use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::prompt::{assistant_tool_message, build_messages, last_user_text, parse_args};
use super::stream::{self, Cancel, RequestPlan, StepOutcome, StreamError, ToolCall};
use super::{kinds::Kind, leak, ModelRef, StreamEvent};
use crate::config::{ConfigState, WebSearchMode};
use crate::tools::{self, ToolContext};

const MAX_ROUNDS: usize = 4;
const MAX_SEARCHES: usize = 5;
// Fetches are free; this is only a safety bound, effectively unlimited.
const MAX_FETCHES: usize = 50;
const STOP_HINT: &str =
    "web_search is used up for this turn — you may still call web_fetch, or answer now using the results above";
const LEAK_NOTE: &str = "_(the model tried to call tools in an unsupported format)_";

fn wrap_up_message() -> Value {
    json!({
        "role": "user",
        "content": "Web search is used up for this turn. Do not call web_search again. You may still call web_fetch on specific URLs, then answer using the results above.",
    })
}

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

    // Tool calling is opt-in per profile; web search follows the mode and
    // toggle, not the tools list, so it works even when no tools are listed.
    let tool_calling = request.tool_calling;
    let web_enabled = tool_calling && !request.web_search.is_off();
    let forced =
        web_enabled && (request.web_search == WebSearchMode::Always || request.web_toggle);
    let supports = if request.tools.is_empty() && !web_enabled {
        false
    } else {
        supports_tools(kind, &model.model, &settings, key.as_deref()).await
    };

    let mut tool_names: Vec<String> = if tool_calling {
        request
            .tools
            .iter()
            .filter(|tool| tool.as_str() != "web_search" || web_enabled)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    if web_enabled {
        // Fetch accompanies search; both are added even when the profile lists
        // no tools, since web search is driven by the mode/toggle.
        for tool in ["web_search", "web_fetch"] {
            if !tool_names.iter().any(|name| name == tool) {
                tool_names.push(tool.to_string());
            }
        }
    }

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

    // Tool definitions must stay identical for the whole turn; we stop tool use
    // through results/tool_choice, never by dropping definitions mid-turn.
    let schemas = tools::schemas(&tool_names);
    let tools_value = if schemas.is_empty() {
        None
    } else {
        Some(json!(schemas))
    };

    let (mut tokens_in, mut tokens_out) = (0u32, 0u32);
    let mut cost = 0.0f64;
    let mut provider = None;
    let mut first = true;
    let mut finished = false;
    let mut wrap_up = false;
    let mut searches = 0usize;
    let mut fetches = 0usize;

    for _ in 0..MAX_ROUNDS {
        let can_search = tool_names.iter().any(|name| name == "web_search");
        // Tools stay available after the wrap-up so web_fetch keeps working;
        // further searches are refused per call instead.
        let tool_choice = if first && forced && can_search {
            json!({ "type": "function", "function": { "name": "web_search" } })
        } else if tools_value.is_some() {
            json!("auto")
        } else {
            json!("none")
        };

        let outcome = match run_plan(
            kind,
            &settings,
            &model.model,
            &messages,
            request.temperature,
            request.reasoning.as_deref(),
            tools_value.as_ref(),
            &tool_choice,
            key.as_deref(),
            &channel,
            &cancel,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(StreamError::Cancelled) => return Ok(()),
            Err(StreamError::Failed(message)) => {
                let _ = channel.send(StreamEvent::Error { message });
                return Ok(());
            }
        };
        accumulate(&outcome, kind, &mut tokens_in, &mut tokens_out, &mut cost, &mut provider);

        let leaked = leak::has(&outcome.content);

        if outcome.tool_calls.is_empty() {
            if leaked && !wrap_up {
                messages.push(wrap_up_message());
                wrap_up = true;
                first = false;
                continue;
            }
            if leaked {
                let _ = channel.send(StreamEvent::Delta {
                    content: LEAK_NOTE.to_string(),
                });
            }
            finished = true;
            break;
        }

        let calls: Vec<ToolCall> = outcome
            .tool_calls
            .into_iter()
            .filter(|call| !call.name.is_empty())
            .collect();

        messages.push(assistant_tool_message(&leak::strip(&outcome.content), &calls));

        let mut hit_limit = false;
        for call in &calls {
            let search = call.name == "web_search";
            let fetch = call.name == "web_fetch" || call.name == "fetch_url";
            let allowed = (!search || searches < MAX_SEARCHES) && (!fetch || fetches < MAX_FETCHES);

            if !allowed {
                // Our own budget, not a provider failure: tell the model, but
                // keep it off the UI so it isn't shown as a failed step.
                hit_limit = true;
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.id,
                    "content": format!("error: {STOP_HINT}"),
                }));
                continue;
            }

            if search {
                searches += 1;
            }
            if fetch {
                fetches += 1;
            }

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

        if hit_limit && !wrap_up {
            messages.push(wrap_up_message());
            wrap_up = true;
        }

        first = false;
    }

    // The round cap was hit mid tool-loop; ask once more for prose with tools
    // still defined (some models ignore a missing tool list and leak markup).
    if !finished {
        if !wrap_up {
            messages.push(wrap_up_message());
        }
        match run_plan(
            kind,
            &settings,
            &model.model,
            &messages,
            request.temperature,
            request.reasoning.as_deref(),
            tools_value.as_ref(),
            &json!("none"),
            key.as_deref(),
            &channel,
            &cancel,
        )
        .await
        {
            Ok(outcome) => {
                accumulate(&outcome, kind, &mut tokens_in, &mut tokens_out, &mut cost, &mut provider);
                if leak::has(&outcome.content) {
                    let _ = channel.send(StreamEvent::Delta {
                        content: LEAK_NOTE.to_string(),
                    });
                }
            }
            Err(StreamError::Cancelled) => return Ok(()),
            Err(StreamError::Failed(message)) => {
                let _ = channel.send(StreamEvent::Error { message });
                return Ok(());
            }
        }
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

#[allow(clippy::too_many_arguments)]
async fn run_plan(
    kind: Kind,
    settings: &crate::config::Settings,
    model: &str,
    messages: &[Value],
    temperature: Option<f32>,
    reasoning: Option<&str>,
    tools: Option<&Value>,
    tool_choice: &Value,
    key: Option<&str>,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<StepOutcome, StreamError> {
    let plan = RequestPlan {
        endpoint: kind.endpoint(settings),
        body: kind.body(
            model,
            &Value::Array(messages.to_vec()),
            temperature,
            reasoning,
            tools,
            tool_choice,
        ),
        key: key.map(str::to_string),
    };
    stream::run_step(&plan, channel, cancel).await
}

fn accumulate(
    outcome: &StepOutcome,
    kind: Kind,
    tokens_in: &mut u32,
    tokens_out: &mut u32,
    cost: &mut f64,
    provider: &mut Option<String>,
) {
    if let Some(usage) = &outcome.usage {
        *tokens_in += usage.prompt_tokens;
        *tokens_out += usage.completion_tokens;
        if !kind.zero_cost() {
            *cost += usage.cost.unwrap_or(0.0);
        }
    }
    if outcome.provider.is_some() {
        *provider = outcome.provider.clone();
    }
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
