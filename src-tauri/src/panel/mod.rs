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

use tauri::{AppHandle, Emitter, Manager};

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
