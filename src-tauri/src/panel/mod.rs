#[cfg(target_os = "macos")]
#[allow(clippy::unused_unit)]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
mod linux;
#[cfg(not(target_os = "macos"))]
pub use linux::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, LogicalPosition, Manager, Monitor, Position};

pub const MIN_HEIGHT: f64 = 88.0;
pub const DEFAULT_COMPACT_WIDTH: f64 = 680.0;

pub struct PanelState {
    expanded: AtomicBool,
    suppressed: AtomicBool,
    compact_height: Mutex<f64>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            expanded: AtomicBool::new(false),
            suppressed: AtomicBool::new(false),
            compact_height: Mutex::new(MIN_HEIGHT),
        }
    }
}

pub fn set_suppressed(app: &AppHandle, value: bool) {
    app.state::<PanelState>()
        .suppressed
        .store(value, Ordering::SeqCst);
}

// Only macOS hides on focus loss; the suppression flag is unused elsewhere.
#[cfg(target_os = "macos")]
pub fn is_suppressed(app: &AppHandle) -> bool {
    app.state::<PanelState>().suppressed.load(Ordering::SeqCst)
}

pub fn is_expanded(app: &AppHandle) -> bool {
    app.state::<PanelState>().expanded.load(Ordering::SeqCst)
}

pub fn resize_compact(app: &AppHandle, height: f64) {
    let state = app.state::<PanelState>();
    if state.expanded.load(Ordering::SeqCst) {
        return;
    }
    let height = height.clamp(MIN_HEIGHT, compact_height(app));
    *state.compact_height.lock().expect("panel lock poisoned") = height;
    resize(app, compact_position(app), compact_width(app), height);
}

pub fn set_expanded(app: &AppHandle, expanded: bool) {
    let state = app.state::<PanelState>();
    state.expanded.store(expanded, Ordering::SeqCst);

    if expanded {
        let ratio = app
            .state::<crate::config::ConfigState>()
            .get()
            .window
            .expanded_screen_ratio;
        let (area_x, area_y, area_w, area_h) = work_area(app);
        let width = area_w * ratio;
        let height = area_h * ratio;
        let position = Position::Logical(LogicalPosition::new(
            area_x + (area_w - width) / 2.0,
            area_y + (area_h - height) / 2.0,
        ));
        resize(app, position, width, height);
    } else {
        let height = *state.compact_height.lock().expect("panel lock poisoned");
        resize(
            app,
            compact_position(app),
            compact_width(app),
            height.clamp(MIN_HEIGHT, compact_height(app)),
        );
    }
}

pub fn toggle_expanded(app: &AppHandle) {
    set_expanded(app, !is_expanded(app));
}

pub fn toggle(app: &AppHandle) {
    if is_visible(app) {
        hide(app, true);
    } else {
        show(app);
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

pub fn compact_position(app: &AppHandle) -> Position {
    let Some(monitor) = active_monitor(app) else {
        return Position::Logical(LogicalPosition::new(0.0, 0.0));
    };

    let scale = monitor.scale_factor();
    let origin = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);

    Position::Logical(LogicalPosition::new(
        origin.x + (size.width - compact_width(app)) / 2.0,
        origin.y + size.height * 0.18,
    ))
}

fn active_monitor(app: &AppHandle) -> Option<Monitor> {
    app.cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())
}

fn compact_width(app: &AppHandle) -> f64 {
    app.state::<crate::config::ConfigState>()
        .get()
        .window
        .compact_width
        .clamp(400.0, 1200.0)
}

// Max height for a filled chat; the empty chat still sizes to its content.
fn compact_height(app: &AppHandle) -> f64 {
    app.state::<crate::config::ConfigState>()
        .get()
        .window
        .compact_height
        .clamp(400.0, 1200.0)
}

#[derive(Default)]
pub struct Sticky(AtomicBool);

pub fn toggle_sticky(app: &AppHandle) -> bool {
    let sticky = app.state::<Sticky>();
    let value = !sticky.0.load(Ordering::Relaxed);
    sticky.0.store(value, Ordering::Relaxed);
    let _ = app.emit("panel://sticky", value);
    value
}

pub fn is_sticky(app: &AppHandle) -> bool {
    app.state::<Sticky>().0.load(Ordering::Relaxed)
}
