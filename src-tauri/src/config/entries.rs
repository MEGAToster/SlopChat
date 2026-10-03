use serde_json::{json, Value};
use tauri::AppHandle;

use super::models_file::{self, ModelsFile};
use super::write::patch_file;
use super::{ModelConfig, Profile};

pub(super) fn patch_models(app: &AppHandle, update: impl FnOnce(&mut Value)) -> Result<(), String> {
    if models_file::read()?.is_none() {
        models_file::write(&ModelsFile::default())?;
    }
    patch_file(app, &crate::paths::models_path(), update)
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

// Adds or updates a named profile in the profiles (matched by name). When
// `previous` is an old name, the old entry is removed so a rename doesn't
// leave two profiles behind.
pub fn save_profile(
    app: &AppHandle,
    profile: &Profile,
    previous: Option<&str>,
) -> Result<(), String> {
    let entry = serde_json::to_value(profile).map_err(|error| error.to_string())?;
    let renamed = previous
        .filter(|prev| *prev != profile.name)
        .map(str::to_string);
    patch_models(app, move |value| upsert_profile(value, entry, renamed.as_deref()))
}

fn upsert_profile(value: &mut Value, entry: Value, previous: Option<&str>) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let profiles = object.entry("profiles").or_insert_with(|| json!([]));
    let Some(profiles) = profiles.as_array_mut() else {
        return;
    };
    if let Some(previous) = previous {
        profiles.retain(|item| item.get("name").and_then(Value::as_str) != Some(previous));
    }
    match profiles
        .iter_mut()
        .find(|item| item.get("name") == entry.get("name"))
    {
        Some(slot) => *slot = entry,
        None => profiles.push(entry),
    }
}

pub fn remove_profile(app: &AppHandle, name: &str) -> Result<(), String> {
    patch_models(app, |value| {
        if let Some(profiles) = value.get_mut("profiles").and_then(Value::as_array_mut) {
            profiles.retain(|entry| entry.get("name").and_then(Value::as_str) != Some(name));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renaming_a_profile_replaces_the_old_entry() {
        let mut value = json!({ "profiles": [{ "name": "old", "model": "openrouter/a" }] });
        upsert_profile(
            &mut value,
            json!({ "name": "new", "model": "openrouter/a" }),
            Some("old"),
        );

        let profiles = value["profiles"].as_array().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0]["name"], "new");
    }
}
