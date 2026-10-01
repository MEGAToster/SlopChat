use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

static REGISTERED: Mutex<Vec<Shortcut>> = Mutex::new(Vec::new());

pub(super) fn parse(keys: &[String]) -> Result<Vec<Shortcut>, String> {
    keys.iter()
        .map(|key| {
            key.parse::<Shortcut>()
                .map_err(|error| format!("invalid keybind {key:?}: {error}"))
        })
        .collect()
}

pub(super) fn apply(app: &AppHandle, shortcuts: &[Shortcut]) -> Result<(), String> {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    for shortcut in shortcuts {
        if let Err(error) = manager.register(*shortcut) {
            restore(app, shortcuts);
            return Err(format!("registering keybind: {error}"));
        }
    }

    *REGISTERED.lock().expect("keybinds lock poisoned") = shortcuts.to_vec();
    Ok(())
}

fn restore(app: &AppHandle, attempted: &[Shortcut]) {
    let previous = REGISTERED.lock().expect("keybinds lock poisoned").clone();
    if previous == attempted {
        return;
    }
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();
    for shortcut in previous {
        let _ = manager.register(shortcut);
    }
}
