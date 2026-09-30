use serde_json::{json, Value};

use super::{ChatRequest, ProviderModel};
use crate::config::Settings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    OpenRouter,
    Ollama,
    Gemini,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::OpenRouter, Kind::Ollama, Kind::Gemini];

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::OpenRouter => "openrouter",
            Kind::Ollama => "ollama",
            Kind::Gemini => "gemini",
        }
    }

    fn default_base_url(self) -> &'static str {
        match self {
            Kind::OpenRouter => "https://openrouter.ai/api/v1",
            Kind::Ollama => "http://localhost:11434/v1",
            Kind::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai/",
        }
    }

    pub fn needs_auth(self) -> bool {
        !matches!(self, Kind::Ollama)
    }

    pub fn zero_cost(self) -> bool {
        matches!(self, Kind::Ollama)
    }

    fn base_url(self, settings: &Settings) -> String {
        settings
            .providers
            .get(self.name())
            .map(|provider| provider.base_url.clone())
            .unwrap_or_else(|| self.default_base_url().to_string())
    }

    pub fn endpoint(self, settings: &Settings) -> String {
        format!(
            "{}/chat/completions",
            self.base_url(settings).trim_end_matches('/')
        )
    }

    pub fn body(self, request: &ChatRequest, model: &str) -> Value {
        let mut body = json!({
            "model": model,
            "messages": super::messages(request),
            "stream": true,
        });

        match self {
            Kind::OpenRouter => body["usage"] = json!({ "include": true }),
            Kind::Ollama => body["stream_options"] = json!({ "include_usage": true }),
            Kind::Gemini => {}
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        body
    }

    fn models_url(self, settings: &Settings) -> String {
        match self {
            Kind::Ollama => format!(
                "{}/api/tags",
                self.base_url(settings)
                    .trim_end_matches('/')
                    .trim_end_matches("/v1")
            ),
            _ => format!(
                "{}/models",
                self.base_url(settings).trim_end_matches('/')
            ),
        }
    }

    pub async fn fetch_models(
        self,
        settings: &Settings,
        key: Option<&str>,
    ) -> Result<Vec<ProviderModel>, String> {
        let mut request = super::client().get(self.models_url(settings));
        if let Some(key) = key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await.map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!(
                "{} models request failed: {}",
                self.name(),
                response.status()
            ));
        }

        let body: Value = response.json().await.map_err(|error| error.to_string())?;
        Ok(match self {
            Kind::Ollama => parse_ollama(&body, self.name()),
            _ => parse_openai(&body, self.name()),
        })
    }
}

fn parse_openai(body: &Value, provider: &str) -> Vec<ProviderModel> {
    body.get("data")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let id = entry.get("id")?.as_str()?.to_string();
                    let name = entry
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    Some(ProviderModel {
                        provider: provider.to_string(),
                        id,
                        name,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_ollama(body: &Value, provider: &str) -> Vec<ProviderModel> {
    body.get("models")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let name = entry
                        .get("name")
                        .or_else(|| entry.get("model"))
                        .and_then(Value::as_str)?
                        .to_string();
                    Some(ProviderModel {
                        provider: provider.to_string(),
                        id: name.clone(),
                        name: Some(name),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
