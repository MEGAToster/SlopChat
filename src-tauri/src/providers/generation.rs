use std::time::Duration;

use serde_json::Value;

use super::kinds::Kind;
use crate::config::Settings;

const TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_DELAYS_MS: [u64; 3] = [400, 800, 1500];

// Looks up the upstream host for an OpenRouter generation when the stream didn't name it.
pub async fn host(settings: &Settings, key: &str, id: &str) -> Option<String> {
    let url = format!(
        "{}/generation?id={}",
        Kind::OpenRouter.base_url(settings).trim_end_matches('/'),
        id
    );
    if let Some(host) = probe(&url, key).await {
        return Some(host);
    }
    for delay in RETRY_DELAYS_MS {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        if let Some(host) = probe(&url, key).await {
            return Some(host);
        }
    }
    None
}

async fn probe(url: &str, key: &str) -> Option<String> {
    let response = super::client()
        .get(url)
        .bearer_auth(key)
        .timeout(TIMEOUT)
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body: Value = response.json().await.ok()?;
    parse_host(&body)
}

fn parse_host(body: &Value) -> Option<String> {
    body.get("data")?
        .get("provider_name")?
        .as_str()
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_provider_name() {
        let body = json!({ "data": { "provider_name": "DeepInfra", "id": "gen-1" } });
        assert_eq!(parse_host(&body).as_deref(), Some("DeepInfra"));
    }

    #[test]
    fn missing_field_is_none() {
        assert!(parse_host(&json!({ "data": {} })).is_none());
    }

    #[test]
    fn null_data_is_none() {
        assert!(parse_host(&json!({ "data": null })).is_none());
    }

    #[test]
    fn empty_name_is_none() {
        assert!(parse_host(&json!({ "data": { "provider_name": "" } })).is_none());
    }
}
