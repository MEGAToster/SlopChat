use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::config::{self, ConfigState, Settings, Startup};
use crate::panel;
use crate::paths;
use crate::providers::client::Streams;
use crate::providers::kinds::Kind;
use crate::providers::{self, ChatRequest, StreamEvent};
use crate::secrets;
use crate::store::{chats, messages, search, Attachment, Chat, ChatView, Message, NewMessage, Store};

#[tauri::command]
pub fn show_panel(app: AppHandle) {
    panel::show(&app);
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    panel::hide(&app, true);
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
pub fn set_window_style(app: AppHandle, scheme: String, vibrancy: String, opacity: f64) {
    panel::set_window_style(&app, &scheme, &vibrancy, opacity);
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
pub fn store_api_key(provider: String, key: String) -> Result<(), String> {
    secrets::set(&provider, &key)
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
    let resume_last = matches!(settings.startup, Startup::Last);
    chats::initial(&store.conn(), resume_last, settings.resume_within_minutes)
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
    let conn = store.conn();
    match chats::get(&conn, &chat_id)? {
        Some(chat) => chats::view(&conn, chat).map(Some),
        None => Ok(None),
    }
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
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|error| format!("decoding attachment: {error}"))?;

    let id = crate::store::new_id("a");
    let extension = std::path::Path::new(&name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    let dir = paths::attachments_dir();
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("{id}{extension}"));
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;

    Ok(Attachment {
        id,
        path: path.to_string_lossy().to_string(),
        mime: Some(mime),
    })
}

#[tauri::command(async)]
pub fn read_attachment(path: String) -> Result<String, String> {
    use base64::Engine;

    let dir = paths::attachments_dir();
    let canonical = std::fs::canonicalize(&path).map_err(|error| error.to_string())?;
    let root = std::fs::canonicalize(&dir).unwrap_or(dir);
    if !canonical.starts_with(&root) {
        return Err("attachment path is outside the attachments directory".to_string());
    }

    let bytes = std::fs::read(&canonical).map_err(|error| error.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<providers::ProviderModel>, String> {
    let settings = app.state::<ConfigState>().get();
    let openrouter_key = providers::api_key(&settings, Kind::OpenRouter.name())?;
    let gemini_key = providers::api_key(&settings, Kind::Gemini.name())?;

    let openrouter = async {
        match openrouter_key.as_deref() {
            Some(key) => Kind::OpenRouter
                .fetch_models(&settings, Some(key))
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };
    let ollama = async {
        Kind::Ollama
            .fetch_models(&settings, None)
            .await
            .unwrap_or_default()
    };
    let gemini = async {
        match gemini_key.as_deref() {
            Some(key) => Kind::Gemini
                .fetch_models(&settings, Some(key))
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };

    let (openrouter, ollama, gemini) = tokio::join!(openrouter, ollama, gemini);
    let mut models = openrouter;
    models.extend(ollama);
    models.extend(gemini);
    Ok(models)
}

#[tauri::command(async)]
pub fn open_settings_file() -> Result<(), String> {
    config::ensure_file()?;
    let path = paths::settings_path();

    if let Some(editor) = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .ok()
        .filter(|editor| !editor.trim().is_empty())
    {
        return std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("exec {editor} \"$1\""))
            .arg("sh")
            .arg(&path)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string());
    }

    open_path(&path)
}

#[tauri::command(async)]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")) {
        return Err(format!("refusing to open unsupported URL: {url}"));
    }
    open_path(std::path::Path::new(&url))
}

#[tauri::command(async)]
pub fn reload_config(app: AppHandle) {
    config::reload(&app);
}

#[tauri::command(async)]
pub fn add_model(app: AppHandle, model: String) -> Result<(), String> {
    config::add_model(&app, &model)
}

#[tauri::command(async)]
pub fn remove_model(app: AppHandle, model: String) -> Result<(), String> {
    config::remove_model(&app, &model)
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
pub fn load_user_themes() -> Vec<config::UserTheme> {
    config::load_user_themes()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchUsage {
    pub requests: i64,
    pub cost: f64,
    pub budget: f64,
}

#[tauri::command(async)]
pub fn search_usage(app: AppHandle) -> SearchUsage {
    let (requests, cost) = crate::tools::budget::usage(&app);
    let budget = app.state::<ConfigState>().get().search.monthly_budget;
    SearchUsage {
        requests,
        cost,
        budget,
    }
}

fn open_path(target: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(target_os = "macos"))]
    let opener = "xdg-open";

    std::process::Command::new(opener)
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
