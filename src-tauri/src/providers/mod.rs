mod agent;
pub mod capabilities;
mod chunks;
pub mod client;
pub mod kinds;
mod leak;
mod prompt;
mod sse;
mod stream;

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::{Settings, WebSearchMode};
use crate::secrets;

pub fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(300))
            .build()
            .expect("http client")
    })
}

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
    pub content: serde_json::Value,
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
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_true")]
    pub tool_calling: bool,
    #[serde(default)]
    pub web_search: WebSearchMode,
    #[serde(default)]
    pub web_toggle: bool,
    #[serde(default)]
    pub reasoning: Option<String>,
    pub messages: Vec<ChatMessage>,
}

fn default_true() -> bool {
    true
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
pub struct Source {
    pub n: u32,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum StreamEvent {
    Delta {
        content: String,
    },
    Reasoning,
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        id: String,
        ok: bool,
        summary: String,
        sources: Vec<Source>,
    },
    Usage {
        tokens_in: u32,
        tokens_out: u32,
        cost: Option<f64>,
        provider: Option<String>,
    },
    Done,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub provider: String,
    pub id: String,
    pub name: Option<String>,
}

pub async fn list_all(settings: &Settings) -> Vec<ProviderModel> {
    let openrouter_key = api_key(settings, kinds::Kind::OpenRouter.name())
        .ok()
        .flatten();
    let gemini_key = api_key(settings, kinds::Kind::Gemini.name())
        .ok()
        .flatten();

    let openrouter = async {
        match openrouter_key.as_deref() {
            Some(key) => kinds::Kind::OpenRouter
                .fetch_models(settings, Some(key))
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };
    let ollama = async {
        kinds::Kind::Ollama
            .fetch_models(settings, None)
            .await
            .unwrap_or_default()
    };
    let gemini = async {
        match gemini_key.as_deref() {
            Some(key) => kinds::Kind::Gemini
                .fetch_models(settings, Some(key))
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };

    let (openrouter, ollama, gemini) = tokio::join!(openrouter, ollama, gemini);
    let mut models = openrouter;
    models.extend(ollama);
    models.extend(gemini);
    models
}

pub fn api_key(settings: &Settings, provider: &str) -> Result<Option<String>, String> {
    if let Some(configured) = settings
        .providers
        .get(provider)
        .and_then(|provider| provider.api_key.as_deref())
    {
        return match configured.strip_prefix("env:") {
            Some(variable) => Ok(std::env::var(variable).ok()),
            None => Err(format!(
                "providers.{provider}.apiKey must use an env:VAR reference; raw keys can't live in settings.json"
            )),
        };
    }

    secrets::get(provider)
}

// Falls back to the conventional <NAME>_API_KEY environment variable.
pub fn search_key(settings: &Settings, provider: &str) -> Result<Option<String>, String> {
    if let Some(key) = api_key(settings, provider)? {
        return Ok(Some(key));
    }
    let variable = format!("{}_API_KEY", provider.to_ascii_uppercase());
    Ok(std::env::var(variable).ok().filter(|value| !value.trim().is_empty()))
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
            provider: Some("OpenAI".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"type":"usage","tokensIn":1,"tokensOut":2,"cost":0.5,"provider":"OpenAI"}"#
        );

        let delta = serde_json::to_string(&StreamEvent::Delta {
            content: "hi".to_string(),
        })
        .unwrap();
        assert_eq!(delta, r#"{"type":"delta","content":"hi"}"#);
    }
}
