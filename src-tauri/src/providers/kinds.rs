use serde_json::{json, Value};

use super::ProviderModel;
use crate::config::Settings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    OpenRouter,
    Ollama,
    Gemini,
}

pub struct Limits<'a> {
    pub reasoning: Option<&'a str>,
    pub reasoning_max_tokens: u32,
    pub max_completion_tokens: Option<u32>,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl Kind {
    fn from_name(name: &str) -> Option<Self> {
        [Kind::OpenRouter, Kind::Ollama, Kind::Gemini]
            .into_iter()
            .find(|kind| kind.name() == name)
    }

    pub fn resolve(provider: &str) -> Result<Self, String> {
        Self::from_name(provider).ok_or_else(|| format!("provider {provider:?} is not supported"))
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

    pub(crate) fn base_url(self, settings: &Settings) -> String {
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

    pub fn body(
        self,
        model: &str,
        messages: &[Value],
        temperature: Option<f32>,
        reasoning: Option<&str>,
        tools: Option<&Value>,
        tool_choice: &Value,
    ) -> Value {
        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });

        match self {
            Kind::OpenRouter => body["usage"] = json!({ "include": true }),
            Kind::Ollama => body["stream_options"] = json!({ "include_usage": true }),
            Kind::Gemini => {}
        }

        if let Some(temperature) = temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(effort) = reasoning {
            if matches!(self, Kind::OpenRouter) {
                body["reasoning"] = json!({ "effort": effort });
            }
        }

        if let Some(tools) = tools {
            body["tools"] = tools.clone();
            body["tool_choice"] = tool_choice.clone();
            // Routing (:floor) must only pick providers that honor tool calls,
            // otherwise the raw tool syntax leaks through as text.
            if matches!(self, Kind::OpenRouter) {
                body["provider"] = json!({ "require_parameters": true });
            }
        }

        body
    }

    // OpenRouter-only generation controls: reasoning budget, completion cap,
    // and mild repetition penalties for models that otherwise loop.
    pub fn apply_limits(self, body: &mut Value, limits: &Limits<'_>) {
        if !matches!(self, Kind::OpenRouter) {
            return;
        }
        if limits.reasoning.is_some() && limits.reasoning_max_tokens > 0 {
            body["reasoning"]["max_tokens"] = json!(limits.reasoning_max_tokens);
        }
        if let Some(max) = limits.max_completion_tokens {
            body["max_completion_tokens"] = json!(max);
        }
        if limits.frequency_penalty > 0.0 {
            body["frequency_penalty"] = json!(limits.frequency_penalty);
        }
        if limits.presence_penalty > 0.0 {
            body["presence_penalty"] = json!(limits.presence_penalty);
        }
    }

    pub(crate) fn models_url(self, settings: &Settings) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openrouter_requires_tool_capable_providers_when_tools_are_sent() {
        let tools = json!([{ "type": "function" }]);
        let body = Kind::OpenRouter.body("m", &[], None, None, Some(&tools), &json!("auto"));
        assert_eq!(body["provider"]["require_parameters"], json!(true));
    }

    #[test]
    fn no_provider_hint_without_tools() {
        let body = Kind::OpenRouter.body("m", &[], None, None, None, &json!("none"));
        assert!(body.get("provider").is_none());
    }

    #[test]
    fn openrouter_sends_reasoning_effort() {
        let body = Kind::OpenRouter.body("m", &[], None, Some("high"), None, &json!("none"));
        assert_eq!(body["reasoning"]["effort"], json!("high"));
    }

    #[test]
    fn other_kinds_ignore_reasoning_effort() {
        let body = Kind::Ollama.body("m", &[], None, Some("high"), None, &json!("none"));
        assert!(body.get("reasoning").is_none());
    }
}
