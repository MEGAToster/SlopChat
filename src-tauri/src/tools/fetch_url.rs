use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;

use serde_json::{json, Value};

use super::{html, ToolContext, ToolOutput};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_CHARS: usize = 8000;

pub fn schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "web_fetch",
            "description": "Fetch a web page and return its text. Read-only.",
            "parameters": {
                "type": "object",
                "properties": { "url": { "type": "string" } },
                "required": ["url"]
            }
        }
    })
}

pub async fn run(args: Value, _context: ToolContext) -> Result<ToolOutput, String> {
    let url = args
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    guard(&url)?;

    let response = crate::providers::client()
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("fetch failed: {}", response.status()));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.contains("text/html") && !content_type.contains("text/plain") {
        return Err(format!("unsupported content type: {content_type}"));
    }

    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("page too large".to_string());
    }

    let text = html::to_text(&String::from_utf8_lossy(&bytes));
    let content: String = text.chars().take(MAX_CHARS).collect();

    Ok(ToolOutput {
        content,
        summary: "page fetched".to_string(),
        sources: Vec::new(),
        cost: 0.0,
    })
}

fn guard(url: &str) -> Result<(), String> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .ok_or_else(|| "only http/https URLs are allowed".to_string())?;

    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_port.split(':').next().unwrap_or(host_port);

    if host.is_empty() || host.eq_ignore_ascii_case("localhost") || host.ends_with(".local") {
        return Err("refusing to fetch a local address".to_string());
    }

    let addresses = (host, 0)
        .to_socket_addrs()
        .map_err(|error| error.to_string())?;

    for address in addresses {
        if blocked(address.ip()) {
            return Err("refusing to fetch a private address".to_string());
        }
    }

    Ok(())
}

fn blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified()
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

    #[test]
    fn rejects_local_and_non_http() {
        assert!(guard("http://localhost:11434").is_err());
        assert!(guard("http://192.168.1.1").is_err());
        assert!(guard("http://[::1]/").is_err());
        assert!(guard("file:///etc/passwd").is_err());
        assert!(guard("http://127.0.0.1").is_err());
    }
}
