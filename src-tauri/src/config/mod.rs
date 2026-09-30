mod keybinds;
mod watch;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::paths;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub keybinds: Keybinds,
    pub providers: BTreeMap<String, Provider>,
    pub profiles: Vec<Profile>,
    pub startup: Startup,
    pub resume_within_minutes: u64,
    pub theme: String,
    pub window: WindowConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Keybinds {
    pub toggle: Vec<String>,
    pub palette: Vec<String>,
    pub sidebar: Vec<String>,
    pub prev_chat: Vec<String>,
    pub next_chat: Vec<String>,
    pub cycle_profile: Vec<String>,
    pub sticky: Vec<String>,
    pub stop: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default = "default_true")]
    pub use_context: bool,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Startup {
    #[default]
    Last,
    New,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowConfig {
    pub compact_width: f64,
    pub expanded_screen_ratio: f64,
}

pub struct ConfigState {
    settings: RwLock<Settings>,
    error: Mutex<Option<String>>,
}

impl ConfigState {
    fn new(settings: Settings) -> Self {
        Self {
            settings: RwLock::new(settings),
            error: Mutex::new(None),
        }
    }

    pub fn get(&self) -> Settings {
        self.settings.read().expect("config lock poisoned").clone()
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().expect("config lock poisoned").clone()
    }

    fn set(&self, settings: Settings) {
        *self.settings.write().expect("config lock poisoned") = settings;
    }

    fn set_error(&self, error: Option<String>) {
        *self.error.lock().expect("config lock poisoned") = error;
    }
}

pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (settings, load_error) = match load() {
        Ok(settings) => (settings, None),
        Err(error) => (Settings::default(), Some(error)),
    };

    app.manage(ConfigState::new(settings.clone()));

    if let Some(error) = load_error {
        report_error(app, error);
    } else {
        apply_toggle(app, &settings.keybinds.toggle);
    }

    watch::start(app.clone())?;
    Ok(())
}

pub(crate) fn reload(app: &AppHandle) {
    let settings = match load() {
        Ok(settings) => settings,
        Err(error) => {
            report_error(app, error);
            return;
        }
    };

    let shortcuts = match keybinds::parse(&settings.keybinds.toggle) {
        Ok(shortcuts) => shortcuts,
        Err(error) => {
            report_error(app, error);
            return;
        }
    };

    let state = app.state::<ConfigState>();
    let toggle_changed = state.get().keybinds.toggle != settings.keybinds.toggle;

    if toggle_changed {
        if let Err(error) = keybinds::apply(app, &shortcuts) {
            report_error(app, error);
            return;
        }
    }

    state.set(settings.clone());
    state.set_error(None);
    let _ = app.emit("config://changed", &settings);
}

fn apply_toggle(app: &AppHandle, keys: &[String]) {
    let result = keybinds::parse(keys).and_then(|shortcuts| keybinds::apply(app, &shortcuts));
    if let Err(error) = result {
        report_error(app, error);
    }
}

fn report_error(app: &AppHandle, error: String) {
    let message = format!("Config not applied, still using previous settings — {error}");
    app.state::<ConfigState>().set_error(Some(message.clone()));
    let _ = app.emit("config://error", &message);
}

fn load() -> Result<Settings, String> {
    let path = paths::settings_path();

    if !path.exists() {
        let settings = Settings::default();
        write(&path, &settings)?;
        return Ok(settings);
    }

    let text =
        fs::read_to_string(&path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("settings.json: {error}"))
}

fn write(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("creating {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| format!("writing {}: {error}", path.display()))
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            keybinds: Keybinds::default(),
            providers: default_providers(),
            profiles: default_profiles(),
            startup: Startup::default(),
            resume_within_minutes: 10,
            theme: "default-dark".to_string(),
            window: WindowConfig::default(),
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            toggle: vec!["Alt+Space".to_string()],
            palette: vec!["Ctrl+P".to_string()],
            sidebar: vec!["Ctrl+H".to_string()],
            prev_chat: vec!["Ctrl+K".to_string()],
            next_chat: vec!["Ctrl+J".to_string()],
            cycle_profile: vec!["Shift+Tab".to_string()],
            sticky: vec!["Ctrl+Shift+P".to_string()],
            stop: vec!["Ctrl+D".to_string()],
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            compact_width: 680.0,
            expanded_screen_ratio: 0.7,
        }
    }
}

fn default_providers() -> BTreeMap<String, Provider> {
    let mut providers = BTreeMap::new();
    providers.insert(
        "openrouter".to_string(),
        Provider {
            base_url: "https://openrouter.ai/api/v1".to_string(),
            api_key: None,
        },
    );
    providers.insert(
        "ollama".to_string(),
        Provider {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
        },
    );
    providers.insert(
        "gemini".to_string(),
        Provider {
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".to_string(),
            api_key: None,
        },
    );
    providers
}

fn default_profiles() -> Vec<Profile> {
    vec![
        Profile {
            name: "default".to_string(),
            model: "openrouter/anthropic/claude-sonnet-5".to_string(),
            system: None,
            temperature: None,
            use_context: true,
        },
        Profile {
            name: "cheap".to_string(),
            model: "openrouter/deepseek/deepseek-chat:floor".to_string(),
            system: None,
            temperature: None,
            use_context: true,
        },
        Profile {
            name: "complex".to_string(),
            model: "openrouter/anthropic/claude-opus-5-5".to_string(),
            system: None,
            temperature: None,
            use_context: true,
        },
        Profile {
            name: "translate".to_string(),
            model: "openrouter/google/gemini-2.5-flash".to_string(),
            system: Some(
                "Translate the user's text. Detect the source language; translate to English unless another target is given."
                    .to_string(),
            ),
            temperature: None,
            use_context: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_yields_documented_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.keybinds.toggle, vec!["Alt+Space".to_string()]);
        assert_eq!(settings.keybinds.stop, vec!["Ctrl+D".to_string()]);
        assert!(settings.providers.contains_key("openrouter"));
        assert_eq!(settings.profiles.len(), 4);
        assert_eq!(settings.theme, "default-dark");
        assert_eq!(settings.window.compact_width, 680.0);
    }

    #[test]
    fn partial_keybinds_keep_other_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"keybinds":{"toggle":["Ctrl+Space"]}}"#).unwrap();
        assert_eq!(settings.keybinds.toggle, vec!["Ctrl+Space".to_string()]);
        assert_eq!(settings.keybinds.palette, Keybinds::default().palette);
        assert_eq!(settings.resume_within_minutes, 10);
    }

    #[test]
    fn profiles_default_to_using_context() {
        let settings: Settings =
            serde_json::from_str(r#"{"profiles":[{"name":"x","model":"y"}]}"#).unwrap();
        assert!(settings.profiles[0].use_context);
    }
}
