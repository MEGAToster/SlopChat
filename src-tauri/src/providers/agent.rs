use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::prompt::{assistant_tool_message, build_messages, last_user_text, parse_args};
use super::stream::{self, Cancel, RequestPlan, StreamError, ToolCall};
use super::{kinds::Kind, ModelRef, StreamEvent};
use crate::config::{ConfigState, WebSearchMode};
use crate::tools::{self, ToolContext};

const MAX_ROUNDS: usize = 4;

pub async fn run(
    app: &AppHandle,
    request: super::ChatRequest,
    channel: Channel<StreamEvent>,
    cancel: Cancel,
) -> Result<(), String> {
    let model = ModelRef::parse(&request.model)?;
    let kind = Kind::from_name(&model.provider)
        .ok_or_else(|| format!("provider {:?} is not supported", model.provider))?;
    let settings = app.state::<ConfigState>().get();

    let key = if kind.needs_auth() {
        Some(
            super::api_key(&settings, kind.name())?
                .ok_or_else(|| format!("no API key stored for {}", kind.name()))?,
        )
    } else {
        None
    };

    let mut messages = build_messages(&request);

    let has_web = request.tools.iter().any(|tool| tool == "web_search");
    let web_enabled = has_web && !request.web_search.is_off();
    let forced =
        web_enabled && (request.web_search == WebSearchMode::Always || request.web_toggle);
    let supports = supports_tools(kind, &model.model, &settings, key.as_deref()).await;

    let mut tool_names: Vec<String> = request
        .tools
        .iter()
        .filter(|tool| tool.as_str() != "web_search" || web_enabled)
        .cloned()
        .collect();

    let mut search_cost = 0.0;
    let mut source_counter = 0u32;

    if forced && !supports {
        let query = last_user_text(&request);
        let (content, _, cost, next) = run_web_search(app, &query, source_counter, &channel, &cancel).await;
        search_cost += cost;
        source_counter = next;
        messages.push(json!({ "role": "system", "content": format!("Web results (cite as [n]):\n{content}") }));
        tool_names.clear();
    } else if !supports {
        tool_names.clear();
    }

    let (mut tokens_in, mut tokens_out) = (0u32, 0u32);
    let mut cost = 0.0f64;
    let mut provider = None;
    let mut first = true;

    for _ in 0..MAX_ROUNDS {
        let tools = tools::schemas(&tool_names);
        let tools_value = if tools.is_empty() { None } else { Some(json!(tools)) };
        let tool_choice = if first && forced && !tool_names.is_empty() {
            json!({ "type": "function", "function": { "name": "web_search" } })
        } else if tools_value.is_some() {
            json!("auto")
        } else {
            json!("none")
        };

        let plan = RequestPlan {
            endpoint: kind.endpoint(&settings),
            body: kind.body(
                &model.model,
                &Value::Array(messages.clone()),
                request.temperature,
                tools_value.as_ref(),
                &tool_choice,
            ),
            key: key.clone(),
        };

        let outcome = match stream::run_step(&plan, &channel, &cancel).await {
            Ok(outcome) => outcome,
            Err(StreamError::Cancelled) => return Ok(()),
            Err(StreamError::Failed(message)) => {
                let _ = channel.send(StreamEvent::Error { message });
                return Ok(());
            }
        };

        if let Some(usage) = outcome.usage {
            tokens_in += usage.prompt_tokens;
            tokens_out += usage.completion_tokens;
            if !kind.zero_cost() {
                cost += usage.cost.unwrap_or(0.0);
            }
        }
        if outcome.provider.is_some() {
            provider = outcome.provider;
        }

        if outcome.tool_calls.is_empty() {
            break;
        }

        let calls: Vec<ToolCall> = outcome
            .tool_calls
            .into_iter()
            .filter(|call| !call.name.is_empty())
            .collect();

        messages.push(assistant_tool_message(&outcome.content, &calls));

        for call in &calls {
            let _ = channel.send(StreamEvent::ToolCall {
                id: call.id.clone(),
                name: call.name.clone(),
                args: parse_args(&call.arguments),
            });

            let (content, summary, sources, call_cost) =
                execute_tool(app, call, source_counter, &channel, &cancel).await;
            search_cost += call_cost;
            source_counter += sources.len() as u32;

            let _ = channel.send(StreamEvent::ToolResult {
                id: call.id.clone(),
                ok: !content.starts_with("error:"),
                summary,
                sources,
            });

            messages.push(json!({
                "role": "tool",
                "tool_call_id": call.id,
                "content": content,
            }));
        }

        first = false;
    }

    let total_cost = if kind.zero_cost() && search_cost == 0.0 {
        Some(0.0)
    } else {
        Some(cost + search_cost)
    };

    let _ = channel.send(StreamEvent::Usage {
        tokens_in,
        tokens_out,
        cost: total_cost,
        provider: provider.or_else(|| Some(kind.name().to_string())),
    });
    let _ = channel.send(StreamEvent::Done);
    Ok(())
}

async fn execute_tool(
    app: &AppHandle,
    call: &ToolCall,
    start_index: u32,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> (String, String, Vec<super::Source>, f64) {
    let settings = app.state::<ConfigState>().get();
    let context = ToolContext {
        app: app.clone(),
        search: settings.search.clone(),
        start_index,
    };

    let result = tokio::select! {
        _ = cancel.notify.notified() => None,
        result = tools::run(&call.name, parse_args(&call.arguments), context) => Some(result),
    };

    match result {
        Some(Ok(output)) => (output.content, output.summary, output.sources, output.cost),
        Some(Err(error)) => (format!("error: {error}"), error, Vec::new(), 0.0),
        None => {
            let _ = channel.send(StreamEvent::Error {
                message: "cancelled".to_string(),
            });
            ("error: cancelled".to_string(), "cancelled".to_string(), Vec::new(), 0.0)
        }
    }
}

async fn run_web_search(
    app: &AppHandle,
    query: &str,
    start_index: u32,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> (String, String, f64, u32) {
    let settings = app.state::<ConfigState>().get();
    let context = ToolContext {
        app: app.clone(),
        search: settings.search.clone(),
        start_index,
    };
    let args = json!({ "query": query });
    let id = "auto_search".to_string();
    let _ = channel.send(StreamEvent::ToolCall {
        id: id.clone(),
        name: "web_search".to_string(),
        args: args.clone(),
    });

    let result = tokio::select! {
        _ = cancel.notify.notified() => None,
        result = tools::run("web_search", args, context) => Some(result),
    };

    match result {
        Some(Ok(output)) => {
            let count = output.sources.len() as u32;
            let _ = channel.send(StreamEvent::ToolResult {
                id,
                ok: true,
                summary: output.summary.clone(),
                sources: output.sources,
            });
            (output.content, output.summary, output.cost, start_index + count)
        }
        Some(Err(error)) => {
            let _ = channel.send(StreamEvent::ToolResult {
                id,
                ok: false,
                summary: error.clone(),
                sources: Vec::new(),
            });
            (format!("error: {error}"), error, 0.0, start_index)
        }
        None => (String::new(), String::new(), 0.0, start_index),
    }
}

async fn supports_tools(
    kind: Kind,
    model: &str,
    settings: &crate::config::Settings,
    key: Option<&str>,
) -> bool {
    super::capabilities::supports(kind, model, settings, key).await
}
