use std::fs;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tauri::AppHandle;

use super::reload;
use crate::paths;

const DEBOUNCE: Duration = Duration::from_millis(200);

pub fn start(app: AppHandle) -> notify::Result<()> {
    let dir = paths::config_dir();
    fs::create_dir_all(&dir)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;

    std::thread::spawn(move || {
        let _watcher = watcher;
        while let Ok(event) = rx.recv() {
            if !touches_settings(&event) {
                continue;
            }
            std::thread::sleep(DEBOUNCE);
            while rx.try_recv().is_ok() {}
            reload(&app);
        }
    });

    Ok(())
}

fn touches_settings(event: &notify::Result<notify::Event>) -> bool {
    matches!(event, Ok(event) if event.paths.iter().any(|path| is_settings_file(path)))
}

fn is_settings_file(path: &std::path::Path) -> bool {
    path.file_name().is_some_and(|name| name == "settings.json")
}
