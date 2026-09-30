use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tauri::AppHandle;

use super::Settings;
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

pub fn add_model(app: &AppHandle, model: &str) -> Result<(), String> {
    patch(app, |value| {
        let Some(object) = value.as_object_mut() else {
            return;
        };
        let models = object
            .entry("models")
            .or_insert_with(|| json!([]))
            .as_array_mut();
        if let Some(models) = models {
            if !models.iter().any(|entry| entry.as_str() == Some(model)) {
                models.push(json!(model));
            }
        }
    })
}

pub fn remove_model(app: &AppHandle, model: &str) -> Result<(), String> {
    patch(app, |value| {
        if let Some(models) = value.get_mut("models").and_then(Value::as_array_mut) {
            models.retain(|entry| entry.as_str() != Some(model));
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
