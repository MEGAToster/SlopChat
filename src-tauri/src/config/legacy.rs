use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use super::defaults;
use super::settings::{
    ExaSettings, Limit, ModelConfig, SearxngSettings, SearchSettings, Settings,
};

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct RawSearchSettings {
    providers: Vec<String>,
    num_results: u32,
    limits: BTreeMap<String, Limit>,
    exa: Option<ExaSettings>,
    searxng: Option<SearxngSettings>,
    reader_fallback: bool,
    snippet_chars: u32,
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
            searxng: None,
            reader_fallback: true,
            snippet_chars: 1500,
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
            searxng: raw.searxng.unwrap_or_default(),
            reader_fallback: raw.reader_fallback,
            snippet_chars: raw.snippet_chars,
        }
    }
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
            max_completion_tokens: ModelConfig::default().max_completion_tokens,
            routing: None,
        });
        changed = true;
    }
    changed
}

// Accept plain-string entries in `models`.
pub(super) fn normalize_models<'de, D>(deserializer: D) -> Result<Vec<ModelConfig>, D::Error>
where
    D: Deserializer<'de>,
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
pub(super) fn normalize_stop<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
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

pub(super) fn normalize_tools<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let tools = Vec::<String>::deserialize(deserializer)?;
    Ok(tools.into_iter().map(normalize_tool).collect())
}

pub(super) fn normalize_tools_opt<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let tools = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(tools.map(|list| list.into_iter().map(normalize_tool).collect()))
}
