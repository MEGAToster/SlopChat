mod brave;
mod exa;
mod firecrawl;
mod hit;
mod quota;
mod searxng;
mod serper;
mod tavily;
pub mod allowance;

use hit::Batch;
use quota::classify;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::{html, ToolContext, ToolOutput};
use crate::text::truncate;
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
        if let Some(reason) = endpoint_skip(name, &search) {
            only_exhausted = false;
            reasons.push(reason);
            continue;
        }

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
        if allowance::is_exhausted(&context.app, name) || !allowance::allowed(&context.app, name, &limit) {
            reasons.push(format!("{name}: free allowance used"));
            continue;
        }

        let backend = BackendCtx {
            key: &key,
            settings: &search,
        };

        match dispatch(name, &query, num_results, &backend).await {
            Ok(batch) => {
                allowance::record(&context.app, name, batch.units, batch.cost_usd);
                if context.deep && name != "searxng" {
                    notify_paid(&context.app, name, reasons.first());
                }
                return Ok(format_output(batch, &context, name, &search));
            }
            Err(SearchError::Exhausted(message)) => {
                allowance::mark_exhausted(&context.app, name);
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

// A launch-time health check; reuses the throttled search path so it also
// exercises the token and produces the same error messages.
pub async fn probe_searxng(app: &AppHandle) -> Option<String> {
    let settings = app.state::<ConfigState>().get();
    let search = settings.search.clone();
    if !probe_configured(&search) {
        return None;
    }
    // A missing token, or a locked/unavailable keychain, just means "no probe".
    let key = crate::providers::search_key(&settings, "searxng").ok().flatten()?;
    let backend = BackendCtx {
        key: &key,
        settings: &search,
    };
    match searxng::search("test", 1, &backend).await {
        Ok(_) => None,
        Err(SearchError::Failed(message)) | Err(SearchError::Exhausted(message)) => Some(message),
    }
}

fn probe_configured(search: &SearchSettings) -> bool {
    !search.searxng.base_url.trim().is_empty()
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
        "searxng" => searxng::search(query, n, ctx).await,
        other => Err(SearchError::Failed(format!("unknown provider {other}"))),
    }
}

// SearXNG carries its endpoint in settings instead of hardcoding one.
fn endpoint_skip(name: &str, search: &SearchSettings) -> Option<String> {
    if name != "searxng" {
        return None;
    }
    search
        .searxng
        .base_url
        .trim()
        .is_empty()
        .then(|| format!("{name}: no base URL"))
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

static NOTIFIED: AtomicBool = AtomicBool::new(false);

fn notify_unavailable(app: &AppHandle) {
    if NOTIFIED.swap(true, Ordering::Relaxed) {
        return;
    }
    let _ = app.emit("search://unavailable", UNAVAILABLE);
}

static PAID_NOTIFIED: AtomicBool = AtomicBool::new(false);

// Reset at the start of every turn so a Deep turn warns at most once.
pub fn reset_paid_notice() {
    PAID_NOTIFIED.store(false, Ordering::Relaxed);
}

fn notify_paid(app: &AppHandle, provider: &str, skipped: Option<&String>) {
    if PAID_NOTIFIED.swap(true, Ordering::Relaxed) {
        return;
    }
    let skipped = skipped.map(|value| format!(" ({value})")).unwrap_or_default();
    let _ = app.emit(
        "search://paid",
        format!(
            "Deep research used paid search ({provider}){skipped} — set up SearXNG in Settings → Web search."
        ),
    );
}


#[cfg(test)]
mod tests;
