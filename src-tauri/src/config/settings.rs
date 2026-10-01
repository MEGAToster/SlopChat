use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::defaults;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub keybinds: Keybinds,
    pub providers: BTreeMap<String, Provider>,
    pub profiles: Vec<Profile>,
    #[serde(default = "defaults::default_models", deserialize_with = "normalize_models")]
    pub models: Vec<ModelConfig>,
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
    #[serde(default, deserialize_with = "normalize_stop")]
    pub stop: Vec<String>,
    pub scroll_up: Vec<String>,
    pub scroll_down: Vec<String>,
    pub web_toggle: Vec<String>,
    pub reasoning: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

// A registered model. This is the single source of truth for settings.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ModelConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default = "defaults::default_true")]
    pub use_context: bool,
    #[serde(deserialize_with = "normalize_tools")]
    pub tools: Vec<String>,
    #[serde(default = "defaults::default_true")]
    pub tool_calling: bool,
    pub web_search: WebSearchMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

// Per-profile overrides; `None` inherits from the pointed registered model.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProfileOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_context: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "normalize_tools_opt"
    )]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<WebSearchMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

// A named pointer to a registered model, plus optional overrides.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(flatten)]
    pub overrides: ProfileOverrides,
}

// A config may express a model only as a full profile; register it so profiles
// are pointers, keeping overrides when the model is already registered.
pub(crate) fn migrate_profiles(settings: &mut Settings) -> bool {
    let Settings {
        profiles, models, ..
    } = settings;
    let mut changed = false;
    for profile in profiles.iter_mut() {
        if models.iter().any(|entry| entry.model == profile.model) {
            continue;
        }
        let overrides = &mut profile.overrides;
        models.push(ModelConfig {
            model: profile.model.clone(),
            system: overrides.system.take(),
            temperature: overrides.temperature.take(),
            use_context: overrides.use_context.take().unwrap_or(true),
            tools: overrides.tools.take().unwrap_or_default(),
            tool_calling: overrides.tool_calling.take().unwrap_or(true),
            web_search: overrides.web_search.take().unwrap_or_default(),
            reasoning: overrides.reasoning.take(),
        });
        changed = true;
    }
    changed
}

// Accept plain-string entries in `models`.
fn normalize_models<'de, D>(deserializer: D) -> Result<Vec<ModelConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Vec::<serde_json::Value>::deserialize(deserializer)?;
    raw.into_iter()
        .map(|value| match value {
            serde_json::Value::String(model) => Ok(ModelConfig {
                model,
                ..ModelConfig::default()
            }),
            other => serde_json::from_value(other).map_err(serde::de::Error::custom),
        })
        .collect()
}

// Accept Ctrl+C for the stop shortcut.
fn normalize_stop<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let keys = Vec::<String>::deserialize(deserializer)?;
    if keys.len() == 1 && keys[0] == "Ctrl+C" {
        return Ok(vec!["Ctrl+Q".to_string()]);
    }
    Ok(keys)
}

// Accept `fetch_url` as an alias for `web_fetch`.
fn normalize_tool(tool: String) -> String {
    if tool == "fetch_url" {
        "web_fetch".to_string()
    } else {
        tool
    }
}

fn normalize_tools<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let tools = Vec::<String>::deserialize(deserializer)?;
    Ok(tools.into_iter().map(normalize_tool).collect())
}

fn normalize_tools_opt<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let tools = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(tools.map(|list| list.into_iter().map(normalize_tool).collect()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_yields_documented_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.keybinds.toggle, vec!["Alt+Space".to_string()]);
        assert_eq!(settings.keybinds.stop, vec!["Ctrl+Q".to_string()]);
        assert_eq!(settings.keybinds.scroll_up, vec!["Ctrl+U".to_string()]);
        assert!(settings.providers.contains_key("openrouter"));
        assert_eq!(settings.profiles.len(), 4);
        assert_eq!(settings.models.len(), 4);
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
    fn old_full_profiles_migrate_to_registered_models() {
        let mut settings: Settings = serde_json::from_str(
            r#"{"profiles":[{"name":"x","model":"openrouter/a","system":"be terse","tools":["fetch_url"],"useContext":false}]}"#,
        )
        .unwrap();
        assert!(!settings.models.iter().any(|entry| entry.model == "openrouter/a"));

        assert!(migrate_profiles(&mut settings));

        let model = settings
            .models
            .iter()
            .find(|entry| entry.model == "openrouter/a")
            .unwrap();
        assert_eq!(model.system.as_deref(), Some("be terse"));
        assert_eq!(model.tools, vec!["web_fetch".to_string()]);
        assert!(!model.use_context);
        assert_eq!(settings.profiles[0].model, "openrouter/a");
        assert!(settings.profiles[0].overrides.system.is_none());
    }

    #[test]
    fn migration_keeps_overrides_when_the_model_already_exists() {
        let mut settings: Settings = serde_json::from_str(
            r#"{"models":[{"model":"openrouter/a"}],"profiles":[{"name":"x","model":"openrouter/a","system":"be terse"}]}"#,
        )
        .unwrap();

        assert!(!migrate_profiles(&mut settings));
        assert_eq!(
            settings.profiles[0].overrides.system.as_deref(),
            Some("be terse")
        );
    }

    #[test]
    fn legacy_string_models_become_configs() {
        let settings: Settings =
            serde_json::from_str(r#"{"models":["openrouter/a","openrouter/b"]}"#).unwrap();
        assert_eq!(settings.models.len(), 2);
        assert_eq!(settings.models[0].model, "openrouter/a");
        assert!(settings.models[0].tool_calling);
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
    fn legacy_ctrl_c_stop_migrates_to_ctrl_q() {
        let settings: Settings =
            serde_json::from_str(r#"{"keybinds":{"stop":["Ctrl+C"]}}"#).unwrap();
        assert_eq!(settings.keybinds.stop, vec!["Ctrl+Q".to_string()]);
    }

    #[test]
    fn custom_stop_binding_is_kept() {
        let settings: Settings =
            serde_json::from_str(r#"{"keybinds":{"stop":["Alt+X"]}}"#).unwrap();
        assert_eq!(settings.keybinds.stop, vec!["Alt+X".to_string()]);
    }

    #[test]
    fn legacy_fetch_url_tool_is_renamed() {
        let settings: Settings = serde_json::from_str(
            r#"{"profiles":[{"name":"x","model":"y","tools":["web_search","fetch_url"]}]}"#,
        )
        .unwrap();
        assert_eq!(
            settings.profiles[0].overrides.tools,
            Some(vec!["web_search".to_string(), "web_fetch".to_string()])
        );
    }

    #[test]
    fn raw_api_keys_are_rejected() {
        let settings: Settings =
            serde_json::from_str(r#"{"providers":{"openrouter":{"baseUrl":"x","apiKey":"sk-123"}}}"#)
                .unwrap();
        assert!(crate::providers::api_key(&settings, "openrouter").is_err());
    }
}
