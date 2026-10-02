use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tauri::AppHandle;

use super::{Advanced, ModelConfig, Profile, Settings, Startup, WindowConfig};
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
    let mut settings: Settings =
        serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))?;

    // One-time: register models that only existed as full profiles.
    if super::migrate_profiles(&mut settings) {
        let json = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
        write_atomic(&path, &json)?;
    }

    Ok(settings)
}

pub fn ensure_file() -> Result<(), String> {
    load().map(|_| ())
}

fn patch(app: &AppHandle, update: impl FnOnce(&mut Value)) -> Result<(), String> {
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

// Adds/updates a model entry. When `previous` is an old ref, profiles pointing
// at it follow to the new ref.
pub fn save_model(app: &AppHandle, config: &ModelConfig, previous: Option<&str>) -> Result<(), String> {
    let entry = serde_json::to_value(config).map_err(|error| error.to_string())?;
    let new_ref = config.model.clone();
    let renamed = previous
        .filter(|prev| *prev != new_ref)
        .map(str::to_string);

    patch(app, move |value| {
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
    patch(app, |value| {
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
