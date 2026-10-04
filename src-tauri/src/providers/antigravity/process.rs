use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStderr, Command};

use super::super::stream::Cancel;
use super::events::{parse_line, AgyEvent, AgyResult};
use super::exec::{AgyError, ExecOptions};

// `--print-timeout` should fire first; this is the hard kill if it doesn't.
#[cfg(not(test))]
const KILL_GRACE: Duration = Duration::from_secs(10);
// Tests cannot wait ten seconds for the hard kill.
#[cfg(test)]
const KILL_GRACE: Duration = Duration::from_millis(250);
const STDERR_CAP: usize = 64 * 1024;

pub async fn run(
    path: &Path,
    opts: &ExecOptions,
    cancel: &Cancel,
    mut on_event: impl FnMut(&AgyEvent),
) -> Result<AgyResult, AgyError> {
    let workspace = workspace()?;
    let mut command = Command::new(path);
    command
        .current_dir(workspace)
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--sandbox")
        .arg("--print-timeout")
        .arg(format!("{}s", opts.timeout.as_secs().max(1)))
        .arg("--model")
        .arg(&opts.model);
    if let Some(conversation) = &opts.conversation {
        command.arg("--conversation").arg(conversation);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command.spawn().map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            super::binary::invalidate();
            AgyError::NotInstalled
        }
        _ => AgyError::Failed(error.to_string()),
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        let prompt = serde_json::to_string(&json!({
            "event": "user",
            "message": { "content": opts.prompt },
        }))
        .unwrap_or_default();
        // A write failure means `agy` already exited; the read loop reports why.
        let _ = stdin.write_all(format!("{prompt}\n").as_bytes()).await;
        let _ = stdin.flush().await;
    }

    let stderr_task = child.stderr.take().map(|stderr| tokio::spawn(read_tail(stderr)));
    let stdout = child.stdout.take().expect("piped stdout");
    let mut lines = BufReader::new(stdout).lines();

    let deadline = tokio::time::Instant::now() + opts.timeout + KILL_GRACE;
    let mut result: Option<AgyResult> = None;
    let mut cancelled = false;
    let mut timed_out = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                cancelled = true;
                break;
            }
            _ = tokio::time::sleep_until(deadline) => {
                timed_out = true;
                break;
            }
            line = lines.next_line() => match line {
                Ok(Some(line)) => {
                    if let Some(event) = parse_line(&line) {
                        if let AgyEvent::Result(value) = &event {
                            result = Some(value.clone());
                        }
                        on_event(&event);
                    }
                }
                Ok(None) => break,
                Err(error) => return Err(AgyError::Failed(error.to_string())),
            },
        }
    }

    if cancelled || timed_out {
        let _ = child.start_kill();
    }
    let _ = child.wait().await;
    let stderr = match stderr_task {
        Some(task) => task.await.unwrap_or_default(),
        None => String::new(),
    };

    if cancelled {
        return Err(AgyError::Cancelled);
    }
    if timed_out {
        return Err(AgyError::TimedOut);
    }
    classify(opts, result, &stderr)
}

fn classify(
    opts: &ExecOptions,
    result: Option<AgyResult>,
    stderr: &str,
) -> Result<AgyResult, AgyError> {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("authentication required")
        || lower.contains("not logged in")
        || lower.contains("not signed in")
    {
        return Err(AgyError::NotSignedIn);
    }

    match result {
        Some(result) if result.is_success() => {
            // A rejected resume id comes back as a *new* conversation id, silently.
            if let Some(requested) = &opts.conversation {
                if !result.conversation_id.is_empty() && &result.conversation_id != requested {
                    return Err(AgyError::UnknownConversation);
                }
            }
            Ok(result)
        }
        Some(result) => {
            let message = result
                .error
                .clone()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| result.status.clone());
            if unknown_model(&message) {
                Err(AgyError::UnknownModel(message))
            } else {
                Err(AgyError::Failed(message))
            }
        }
        None => {
            let last = stderr
                .lines()
                .rfind(|line| !line.trim().is_empty())
                .unwrap_or("")
                .to_string();
            if unknown_model(&last) {
                Err(AgyError::UnknownModel(last))
            } else if last.is_empty() {
                Err(AgyError::Failed("agy ended without a result".to_string()))
            } else {
                Err(AgyError::Failed(last))
            }
        }
    }
}

fn unknown_model(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("is not recognized as a known model") || lower.contains("invalid model selection")
}

async fn read_tail(stderr: ChildStderr) -> String {
    let mut reader = BufReader::new(stderr);
    let mut buffer = Vec::new();
    if reader.read_to_end(&mut buffer).await.is_err() {
        return String::new();
    }
    let tail = if buffer.len() > STDERR_CAP {
        &buffer[buffer.len() - STDERR_CAP..]
    } else {
        &buffer[..]
    };
    String::from_utf8_lossy(tail).into_owned()
}

fn workspace() -> Result<PathBuf, AgyError> {
    let dir = crate::paths::data_dir().join("antigravity").join("workspace");
    std::fs::create_dir_all(&dir).map_err(|error| {
        AgyError::Failed(format!("creating the Antigravity workspace failed: {error}"))
    })?;
    Ok(dir)
}


#[cfg(test)]
mod tests;
#[cfg(all(test, unix))]
mod fake_binary;
