use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Position, Size, WebviewUrl, WebviewWindowBuilder,
};

use super::{DEFAULT_COMPACT_WIDTH, MIN_HEIGHT};

const LABEL: &str = "main-panel";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("SlopChat")
        .inner_size(DEFAULT_COMPACT_WIDTH, MIN_HEIGHT)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .visible(false)
        .build()?;

    Ok(())
}

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        if !super::is_expanded(app) {
            let _ = window.set_position(super::compact_position(app));
        }
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("panel://shown", ());
    }
}

pub fn hide(app: &AppHandle, _restore: bool) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn set_window_style(app: &AppHandle, scheme: &str, _vibrancy: &str) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let theme = if scheme == "light" {
            tauri::Theme::Light
        } else {
            tauri::Theme::Dark
        };
        let _ = window.set_theme(Some(theme));
    }
}

pub fn resize(app: &AppHandle, position: Position, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.set_position(position);
        let _ = window.set_size(Size::Logical(LogicalSize::new(width, height)));
    }
}
