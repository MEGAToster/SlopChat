use tauri::AppHandle;

use crate::panel;

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
pub fn set_suppress_hide(app: AppHandle, value: bool) {
    panel::set_suppressed(&app, value);
}

#[tauri::command(async)]
pub fn read_clipboard() -> Result<crate::clipboard::ClipboardContent, String> {
    crate::clipboard::read()
}

#[tauri::command(async)]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")) {
        return Err(format!("refusing to open unsupported URL: {url}"));
    }
    crate::platform::open_path(std::path::Path::new(&url))
}
