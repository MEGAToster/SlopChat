use tauri::{AppHandle, State};

use crate::config::{self, ConfigState, Settings};
use crate::store::{chats, Store};

#[tauri::command]
pub fn get_settings(config: State<ConfigState>) -> Settings {
    config.get()
}

#[tauri::command]
pub fn config_error(config: State<ConfigState>) -> Option<String> {
    config.error()
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

#[tauri::command(async)]
pub fn set_theme(app: AppHandle, theme: String) -> Result<(), String> {
    config::set_theme(&app, &theme)
}

#[tauri::command(async)]
pub fn set_style(app: AppHandle, style: String) -> Result<(), String> {
    config::set_style(&app, &style)
}

#[tauri::command(async)]
pub fn set_reading_width(app: AppHandle, width: String) -> Result<(), String> {
    match width.as_str() {
        "narrow" | "wide" | "full" => config::set_reading_width(&app, &width),
        other => Err(format!("invalid reading width {other:?}")),
    }
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
pub fn set_paste_as_file(app: AppHandle, chars: u64) -> Result<(), String> {
    config::set_paste_as_file(&app, chars.min(10_000_000))
}

#[tauri::command(async)]
pub fn set_reader_fallback(app: AppHandle, enabled: bool) -> Result<(), String> {
    config::set_reader_fallback(&app, enabled)
}

#[tauri::command(async)]
pub fn set_searxng_url(app: AppHandle, url: String) -> Result<(), String> {
    config::set_searxng_base_url(&app, &url)
}

#[tauri::command(async)]
pub fn set_searxng_interval(app: AppHandle, ms: u64) -> Result<(), String> {
    config::set_searxng_min_interval(&app, ms)
}

#[tauri::command]
pub async fn probe_searxng(app: AppHandle) -> Option<String> {
    crate::tools::search::probe_searxng(&app).await
}

#[tauri::command(async)]
pub fn set_agent_log(app: AppHandle, enabled: bool) -> Result<(), String> {
    config::set_agent_log(&app, enabled)
}

#[tauri::command(async)]
pub fn save_window(app: AppHandle, window: config::WindowConfig) -> Result<(), String> {
    config::save_window(&app, &window)
}

#[tauri::command(async)]
pub fn save_font(app: AppHandle, font: config::Font) -> Result<(), String> {
    let mut font = font;
    if let Some(size) = font.size {
        font.size = Some(size.clamp(10.0, 24.0));
    }
    config::save_font(&app, &font)
}

#[tauri::command(async)]
pub fn load_user_themes() -> Vec<config::UserTheme> {
    config::load_user_themes()
}
