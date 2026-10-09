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
    pub reading_width: ReadingWidth,
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearxngSettings {
    pub base_url: String,
    pub min_interval_ms: u64,
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
    pub zoom_in: Vec<String>,
    pub zoom_out: Vec<String>,
    pub zoom_reset: Vec<String>,
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
pub enum ReadingWidth {
    Narrow,
    Full,
    #[default]
    #[serde(other)]
    Wide,
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
mod tests;
