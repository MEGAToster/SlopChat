use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

use super::prompt::{assistant_tool_message, build_messages, last_user_text, parse_args};
use super::stream::{self, Cancel, RequestPlan, StepOutcome, StreamError, ToolCall};
use super::{kinds::Kind, leak, ModelRef, StreamEvent};
use crate::config::{ConfigState, WebSearchMode};
use crate::tools;

mod exec;
use exec::{execute_tool, run_web_search, ToolOutcome};

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
) {
    let model = match ModelRef::parse(&request.model) {
        Ok(model) => model,
        Err(error) => return emit_error(&channel, error),
    };

    match run_turn(app, &request, &channel, &cancel, &model).await {
        Ok(()) => {}
        Err(TurnError::Unavailable(message)) => {
            let fallback = request
                .fallback
                .as_deref()
                .filter(|value| !value.is_empty());
            let Some(fallback) = fallback else {
                return emit_error(&channel, message);
            };

            let fallback_model = match ModelRef::parse(fallback) {
                Ok(model) => model,
                Err(error) => return emit_error(&channel, error),
            };

            let _ = app.emit(
                "model://fallback",
                format!("{} unavailable — using {fallback}", request.model),
            );

            if let Err(error) = run_turn(app, &request, &channel, &cancel, &fallback_model).await {
                emit_error(&channel, error.message());
            }
        }
        Err(TurnError::Failed(message)) => emit_error(&channel, message),
    }
}

fn emit_error(channel: &Channel<StreamEvent>, message: String) {
    let _ = channel.send(StreamEvent::Error { message });
}

enum TurnError {
    Unavailable(String),
    Failed(String),
}

impl TurnError {
    fn message(self) -> String {
        match self {
            TurnError::Unavailable(message) | TurnError::Failed(message) => message,
        }
    }
}

async fn run_turn(
    app: &AppHandle,
    request: &super::ChatRequest,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
    model: &ModelRef,
) -> Result<(), TurnError> {
    let kind = Kind::resolve(&model.provider).map_err(TurnError::Failed)?;
    let settings = app.state::<ConfigState>().get();
    let tracing = settings.debug.agent_log;
    let turn_id = format!("{}-{}", request.chat_id, now_millis());

    // Clamp so a bad setting can't stall or unbound the turn.
    let max_rounds = settings.advanced.max_rounds.clamp(1, 10) as usize;
    let max_searches = settings.advanced.max_searches.clamp(1, 20) as usize;
    let max_fetches = settings.advanced.max_fetches.clamp(0, 100) as usize;

    let key = if kind.needs_auth() {
        Some(
            super::api_key(&settings, kind.name())
                .map_err(TurnError::Failed)?
                .ok_or_else(|| TurnError::Failed(format!("no API key stored for {}", kind.name())))?,
        )
    } else {
        None
    };

    // Tool calling is opt-in per profile; web search follows the mode and
    // toggle, not the tools list, so it works even when no tools are listed.
    let tool_calling = request.tool_calling;
    let web_enabled = tool_calling && !request.web_search.is_off();
    let forced =
        web_enabled && (request.web_search == WebSearchMode::Always || request.web_toggle);
    let supports = if request.tools.is_empty() && !web_enabled {
        false
    } else {
        super::capabilities::supports(kind, &model.model, &settings, key.as_deref()).await
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

    // The guidance only helps when tools are actually offered to the model.
    let mut messages = build_messages(request, supports && !tool_names.is_empty());

    let mut search_cost = 0.0;
    let mut source_counter = 0u32;

    if forced && !supports {
        let query = last_user_text(request);
        let (content, cost, next) =
            run_web_search(app, &query, source_counter, channel, cancel).await;
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

    // Multi-step research drifts at high temperature, so default it once tools
    // are in play — but not for reasoning models, which reject `temperature`.
    let reasoning_model = !tool_names.is_empty()
        && super::capabilities::reasoning(kind, &model.model, &settings, key.as_deref())
            .await
            .is_some();
    let temperature = request
        .temperature
        .or_else(|| (!tool_names.is_empty() && !reasoning_model).then_some(0.3));

    let turn = Turn {
        kind,
        settings: &settings,
        model: model.model.as_str(),
        temperature,
        reasoning: request.reasoning.as_deref(),
        key: key.as_deref(),
    };

    let (mut tokens_in, mut tokens_out) = (0u32, 0u32);
    let mut cost = 0.0f64;
    let mut provider = None;
    let mut first = true;
    let mut finished = false;
    let mut wrap_up = false;
    let mut searches = 0usize;
    let mut fetches = 0usize;
    let mut round = 0u32;

    for _ in 0..max_rounds {
        if cancel.is_cancelled() {
            return Ok(());
        }
        round += 1;
        let can_search = tool_names.iter().any(|name| name == "web_search");
        // Tools stay available after the wrap-up so web_fetch keeps working;
        // further searches are refused per call instead.
        // The user asked for the web, so make the first call a search.
        let tool_choice = if first && forced && can_search {
            json!({ "type": "function", "function": { "name": "web_search" } })
        } else if tools_value.is_some() {
            json!("auto")
        } else {
            json!("none")
        };

        if tracing {
            super::trace::append(&json!({
                "turn": turn_id, "round": round, "model": model.model,
                "provider": kind.name(), "toolChoice": tool_choice, "tools": tool_names,
            }));
        }

        let outcome = match turn
            .step(&messages, tools_value.as_ref(), &tool_choice, round, channel, cancel)
            .await
        {
            Ok(outcome) => outcome,
            Err(StreamError::Cancelled) => return Ok(()),
            // The initial request failing means the model is unavailable; the
            // caller may retry with a fallback. Later failures are reported.
            Err(StreamError::Unavailable(message)) if first => {
                return Err(TurnError::Unavailable(message));
            }
            Err(StreamError::Unavailable(message) | StreamError::Failed(message)) => {
                emit_error(channel, message);
                return Ok(());
            }
        };
        accumulate(&outcome, kind, &mut tokens_in, &mut tokens_out, &mut cost, &mut provider);

        if tracing {
            super::trace::append(&json!({
                "turn": turn_id, "round": round,
                "finish": if outcome.tool_calls.is_empty() { "stop" } else { "tool_calls" },
                "usage": outcome.usage.as_ref().map(|usage| json!({
                    "in": usage.prompt_tokens, "out": usage.completion_tokens,
                })),
                "provider": outcome.provider,
            }));
        }

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
                    round,
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

        // A research round is usually one search or a batch of page fetches;
        // fetches are independent, so run them together (max 3 in flight).
        if calls.len() > 1 && calls.iter().all(is_fetch) {
            for chunk in calls.chunks(3) {
                let mut handles = Vec::new();
                for call in chunk {
                    if fetches >= max_fetches {
                        hit_limit = true;
                        emit_call(channel, call, true);
                        messages.push(json!({
                            "role": "tool", "tool_call_id": call.id,
                            "content": format!("error: {STOP_HINT}"),
                        }));
                        continue;
                    }
                    fetches += 1;
                    emit_call(channel, call, false);
                    let worker = app.clone();
                    let owned = call.clone();
                    let stop = cancel.clone();
                    handles.push((
                        call.name.clone(),
                        call.id.clone(),
                        tauri::async_runtime::spawn(async move {
                            execute_tool(&worker, &owned, 0, &stop).await
                        }),
                    ));
                }

                for (name, id, handle) in handles {
                    let result = handle.await.unwrap_or_else(|_| ToolOutcome {
                        content: "error: cancelled".to_string(),
                        summary: "cancelled".to_string(),
                        sources: Vec::new(),
                        cost: 0.0,
                    });
                    search_cost += result.cost;
                    if tracing {
                        trace_result(&turn_id, round, &name, &result);
                    }
                    send_result(channel, &id, &result);
                    messages.push(json!({
                        "role": "tool", "tool_call_id": id, "content": result.content,
                    }));
                }

                if cancel.is_cancelled() {
                    return Ok(());
                }
            }
        } else {
            for call in &calls {
                let search = call.name == "web_search";
                let fetch = is_fetch(call);
                let allowed = (!search || searches < max_searches) && (!fetch || fetches < max_fetches);

                if !allowed {
                    hit_limit = true;
                    emit_call(channel, call, true);
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

                emit_call(channel, call, false);
                if tracing {
                    super::trace::append(&json!({
                        "turn": turn_id, "round": round,
                        "call": { "name": call.name, "args": parse_args(&call.arguments) },
                    }));
                }

                let result = execute_tool(app, call, source_counter, cancel).await;
                if cancel.is_cancelled() {
                    return Ok(());
                }
                search_cost += result.cost;
                source_counter += result.sources.len() as u32;
                if tracing {
                    trace_result(&turn_id, round, &call.name, &result);
                }

                send_result(channel, &call.id, &result);
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.id,
                    "content": result.content,
                }));
            }
        }

        if hit_limit && !wrap_up {
            messages.push(wrap_up_message());
            wrap_up = true;
        }

        first = false;
    }

    // The round cap was hit mid tool-loop; ask once more for prose with tools
    // still defined (some models ignore a missing tool list and leak markup).
    if !finished && !cancel.is_cancelled() {
        round += 1;
        if !wrap_up {
            messages.push(wrap_up_message());
        }
        match turn
            .step(&messages, tools_value.as_ref(), &json!("none"), round, channel, cancel)
            .await
        {
            Ok(outcome) => {
                accumulate(&outcome, kind, &mut tokens_in, &mut tokens_out, &mut cost, &mut provider);
                if leak::has(&outcome.content) {
                    let _ = channel.send(StreamEvent::Delta {
                        content: LEAK_NOTE.to_string(),
                        round,
                    });
                }
            }
            Err(StreamError::Cancelled) => return Ok(()),
            Err(StreamError::Unavailable(message) | StreamError::Failed(message)) => {
                emit_error(channel, message);
                return Ok(());
            }
        }
    }

    let total_cost = Some(cost + search_cost);

    if tracing {
        super::trace::append(&json!({
            "turn": turn_id, "done": true, "tokensIn": tokens_in, "tokensOut": tokens_out,
            "cost": total_cost, "searches": searches, "fetches": fetches,
        }));
    }

    let _ = channel.send(StreamEvent::Usage {
        tokens_in,
        tokens_out,
        cost: total_cost,
        provider: provider.or_else(|| Some(kind.name().to_string())),
    });
    let _ = channel.send(StreamEvent::Done);
    Ok(())
}

fn is_fetch(call: &ToolCall) -> bool {
    call.name == "web_fetch" || call.name == "fetch_url"
}

fn emit_call(channel: &Channel<StreamEvent>, call: &ToolCall, skipped: bool) {
    let _ = channel.send(StreamEvent::ToolCall {
        id: call.id.clone(),
        name: call.name.clone(),
        args: parse_args(&call.arguments),
    });
    if skipped {
        let _ = channel.send(StreamEvent::ToolResult {
            id: call.id.clone(),
            ok: false,
            skipped: true,
            summary: "skipped (limit)".to_string(),
            sources: Vec::new(),
        });
    }
}

fn send_result(channel: &Channel<StreamEvent>, id: &str, result: &ToolOutcome) {
    let _ = channel.send(StreamEvent::ToolResult {
        id: id.to_string(),
        ok: !result.content.starts_with("error:"),
        skipped: false,
        summary: result.summary.clone(),
        sources: result.sources.clone(),
    });
}

fn trace_result(turn: &str, round: u32, name: &str, result: &ToolOutcome) {
    super::trace::append(&json!({
        "turn": turn, "round": round,
        "result": {
            "name": name,
            "ok": !result.content.starts_with("error:"),
            "summary": result.summary,
            "bytes": result.content.len(),
            "head": result.content.chars().take(300).collect::<String>(),
        },
    }));
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0)
}

struct Turn<'a> {
    kind: Kind,
    settings: &'a crate::config::Settings,
    model: &'a str,
    temperature: Option<f32>,
    reasoning: Option<&'a str>,
    key: Option<&'a str>,
}

impl Turn<'_> {
    async fn step(
        &self,
        messages: &[Value],
        tools: Option<&Value>,
        tool_choice: &Value,
        round: u32,
        channel: &Channel<StreamEvent>,
        cancel: &Cancel,
    ) -> Result<StepOutcome, StreamError> {
        let plan = RequestPlan {
            endpoint: self.kind.endpoint(self.settings),
            body: self.kind.body(
                self.model,
                messages,
                self.temperature,
                self.reasoning,
                tools,
                tool_choice,
            ),
            key: self.key,
        };
        stream::run_step(&plan, round, channel, cancel).await
    }
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
