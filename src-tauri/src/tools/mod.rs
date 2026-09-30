pub mod budget;
mod fetch_url;
mod html;
mod web_search;

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
            "web_search" => Some(web_search::schema()),
            "fetch_url" => Some(fetch_url::schema()),
            _ => None,
        })
        .collect()
}

pub async fn run(name: &str, args: Value, context: ToolContext) -> Result<ToolOutput, String> {
    match name {
        "web_search" => web_search::run(args, context).await,
        "fetch_url" => fetch_url::run(args, context).await,
        other => Err(format!("unknown tool {other}")),
    }
}
