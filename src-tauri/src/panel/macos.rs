use std::sync::atomic::{AtomicI32, Ordering};

use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, WebviewUrl,
};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

use super::{COMPACT_WIDTH, MIN_HEIGHT};

const LABEL: &str = "main-panel";

static FRONTMOST_PID: AtomicI32 = AtomicI32::new(-1);

fn remember_frontmost() {
    use tauri_nspanel::objc2_app_kit::NSWorkspace;

    if let Some(application) = NSWorkspace::sharedWorkspace().frontmostApplication() {
        let pid = application.processIdentifier();
        if pid != std::process::id() as i32 {
            FRONTMOST_PID.store(pid, Ordering::SeqCst);
        }
    }
}

fn restore_frontmost() {
    use tauri_nspanel::objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

    let pid = FRONTMOST_PID.load(Ordering::SeqCst);
    if pid <= 0 {
        return;
    }
    if let Some(application) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
        let _ = application.activateWithOptions(NSApplicationActivationOptions::empty());
    }
}

tauri_panel! {
    panel!(SlopPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })

    panel_event!(SlopPanelEvents {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> ()
    })
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let panel = PanelBuilder::<_, SlopPanel>::new(app, LABEL)
        .url(WebviewUrl::App("index.html".into()))
        .title("SlopChat")
        .position(compact_position(None))
        .size(Size::Logical(LogicalSize::new(COMPACT_WIDTH, MIN_HEIGHT)))
        .resizable(true)
        .level(PanelLevel::Floating)
        .has_shadow(true)
        .corner_radius(12.0)
        .collection_behavior(
            CollectionBehavior::new()
                .can_join_all_spaces()
                .full_screen_auxiliary()
                .transient(),
        )
        .hides_on_deactivate(false)
        .works_when_modal(true)
        .with_window(|window| window.decorations(false).transparent(true))
        .add_style_mask(StyleMask::empty().nonactivating_panel())
        .no_activate(true)
        .build()?;

    let handler = SlopPanelEvents::new();
    let handle = app.clone();
    handler.window_did_resign_key(move |_| {
        if !super::is_sticky(&handle) {
            hide(&handle);
        }
    });
    panel.set_event_handler(Some(handler.as_ref()));

    Ok(())
}

pub fn show(app: &AppHandle) {
    let Ok(panel) = app.get_webview_panel(LABEL) else {
        return;
    };

    if !super::is_expanded(app) {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.set_position(compact_position(active_monitor(app).as_ref()));
        }
    }

    remember_frontmost();
    panel.show_and_make_key();
    let _ = app.emit("panel://shown", ());
}

pub fn hide(app: &AppHandle) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        panel.hide();
    }
    restore_frontmost();
}

pub fn toggle(app: &AppHandle) {
    if is_visible(app) {
        hide(app);
    } else {
        show(app);
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_panel(LABEL)
        .map(|panel| panel.is_visible())
        .unwrap_or(false)
}

pub fn apply_layout(app: &AppHandle, width: f64, height: f64, centered: bool) {
    let monitor = active_monitor(app);
    let position = if centered {
        centered_position(monitor.as_ref(), width, height)
    } else {
        compact_position(monitor.as_ref())
    };

    if let Ok(panel) = app.get_webview_panel(LABEL) {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.set_position(position);
        }
        panel.set_content_size(width, height);
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
        origin.x + (size.width - COMPACT_WIDTH) / 2.0,
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
