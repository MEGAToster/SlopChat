use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::client;
use crate::config::ConfigState;

pub(crate) enum VerifyError {
    Rejected,
    Unreachable,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    Ok,
    Rejected,
    Unreachable,
}

impl ApiKeyStatus {
    pub fn of(result: Result<(), VerifyError>) -> Self {
        match result {
            Ok(()) => Self::Ok,
            Err(VerifyError::Rejected) => Self::Rejected,
            Err(VerifyError::Unreachable) => Self::Unreachable,
        }
    }
}

// A cheap authenticated probe; OpenRouter's /key returns 401 for a bad key.
pub(crate) async fn verify_openrouter(base_url: &str, key: &str) -> Result<(), VerifyError> {
    let url = format!("{}/key", base_url.trim_end_matches('/'));
    let response = client()
        .get(url)
        .bearer_auth(key)
        .send()
        .await
        .map_err(|_| VerifyError::Unreachable)?;

    if response.status().is_success() {
        Ok(())
    } else if matches!(response.status().as_u16(), 401 | 403) {
        Err(VerifyError::Rejected)
    } else {
        Err(VerifyError::Unreachable)
    }
}

// Only OpenRouter has a cheap key-check endpoint; other providers skip the probe.
pub async fn verify_provider(app: &AppHandle, provider: &str, key: &str) -> ApiKeyStatus {
    if provider != "openrouter" {
        return ApiKeyStatus::Ok;
    }
    let base_url = app
        .state::<ConfigState>()
        .get()
        .providers
        .get("openrouter")
        .map(|entry| entry.base_url.clone())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
    ApiKeyStatus::of(verify_openrouter(&base_url, key).await)
}
