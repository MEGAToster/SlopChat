use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};

use super::kinds::Kind;
use crate::config::Settings;

fn cache() -> &'static Mutex<HashMap<String, bool>> {
    static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn supports(kind: Kind, model: &str, settings: &Settings, key: Option<&str>) -> bool {
    if kind == Kind::Gemini {
        return true;
    }

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
    let request = super::client()
        .get(Kind::OpenRouter.models_url(settings))
        .bearer_auth(key)
        .timeout(Duration::from_secs(5));

    let Ok(response) = request.send().await else {
        return true;
    };
    let Ok(body) = response.json::<Value>().await else {
        return true;
    };

    body.get("data")
        .and_then(Value::as_array)
        .and_then(|entries| entries.iter().find(|entry| entry.get("id").and_then(Value::as_str) == Some(model)))
        .and_then(|entry| entry.get("supported_parameters"))
        .and_then(Value::as_array)
        .map(|params| params.iter().any(|param| param.as_str() == Some("tools")))
        .unwrap_or(true)
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
