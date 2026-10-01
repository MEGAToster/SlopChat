pub mod budget;
mod fetch_client;
mod fetch_url;
mod html;
mod search;

use serde_json::Value;
use tauri::AppHandle;

use crate::config::SearchSettings;
use crate::providers::Source;

pub struct ToolContext {
    pub app: AppHandle,
    pub search: SearchSettings,
    pub start_index: u32,
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
            "web_fetch" | "fetch_url" => Some(fetch_url::schema()),
            _ => None,
        })
        .collect()
}

pub async fn run(name: &str, args: Value, context: ToolContext) -> Result<ToolOutput, String> {
    match name {
        // Only web_search may call a paid search API; web_fetch stays local (and
        // free) so it never spends provider credits.
        "web_search" => search::run(args, context).await,
        "web_fetch" | "fetch_url" => fetch_url::run(args, context.search.reader_fallback).await,
        other => Err(format!("unknown tool {other}")),
    }
}
