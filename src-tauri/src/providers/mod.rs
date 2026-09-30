pub mod client;
pub mod gemini;
pub mod ollama;
pub mod openrouter;
pub mod sse;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Settings;
use crate::secrets;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

impl ModelRef {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.split_once('/') {
            Some((provider, model)) if !provider.is_empty() && !model.is_empty() => Ok(Self {
                provider: provider.to_string(),
                model: model.to_string(),
            }),
            _ => Err(format!(
                "invalid model reference {value:?} (expected <provider>/<model>)"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub chat_id: String,
    pub model: String,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub use_context: bool,
    pub messages: Vec<ChatMessage>,
}

pub(crate) fn messages(request: &ChatRequest) -> Vec<Value> {
    let mut messages = Vec::new();
    if let Some(system) = request.system.as_deref() {
        if !system.is_empty() {
            messages.push(json!({ "role": "system", "content": system }));
        }
    }
    for message in &request.messages {
        messages.push(json!({ "role": message.role, "content": message.content }));
    }
    messages
}

impl ChatRequest {
    pub fn validate(&self) -> Result<(), String> {
        ModelRef::parse(&self.model)?;
        if self.messages.is_empty() {
            return Err("no messages to send".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub provider: String,
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum StreamEvent {
    Delta { content: String },
    Usage { tokens_in: u32, tokens_out: u32, cost: Option<f64> },
    Done,
    Error { message: String },
}

pub fn api_key(settings: &Settings, provider: &str) -> Result<Option<String>, String> {
    if let Some(configured) = settings
        .providers
        .get(provider)
        .and_then(|provider| provider.api_key.as_deref())
    {
        if let Some(variable) = configured.strip_prefix("env:") {
            return Ok(std::env::var(variable).ok());
        }
        return Ok(Some(configured.to_string()));
    }

    secrets::get(provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_the_first_slash_only() {
        let model = ModelRef::parse("openrouter/anthropic/claude-sonnet-5:floor").unwrap();
        assert_eq!(model.provider, "openrouter");
        assert_eq!(model.model, "anthropic/claude-sonnet-5:floor");
    }

    #[test]
    fn rejects_incomplete_references() {
        assert!(ModelRef::parse("openrouter").is_err());
        assert!(ModelRef::parse("/model").is_err());
        assert!(ModelRef::parse("provider/").is_err());
        assert!(ModelRef::parse("").is_err());
    }

    #[test]
    fn stream_events_use_camel_case_wire_format() {
        let event = StreamEvent::Usage {
            tokens_in: 1,
            tokens_out: 2,
            cost: Some(0.5),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"type":"usage","tokensIn":1,"tokensOut":2,"cost":0.5}"#);

        let delta = serde_json::to_string(&StreamEvent::Delta {
            content: "hi".to_string(),
        })
        .unwrap();
        assert_eq!(delta, r#"{"type":"delta","content":"hi"}"#);
    }
}
