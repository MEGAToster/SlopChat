use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use reqwest::Url;
use tokio::net::lookup_host;

pub struct Target {
    pub url: Url,
    pub host: String,
    pub addr: SocketAddr,
}

// Parse with the same URL rules reqwest uses, then pin the resolved address so
// a redirect or a re-resolving DNS server cannot swap in a private host.
pub async fn resolve_target(url: &str) -> Result<Target, String> {
    let parsed = Url::parse(url).map_err(|_| "only http/https URLs are allowed".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("only http/https URLs are allowed".to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?
        .to_string();
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".local") {
        return Err("refusing to fetch a local address".to_string());
    }

    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| "URL has no port".to_string())?;

    let addr = match literal(&host) {
        Some(ip) => SocketAddr::new(ip, port),
        None => resolve_host(&host, port).await?,
    };
    if blocked(addr.ip()) {
        return Err("refusing to fetch a private address".to_string());
    }

    Ok(Target {
        url: parsed,
        host,
        addr,
    })
}

// `host_str()` may keep the brackets around an IPv6 literal.
fn literal(host: &str) -> Option<IpAddr> {
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .ok()
}

async fn resolve_host(host: &str, port: u16) -> Result<SocketAddr, String> {
    let addresses: Vec<SocketAddr> = lookup_host((host, port))
        .await
        .map_err(|error| error.to_string())?
        .collect();
    if addresses.is_empty() {
        return Err("could not resolve host".to_string());
    }
    if addresses.iter().any(|address| blocked(address.ip())) {
        return Err("refusing to fetch a private address".to_string());
    }
    Ok(addresses[0])
}

fn blocked(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets[0] == 0
                || ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_broadcast()
                || is_cgnat(ip)
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
        }
    }
}

fn is_cgnat(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (64..128).contains(&octets[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_local_and_non_http() {
        assert!(resolve_target("http://localhost:11434").await.is_err());
        assert!(resolve_target("http://192.168.1.1").await.is_err());
        assert!(resolve_target("http://[::1]/").await.is_err());
        assert!(resolve_target("file:///etc/passwd").await.is_err());
        assert!(resolve_target("http://127.0.0.1").await.is_err());
    }

    #[tokio::test]
    async fn rejects_whatwg_authority_tricks() {
        assert!(resolve_target("http://127.0.0.1\\@evil.com").await.is_err());
        assert!(resolve_target("http://evil.com\\@127.0.0.1").await.is_err());
        assert!(resolve_target("http://0.0.0.0/").await.is_err());
    }

    #[tokio::test]
    async fn rejects_ipv4_mapped_ipv6() {
        assert!(resolve_target("http://[::ffff:127.0.0.1]/").await.is_err());
        assert!(resolve_target("http://[::ffff:192.168.0.1]/").await.is_err());
    }

    #[tokio::test]
    async fn allows_public_ip_literals() {
        assert!(resolve_target("http://1.1.1.1/").await.is_ok());
        assert!(resolve_target("https://[2606:4700:4700::1111]/").await.is_ok());
    }
}
