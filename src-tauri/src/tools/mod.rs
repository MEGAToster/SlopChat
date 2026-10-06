mod fetch_client;
mod fetch_url;
mod html;
pub mod search;
mod ssrf;

use serde_json::{json, Value};
use tauri::AppHandle;

use crate::config::SearchSettings;
use crate::providers::Source;

pub struct ToolContext {
    pub app: AppHandle,
    pub search: SearchSettings,
    pub start_index: u32,
    // Deep turns warn once when a paid provider answers.
    pub deep: bool,
}

pub struct ToolOutput {
    pub content: String,
    pub summary: String,
    pub sources: Vec<Source>,
    pub cost: f64,
}

pub fn schemas(names: &[String]) -> Vec<Value> {
    names
        .iter()
        .filter_map(|name| match name.as_str() {
            "web_search" => Some(search::schema()),
            "web_fetch" => Some(fetch_url::schema()),
            // Executed by the agent's round loop, not through `run`.
            "research" => Some(research_schema()),
            _ => None,
        })
        .collect()
}

fn research_schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "research",
            "description": "Delegate one research sub-question to a cheap sub-agent that searches and reads pages, then returns dense notes with inline [n] citations. Plan 3-6 sub-questions and call this once per sub-question.",
            "parameters": {
                "type": "object",
                "properties": { "question": { "type": "string" } },
                "required": ["question"]
            }
        }
    })
}

pub async fn run(name: &str, args: Value, context: ToolContext) -> Result<ToolOutput, String> {
    match name {
        // Only web_search may call a paid search API; web_fetch stays local (and
        // free) so it never spends provider credits.
        "web_search" => search::run(args, context).await,
        "web_fetch" => fetch_url::run(args, context.search.reader_fallback).await,
        "research" => Err("research is handled by the agent loop".to_string()),
        other => Err(format!("unknown tool {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_schema_is_offered_by_name() {
        let schemas = schemas(&["research".to_string()]);
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0]["function"]["name"], "research");
    }
}
