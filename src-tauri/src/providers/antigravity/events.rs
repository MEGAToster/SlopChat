use serde::Serialize;
use serde_json::Value;

// The subset of `agy`'s NDJSON we act on. Unknown events and unknown fields are
// ignored so a newer `agy` cannot break the stream.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AgyEvent {
    Init {
        conversation_id: Option<String>,
    },
    Text(String),
    ToolCall {
        key: String,
        name: String,
        params: Value,
    },
    ToolResult {
        key: String,
        name: String,
        params: Value,
        ok: bool,
        summary: String,
    },
    Result(AgyResult),
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgyResult {
    pub conversation_id: String,
    pub status: String,
    pub response: String,
    pub error: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub thinking_tokens: u32,
}

impl AgyResult {
    pub fn is_success(&self) -> bool {
        self.status == "SUCCESS"
    }
}

// A malformed line yields None; the caller logs and skips it.
pub fn parse_line(line: &str) -> Option<AgyEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    match value.get("event")?.as_str()? {
        "init" => Some(AgyEvent::Init {
            conversation_id: string(&value, "conversation_id"),
        }),
        "step_update" => Some(
            value
                .get("step_update")
                .map(parse_step)
                .unwrap_or(AgyEvent::Other),
        ),
        "result" => value.get("result").map(parse_result).map(AgyEvent::Result),
        _ => Some(AgyEvent::Other),
    }
}

fn parse_step(step: &Value) -> AgyEvent {
    match string(step, "step_type").as_deref() {
        Some("agent_response") => match string(step, "text_delta") {
            Some(delta) => AgyEvent::Text(delta),
            None => AgyEvent::Other,
        },
        Some("tool") => parse_tool(step),
        _ => AgyEvent::Other,
    }
}

fn parse_tool(step: &Value) -> AgyEvent {
    let key = step
        .get("step_index")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let name = string(step, "tool_name").unwrap_or_default();
    let info = step.get("tool_info");
    let params = info
        .and_then(|info| info.get("parameters"))
        .cloned()
        .unwrap_or(Value::Null);

    match string(step, "state").as_deref() {
        Some("DONE") => AgyEvent::ToolResult {
            key,
            name,
            params,
            ok: true,
            summary: info.and_then(|info| string(info, "output")).unwrap_or_default(),
        },
        Some("ERROR") => AgyEvent::ToolResult {
            key,
            name,
            params,
            ok: false,
            summary: info
                .and_then(|info| info.get("error"))
                .and_then(|error| string(error, "message"))
                .unwrap_or_default(),
        },
        _ => AgyEvent::ToolCall { key, name, params },
    }
}

fn parse_result(result: &Value) -> AgyResult {
    let usage = result.get("usage");
    AgyResult {
        conversation_id: string(result, "conversation_id").unwrap_or_default(),
        status: string(result, "status").unwrap_or_default(),
        response: string(result, "response").unwrap_or_default(),
        error: string(result, "error").filter(|error| !error.is_empty()),
        input_tokens: tokens(usage, "input_tokens"),
        output_tokens: tokens(usage, "output_tokens"),
        thinking_tokens: tokens(usage, "thinking_tokens"),
    }
}

fn string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn tokens(usage: Option<&Value>, key: &str) -> u32 {
    usage
        .and_then(|usage| usage.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(fixture: &str) -> Vec<AgyEvent> {
        fixture.lines().filter_map(parse_line).collect()
    }

    #[test]
    fn parses_a_text_turn() {
        let events = parse_all(include_str!("fixtures/text.ndjson"));
        assert!(matches!(events.first(), Some(AgyEvent::Init { .. })));
        assert!(events.iter().any(|event| event == &AgyEvent::Text("hello there".to_string())));
        let result = events
            .iter()
            .find_map(|event| match event {
                AgyEvent::Result(result) => Some(result),
                _ => None,
            })
            .expect("result event");
        assert!(result.is_success());
        assert_eq!(result.response, "hello there\n");
        assert_eq!(result.thinking_tokens, 31);
    }

    #[test]
    fn parses_a_tool_call_and_its_summary() {
        let events = parse_all(include_str!("fixtures/tool.ndjson"));
        assert!(events.iter().any(|event| matches!(
            event,
            AgyEvent::ToolCall { name, .. } if name == "view_file"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            AgyEvent::ToolResult { ok: true, summary, .. } if summary == "2 lines, 17 bytes"
        )));
    }

    #[test]
    fn parses_a_tool_error() {
        let events = parse_all(include_str!("fixtures/tool_error.ndjson"));
        assert!(events.iter().any(|event| matches!(
            event,
            AgyEvent::ToolResult { ok: false, summary, .. } if summary.contains("path is a directory")
        )));
    }

    #[test]
    fn parses_an_error_result() {
        let events = parse_all(include_str!("fixtures/unknown_model.ndjson"));
        let result = events
            .iter()
            .find_map(|event| match event {
                AgyEvent::Result(result) => Some(result),
                _ => None,
            })
            .expect("result event");
        assert!(!result.is_success());
        assert!(result
            .error
            .as_deref()
            .unwrap()
            .contains("not recognized as a known model"));
    }

    #[test]
    fn ignores_malformed_lines_and_unknown_events() {
        assert_eq!(parse_line("{not json"), None);
        assert_eq!(parse_line("{}"), None);
        assert_eq!(parse_line(r#"{"event":"checkpoint"}"#), Some(AgyEvent::Other));
        assert_eq!(parse_line(""), None);
    }
}
