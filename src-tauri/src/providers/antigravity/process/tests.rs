    use super::*;

    fn options(conversation: Option<&str>) -> ExecOptions {
        ExecOptions {
            model: "gemini-3.8-flash-medium".to_string(),
            conversation: conversation.map(str::to_string),
            prompt: "hi".to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    fn result(conversation: &str, status: &str, error: Option<&str>) -> AgyResult {
        AgyResult {
            conversation_id: conversation.to_string(),
            status: status.to_string(),
            response: String::new(),
            error: error.map(str::to_string),
            input_tokens: 0,
            output_tokens: 0,
            thinking_tokens: 0,
        }
    }

    #[test]
    fn a_different_conversation_id_means_the_resume_was_rejected() {
        let opts = options(Some("wanted"));
        let got = result("other", "SUCCESS", None);
        assert!(matches!(
            classify(&opts, Some(got), ""),
            Err(AgyError::UnknownConversation)
        ));
    }

    #[test]
    fn a_matching_conversation_id_resumes() {
        let opts = options(Some("wanted"));
        let got = result("wanted", "SUCCESS", None);
        assert!(classify(&opts, Some(got), "").is_ok());
    }

    #[test]
    fn a_fresh_run_accepts_any_conversation_id() {
        let opts = options(None);
        let got = result("brand-new", "SUCCESS", None);
        assert!(classify(&opts, Some(got), "").is_ok());
    }

    #[test]
    fn an_unknown_model_is_classified() {
        let opts = options(None);
        let got = result("", "ERROR", Some("invalid model selection (--model \"nope\")"));
        assert!(matches!(
            classify(&opts, Some(got), ""),
            Err(AgyError::UnknownModel(_))
        ));
    }

    #[test]
    fn authentication_errors_are_detected_from_stderr() {
        let opts = options(None);
        assert!(matches!(
            classify(&opts, None, "Error: authentication required. Run 'agy' to log in."),
            Err(AgyError::NotSignedIn)
        ));
    }

    #[test]
    fn a_missing_result_uses_the_last_stderr_line() {
        let opts = options(None);
        assert!(matches!(
            classify(&opts, None, "line one\nline two\n"),
            Err(AgyError::Failed(message)) if message == "line two"
        ));
    }
