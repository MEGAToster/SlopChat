use std::collections::BTreeMap;

use super::{
    Advanced, Debug, ExaSettings, Keybinds, Limit, ModelConfig, Profile, ProfileOverrides,
    Provider, SearchSettings, Settings, Startup, WebSearchMode, WindowConfig,
};

impl Default for Advanced {
    fn default() -> Self {
        Self {
            max_searches: 5,
            max_fetches: 50,
            max_rounds: 6,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            system: None,
            temperature: None,
            use_context: true,
            tools: Vec::new(),
            tool_calling: true,
            web_search: WebSearchMode::Auto,
            reasoning: None,
        }
    }
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            providers: default_search_providers(),
            num_results: 5,
            limits: default_search_limits(),
            exa: ExaSettings::default(),
            reader_fallback: true,
            snippet_chars: 1500,
        }
    }
}

impl Default for ExaSettings {
    fn default() -> Self {
        Self {
            search_type: "instant".to_string(),
            contents: "highlights".to_string(),
            max_characters: 1500,
        }
    }
}

pub(super) fn default_search_providers() -> Vec<String> {
    ["exa", "serper", "tavily", "brave"]
        .iter()
        .map(|name| name.to_string())
        .collect()
}

pub(super) fn default_search_limits() -> BTreeMap<String, Limit> {
    let mut limits = BTreeMap::new();
    limits.insert(
        "exa".to_string(),
        Limit {
            monthly_usd: Some(10.0),
            ..Limit::default()
        },
    );
    limits.insert(
        "serper".to_string(),
        Limit {
            total: Some(2500),
            ..Limit::default()
        },
    );
    limits.insert(
        "tavily".to_string(),
        Limit {
            monthly: Some(1000),
            ..Limit::default()
        },
    );
    limits.insert(
        "brave".to_string(),
        Limit {
            monthly_usd: Some(5.0),
            ..Limit::default()
        },
    );
    limits
}

pub(super) fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            keybinds: Keybinds::default(),
            providers: default_providers(),
            profiles: default_profiles(),
            models: default_models(),
            startup: Startup::default(),
            resume_within_minutes: 10,
            theme: "default-dark".to_string(),
            style: "auto".to_string(),
            font: None,
            search: SearchSettings::default(),
            advanced: Advanced::default(),
            debug: Debug::default(),
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
            stop: vec!["Ctrl+Q".to_string()],
            scroll_up: vec!["Ctrl+U".to_string()],
            scroll_down: vec!["Ctrl+D".to_string()],
            web_toggle: vec!["Ctrl+W".to_string()],
            reasoning: vec!["Ctrl+R".to_string()],
            settings: vec!["Ctrl+,".to_string()],
            models: vec!["Ctrl+M".to_string()],
            follow: vec!["Ctrl+L".to_string()],
            focus_sidebar: vec!["Ctrl+S".to_string()],
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

pub(super) fn default_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig {
            model: "openrouter/anthropic/claude-sonnet-5".to_string(),
            tools: vec!["web_search".to_string(), "web_fetch".to_string()],
            ..ModelConfig::default()
        },
        ModelConfig {
            model: "openrouter/deepseek/deepseek-chat:floor".to_string(),
            tool_calling: false,
            ..ModelConfig::default()
        },
        ModelConfig {
            model: "openrouter/anthropic/claude-opus-5-5".to_string(),
            ..ModelConfig::default()
        },
        ModelConfig {
            model: "openrouter/google/gemini-2.5-flash".to_string(),
            system: Some(
                "Translate the user's text. Detect the source language; translate to English unless another target is given."
                    .to_string(),
            ),
            use_context: false,
            tool_calling: false,
            ..ModelConfig::default()
        },
    ]
}

fn profile(name: &str, model: &str) -> Profile {
    Profile {
        name: name.to_string(),
        model: model.to_string(),
        fallback: None,
        color: None,
        overrides: ProfileOverrides::default(),
    }
}

fn default_profiles() -> Vec<Profile> {
    vec![
        profile("default", "openrouter/anthropic/claude-sonnet-5"),
        profile("cheap", "openrouter/deepseek/deepseek-chat:floor"),
        profile("complex", "openrouter/anthropic/claude-opus-5-5"),
        profile("translate", "openrouter/google/gemini-2.5-flash"),
    ]
}
