use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, WebviewUrl,
    WebviewWindowBuilder,
};

const LABEL: &str = "main-panel";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("SlopChat")
        .inner_size(super::COMPACT_WIDTH, super::MIN_HEIGHT)
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
            let _ = window.set_position(compact_position(active_monitor(app).as_ref()));
        }
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

pub fn apply_layout(app: &AppHandle, width: f64, height: f64, centered: bool) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let monitor = active_monitor(app);
        let position = if centered {
            centered_position(monitor.as_ref(), width, height)
        } else {
            compact_position(monitor.as_ref())
        };
        let _ = window.set_position(position);
        let _ = window.set_size(Size::Logical(LogicalSize::new(width, height)));
    }
}

pub fn screen_size(app: &AppHandle) -> (f64, f64) {
    match active_monitor(app) {
        Some(monitor) => {
            let size = monitor.size().to_logical::<f64>(monitor.scale_factor());
            (size.width, size.height)
        }
        None => (1440.0, 900.0),
    }
}

fn active_monitor(app: &AppHandle) -> Option<Monitor> {
    app.cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
}

fn compact_position(monitor: Option<&Monitor>) -> Position {
    let Some(monitor) = monitor else {
        return Position::Logical(LogicalPosition::new(0.0, 0.0));
    };

    let scale = monitor.scale_factor();
    let origin = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);

    Position::Logical(LogicalPosition::new(
        origin.x + (size.width - super::COMPACT_WIDTH) / 2.0,
        origin.y + size.height * 0.18,
    ))
}

fn centered_position(monitor: Option<&Monitor>, width: f64, height: f64) -> Position {
    let Some(monitor) = monitor else {
        return Position::Logical(LogicalPosition::new(0.0, 0.0));
    };

    let scale = monitor.scale_factor();
    let origin = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);

    Position::Logical(LogicalPosition::new(
        origin.x + (size.width - width) / 2.0,
        origin.y + (size.height - height) / 2.0,
    ))
}
