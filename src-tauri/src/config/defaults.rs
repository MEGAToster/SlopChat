use std::collections::BTreeMap;

use super::{
    ExaSettings, Keybinds, Limit, Profile, Provider, SearchSettings, Settings, Startup,
    WebSearchMode, WindowConfig,
};

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            providers: default_search_providers(),
            num_results: 5,
            limits: default_search_limits(),
            exa: ExaSettings::default(),
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
            models: Vec::new(),
            startup: Startup::default(),
            resume_within_minutes: 10,
            theme: "default-dark".to_string(),
            style: "auto".to_string(),
            font: None,
            search: SearchSettings::default(),
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
            stop: vec!["Ctrl+C".to_string()],
            scroll_up: vec!["Ctrl+U".to_string()],
            scroll_down: vec!["Ctrl+D".to_string()],
            web_toggle: vec!["Ctrl+W".to_string()],
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
            color: None,
            tools: vec!["web_search".to_string(), "web_fetch".to_string()],
            tool_calling: true,
            web_search: WebSearchMode::Auto,
        },
        Profile {
            name: "cheap".to_string(),
            model: "openrouter/deepseek/deepseek-chat:floor".to_string(),
            system: None,
            temperature: None,
            use_context: true,
            color: None,
            tools: Vec::new(),
            tool_calling: false,
            web_search: WebSearchMode::Auto,
        },
        Profile {
            name: "complex".to_string(),
            model: "openrouter/anthropic/claude-opus-5-5".to_string(),
            system: None,
            temperature: None,
            use_context: true,
            color: None,
            tools: Vec::new(),
            tool_calling: true,
            web_search: WebSearchMode::Auto,
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
            color: None,
            tools: Vec::new(),
            tool_calling: false,
            web_search: WebSearchMode::Auto,
        },
    ]
}
