use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};

use super::kinds::Kind;
use crate::config::Settings;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningInfo {
    pub supported_efforts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_effort: Option<String>,
}

fn cache() -> &'static Mutex<HashMap<String, bool>> {
    static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn reasoning_cache() -> &'static Mutex<HashMap<String, Option<ReasoningInfo>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<ReasoningInfo>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Effort levels OpenRouter accepts when a model leaves `supported_efforts` null.
const ALL_EFFORTS: &[&str] = &["max", "xhigh", "high", "medium", "low", "minimal", "none"];

pub async fn reasoning(
    kind: Kind,
    model: &str,
    settings: &Settings,
    key: Option<&str>,
) -> Option<ReasoningInfo> {
    if kind != Kind::OpenRouter {
        return None;
    }

    let cache_key = format!("{}:{model}", kind.name());
    if let Some(value) = reasoning_cache()
        .lock()
        .expect("reasoning cache")
        .get(&cache_key)
    {
        return value.clone();
    }

    let info = openrouter_reasoning(settings, key, model).await;
    reasoning_cache()
        .lock()
        .expect("reasoning cache")
        .insert(cache_key, info.clone());
    info
}

async fn fetch_models(settings: &Settings, key: &str) -> Option<Value> {
    let response = super::client()
        .get(Kind::OpenRouter.models_url(settings))
        .bearer_auth(key)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    response.json::<Value>().await.ok()
}

async fn openrouter_reasoning(
    settings: &Settings,
    key: Option<&str>,
    model: &str,
) -> Option<ReasoningInfo> {
    let body = fetch_models(settings, key?).await?;
    let entry = find_entry(body.get("data")?.as_array()?, model)?;
    let reasoning = entry.get("reasoning")?;

    let supported_efforts = match reasoning.get("supported_efforts") {
        Some(Value::Null) => ALL_EFFORTS.iter().map(|value| value.to_string()).collect(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => return None,
    };

    Some(ReasoningInfo {
        supported_efforts,
        default_effort: reasoning
            .get("default_effort")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

pub async fn supports(kind: Kind, model: &str, settings: &Settings, key: Option<&str>) -> bool {
    let cache_key = format!("{}:{model}", kind.name());
    if let Some(value) = cache().lock().expect("capability cache").get(&cache_key) {
        return *value;
    }

    let supported = match kind {
        Kind::OpenRouter => openrouter(settings, key, model).await,
        Kind::Ollama => ollama(settings, model).await,
        Kind::Gemini => true,
    };

    cache()
        .lock()
        .expect("capability cache")
        .insert(cache_key, supported);
    supported
}

async fn openrouter(settings: &Settings, key: Option<&str>, model: &str) -> bool {
    let Some(key) = key else {
        return true;
    };
    let Some(body) = fetch_models(settings, key).await else {
        return true;
    };

    body.get("data")
        .and_then(Value::as_array)
        .and_then(|entries| find_entry(entries, model))
        .and_then(|entry| entry.get("supported_parameters"))
        .and_then(Value::as_array)
        .map(|params| params.iter().any(|param| param.as_str() == Some("tools")))
        .unwrap_or(true)
}

// `:floor`, `:nitro`, … are routing variants of the base model id.
fn base_model(model: &str) -> &str {
    model.split(':').next().unwrap_or(model)
}

fn find_entry<'a>(entries: &'a [Value], model: &str) -> Option<&'a Value> {
    let base = base_model(model);
    entries.iter().find(|entry| {
        entry
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == base)
    })
}

async fn ollama(settings: &Settings, model: &str) -> bool {
    let root = Kind::Ollama
        .base_url(settings)
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .to_string();
    let url = format!("{root}/api/show");

    let Ok(response) = super::client()
        .post(url)
        .json(&json!({ "model": model }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
    else {
        return false;
    };
    let Ok(body) = response.json::<Value>().await else {
        return false;
    };

    body.get("capabilities")
        .and_then(Value::as_array)
        .map(|caps| caps.iter().any(|cap| cap.as_str() == Some("tools")))
        .unwrap_or(false)
}
