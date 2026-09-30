mod defaults;
mod keybinds;
mod themes;
mod watch;
mod write;

pub use themes::UserTheme;

use std::collections::BTreeMap;
use std::sync::{Mutex, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub keybinds: Keybinds,
    pub providers: BTreeMap<String, Provider>,
    pub profiles: Vec<Profile>,
    pub models: Vec<String>,
    pub startup: Startup,
    pub resume_within_minutes: u64,
    pub theme: String,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
    pub window: WindowConfig,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Font {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mono: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
}

fn default_style() -> String {
    "auto".to_string()
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
    pub scroll_up: Vec<String>,
    pub scroll_down: Vec<String>,
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
    #[serde(default = "defaults::default_true")]
    pub use_context: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
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
        self.settings
            .read()
            .expect("config lock poisoned")
            .clone()
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
    let (settings, load_error) = match write::load() {
        Ok(settings) => (settings, None),
        Err(error) => (Settings::default(), Some(error)),
    };

    app.manage(ConfigState::new(settings.clone()));

    if let Some(error) = load_error {
        report_error(app, error);
    }

    apply_toggle(app, &settings.keybinds.toggle);
    watch::start(app.clone())?;
    Ok(())
}

pub(crate) fn reload(app: &AppHandle) {
    let settings = match write::load() {
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

pub fn ensure_file() -> Result<(), String> {
    write::ensure_file()
}

pub fn add_model(app: &AppHandle, model: &str) -> Result<(), String> {
    write::add_model(app, model)
}

pub fn remove_model(app: &AppHandle, model: &str) -> Result<(), String> {
    write::remove_model(app, model)
}

pub fn set_theme(app: &AppHandle, theme: &str) -> Result<(), String> {
    write::set_theme(app, theme)
}

pub fn set_style(app: &AppHandle, style: &str) -> Result<(), String> {
    write::set_style(app, style)
}

pub fn load_user_themes() -> Vec<UserTheme> {
    themes::load_user()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_yields_documented_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.keybinds.toggle, vec!["Alt+Space".to_string()]);
        assert_eq!(settings.keybinds.stop, vec!["Ctrl+C".to_string()]);
        assert_eq!(settings.keybinds.scroll_up, vec!["Ctrl+U".to_string()]);
        assert!(settings.providers.contains_key("openrouter"));
        assert_eq!(settings.profiles.len(), 4);
        assert!(settings.models.is_empty());
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

    #[test]
    fn raw_api_keys_are_rejected() {
        let settings: Settings =
            serde_json::from_str(r#"{"providers":{"openrouter":{"baseUrl":"x","apiKey":"sk-123"}}}"#)
                .unwrap();
        assert!(crate::providers::api_key(&settings, "openrouter").is_err());
    }
}
