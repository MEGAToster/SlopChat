use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tauri::AppHandle;

use super::{ModelConfig, Profile, Settings};
use crate::paths;

pub(super) fn load() -> Result<Settings, String> {
    let path = paths::settings_path();

    if !path.exists() {
        let settings = Settings::default();
        write_atomic(&path, &serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?)?;
        return Ok(settings);
    }

    let text =
        fs::read_to_string(&path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))
}

pub fn ensure_file() -> Result<(), String> {
    load().map(|_| ())
}

pub fn patch(app: &AppHandle, update: impl FnOnce(&mut Value)) -> Result<(), String> {
    let path = paths::settings_path();
    let text =
        fs::read_to_string(&path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let mut value: Value =
        serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))?;

    update(&mut value);

    let json = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    write_atomic(&path, &json)?;
    super::reload(app);
    Ok(())
}

// Adds or updates a model entry (matched by its `model` reference).
pub fn save_model(app: &AppHandle, config: &ModelConfig) -> Result<(), String> {
    let entry = serde_json::to_value(config).map_err(|error| error.to_string())?;
    patch(app, move |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        let models = object
            .entry("models")
            .or_insert_with(|| json!([]))
            .as_array_mut();
        let Some(models) = models else { return };
        for item in models.iter_mut() {
            if item.is_string() {
                let id = item.as_str().unwrap_or_default().to_string();
                *item = json!({ "model": id });
            }
        }
        match models
            .iter_mut()
            .find(|item| item.get("model") == entry.get("model"))
        {
            Some(slot) => *slot = entry,
            None => models.push(entry),
        }
    })
}

pub fn remove_model(app: &AppHandle, model: &str) -> Result<(), String> {
    patch(app, |value| {
        if let Some(models) = value.get_mut("models").and_then(Value::as_array_mut) {
            models.retain(|entry| entry.get("model").and_then(Value::as_str) != Some(model));
        }
    })
}

// Adds or updates a named profile in the rotation (matched by name).
pub fn save_profile(app: &AppHandle, profile: &Profile) -> Result<(), String> {
    let entry = serde_json::to_value(profile).map_err(|error| error.to_string())?;
    patch(app, move |value| {
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
    patch(app, |value| {
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

fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("creating {}: {error}", parent.display()))?;

    let temp = parent.join("settings.json.tmp");
    fs::write(&temp, contents).map_err(|error| format!("writing {}: {error}", temp.display()))?;
    fs::rename(&temp, path).map_err(|error| format!("renaming into {}: {error}", path.display()))?;
    Ok(())
}
