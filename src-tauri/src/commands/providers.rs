use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use crate::config::ConfigState;
use crate::providers::client::Streams;
use crate::providers::{self, ChatRequest, StreamEvent};
use crate::secrets;

#[tauri::command(async)]
pub fn has_api_key(app: AppHandle, provider: String) -> Result<bool, String> {
    let settings = app.state::<ConfigState>().get();
    providers::api_key(&settings, &provider).map(|key| key.is_some())
}

#[tauri::command(async)]
pub fn store_api_key(app: AppHandle, provider: String, key: String) -> Result<(), String> {
    providers::store_key(&app, &provider, &key)
}

#[tauri::command]
pub async fn verify_api_key(
    app: AppHandle,
    provider: String,
    key: String,
) -> Result<providers::verify::ApiKeyStatus, String> {
    Ok(providers::verify::verify_provider(&app, &provider, &key).await)
}

#[tauri::command(async)]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    secrets::delete(&provider)
}

#[tauri::command]
pub async fn send(
    app: AppHandle,
    channel: Channel<StreamEvent>,
    request: ChatRequest,
) -> Result<(), String> {
    request.validate()?;
    providers::client::stream(&app, request, channel).await
}

#[tauri::command]
pub fn stop(app: AppHandle, chat_id: String) {
    app.state::<Streams>().stop(&chat_id);
}

#[tauri::command(async)]
pub async fn antigravity_status() -> providers::antigravity::binary::Status {
    providers::antigravity::binary::status().await
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<providers::ProviderModel>, String> {
    let settings = app.state::<ConfigState>().get();
    Ok(providers::list_all(&settings).await)
}

#[tauri::command]
pub async fn model_reasoning(
    app: AppHandle,
    model: String,
) -> Result<Option<providers::capabilities::ReasoningInfo>, String> {
    providers::reasoning_for(&app, &model).await
}

#[tauri::command(async)]
pub fn open_agent_log() -> Result<(), String> {
    providers::trace::ensure()?;
    crate::platform::open_path(&providers::trace::path())
}

#[tauri::command(async)]
pub fn search_usage(app: AppHandle) -> Vec<crate::tools::search::allowance::SearchProviderUsage> {
    crate::tools::search::allowance::report(&app)
}
