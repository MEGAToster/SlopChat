use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::ChatRequest;
use crate::config::Settings;

pub const PROVIDER: &str = "openrouter";
const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

fn base_url(settings: &Settings) -> String {
    settings
        .providers
        .get(PROVIDER)
        .map(|provider| provider.base_url.clone())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

pub fn endpoint(settings: &Settings) -> String {
    format!("{}/chat/completions", base_url(settings).trim_end_matches('/'))
}

pub fn body(request: &ChatRequest, model: &str) -> Value {
    let mut messages = Vec::new();
    if let Some(system) = request.system.as_deref() {
        if !system.is_empty() {
            messages.push(json!({ "role": "system", "content": system }));
        }
    }
    for message in &request.messages {
        messages.push(json!({ "role": message.role, "content": message.content }));
    }

    let mut body = json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "usage": { "include": true },
    });

    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }

    body
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

pub async fn models(settings: &Settings, key: &str) -> Result<Vec<Model>, String> {
    #[derive(Deserialize)]
    struct Response {
        data: Vec<Model>,
    }

    let url = format!("{}/models", base_url(settings).trim_end_matches('/'));
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(key)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("models request failed: {}", response.status()));
    }

    let body: Response = response.json().await.map_err(|error| error.to_string())?;
    Ok(body.data)
}
