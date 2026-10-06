use std::fs;
use std::path::Path;
use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::AppHandle;

use super::models_file::{self, ModelsFile};
use super::{Advanced, Settings, Startup, WindowConfig};
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

pub fn set_theme(app: &AppHandle, theme: &str) -> Result<(), String> {
    patch(app, |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("theme".to_string(), json!(theme));
        }
    })
}

pub fn set_style(app: &AppHandle, style: &str) -> Result<(), String> {
    patch(app, |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("style".to_string(), json!(style));
        }
    })
}

pub fn save_advanced(app: &AppHandle, advanced: &Advanced) -> Result<(), String> {
    let entry = serde_json::to_value(advanced).map_err(|error| error.to_string())?;
    patch(app, move |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("advanced".to_string(), entry);
        }
    })
}

pub fn set_startup(app: &AppHandle, startup: Startup) -> Result<(), String> {
    let entry = serde_json::to_value(startup).map_err(|error| error.to_string())?;
    patch(app, move |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("startup".to_string(), entry);
        }
    })
}

pub fn set_resume_within(app: &AppHandle, minutes: u64) -> Result<(), String> {
    patch(app, move |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("resumeWithinMinutes".to_string(), json!(minutes));
        }
    })
}

pub fn set_paste_as_file(app: &AppHandle, chars: u64) -> Result<(), String> {
    patch(app, move |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("pasteAsFileChars".to_string(), json!(chars));
        }
    })
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
    let entry = serde_json::to_value(window).map_err(|error| error.to_string())?;
    patch(app, move |value| {
        if let Some(object) = value.as_object_mut() {
            object.insert("window".to_string(), entry);
        }
    })
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
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("slopchat-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn migrates_models_out_of_settings() {
        let dir = temp_dir("migrate");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(
            &settings_path,
            r#"{"theme":"default-dark","models":[{"model":"openrouter/a"}],"profiles":[{"name":"x","model":"openrouter/a"}]}"#,
        )
        .unwrap();

        let settings = load_from(&settings_path, &models_path).unwrap();

        let file: ModelsFile =
            serde_json::from_str(&fs::read_to_string(&models_path).unwrap()).unwrap();
        assert_eq!(file.models[0].model, "openrouter/a");
        assert_eq!(file.profiles[0].name, "x");

        let stored: Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert!(stored.get("models").is_none());
        assert!(stored.get("profiles").is_none());
        assert_eq!(settings.models[0].model, "openrouter/a");
    }

    #[test]
    fn models_file_overrides_settings() {
        let dir = temp_dir("override");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(&settings_path, r#"{"models":[{"model":"openrouter/old"}]}"#).unwrap();
        fs::write(
            &models_path,
            r#"{"models":[{"model":"openrouter/new"}],"profiles":[]}"#,
        )
        .unwrap();

        let settings = load_from(&settings_path, &models_path).unwrap();

        assert_eq!(settings.models.len(), 1);
        assert_eq!(settings.models[0].model, "openrouter/new");
        // settings.json is left untouched when models.json is the source of truth.
        let stored: Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert!(stored.get("models").is_some());
    }

    #[test]
    fn malformed_models_file_errors() {
        let dir = temp_dir("malformed");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(&settings_path, "{}").unwrap();
        fs::write(&models_path, "{not json").unwrap();

        assert!(load_from(&settings_path, &models_path).is_err());
    }

    #[test]
    fn toggles_create_missing_sections() {
        let mut value = json!({});
        set_search_flag(&mut value, "readerFallback", true);
        assert_eq!(value["search"]["readerFallback"], json!(true));

        let mut value = json!({});
        set_debug_flag(&mut value, "agentLog", true);
        assert_eq!(value["debug"]["agentLog"], json!(true));
    }
}
