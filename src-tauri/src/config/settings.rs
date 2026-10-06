use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::defaults;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub keybinds: Keybinds,
    pub providers: BTreeMap<String, Provider>,
    pub profiles: Vec<Profile>,
    #[serde(default = "defaults::default_models", deserialize_with = "super::legacy::normalize_models")]
    pub models: Vec<ModelConfig>,
    pub startup: Startup,
    pub resume_within_minutes: u64,
    pub paste_as_file_chars: u64,
    pub theme: String,
    pub style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
    pub search: SearchSettings,
    pub advanced: Advanced,
    pub debug: DebugSettings,
    pub window: WindowConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Advanced {
    pub max_searches: u32,
    pub max_fetches: u32,
    pub max_rounds: u32,
    pub response_timeout_secs: u32,
    pub max_turn_secs: u32,
    pub max_fetched_chars: u64,
    pub reasoning_max_tokens: u32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub deep: DeepSettings,
}

// Budgets for a Deep research turn; they replace the normal advanced limits.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DeepSettings {
    pub max_rounds: u32,
    pub max_searches: u32,
    pub max_fetches: u32,
    pub max_fetched_chars: u64,
    pub max_turn_secs: u32,
    pub subagent_model: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DebugSettings {
    pub agent_log: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "super::legacy::RawSearchSettings")]
pub struct SearchSettings {
    pub providers: Vec<String>,
    pub num_results: u32,
    pub limits: BTreeMap<String, Limit>,
    pub exa: ExaSettings,
    pub searxng: SearxngSettings,
    pub reader_fallback: bool,
    pub snippet_chars: u32,
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

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearxngSettings {
    pub base_url: String,
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
    #[serde(default, deserialize_with = "super::legacy::normalize_stop")]
    pub stop: Vec<String>,
    pub scroll_up: Vec<String>,
    pub scroll_down: Vec<String>,
    pub web_toggle: Vec<String>,
    pub reasoning: Vec<String>,
    pub settings: Vec<String>,
    pub models: Vec<String>,
    pub follow: Vec<String>,
    pub focus_sidebar: Vec<String>,
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
    pub use_context: bool,
    #[serde(deserialize_with = "super::legacy::normalize_tools")]
    pub tools: Vec<String>,
    pub tool_calling: bool,
    pub web_search: WebSearchMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<Routing>,
}

// OpenRouter provider-routing hints; other providers ignore them.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Routing {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<MaxPrice>,
}

// USD per 1M tokens.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<f64>,
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
        deserialize_with = "super::legacy::normalize_tools_opt"
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchMode {
    #[default]
    #[serde(alias = "always")]
    Auto,
    Off,
    Deep,
}

impl WebSearchMode {
    pub fn is_off(self) -> bool {
        self == WebSearchMode::Off
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Startup {
    Pinned,
    #[default]
    #[serde(alias = "last")]
    Latest,
    New,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowConfig {
    pub compact_width: f64,
    pub compact_height: f64,
    pub expanded_screen_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::super::legacy::migrate_profiles;
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
    fn advanced_has_documented_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.advanced.max_searches, 5);
        assert_eq!(settings.advanced.max_fetches, 50);
        assert_eq!(settings.advanced.max_rounds, 5);
        assert_eq!(settings.advanced.deep.max_rounds, 15);
        assert_eq!(settings.advanced.deep.max_searches, 30);
        assert_eq!(settings.advanced.deep.max_turn_secs, 900);
        assert!(settings.advanced.deep.subagent_model.is_empty());
        assert_eq!(settings.startup, Startup::Latest);
        assert!(!settings.debug.agent_log);
    }

    #[test]
    fn web_modes_parse_always_as_auto_and_add_deep() {
        assert_eq!(
            serde_json::from_str::<WebSearchMode>("\"always\"").unwrap(),
            WebSearchMode::Auto
        );
        assert_eq!(
            serde_json::from_str::<WebSearchMode>("\"auto\"").unwrap(),
            WebSearchMode::Auto
        );
        assert_eq!(
            serde_json::from_str::<WebSearchMode>("\"deep\"").unwrap(),
            WebSearchMode::Deep
        );
        assert_eq!(
            serde_json::from_str::<WebSearchMode>("\"off\"").unwrap(),
            WebSearchMode::Off
        );
    }

    #[test]
    fn open_modes_parse_and_legacy_last_maps_to_latest() {
        let pinned: Settings = serde_json::from_str(r#"{"startup":"pinned"}"#).unwrap();
        assert_eq!(pinned.startup, Startup::Pinned);
        let latest: Settings = serde_json::from_str(r#"{"startup":"latest"}"#).unwrap();
        assert_eq!(latest.startup, Startup::Latest);
        let legacy: Settings = serde_json::from_str(r#"{"startup":"last"}"#).unwrap();
        assert_eq!(legacy.startup, Startup::Latest);
        let new: Settings = serde_json::from_str(r#"{"startup":"new"}"#).unwrap();
        assert_eq!(new.startup, Startup::New);
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
        assert_eq!(settings.search.providers.len(), 6);
        assert_eq!(settings.search.providers[0], "searxng");
        assert_eq!(settings.search.providers.last().unwrap(), "serper");
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
