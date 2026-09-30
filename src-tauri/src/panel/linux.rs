use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, WebviewUrl,
    WebviewWindowBuilder,
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
            let _ = window.set_position(compact_position(app));
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

pub fn toggle(app: &AppHandle) {
    if is_visible(app) {
        hide(app, true);
    } else {
        show(app);
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn set_window_style(app: &AppHandle, scheme: &str, _vibrancy: &str, _opacity: f64) {
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

pub fn work_area(app: &AppHandle) -> (f64, f64, f64, f64) {
    match active_monitor(app) {
        Some(monitor) => {
            let scale = monitor.scale_factor();
            let area = monitor.work_area();
            let position = area.position.to_logical::<f64>(scale);
            let size = area.size.to_logical::<f64>(scale);
            (position.x, position.y, size.width, size.height)
        }
        None => (0.0, 0.0, 1440.0, 900.0),
    }
}

fn active_monitor(app: &AppHandle) -> Option<Monitor> {
    app.cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
}

pub fn compact_position(app: &AppHandle) -> Position {
    let Some(monitor) = active_monitor(app) else {
        return Position::Logical(LogicalPosition::new(0.0, 0.0));
    };

    let scale = monitor.scale_factor();
    let origin = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);

    Position::Logical(LogicalPosition::new(
        origin.x + (size.width - DEFAULT_COMPACT_WIDTH) / 2.0,
        origin.y + size.height * 0.18,
    ))
}
