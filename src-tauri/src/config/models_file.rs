use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::write;
use super::{ModelConfig, Profile};

// The git-syncable half of the config: registered models and the rotation.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ModelsFile {
    pub models: Vec<ModelConfig>,
    pub profiles: Vec<Profile>,
}

pub(super) fn read() -> Result<Option<ModelsFile>, String> {
    read_at(&crate::paths::models_path())
}

pub(super) fn write(file: &ModelsFile) -> Result<(), String> {
    write_at(file, &crate::paths::models_path())
}

pub(super) fn read_at(path: &Path) -> Result<Option<ModelsFile>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text =
        fs::read_to_string(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let file: ModelsFile =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Some(file))
}

pub(super) fn write_at(file: &ModelsFile, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(file).map_err(|error| error.to_string())?;
    write::write_atomic(path, &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_models_and_profiles() {
        let file = ModelsFile {
            models: vec![ModelConfig {
                model: "openrouter/a".to_string(),
                ..ModelConfig::default()
            }],
            profiles: Vec::new(),
        };
        let json = serde_json::to_string(&file).unwrap();
        let parsed: ModelsFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.models[0].model, "openrouter/a");
    }

    #[test]
    fn empty_object_yields_empty_lists() {
        let file: ModelsFile = serde_json::from_str("{}").unwrap();
        assert!(file.models.is_empty());
        assert!(file.profiles.is_empty());
    }
}
