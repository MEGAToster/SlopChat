use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::config::{self, ConfigState, Settings};
use crate::panel;
use crate::providers::client::Streams;
use crate::providers::{self, ChatRequest, StreamEvent};
use crate::secrets;
use crate::store::{chats, messages, search, Attachment, Chat, ChatView, Message, NewMessage, Store};

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    panel::hide(&app, true);
}

#[tauri::command]
pub fn resize_panel(app: AppHandle, height: f64) {
    panel::resize_compact(&app, height);
}

#[tauri::command]
pub fn resize_overlay(app: AppHandle, height: f64) {
    panel::resize_overlay(&app, height);
}

#[tauri::command]
pub fn set_panel_expanded(app: AppHandle, expanded: bool) {
    panel::set_expanded(&app, expanded);
}

#[tauri::command]
pub fn set_window_style(app: AppHandle, scheme: String, vibrancy: String) {
    panel::set_window_style(&app, &scheme, &vibrancy);
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

#[tauri::command(async)]
pub fn has_api_key(app: AppHandle, provider: String) -> Result<bool, String> {
    let settings = app.state::<ConfigState>().get();
    providers::api_key(&settings, &provider).map(|key| key.is_some())
}

#[tauri::command(async)]
pub fn store_api_key(app: AppHandle, provider: String, key: String) -> Result<(), String> {
    providers::store_key(&app, &provider, &key)
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
pub fn initial_chat(app: AppHandle, store: State<Store>) -> Result<Option<ChatView>, String> {
    let settings = app.state::<ConfigState>().get();
    chats::initial(&store.conn(), settings.startup, settings.resume_within_minutes)
}

#[tauri::command(async)]
pub fn list_chats(store: State<Store>) -> Result<Vec<Chat>, String> {
    chats::list(&store.conn())
}

#[tauri::command(async)]
pub fn create_chat(store: State<Store>, profile: String) -> Result<Chat, String> {
    chats::create(&store.conn(), &profile)
}

#[tauri::command(async)]
pub fn get_chat(store: State<Store>, chat_id: String) -> Result<Option<ChatView>, String> {
    chats::view_by_id(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn delete_chat(store: State<Store>, chat_id: String) -> Result<(), String> {
    chats::delete(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn toggle_chat_pin(store: State<Store>, chat_id: String) -> Result<Chat, String> {
    chats::toggle_pin(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn set_chat_profile(
    store: State<Store>,
    chat_id: String,
    profile: String,
) -> Result<(), String> {
    chats::set_profile(&store.conn(), &chat_id, &profile)
}

#[tauri::command(async)]
pub fn rename_chat(store: State<Store>, chat_id: String, title: String) -> Result<(), String> {
    chats::rename(&store.conn(), &chat_id, &title)
}

#[tauri::command(async)]
pub fn set_chat_model(store: State<Store>, chat_id: String, model: Option<String>) -> Result<(), String> {
    chats::set_model(&store.conn(), &chat_id, model.as_deref())
}

#[tauri::command(async)]
pub fn append_message(store: State<Store>, message: NewMessage) -> Result<Message, String> {
    messages::append(&store.conn(), &message)
}

#[tauri::command(async)]
pub fn rewind_to(
    store: State<Store>,
    chat_id: String,
    message_id: String,
    content: String,
) -> Result<(), String> {
    messages::rewind(&store.conn(), &chat_id, &message_id, &content)
}

#[tauri::command(async)]
pub fn search_chats(store: State<Store>, query: String) -> Result<Vec<Chat>, String> {
    search::chats(&store.conn(), &query, 20)
}

#[tauri::command]
pub fn set_suppress_hide(app: AppHandle, value: bool) {
    panel::set_suppressed(&app, value);
}

#[tauri::command(async)]
pub fn save_attachment(name: String, mime: String, data: String) -> Result<Attachment, String> {
    crate::store::attachments::save(&name, &mime, &data)
}

#[tauri::command(async)]
pub fn read_attachment(path: String) -> Result<String, String> {
    crate::store::attachments::read(&path)
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<providers::ProviderModel>, String> {
    let settings = app.state::<ConfigState>().get();
    Ok(providers::list_all(&settings).await)
}

#[tauri::command(async)]
pub fn open_settings_file() -> Result<(), String> {
    config::open_file()
}

#[tauri::command(async)]
pub fn open_models_file() -> Result<(), String> {
    config::open_models_file()
}

#[tauri::command(async)]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")) {
        return Err(format!("refusing to open unsupported URL: {url}"));
    }
    crate::platform::open_path(std::path::Path::new(&url))
}

#[tauri::command(async)]
pub fn reload_config(app: AppHandle) {
    config::reload(&app);
}

#[tauri::command(async)]
pub fn save_model(
    app: AppHandle,
    model: config::ModelConfig,
    previous: Option<String>,
) -> Result<(), String> {
    config::save_model(&app, &model, previous.as_deref())
}

#[tauri::command(async)]
pub fn remove_model(app: AppHandle, model: String) -> Result<(), String> {
    config::remove_model(&app, &model)
}

#[tauri::command(async)]
pub fn save_profile(
    app: AppHandle,
    store: State<Store>,
    profile: config::Profile,
    previous: Option<String>,
) -> Result<(), String> {
    config::save_profile(&app, &profile, previous.as_deref())?;
    if let Some(previous) = previous.as_deref() {
        if previous != profile.name {
            chats::rename_profile(&store.conn(), previous, &profile.name)?;
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn remove_profile(app: AppHandle, name: String) -> Result<(), String> {
    config::remove_profile(&app, &name)
}

#[tauri::command]
pub async fn model_reasoning(
    app: AppHandle,
    model: String,
) -> Result<Option<providers::capabilities::ReasoningInfo>, String> {
    providers::reasoning_for(&app, &model).await
}

#[tauri::command(async)]
pub fn set_theme(app: AppHandle, theme: String) -> Result<(), String> {
    config::set_theme(&app, &theme)
}

#[tauri::command(async)]
pub fn set_style(app: AppHandle, style: String) -> Result<(), String> {
    config::set_style(&app, &style)
}

#[tauri::command(async)]
pub fn save_advanced(app: AppHandle, advanced: config::Advanced) -> Result<(), String> {
    config::save_advanced(&app, &advanced)
}

#[tauri::command(async)]
pub fn set_startup(app: AppHandle, startup: config::Startup) -> Result<(), String> {
    config::set_startup(&app, startup)
}

#[tauri::command(async)]
pub fn set_resume_within(app: AppHandle, minutes: u64) -> Result<(), String> {
    config::set_resume_within(&app, minutes)
}

#[tauri::command(async)]
pub fn set_reader_fallback(app: AppHandle, enabled: bool) -> Result<(), String> {
    config::set_reader_fallback(&app, enabled)
}

#[tauri::command(async)]
pub fn set_agent_log(app: AppHandle, enabled: bool) -> Result<(), String> {
    config::set_agent_log(&app, enabled)
}

#[tauri::command(async)]
pub fn open_agent_log() -> Result<(), String> {
    providers::trace::ensure()?;
    crate::platform::open_path(&providers::trace::path())
}

#[tauri::command(async)]
pub fn save_window(app: AppHandle, window: config::WindowConfig) -> Result<(), String> {
    config::save_window(&app, &window)
}

#[tauri::command(async)]
pub fn load_user_themes() -> Vec<config::UserTheme> {
    config::load_user_themes()
}

#[tauri::command(async)]
pub fn search_usage(app: AppHandle) -> Vec<crate::tools::search::allowance::SearchProviderUsage> {
    crate::tools::search::allowance::report(&app)
}
