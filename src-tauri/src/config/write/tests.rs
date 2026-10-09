    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("slopchat-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn migrates_models_out_of_settings() {
        let dir = temp_dir("migrate");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(
            &settings_path,
            r#"{"theme":"default-dark","models":[{"model":"openrouter/a"}],"profiles":[{"name":"x","model":"openrouter/a"}]}"#,
        )
        .unwrap();

        let settings = load_from(&settings_path, &models_path).unwrap();

        let file: ModelsFile =
            serde_json::from_str(&fs::read_to_string(&models_path).unwrap()).unwrap();
        assert_eq!(file.models[0].model, "openrouter/a");
        assert_eq!(file.profiles[0].name, "x");

        let stored: Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert!(stored.get("models").is_none());
        assert!(stored.get("profiles").is_none());
        assert_eq!(settings.models[0].model, "openrouter/a");
    }

    #[test]
    fn models_file_overrides_settings() {
        let dir = temp_dir("override");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(&settings_path, r#"{"models":[{"model":"openrouter/old"}]}"#).unwrap();
        fs::write(
            &models_path,
            r#"{"models":[{"model":"openrouter/new"}],"profiles":[]}"#,
        )
        .unwrap();

        let settings = load_from(&settings_path, &models_path).unwrap();

        assert_eq!(settings.models.len(), 1);
        assert_eq!(settings.models[0].model, "openrouter/new");
        // settings.json is left untouched when models.json is the source of truth.
        let stored: Value =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert!(stored.get("models").is_some());
    }

    #[test]
    fn malformed_models_file_errors() {
        let dir = temp_dir("malformed");
        let settings_path = dir.join("settings.json");
        let models_path = dir.join("models.json");
        fs::write(&settings_path, "{}").unwrap();
        fs::write(&models_path, "{not json").unwrap();

        assert!(load_from(&settings_path, &models_path).is_err());
    }

    #[test]
    fn toggles_create_missing_sections() {
        let mut value = json!({});
        set_search_flag(&mut value, "readerFallback", true);
        assert_eq!(value["search"]["readerFallback"], json!(true));

        let mut value = json!({});
        set_debug_flag(&mut value, "agentLog", true);
        assert_eq!(value["debug"]["agentLog"], json!(true));
    }
