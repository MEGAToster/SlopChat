use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::config::{ConfigState, Settings, Startup};
use crate::panel;
use crate::providers::client::Streams;
use crate::providers::{self, ChatRequest, StreamEvent};
use crate::secrets;
use crate::store::{chats, messages, Chat, ChatView, Message, NewMessage, Store};

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
    panel::resize_compact(&app, height);
}

#[tauri::command]
pub fn set_panel_expanded(app: AppHandle, expanded: bool) {
    panel::set_expanded(&app, expanded);
}

#[tauri::command]
pub fn toggle_panel_expanded(app: AppHandle) {
    panel::toggle_expanded(&app);
}

#[tauri::command]
pub fn is_panel_expanded(app: AppHandle) -> bool {
    panel::is_expanded(&app)
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

#[tauri::command]
pub fn initial_chat(app: AppHandle, store: State<Store>) -> Result<Option<ChatView>, String> {
    let settings = app.state::<ConfigState>().get();
    let resume_last = matches!(settings.startup, Startup::Last);
    chats::initial(&store.conn(), resume_last, settings.resume_within_minutes)
}

#[tauri::command]
pub fn list_chats(store: State<Store>) -> Result<Vec<Chat>, String> {
    chats::list(&store.conn())
}

#[tauri::command]
pub fn create_chat(store: State<Store>, profile: String) -> Result<Chat, String> {
    chats::create(&store.conn(), &profile)
}

#[tauri::command]
pub fn get_chat(store: State<Store>, chat_id: String) -> Result<Option<ChatView>, String> {
    let conn = store.conn();
    match chats::get(&conn, &chat_id)? {
        Some(chat) => chats::view(&conn, chat).map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn delete_chat(store: State<Store>, chat_id: String) -> Result<(), String> {
    chats::delete(&store.conn(), &chat_id)
}

#[tauri::command]
pub fn toggle_chat_pin(store: State<Store>, chat_id: String) -> Result<Chat, String> {
    chats::toggle_pin(&store.conn(), &chat_id)
}

#[tauri::command]
pub fn set_chat_profile(
    store: State<Store>,
    chat_id: String,
    profile: String,
) -> Result<(), String> {
    chats::set_profile(&store.conn(), &chat_id, &profile)
}

#[tauri::command]
pub fn append_message(store: State<Store>, message: NewMessage) -> Result<Message, String> {
    messages::append(&store.conn(), &message)
}

#[tauri::command]
pub fn rewind_to(
    store: State<Store>,
    chat_id: String,
    message_id: String,
    content: String,
) -> Result<(), String> {
    messages::rewind(&store.conn(), &chat_id, &message_id, &content)
}
