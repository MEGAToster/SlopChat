use std::fs;
use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use super::models_file::{self, ModelsFile};
use super::{Advanced, Font, Settings, Startup, WindowConfig};
use crate::paths;

// Read-modify-write on a single file must not interleave between async commands.
static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn load() -> Result<Settings, String> {
    load_from(&paths::settings_path(), &paths::models_path())
}

fn load_from(settings_path: &Path, models_path: &Path) -> Result<Settings, String> {
    let mut settings = if settings_path.exists() {
        let text = fs::read_to_string(settings_path)
            .map_err(|error| format!("reading {}: {error}", settings_path.display()))?;
        serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))?
    } else {
        Settings::default()
    };

    match models_file::read_at(models_path)? {
        Some(file) => {
            settings.models = file.models;
            settings.profiles = file.profiles;
        }
        None => {
            // One-time: models and profiles move out of settings.json.
            models_file::write_at(&snapshot(&settings), models_path)?;
            strip_models(settings_path)?;
        }
    }

    super::defaults::ensure_builtin_providers(&mut settings);

    // Register models that only existed as full profiles.
    if super::migrate_profiles(&mut settings) {
        models_file::write_at(&snapshot(&settings), models_path)?;
    }

    Ok(settings)
}

fn snapshot(settings: &Settings) -> ModelsFile {
    ModelsFile {
        models: settings.models.clone(),
        profiles: settings.profiles.clone(),
    }
}

pub fn ensure_file() -> Result<(), String> {
    load().map(|_| ())
}

// Removes the migrated keys, creating settings.json if needed.
fn strip_models(path: &Path) -> Result<(), String> {
    let mut value = if path.exists() {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("reading {}: {error}", path.display()))?;
        serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))?
    } else {
        serde_json::to_value(Settings::default()).map_err(|error| error.to_string())?
    };

    if let Some(object) = value.as_object_mut() {
        object.remove("models");
        object.remove("profiles");
    }

    let json = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    write_atomic(path, &json)
}

fn patch(app: &AppHandle, update: impl FnOnce(&mut Value)) -> Result<(), String> {
    patch_file(app, &paths::settings_path(), update)
}

pub(super) fn patch_file(
    app: &AppHandle,
    path: &Path,
    update: impl FnOnce(&mut Value),
) -> Result<(), String> {
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let text =
        fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let mut value: Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;

    update(&mut value);

    let json = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    write_atomic(path, &json)?;
    super::reload(app);
    Ok(())
}

// Most writers just drop one serialized value under a settings key.
fn set_entry(app: &AppHandle, key: &str, entry: Value) -> Result<(), String> {
    let key = key.to_string();
    patch(app, move |root| {
        if let Some(object) = root.as_object_mut() {
            object.insert(key, entry);
        }
    })
}

fn set_scalar(app: &AppHandle, key: &str, value: impl Serialize) -> Result<(), String> {
    let entry = serde_json::to_value(value).map_err(|error| error.to_string())?;
    set_entry(app, key, entry)
}

pub fn set_theme(app: &AppHandle, theme: &str) -> Result<(), String> {
    set_scalar(app, "theme", theme)
}

pub fn set_style(app: &AppHandle, style: &str) -> Result<(), String> {
    set_scalar(app, "style", style)
}

pub fn set_reading_width(app: &AppHandle, width: &str) -> Result<(), String> {
    set_scalar(app, "readingWidth", width)
}

pub fn save_advanced(app: &AppHandle, advanced: &Advanced) -> Result<(), String> {
    set_scalar(app, "advanced", advanced)
}

pub fn set_startup(app: &AppHandle, startup: Startup) -> Result<(), String> {
    set_scalar(app, "startup", startup)
}

pub fn set_resume_within(app: &AppHandle, minutes: u64) -> Result<(), String> {
    set_scalar(app, "resumeWithinMinutes", minutes)
}

pub fn set_paste_as_file(app: &AppHandle, chars: u64) -> Result<(), String> {
    set_scalar(app, "pasteAsFileChars", chars)
}

pub fn set_reader_fallback(app: &AppHandle, enabled: bool) -> Result<(), String> {
    patch(app, move |value| set_search_flag(value, "readerFallback", enabled))
}

pub fn set_searxng_base_url(app: &AppHandle, url: &str) -> Result<(), String> {
    let url = url.trim().to_string();
    patch(app, move |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        let search = object.entry("search").or_insert_with(|| json!({}));
        if let Some(search) = search.as_object_mut() {
            let searxng = search.entry("searxng").or_insert_with(|| json!({}));
            if let Some(searxng) = searxng.as_object_mut() {
                searxng.insert("baseUrl".to_string(), json!(url));
            }
        }
    })
}

// Creates the section when settings.json predates it, so a toggle always lands.
fn set_search_flag(value: &mut Value, key: &str, enabled: bool) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let search = object.entry("search").or_insert_with(|| json!({}));
    if let Some(search) = search.as_object_mut() {
        search.insert(key.to_string(), json!(enabled));
    }
}

pub fn save_window(app: &AppHandle, window: &WindowConfig) -> Result<(), String> {
    set_scalar(app, "window", window)
}

pub fn save_font(app: &AppHandle, font: &Font) -> Result<(), String> {
    let empty = font.ui.is_none() && font.mono.is_none() && font.size.is_none();
    if empty {
        patch(app, |value| {
            if let Some(object) = value.as_object_mut() {
                object.remove("font");
            }
        })
    } else {
        set_scalar(app, "font", font)
    }
}

pub fn set_agent_log(app: &AppHandle, enabled: bool) -> Result<(), String> {
    patch(app, move |value| set_debug_flag(value, "agentLog", enabled))
}

fn set_debug_flag(value: &mut Value, key: &str, enabled: bool) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let debug = object.entry("debug").or_insert_with(|| json!({}));
    if let Some(debug) = debug.as_object_mut() {
        debug.insert(key.to_string(), json!(enabled));
    }
}

pub(super) fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("creating {}: {error}", parent.display()))?;

    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no file name", path.display()))?;
    let temp = parent.join(format!("{}.tmp", name.to_string_lossy()));
    fs::write(&temp, contents).map_err(|error| format!("writing {}: {error}", temp.display()))?;
    fs::rename(&temp, path).map_err(|error| format!("renaming into {}: {error}", path.display()))?;
    Ok(())
}


#[cfg(test)]
mod tests;
