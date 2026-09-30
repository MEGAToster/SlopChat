use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, WebviewUrl,
};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
};

const LABEL: &str = "main-panel";
const COMPACT_WIDTH: f64 = 680.0;
const COMPACT_HEIGHT: f64 = 160.0;
const MAX_HEIGHT: f64 = 480.0;

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
        .position(compact_position(app, None))
        .size(Size::Logical(LogicalSize::new(
            COMPACT_WIDTH,
            COMPACT_HEIGHT,
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

    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten());

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.set_position(compact_position(app, monitor.as_ref()));
    }

    panel.show_and_make_key();
    let _ = app.emit("panel://shown", ());
}

pub fn hide(app: &AppHandle) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        panel.hide();
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
    app.get_webview_panel(LABEL)
        .map(|panel| panel.is_visible())
        .unwrap_or(false)
}

pub fn resize(app: &AppHandle, height: f64) {
    if let Ok(panel) = app.get_webview_panel(LABEL) {
        panel.set_content_size(COMPACT_WIDTH, height.clamp(COMPACT_HEIGHT, MAX_HEIGHT));
    }
}

fn compact_position(app: &AppHandle, monitor: Option<&Monitor>) -> Position {
    let fallback = app.primary_monitor().ok().flatten();
    let monitor = monitor.or(fallback.as_ref());

    let (x, y) = match monitor {
        Some(monitor) => {
            let scale = monitor.scale_factor();
            let origin = monitor.position().to_logical::<f64>(scale);
            let size = monitor.size().to_logical::<f64>(scale);
            (
                origin.x + (size.width - COMPACT_WIDTH) / 2.0,
                origin.y + size.height * 0.18,
            )
        }
        None => (0.0, 0.0),
    };

    Position::Logical(LogicalPosition::new(x, y))
}
