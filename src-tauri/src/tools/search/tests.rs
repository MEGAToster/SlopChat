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
    fn searxng_needs_a_base_url() {
        let default = SearchSettings::default();
        assert_eq!(
            endpoint_skip("searxng", &default).as_deref(),
            Some("searxng: no base URL")
        );
        assert!(endpoint_skip("exa", &default).is_none());

        let configured = SearchSettings {
            searxng: crate::config::SearxngSettings {
                base_url: "http://localhost:8080".to_string(),
                ..Default::default()
            },
            ..SearchSettings::default()
        };
        assert!(endpoint_skip("searxng", &configured).is_none());
    }

    #[test]
    fn probe_skips_until_a_base_url_is_set() {
        assert!(!probe_configured(&SearchSettings::default()));

        let configured = SearchSettings {
            searxng: crate::config::SearxngSettings {
                base_url: "http://localhost:8080".to_string(),
                ..Default::default()
            },
            ..SearchSettings::default()
        };
        assert!(probe_configured(&configured));
    }
