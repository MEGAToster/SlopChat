use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn parse(keys: &[String]) -> Result<Vec<Shortcut>, String> {
    keys.iter()
        .map(|key| {
            key.parse::<Shortcut>()
                .map_err(|error| format!("invalid keybind {key:?}: {error}"))
        })
        .collect()
}

pub fn apply(app: &AppHandle, shortcuts: &[Shortcut]) -> Result<(), String> {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    for shortcut in shortcuts {
        manager
            .register(*shortcut)
            .map_err(|error| format!("registering keybind: {error}"))?;
    }

    Ok(())
}
