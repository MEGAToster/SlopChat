use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use super::{budget, ToolContext, ToolOutput};
use crate::config::{ConfigState, SearchSettings};
use crate::providers::Source;

pub fn schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "web_search",
            "description": "Search the web for current information. Returns numbered results.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "numResults": { "type": "integer", "minimum": 1, "maximum": 10 }
                },
                "required": ["query"]
            }
        }
    })
}

pub async fn run(args: Value, context: ToolContext) -> Result<ToolOutput, String> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if query.is_empty() {
        return Err("missing query".to_string());
    }

    if budget::remaining(&context.app, context.search.monthly_budget) < 0.02 {
        return Err("monthly search budget reached".to_string());
    }

    let key = exa_key(&context.app)?;
    let requested = args
        .get("numResults")
        .and_then(Value::as_u64)
        .unwrap_or(context.search.num_results as u64) as u32;
    let cap = context.search.num_results.clamp(1, 10);
    let num_results = requested.clamp(1, cap);

    let body = json!({
        "query": query,
        "type": context.search.search_type,
        "numResults": num_results,
        "contents": contents(&context.search),
    });

    let response = crate::providers::client()
        .post("https://api.exa.ai/search")
        .header("x-api-key", key)
        .timeout(Duration::from_secs(15))
        .json(&body)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("exa search failed: {}", response.status()));
    }

    let value: Value = response.json().await.map_err(|error| error.to_string())?;
    let cost = value
        .get("costDollars")
        .and_then(|cost| cost.get("total"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    budget::record(&context.app, cost);

    let results = value
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut content = String::new();
    let mut sources = Vec::new();

    for (index, result) in results.iter().enumerate() {
        let n = context.start_index + index as u32 + 1;
        let title = result
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let url = result
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let published = result
            .get("publishedDate")
            .and_then(Value::as_str)
            .unwrap_or("");

        content.push_str(&format!("[{n}] {title} — {url}"));
        if !published.is_empty() {
            content.push_str(&format!(" ({published})"));
        }
        content.push('\n');

        if let Some(highlights) = result.get("highlights").and_then(Value::as_array) {
            for highlight in highlights {
                if let Some(text) = highlight.as_str() {
                    content.push_str("  ");
                    content.push_str(text);
                    content.push('\n');
                }
            }
        } else if let Some(text) = result.get("text").and_then(Value::as_str) {
            content.push_str("  ");
            content.push_str(text);
            content.push('\n');
        }

        sources.push(Source { n, title, url });
    }

    Ok(ToolOutput {
        content,
        summary: format!("{} results", sources.len()),
        sources,
        cost,
    })
}

fn contents(search: &SearchSettings) -> Value {
    match search.contents.as_str() {
        "highlights" => json!({ "highlights": { "maxCharacters": search.max_characters } }),
        "text" => json!({ "text": { "maxCharacters": search.max_characters } }),
        _ => json!({}),
    }
}

fn exa_key(app: &AppHandle) -> Result<String, String> {
    let settings = app.state::<ConfigState>().get();
    if let Some(configured) = settings.providers.get("exa").and_then(|p| p.api_key.as_deref()) {
        return match configured.strip_prefix("env:") {
            Some(variable) => std::env::var(variable).map_err(|_| format!("{variable} is not set")),
            None => Err("providers.exa.apiKey must use env:VAR".to_string()),
        };
    }
    if let Some(key) = crate::secrets::get("exa")? {
        return Ok(key);
    }
    std::env::var("EXA_API_KEY").map_err(|_| "no Exa API key stored".to_string())
}
