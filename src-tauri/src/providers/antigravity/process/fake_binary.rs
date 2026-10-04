    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::*;
    use crate::providers::antigravity::{binary, exec};

    fn write_script(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("slopchat-agy-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("agy");
        fs::write(&path, body).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    fn fixture(name: &str) -> String {
        format!(
            "{}/src/providers/antigravity/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn options(prompt: &str) -> ExecOptions {
        ExecOptions {
            model: "gemini-3.8-flash-medium".to_string(),
            conversation: None,
            prompt: prompt.to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    #[tokio::test]
    async fn reads_a_full_fixture_into_events_and_a_result() {
        let script = write_script("cat", &format!("#!/bin/sh\ncat '{}'\n", fixture("tool.ndjson")));
        let mut events = Vec::new();
        let result = run(&script, &options("list"), &Cancel::new(), |event| {
            events.push(event.clone())
        })
        .await
        .unwrap();

        assert_eq!(result.response, "SECRET_MARKER_42\n");
        assert!(events.iter().any(
            |event| matches!(event, AgyEvent::ToolCall { name, .. } if name == "view_file")
        ));
    }

    #[tokio::test]
    async fn cancel_kills_the_child() {
        let pidfile = std::env::temp_dir().join(format!("slopchat-agy-pid-{}", std::process::id()));
        let _ = fs::remove_file(&pidfile);
        let script = write_script(
            "cancel",
            &format!("#!/bin/sh\necho $$ > '{}'\nexec sleep 30\n", pidfile.display()),
        );

        let cancel = Cancel::new();
        let stopper = cancel.clone();
        let watch = pidfile.clone();
        tokio::spawn(async move {
            // Cancel once the child has proven it started, not on a fixed timer.
            for _ in 0..200 {
                if watch.exists() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            stopper.cancel();
        });

        let result = run(&script, &options("hi"), &cancel, |_| {}).await;
        assert!(matches!(result, Err(AgyError::Cancelled)));

        let pid = fs::read_to_string(&pidfile).expect("the child wrote its pid");
        let alive = std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.trim())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        assert!(!alive, "agy child was still running");
    }

    #[tokio::test]
    async fn a_stalled_child_times_out() {
        let script = write_script("stall", "#!/bin/sh\nexec sleep 30\n");
        let mut opts = options("hi");
        opts.timeout = Duration::from_secs(1);
        let result = run(&script, &opts, &Cancel::new(), |_| {}).await;
        assert!(matches!(result, Err(AgyError::TimedOut)));
    }

    #[tokio::test]
    async fn authentication_failure_is_reported() {
        let script = write_script("auth", "#!/bin/sh\necho 'authentication required' >&2\nexit 1\n");
        let result = run(&script, &options("hi"), &Cancel::new(), |_| {}).await;
        assert!(matches!(result, Err(AgyError::NotSignedIn)));
    }

    #[tokio::test]
    async fn writes_one_json_line_and_closes_stdin() {
        let outfile = std::env::temp_dir().join(format!("slopchat-agy-in-{}", std::process::id()));
        let _ = fs::remove_file(&outfile);
        let script = write_script("stdin", &format!("#!/bin/sh\ncat > '{}'\n", outfile.display()));

        let result = run(&script, &options("hello world"), &Cancel::new(), |_| {}).await;
        assert!(matches!(result, Err(AgyError::Failed(_))));

        let written = fs::read_to_string(&outfile).unwrap();
        assert_eq!(written.lines().count(), 1);
        let value: serde_json::Value = serde_json::from_str(written.trim()).unwrap();
        assert_eq!(value["event"], "user");
        assert_eq!(value["message"]["content"], "hello world");
    }

    #[tokio::test]
    async fn exec_uses_the_binary_override_and_its_version() {
        let script = write_script(
            "exec",
            &format!(
                "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 1.2.16; exit 0; fi\ncat '{}'\n",
                fixture("text.ndjson")
            ),
        );
        binary::set_override_for_tests(Some(script));

        let result = exec::exec(options("hi"), &Cancel::new(), |_| {}).await.unwrap();
        assert_eq!(result.response, "hello there\n");

        binary::set_override_for_tests(None);
    }
