use std::net::SocketAddr;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, LOCATION};
use reqwest::redirect::Policy;

use super::html;
use super::ssrf::{self, Target};

pub const DEFAULT_CHARS: usize = 12_000;

const MAX_CHARS: usize = 20_000;
const MAX_BYTES: usize = 2 * 1024 * 1024;
const MIN_TEXT: usize = 500;
const MIN_RESULT_CHARS: usize = 50;
const MAX_REDIRECTS: usize = 5;
const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15";

pub struct Fetched {
    pub title: Option<String>,
    pub url: String,
    pub text: String,
    pub start: usize,
    pub total: usize,
    pub via_reader: bool,
}

pub async fn get(
    url: &str,
    start: usize,
    max_chars: usize,
    reader_fallback: bool,
) -> Result<Fetched, String> {
    let max_chars = max_chars.clamp(1, MAX_CHARS);
    let target = ssrf::resolve_target(url).await?;

    let (title, text, via_reader) = match download(&target).await {
        Ok(page) => {
            let raw = String::from_utf8_lossy(&page.body).to_string();
            let mut title = html::title(&raw);
            let mut text = if is_markup(&page.content_type) {
                html::to_text(&raw)
            } else {
                raw.trim().to_string()
            };
            let mut via_reader = false;

            if text.chars().count() < MIN_TEXT && reader_fallback {
                if let Ok((reader_title, reader_text)) = reader(url).await {
                    if !reader_text.is_empty() {
                        text = reader_text;
                        via_reader = true;
                        if title.is_none() {
                            title = reader_title;
                        }
                    }
                }
                check_text(&text, start, true)?;
            }
            (title, text, via_reader)
        }
        Err(FetchError::Blocked(_)) if reader_fallback => {
            let (title, text) = reader(url).await.map_err(|error| actionable(&error))?;
            check_text(&text, start, true)?;
            (title, text, true)
        }
        Err(FetchError::Blocked(reason) | FetchError::Message(reason)) => {
            return Err(actionable(&reason))
        }
    };

    Ok(window(title, text, url, start, max_chars, via_reader))
}

// A near-empty page is a failure, not a result, so the model tries another URL.
fn check_text(text: &str, start: usize, reader_tried: bool) -> Result<(), String> {
    if start == 0 && text.chars().count() < MIN_RESULT_CHARS {
        let suffix = if reader_tried {
            "; reader fallback failed"
        } else {
            ""
        };
        return Err(actionable(&format!(
            "no readable text at this URL (likely JS-rendered or a bot check){suffix}"
        )));
    }
    Ok(())
}

fn window(
    title: Option<String>,
    text: String,
    url: &str,
    start: usize,
    max_chars: usize,
    via_reader: bool,
) -> Fetched {
    let total = text.chars().count();
    let slice: String = text.chars().skip(start).take(max_chars).collect();
    Fetched {
        title,
        url: url.to_string(),
        text: slice,
        start,
        total,
        via_reader,
    }
}

struct Page {
    content_type: String,
    body: Vec<u8>,
}

enum FetchError {
    Blocked(String),
    Message(String),
}

// Follow redirects by hand so every hop is re-resolved and re-pinned; reqwest's
// own redirect policy would reuse the first client's pinned address.
async fn download(target: &Target) -> Result<Page, FetchError> {
    let mut url = target.url.clone();
    let mut host = target.host.clone();
    let mut addr = target.addr;

    for _ in 0..=MAX_REDIRECTS {
        let response = build_client(&host, addr)
            .get(url.clone())
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(|error| FetchError::Message(error.to_string()))?;

        let status = response.status();
        if status.is_redirection() {
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| FetchError::Message(format!("HTTP {status} with no Location")))?;
            let next = url
                .join(location)
                .map_err(|error| FetchError::Message(error.to_string()))?;
            let hop = ssrf::resolve_target(next.as_str())
                .await
                .map_err(FetchError::Blocked)?;
            url = hop.url;
            host = hop.host;
            addr = hop.addr;
            continue;
        }

        if !status.is_success() {
            let reason = format!("HTTP {status}");
            return Err(if matches!(status.as_u16(), 403 | 429 | 503) {
                FetchError::Blocked(reason)
            } else {
                FetchError::Message(reason)
            });
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        let mut response = response;
        let mut body = Vec::new();
        while body.len() < MAX_BYTES {
            match response
                .chunk()
                .await
                .map_err(|error| FetchError::Message(error.to_string()))?
            {
                Some(bytes) => body.extend_from_slice(&bytes),
                None => break,
            }
        }
        body.truncate(MAX_BYTES);

        return Ok(Page { content_type, body });
    }

    Err(FetchError::Blocked("too many redirects".to_string()))
}

async fn reader(url: &str) -> Result<(Option<String>, String), String> {
    let endpoint = format!("https://r.jina.ai/{url}");
    let target = ssrf::resolve_target(&endpoint).await?;
    let response = build_client(&target.host, target.addr)
        .get(&endpoint)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("reader HTTP {}", response.status()));
    }
    let text = response.text().await.map_err(|error| error.to_string())?;
    Ok((None, text.trim().to_string()))
}

fn build_client(host: &str, addr: SocketAddr) -> reqwest::Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en"));

    reqwest::Client::builder()
        .user_agent(UA)
        .default_headers(headers)
        .redirect(Policy::none())
        .resolve(host, addr)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(15))
        .build()
        .expect("fetch client")
}

fn is_markup(content_type: &str) -> bool {
    content_type.is_empty()
        || content_type.contains("html")
        || content_type.contains("xml")
}

fn actionable(reason: &str) -> String {
    format!("{reason}. Try another URL from the search results or answer from the snippets.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_empty_text_is_an_error() {
        assert!(check_text("", 0, false).is_err());
        assert!(check_text("short", 0, true)
            .unwrap_err()
            .contains("reader fallback failed"));
        assert!(check_text(&"x".repeat(60), 0, false).is_ok());
        // Paging into a long page is not an emptiness failure.
        assert!(check_text("", 10, false).is_ok());
    }
}
