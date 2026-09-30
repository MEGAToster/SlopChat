use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "main-panel";
const COMPACT_WIDTH: f64 = 680.0;
const COMPACT_HEIGHT: f64 = 160.0;
const MAX_HEIGHT: f64 = 480.0;

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("SlopChat")
        .inner_size(COMPACT_WIDTH, COMPACT_HEIGHT)
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
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("panel://shown", ());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
    }
}

pub fn toggle(app: &AppHandle) {
    if is_visible(app) {
        hide(app);
    } else {
        show(app);
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn resize(app: &AppHandle, height: f64) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.set_size(tauri::LogicalSize::new(
            COMPACT_WIDTH,
            height.clamp(COMPACT_HEIGHT, MAX_HEIGHT),
        ));
    }
}
