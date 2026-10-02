use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tauri::AppHandle;

use super::models_file::{self, ModelsFile};
use super::{Advanced, ModelConfig, Profile, Settings, Startup, WindowConfig};
use crate::paths;

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

fn patch_models(app: &AppHandle, update: impl FnOnce(&mut Value)) -> Result<(), String> {
    if models_file::read()?.is_none() {
        models_file::write(&ModelsFile::default())?;
    }
    patch_file(app, &paths::models_path(), update)
}

fn patch_file(
    app: &AppHandle,
    path: &Path,
    update: impl FnOnce(&mut Value),
) -> Result<(), String> {
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

// Adds/updates a model entry. When `previous` is an old ref, profiles pointing
// at it follow to the new ref.
pub fn save_model(app: &AppHandle, config: &ModelConfig, previous: Option<&str>) -> Result<(), String> {
    let entry = serde_json::to_value(config).map_err(|error| error.to_string())?;
    let new_ref = config.model.clone();
    let renamed = previous
        .filter(|prev| *prev != new_ref)
        .map(str::to_string);

    patch_models(app, move |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };

        {
            let models = object
                .entry("models")
                .or_insert_with(|| json!([]))
                .as_array_mut();
            let Some(models) = models else { return };
            for item in models.iter_mut() {
                if let Some(id) = item.as_str() {
                    *item = json!({ "model": id });
                }
            }

            let existing = renamed.as_deref().unwrap_or(new_ref.as_str());
            match models
                .iter_mut()
                .find(|item| item.get("model").and_then(Value::as_str) == Some(existing))
            {
                Some(slot) => *slot = entry,
                None => models.push(entry),
            }
        }

        if let Some(previous) = renamed.as_deref() {
            if let Some(profiles) = object.get_mut("profiles").and_then(Value::as_array_mut) {
                for profile in profiles.iter_mut() {
                    let points_here = profile.get("model").and_then(Value::as_str) == Some(previous);
                    if points_here {
                        if let Some(map) = profile.as_object_mut() {
                            map.insert("model".to_string(), json!(new_ref));
                        }
                    }
                }
            }
        }
    })
}

pub fn remove_model(app: &AppHandle, model: &str) -> Result<(), String> {
    patch_models(app, |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        if let Some(models) = object.get_mut("models").and_then(Value::as_array_mut) {
            models.retain(|entry| entry.get("model").and_then(Value::as_str) != Some(model));
        }
        // Cascade: drop profiles pointing at the removed model.
        if let Some(profiles) = object.get_mut("profiles").and_then(Value::as_array_mut) {
            profiles.retain(|entry| entry.get("model").and_then(Value::as_str) != Some(model));
        }
    })
}

// Adds or updates a named profile in the rotation (matched by name).
pub fn save_profile(app: &AppHandle, profile: &Profile) -> Result<(), String> {
    let entry = serde_json::to_value(profile).map_err(|error| error.to_string())?;
    patch_models(app, move |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        let profiles = object
            .entry("profiles")
            .or_insert_with(|| json!([]))
            .as_array_mut();
        let Some(profiles) = profiles else { return };
        match profiles
            .iter_mut()
            .find(|item| item.get("name") == entry.get("name"))
        {
            Some(slot) => *slot = entry,
            None => profiles.push(entry),
        }
    })
}

pub fn remove_profile(app: &AppHandle, name: &str) -> Result<(), String> {
    patch_models(app, |value| {
        if let Some(profiles) = value.get_mut("profiles").and_then(Value::as_array_mut) {
            profiles.retain(|entry| entry.get("name").and_then(Value::as_str) != Some(name));
        }
    })
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

pub fn set_reader_fallback(app: &AppHandle, enabled: bool) -> Result<(), String> {
    patch(app, move |value| {
        if let Some(search) = value.get_mut("search").and_then(Value::as_object_mut) {
            search.insert("readerFallback".to_string(), json!(enabled));
        }
    })
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
    patch(app, move |value| {
        if let Some(debug) = value.get_mut("debug").and_then(Value::as_object_mut) {
            debug.insert("agentLog".to_string(), json!(enabled));
        }
    })
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
}
