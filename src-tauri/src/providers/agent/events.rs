use serde_json::{json, Value};
use tauri::ipc::Channel;

use super::super::prompt::parse_args;
use super::super::stream::ToolCall;
use super::super::trace::Ring;
use super::super::StreamEvent;
use crate::tools::ToolOutput;

pub(super) fn is_fetch(call: &ToolCall) -> bool {
    call.name == "web_fetch"
}

pub(super) fn emit_error(channel: &Channel<StreamEvent>, message: String) {
    let _ = channel.send(StreamEvent::Error { message });
}

pub(super) fn emit_call(channel: &Channel<StreamEvent>, call: &ToolCall, skipped: bool) {
    let _ = channel.send(StreamEvent::ToolCall {
        id: call.id.clone(),
        name: call.name.clone(),
        args: parse_args(&call.arguments),
    });
    if skipped {
        let _ = channel.send(StreamEvent::ToolResult {
            id: call.id.clone(),
            ok: false,
            skipped: true,
            summary: "skipped (limit)".to_string(),
            sources: Vec::new(),
        });
    }
}

pub(super) fn send_result(channel: &Channel<StreamEvent>, id: &str, result: &ToolOutput) {
    let _ = channel.send(StreamEvent::ToolResult {
        id: id.to_string(),
        ok: !result.content.starts_with("error:"),
        skipped: false,
        summary: result.summary.clone(),
        sources: result.sources.clone(),
    });
}

// A note for a successful tool result, reused by the final synthesis pass.
pub(super) fn note(call: &ToolCall, result: &ToolOutput) -> Option<String> {
    if result.content.starts_with("error:") {
        return None;
    }
    let args = parse_args(&call.arguments);
    let text = |key: &str| {
        args.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    if call.name == "web_search" {
        Some(format!("## web_search \"{}\"\n{}", text("query"), result.content))
    } else {
        Some(format!("## web_fetch {}\n{}", text("url"), result.content))
    }
}

pub(super) fn trace_result(
    ring: &mut Ring,
    turn: &str,
    round: u32,
    name: &str,
    result: &ToolOutput,
) {
    ring.push(json!({
        "turn": turn, "round": round,
        "result": {
            "name": name,
            "ok": !result.content.starts_with("error:"),
            "summary": result.summary,
            "bytes": result.content.len(),
            "head": result.content.chars().take(300).collect::<String>(),
        },
    }));
}
