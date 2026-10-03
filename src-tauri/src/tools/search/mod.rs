mod brave;
mod exa;
mod firecrawl;
mod hit;
mod serper;
mod tavily;

use hit::Batch;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::{budget, html, ToolContext, ToolOutput};
use crate::config::{ConfigState, SearchSettings};
use crate::providers::Source;

const TIMEOUT: Duration = Duration::from_secs(15);
const UNAVAILABLE: &str = "All free search allowances used — search resumes when they refill.";

pub fn schema() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "web_search",
            "description": "Search the web. Returns numbered results with short snippets. Use specific queries; snippets are not enough for exact numbers — fetch the page.",
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

pub struct BackendCtx<'a> {
    pub key: &'a str,
    pub settings: &'a SearchSettings,
}

pub enum SearchError {
    Exhausted(String),
    Failed(String),
}

// Shared HTTP handling so every backend reports quota failures the same way.
pub(super) async fn send_json(request: reqwest::RequestBuilder) -> Result<Value, SearchError> {
    let response = request
        .send()
        .await
        .map_err(|error| SearchError::Failed(error.to_string()))?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .await
        .map_err(|error| SearchError::Failed(error.to_string()))?;

    if !(200..300).contains(&status) {
        return Err(classify(status, &text));
    }

    serde_json::from_str(&text).map_err(|error| SearchError::Failed(error.to_string()))
}

fn classify(status: u16, body: &str) -> SearchError {
    if matches!(status, 402 | 429 | 432 | 433) || explicit_quota(body) {
        SearchError::Exhausted(reason(body).unwrap_or_else(|| format!("HTTP {status}")))
    } else {
        SearchError::Failed(format!("HTTP {status}"))
    }
}

// Only unambiguous billing phrases; a bare "limit" or "plan" in an error body
// must not lock a provider out for the rest of the month.
fn explicit_quota(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    [
        "out of credits",
        "insufficient credits",
        "not enough credits",
        "quota exceeded",
        "quota has been exceeded",
        "credit limit reached",
        "monthly quota",
        "free allowance",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn reason(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    for pointer in ["/error", "/message", "/detail", "/error/message"] {
        if let Some(text) = value.pointer(pointer).and_then(Value::as_str) {
            return Some(text.to_string());
        }
    }
    None
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

    let settings = context.app.state::<ConfigState>().get();
    let search = context.search.clone();
    let requested = args
        .get("numResults")
        .and_then(Value::as_u64)
        .unwrap_or(search.num_results as u64) as u32;
    let num_results = requested.clamp(1, 10);

    let mut reasons = Vec::new();
    let mut only_exhausted = true;

    for name in &search.providers {
        let key = match crate::providers::search_key(&settings, name) {
            Ok(Some(key)) => key,
            Ok(None) => {
                only_exhausted = false;
                reasons.push(format!("{name}: no key"));
                continue;
            }
            Err(error) => {
                only_exhausted = false;
                reasons.push(format!("{name}: {error}"));
                continue;
            }
        };

        let limit = search.limits.get(name).cloned().unwrap_or_default();
        if budget::is_exhausted(&context.app, name) || !budget::allowed(&context.app, name, &limit) {
            reasons.push(format!("{name}: free allowance used"));
            continue;
        }

        let backend = BackendCtx {
            key: &key,
            settings: &search,
        };

        match dispatch(name, &query, num_results, &backend).await {
            Ok(batch) => {
                budget::record(&context.app, name, batch.units, batch.cost_usd);
                return Ok(format_output(batch, &context, name, &search));
            }
            Err(SearchError::Exhausted(message)) => {
                budget::mark_exhausted(&context.app, name);
                reasons.push(format!("{name}: {message}"));
            }
            Err(SearchError::Failed(message)) => {
                only_exhausted = false;
                reasons.push(format!("{name}: {message}"));
            }
        }
    }

    // Only warn about used-up allowances when that is really why search failed.
    if only_exhausted && !reasons.is_empty() {
        notify_unavailable(&context.app);
    }
    Err(format!("web search unavailable: {}", reasons.join("; ")))
}

async fn dispatch(
    name: &str,
    query: &str,
    n: u32,
    ctx: &BackendCtx<'_>,
) -> Result<Batch, SearchError> {
    match name {
        "exa" => exa::search(query, n, ctx).await,
        "serper" => serper::search(query, n, ctx).await,
        "tavily" => tavily::search(query, n, ctx).await,
        "brave" => brave::search(query, n, ctx).await,
        "firecrawl" => firecrawl::search(query, n, ctx).await,
        other => Err(SearchError::Failed(format!("unknown provider {other}"))),
    }
}

fn format_output(
    batch: Batch,
    context: &ToolContext,
    provider: &str,
    search: &SearchSettings,
) -> ToolOutput {
    let mut content = String::new();
    let mut sources = Vec::new();
    let cap = search.snippet_chars as usize;

    for hit in &batch.hits {
        let snippet = truncate(&html::to_line(&hit.snippet), cap);

        // Answer/knowledge-box leads have no link: keep them as context, never
        // as citations, so numbering only counts real sources.
        if hit.url.is_empty() {
            content.push_str(&format!("{}: {}\n", hit.title, snippet));
            continue;
        }

        let n = context.start_index + sources.len() as u32 + 1;
        content.push_str(&format!("[{n}] {} — {}", hit.title, hit.url));
        if let Some(published) = hit.published.as_deref().filter(|value| !value.is_empty()) {
            content.push_str(&format!(" ({published})"));
        }
        content.push('\n');

        if !snippet.is_empty() {
            content.push_str("  ");
            content.push_str(&snippet);
            content.push('\n');
        }

        sources.push(Source {
            n,
            title: hit.title.clone(),
            url: hit.url.clone(),
        });
    }

    ToolOutput {
        content,
        summary: format!("{} results · {provider}", sources.len()),
        sources,
        cost: batch.cost_usd,
    }
}

fn truncate(text: &str, cap: usize) -> String {
    text.chars().take(cap).collect()
}

static NOTIFIED: AtomicBool = AtomicBool::new(false);

fn notify_unavailable(app: &AppHandle) {
    if NOTIFIED.swap(true, Ordering::Relaxed) {
        return;
    }
    let _ = app.emit("search://unavailable", UNAVAILABLE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_only_explicit_quota_failures_as_exhausted() {
        assert!(matches!(classify(402, ""), SearchError::Exhausted(_)));
        assert!(matches!(classify(429, ""), SearchError::Exhausted(_)));
        assert!(matches!(
            classify(400, "quota exceeded"),
            SearchError::Exhausted(_)
        ));
        // A bare "limit"/"plan" in the body must not lock the provider out.
        assert!(matches!(
            classify(400, "monthly plan limit reached"),
            SearchError::Failed(_)
        ));
        assert!(matches!(classify(401, ""), SearchError::Failed(_)));
        assert!(matches!(classify(500, "boom"), SearchError::Failed(_)));
    }

    #[test]
    fn extracts_a_reason_from_json_bodies() {
        assert_eq!(reason(r#"{"error":"plan limit reached"}"#).unwrap(), "plan limit reached");
        assert_eq!(reason("not json"), None);
    }
}
