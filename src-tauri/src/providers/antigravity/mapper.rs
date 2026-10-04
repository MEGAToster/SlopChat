use std::collections::HashMap;

use super::super::StreamEvent;
use super::events::AgyEvent;

// Turns `agy` events into the UI's stream events. Pure: no I/O, so it can be
// snapshot-tested against captured fixtures.
#[derive(Default)]
pub struct Mapper {
    round: u32,
    text_open: bool,
    tool_seq: u32,
    open: HashMap<String, String>,
    pub streamed_chars: usize,
}

impl Mapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn map(&mut self, event: &AgyEvent) -> Vec<StreamEvent> {
        match event {
            // `agy` starts a fresh answer buffer after a tool, and the UI treats
            // each new round the same way (text before a tool becomes a note).
            AgyEvent::Text(text) if !text.is_empty() => {
                if !self.text_open {
                    self.round += 1;
                    self.text_open = true;
                }
                self.streamed_chars += text.chars().count();
                vec![StreamEvent::Delta {
                    content: text.clone(),
                    round: self.round,
                }]
            }
            AgyEvent::ToolCall { key, name, params } => {
                self.text_open = false;
                let id = self.key(key);
                vec![StreamEvent::ToolCall {
                    id,
                    name: name.clone(),
                    args: params.clone(),
                }]
            }
            AgyEvent::ToolResult {
                key,
                name,
                params,
                ok,
                summary,
            } => {
                let mut events = Vec::new();
                let id = match self.open.remove(key) {
                    Some(id) => id,
                    // A lone result still needs a visible step.
                    None => {
                        let id = self.key(key);
                        events.push(StreamEvent::ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            args: params.clone(),
                        });
                        id
                    }
                };
                self.text_open = false;
                events.push(StreamEvent::ToolResult {
                    id,
                    ok: *ok,
                    skipped: false,
                    summary: short(summary),
                    sources: Vec::new(),
                });
                events
            }
            _ => Vec::new(),
        }
    }

    fn key(&mut self, key: &str) -> String {
        if let Some(id) = self.open.get(key) {
            return id.clone();
        }
        self.tool_seq += 1;
        let id = format!("agy-{}", self.tool_seq);
        self.open.insert(key.to_string(), id.clone());
        id
    }
}

fn short(summary: &str) -> String {
    summary.lines().next().unwrap_or("").chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::antigravity::events::parse_line;
    use serde_json::Value;

    fn run(fixture: &str) -> (Mapper, Vec<StreamEvent>) {
        let mut mapper = Mapper::new();
        let mut events = Vec::new();
        for line in fixture.lines() {
            if let Some(event) = parse_line(line) {
                events.extend(mapper.map(&event));
            }
        }
        (mapper, events)
    }

    fn json(events: &[StreamEvent]) -> Vec<Value> {
        events
            .iter()
            .map(|event| serde_json::to_value(event).unwrap())
            .collect()
    }

    #[test]
    fn text_turn_streams_one_round() {
        let (mapper, events) = run(include_str!("fixtures/text.ndjson"));
        assert_eq!(
            json(&events),
            vec![
                serde_json::json!({"type":"delta","content":"hello there","round":1}),
                serde_json::json!({"type":"delta","content":"\n","round":1}),
            ]
        );
        assert_eq!(mapper.streamed_chars, "hello there\n".chars().count());
    }

    #[test]
    fn a_tool_splits_the_rounds() {
        let (_, events) = run(include_str!("fixtures/tool.ndjson"));
        assert_eq!(
            json(&events),
            vec![
                serde_json::json!({"type":"toolCall","id":"agy-1","name":"view_file","args":{"AbsolutePath":"/tmp/agy-workspace/alpha.txt"}}),
                serde_json::json!({"type":"toolResult","id":"agy-1","ok":true,"skipped":false,"summary":"2 lines, 17 bytes","sources":[]}),
                serde_json::json!({"type":"delta","content":"SECRET_MARKER_42","round":1}),
                serde_json::json!({"type":"delta","content":"\n","round":1}),
            ]
        );
    }

    #[test]
    fn a_tool_error_reports_failure() {
        let (_, events) = run(include_str!("fixtures/tool_error.ndjson"));
        let result = events
            .iter()
            .find_map(|event| match event {
                StreamEvent::ToolResult { ok, summary, .. } => Some((*ok, summary.clone())),
                _ => None,
            })
            .expect("tool result");
        assert!(!result.0);
        assert!(!result.1.is_empty());
        assert!(result.1.chars().count() <= 120);
    }
}
