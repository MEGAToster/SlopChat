use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::config::{ConfigState, Settings};
use crate::panel;
use crate::providers::client::Streams;
use crate::providers::{self, ChatRequest, StreamEvent};
use crate::secrets;

#[tauri::command]
pub fn show_panel(app: AppHandle) {
    panel::show(&app);
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    panel::hide(&app);
}

#[tauri::command]
pub fn toggle_panel(app: AppHandle) {
    panel::toggle(&app);
}

#[tauri::command]
pub fn resize_panel(app: AppHandle, height: f64) {
    panel::resize(&app, height);
}

#[tauri::command]
pub fn toggle_sticky(app: AppHandle) -> bool {
    panel::toggle_sticky(&app)
}

#[tauri::command]
pub fn is_sticky(app: AppHandle) -> bool {
    panel::is_sticky(&app)
}

#[tauri::command]
pub fn get_settings(config: State<ConfigState>) -> Settings {
    config.get()
}

#[tauri::command]
pub fn config_error(config: State<ConfigState>) -> Option<String> {
    config.error()
}

#[tauri::command]
pub fn has_api_key(app: AppHandle, provider: String) -> Result<bool, String> {
    let settings = app.state::<ConfigState>().get();
    providers::api_key(&settings, &provider).map(|key| key.is_some())
}

#[tauri::command]
pub fn store_api_key(provider: String, key: String) -> Result<(), String> {
    secrets::set(&provider, &key)
}

#[tauri::command]
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
