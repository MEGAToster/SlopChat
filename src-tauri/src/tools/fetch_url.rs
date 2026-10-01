use serde_json::{json, Value};

use super::fetch_client::{self, Fetched};
use super::{ToolContext, ToolOutput};

pub fn schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "web_fetch",
            "description": "Fetch the readable text of a URL (usually from search results). Use for details such as pricing tables, specs and changelogs. Supports `start` for long pages.",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": { "type": "string" },
                    "start": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Character offset to continue from on long pages"
                    },
                    "maxChars": { "type": "integer", "minimum": 1, "maximum": 20000 }
                },
                "required": ["url"]
            }
        }
    })
}

pub async fn run(args: Value, context: ToolContext) -> Result<ToolOutput, String> {
    let url = args
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let start = args.get("start").and_then(Value::as_u64).unwrap_or(0) as usize;
    let max_chars = args
        .get("maxChars")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(fetch_client::DEFAULT_CHARS);

    let page = fetch_client::get(&url, start, max_chars, context.search.reader_fallback).await?;

    Ok(ToolOutput {
        content: format_content(&page),
        summary: format_summary(&page),
        sources: Vec::new(),
        cost: 0.0,
    })
}

fn format_content(page: &Fetched) -> String {
    let mut content = String::new();
    if let Some(title) = &page.title {
        content.push_str(&format!("Title: {title}\n"));
    }
    let first = page.start + 1;
    let last = page.start + page.text.chars().count();
    content.push_str(&format!(
        "URL: {} (chars {first}-{last} of {})\n\n",
        page.url, page.total
    ));
    content.push_str(&page.text);
    content
}

fn format_summary(page: &Fetched) -> String {
    let mut summary = format!("{} chars", compact(page.total));
    if page.via_reader {
        summary.push_str(" · via reader");
    }
    summary
}

fn compact(total: usize) -> String {
    if total >= 1000 {
        format!("{:.1}k", total as f64 / 1000.0)
    } else {
        total.to_string()
    }
}
