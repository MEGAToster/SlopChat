use tauri::AppHandle;

use crate::panel;

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
pub fn toggle_sticky(app: AppHandle) -> bool {
    panel::toggle_sticky(&app)
}

#[tauri::command]
pub fn is_sticky(app: AppHandle) -> bool {
    panel::is_sticky(&app)
}
