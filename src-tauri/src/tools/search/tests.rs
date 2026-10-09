    use super::quota::{classify, reason};
    use super::*;

    #[test]
    fn classifies_only_explicit_quota_failures_as_exhausted() {
        assert!(matches!(classify(402, ""), SearchError::Exhausted(_)));
        assert!(matches!(classify(429, ""), SearchError::Exhausted(_)));
        assert!(matches!(
            classify(400, "quota exceeded"),
            SearchError::Exhausted(_)
        ));
        // A bare "limit"/"plan" in the body must not lock the provider out.
        assert!(matches!(
            classify(400, "monthly plan limit reached"),
            SearchError::Failed(_)
        ));
        assert!(matches!(classify(401, ""), SearchError::Failed(_)));
        assert!(matches!(classify(500, "boom"), SearchError::Failed(_)));
    }

    #[test]
    fn extracts_a_reason_from_json_bodies() {
        assert_eq!(reason(r#"{"error":"plan limit reached"}"#).unwrap(), "plan limit reached");
        assert_eq!(reason("not json"), None);
    }

    #[test]
    fn searxng_is_keyless_but_needs_a_base_url() {
        assert!(!needs_key("searxng"));
        assert!(needs_key("exa"));

        let default = SearchSettings::default();
        assert_eq!(
            keyless_skip("searxng", &default).as_deref(),
            Some("searxng: no base URL")
        );

        let configured = SearchSettings {
            searxng: crate::config::SearxngSettings {
                base_url: "http://localhost:8080".to_string(),
            },
            ..SearchSettings::default()
        };
        assert!(keyless_skip("searxng", &configured).is_none());
    }
