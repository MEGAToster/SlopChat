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
    pub search: SearchSettings,
    pub window: WindowConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "RawSearchSettings")]
pub struct SearchSettings {
    pub providers: Vec<String>,
    pub num_results: u32,
    pub limits: BTreeMap<String, Limit>,
    pub exa: ExaSettings,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Limit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExaSettings {
    #[serde(rename = "type")]
    pub search_type: String,
    pub contents: String,
    pub max_characters: u32,
}

// Accepts both the current shape and the legacy flat Exa keys.
#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawSearchSettings {
    providers: Vec<String>,
    num_results: u32,
    limits: BTreeMap<String, Limit>,
    exa: Option<ExaSettings>,
    provider: Option<String>,
    monthly_budget: Option<f64>,
    #[serde(rename = "type")]
    search_type: Option<String>,
    contents: Option<String>,
    max_characters: Option<u32>,
}

impl Default for RawSearchSettings {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            num_results: 5,
            limits: BTreeMap::new(),
            exa: None,
            provider: None,
            monthly_budget: None,
            search_type: None,
            contents: None,
            max_characters: None,
        }
    }
}

impl From<RawSearchSettings> for SearchSettings {
    fn from(raw: RawSearchSettings) -> Self {
        let mut limits = defaults::default_search_limits();
        for (name, limit) in raw.limits {
            limits.insert(name, limit);
        }
        if let Some(budget) = raw.monthly_budget {
            limits.entry("exa".to_string()).or_default().monthly_usd = Some(budget);
        }

        let exa = raw.exa.unwrap_or_else(|| ExaSettings {
            search_type: raw.search_type.unwrap_or_else(|| "instant".to_string()),
            contents: raw.contents.unwrap_or_else(|| "highlights".to_string()),
            max_characters: raw.max_characters.unwrap_or(1500),
        });

        let providers = if raw.providers.is_empty() {
            defaults::default_search_providers()
        } else {
            raw.providers
        };

        Self {
            providers,
            num_results: raw.num_results,
            limits,
            exa,
        }
    }
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
    pub web_toggle: Vec<String>,
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
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub web_search: WebSearchMode,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchMode {
    #[default]
    Auto,
    Always,
    Off,
}

impl WebSearchMode {
    pub fn is_off(self) -> bool {
        self == WebSearchMode::Off
    }
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
    fn legacy_search_keys_map_onto_the_new_shape() {
        let settings: Settings = serde_json::from_str(
            r#"{"search":{"provider":"exa","monthlyBudget":3.5,"type":"fast","contents":"text","maxCharacters":900}}"#,
        )
        .unwrap();
        assert_eq!(settings.search.providers.len(), 4);
        assert_eq!(settings.search.providers[0], "exa");
        assert_eq!(settings.search.limits["exa"].monthly_usd, Some(3.5));
        assert_eq!(settings.search.limits["serper"].total, Some(2500));
        assert_eq!(settings.search.exa.search_type, "fast");
        assert_eq!(settings.search.exa.contents, "text");
        assert_eq!(settings.search.exa.max_characters, 900);
    }

    #[test]
    fn raw_api_keys_are_rejected() {
        let settings: Settings =
            serde_json::from_str(r#"{"providers":{"openrouter":{"baseUrl":"x","apiKey":"sk-123"}}}"#)
                .unwrap();
        assert!(crate::providers::api_key(&settings, "openrouter").is_err());
    }
}
