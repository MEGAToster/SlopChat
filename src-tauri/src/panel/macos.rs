use std::sync::atomic::{AtomicI32, Ordering};

use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, WebviewUrl,
};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

use super::{DEFAULT_COMPACT_WIDTH, MIN_HEIGHT};

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
        .position(compact_position(app))
        .size(Size::Logical(LogicalSize::new(
            DEFAULT_COMPACT_WIDTH,
            MIN_HEIGHT,
        )))
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
    handler.window_did_resign_key(move |_| hide(&handle, false));
    panel.set_event_handler(Some(handler.as_ref()));

    Ok(())
}

pub fn show(app: &AppHandle) {
    let Ok(panel) = app.get_webview_panel(LABEL) else {
        return;
    };

    if !super::is_expanded(app) {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.set_position(compact_position(app));
        }
    }

    remember_frontmost();
    panel.show_and_make_key();
    let _ = app.emit("panel://shown", ());
}

pub fn hide(app: &AppHandle, restore: bool) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        panel.hide();
    }
    if restore {
        restore_frontmost();
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
    app.get_webview_panel(LABEL)
        .map(|panel| panel.is_visible())
        .unwrap_or(false)
}

pub fn set_window_style(app: &AppHandle, scheme: &str, vibrancy: &str, _opacity: f64) {
    use tauri::window::{Effect, EffectState, EffectsBuilder};

    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    let effect = match vibrancy {
        "hud" => Some(Effect::HudWindow),
        "popover" => Some(Effect::Popover),
        "sidebar" => Some(Effect::Sidebar),
        _ => None,
    };

    let config = match effect {
        Some(effect) => EffectsBuilder::new()
            .effect(effect)
            .state(EffectState::Active)
            .radius(12.0)
            .build(),
        None => EffectsBuilder::new().build(),
    };
    let _ = window.set_effects(config);

    let theme = if scheme == "light" {
        tauri::Theme::Light
    } else {
        tauri::Theme::Dark
    };
    let _ = window.set_theme(Some(theme));
}

pub fn resize(app: &AppHandle, position: Position, width: f64, height: f64) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.set_position(position);
        }
        panel.set_content_size(width, height);
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
