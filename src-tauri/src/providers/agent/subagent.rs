use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::AppHandle;
use tokio::sync::Mutex;

use super::super::kinds::Kind;
use super::super::prompt::{assistant_tool_message, parse_args};
use super::super::stream::{Cancel, StreamError};
use super::super::{leak, StreamEvent};
use super::budget::Budget;
use super::exec::execute_tool;
use super::turn::Turn;
use crate::config::Settings;
use crate::tools::{self, ToolOutput};

const SYSTEM: &str = "You are a research sub-agent. Answer exactly one sub-question. Use web_search, then web_fetch the most authoritative sources. Return at most ~1500 tokens of dense notes with inline [n] citations. Do not ask questions and do not call other tools.";
// One sub-agent may not spend the whole turn; the shared budget caps the rest.
const MAX_ROUNDS: u32 = 5;
const TIMEOUT: Duration = Duration::from_secs(120);

// State shared by every sub-agent of one Deep turn.
pub(super) struct Research {
    pub kind: Kind,
    pub model: String,
    pub key: Option<String>,
    pub settings: Settings,
    pub budget: Arc<Mutex<Budget>>,
    pub sources: Arc<AtomicU32>,
}

// Runs one headless round loop for a single sub-question. The UI is not
// involved: the caller forwards the returned notes as one tool result.
pub(super) async fn research(
    app: &AppHandle,
    cfg: &Research,
    question: &str,
    cancel: &Cancel,
) -> ToolOutput {
    let noop: Channel<StreamEvent> = Channel::new(|_| Ok(()));
    let schemas = tools::schemas(&["web_search".to_string(), "web_fetch".to_string()]);
    let tools = json!(schemas);
    let mut messages = vec![
        json!({ "role": "system", "content": SYSTEM }),
        json!({ "role": "user", "content": question }),
    ];

    let turn = Turn {
        kind: cfg.kind,
        settings: &cfg.settings,
        model: &cfg.model,
        temperature: Some(0.3),
        reasoning: None,
        reasoning_max_tokens: 0,
        max_completion_tokens: Some(4096),
        frequency_penalty: 0.0,
        presence_penalty: 0.0,
        routing: None,
        inactivity: Duration::from_secs(60),
        deadline: tokio::time::Instant::now() + TIMEOUT,
        key: cfg.key.as_deref(),
    };

    let mut content = String::new();
    let mut sources = Vec::new();
    let mut cost = 0.0;

    for round in 1..=MAX_ROUNDS {
        let outcome = match turn
            .step(&messages, Some(&tools), &json!("auto"), round, &noop, cancel)
            .await
        {
            Ok(outcome) => outcome,
            Err(StreamError::Cancelled) => break,
            Err(StreamError::Unavailable(message) | StreamError::Failed(message)) => {
                return failure(message)
            }
        };

        let calls: Vec<_> = outcome
            .tool_calls
            .into_iter()
            .filter(|call| !call.name.is_empty())
            .collect();
        if calls.is_empty() {
            if !outcome.content.trim().is_empty() {
                content = outcome.content;
            }
            break;
        }

        messages.push(assistant_tool_message(&leak::strip(&outcome.content), &calls));
        for call in &calls {
            if !cfg.budget.lock().await.allow(call) {
                messages.push(json!({
                    "role": "tool", "tool_call_id": call.id,
                    "content": "error: the research budget is used up — return your notes now",
                }));
                continue;
            }
            let start = cfg.sources.load(Ordering::Relaxed);
            let result = execute_tool(app, call, start, false, cancel).await;
            cfg.sources.fetch_add(result.sources.len() as u32, Ordering::Relaxed);
            cost += result.cost;
            sources.extend(result.sources.iter().cloned());
            messages.push(json!({ "role": "tool", "tool_call_id": call.id, "content": result.content }));
        }
    }

    if content.trim().is_empty() {
        content = "error: the sub-agent returned no notes".to_string();
    }
    ToolOutput {
        content,
        summary: format!("research · {} sources", sources.len()),
        sources,
        cost,
    }
}

fn failure(message: String) -> ToolOutput {
    ToolOutput {
        content: format!("error: {message}"),
        summary: "research failed".to_string(),
        sources: Vec::new(),
        cost: 0.0,
    }
}

// Pulls the sub-question out of a `research` tool call.
pub(super) fn question(raw: &str) -> String {
    parse_args(raw)
        .get("question")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_reads_the_argument_and_tolerates_junk() {
        assert_eq!(question(r#"{"question":"What is X?"}"#), "What is X?");
        assert_eq!(question("{}"), "");
        assert_eq!(question("not json"), "");
    }
}
