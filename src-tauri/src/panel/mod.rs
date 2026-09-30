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

use tauri::{AppHandle, Emitter, Manager};

pub const COMPACT_WIDTH: f64 = 680.0;
pub const MIN_HEIGHT: f64 = 160.0;
pub const MAX_HEIGHT: f64 = 480.0;

pub struct PanelState {
    expanded: AtomicBool,
    compact_height: Mutex<f64>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            expanded: AtomicBool::new(false),
            compact_height: Mutex::new(MIN_HEIGHT),
        }
    }
}

pub fn is_expanded(app: &AppHandle) -> bool {
    app.state::<PanelState>().expanded.load(Ordering::SeqCst)
}

pub fn resize_compact(app: &AppHandle, height: f64) {
    let state = app.state::<PanelState>();
    if state.expanded.load(Ordering::SeqCst) {
        return;
    }
    let height = height.clamp(MIN_HEIGHT, MAX_HEIGHT);
    *state.compact_height.lock().expect("panel lock poisoned") = height;
    apply_layout(app, COMPACT_WIDTH, height, false);
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
        let (screen_width, screen_height) = screen_size(app);
        apply_layout(app, screen_width * ratio, screen_height * ratio, true);
    } else {
        let height = *state.compact_height.lock().expect("panel lock poisoned");
        apply_layout(app, COMPACT_WIDTH, height.clamp(MIN_HEIGHT, MAX_HEIGHT), false);
    }
}

pub fn toggle_expanded(app: &AppHandle) {
    set_expanded(app, !is_expanded(app));
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
