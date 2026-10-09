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
    fn unknown_reading_width_falls_back_to_wide() {
        let settings: Settings = serde_json::from_str(r#"{"readingWidth":"gigantic"}"#).unwrap();
        assert_eq!(settings.reading_width, ReadingWidth::Wide);
        let narrow: Settings = serde_json::from_str(r#"{"readingWidth":"narrow"}"#).unwrap();
        assert_eq!(narrow.reading_width, ReadingWidth::Narrow);
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
        assert_eq!(settings.search.searxng.min_interval_ms, 700);
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
