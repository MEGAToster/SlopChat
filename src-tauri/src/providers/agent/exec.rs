use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::super::prompt::parse_args;
use super::super::stream::{Cancel, ToolCall};
use super::super::StreamEvent;
use crate::config::ConfigState;
use crate::tools::{self, ToolContext, ToolOutput};

pub(super) struct ToolOutcome {
    pub content: String,
    pub summary: String,
    pub sources: Vec<super::super::Source>,
    pub cost: f64,
}

fn tool_context(app: &AppHandle, start_index: u32) -> ToolContext {
    let settings = app.state::<ConfigState>().get();
    ToolContext {
        app: app.clone(),
        search: settings.search.clone(),
        start_index,
    }
}

// None means the turn was cancelled.
async fn call_tool(
    app: &AppHandle,
    name: &str,
    args: Value,
    start_index: u32,
    cancel: &Cancel,
) -> Option<Result<ToolOutput, String>> {
    let context = tool_context(app, start_index);
    tokio::select! {
        _ = cancel.cancelled() => None,
        result = tools::run(name, args, context) => Some(result),
    }
}

pub(super) async fn execute_tool(
    app: &AppHandle,
    call: &ToolCall,
    start_index: u32,
    cancel: &Cancel,
) -> ToolOutcome {
    match call_tool(app, &call.name, parse_args(&call.arguments), start_index, cancel).await {
        Some(Ok(output)) => ToolOutcome {
            content: output.content,
            summary: output.summary,
            sources: output.sources,
            cost: output.cost,
        },
        Some(Err(error)) => ToolOutcome {
            content: format!("error: {error}"),
            summary: error,
            sources: Vec::new(),
            cost: 0.0,
        },
        None => ToolOutcome {
            content: "error: cancelled".to_string(),
            summary: "cancelled".to_string(),
            sources: Vec::new(),
            cost: 0.0,
        },
    }
}

pub(super) async fn run_web_search(
    app: &AppHandle,
    query: &str,
    start_index: u32,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> (String, f64, u32) {
    let args = json!({ "query": query });
    let id = "auto_search".to_string();
    let _ = channel.send(StreamEvent::ToolCall {
        id: id.clone(),
        name: "web_search".to_string(),
        args: args.clone(),
    });

    match call_tool(app, "web_search", args, start_index, cancel).await {
        Some(Ok(output)) => {
            let count = output.sources.len() as u32;
            let _ = channel.send(StreamEvent::ToolResult {
                id,
                ok: true,
                skipped: false,
                summary: output.summary,
                sources: output.sources,
            });
            (output.content, output.cost, start_index + count)
        }
        Some(Err(error)) => {
            let _ = channel.send(StreamEvent::ToolResult {
                id,
                ok: false,
                skipped: false,
                summary: error.clone(),
                sources: Vec::new(),
            });
            (format!("error: {error}"), 0.0, start_index)
        }
        None => (String::new(), 0.0, start_index),
    }
}
