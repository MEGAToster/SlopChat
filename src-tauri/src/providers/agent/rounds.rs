use serde_json::{json, Value};

use super::super::prompt::{assistant_tool_message, parse_args};
use super::super::stream::{StreamError, ToolCall};
use super::super::{leak, StreamEvent};
use crate::tools::ToolOutput;
use super::budget::Budget;
use super::events::{emit_call, is_fetch, note, send_result, trace_result};
use super::exec::execute_tool;
use super::instructions::{
    stop_hint, wrap_up_message, LAST_ROUND_NOTICE, LEAK_NOTE, LENGTH_NOTE,
};
use super::state::{Ctx, State};
use super::turn::{accumulate, TurnError};

pub(super) async fn run(ctx: &Ctx<'_>, state: &mut State) -> Result<(), TurnError> {
    loop {
        if state.finished || ctx.cancel.is_cancelled() {
            break;
        }
        if state.budget.rounds >= state.budget.max_rounds {
            break;
        }
        // Reserve time so the final answer pass always runs.
        if tokio::time::Instant::now() + ctx.reserve >= ctx.turn.deadline {
            break;
        }
        if state.budget.take_last_round_warning() {
            state.messages.push(json!({ "role": "user", "content": LAST_ROUND_NOTICE }));
        }
        state.round += 1;
        state.budget.rounds += 1;

        let can_search = ctx.tool_names.iter().any(|name| name == "web_search");
        // The user asked for the web, so make the first call a search.
        let tool_choice = if state.first && ctx.forced && can_search {
            json!({ "type": "function", "function": { "name": "web_search" } })
        } else if ctx.tools.is_some() {
            json!("auto")
        } else {
            json!("none")
        };

        state.ring.push(json!({
            "turn": ctx.turn_id, "round": state.round, "model": ctx.turn.model,
            "provider": ctx.turn.kind.name(), "toolChoice": tool_choice, "tools": ctx.tool_names,
        }));

        let outcome = match ctx
            .turn
            .step(
                &state.messages,
                ctx.tools.as_ref(),
                &tool_choice,
                state.round,
                ctx.channel,
                ctx.cancel,
            )
            .await
        {
            Ok(outcome) => outcome,
            Err(StreamError::Cancelled) => return Ok(()),
            // Only the initial request may trigger a fallback model.
            Err(StreamError::Unavailable(message)) if state.first => {
                return Err(TurnError::Unavailable(message));
            }
            Err(StreamError::Unavailable(message) | StreamError::Failed(message)) => {
                return Err(TurnError::Failed(message));
            }
        };
        accumulate(&outcome, ctx.turn.kind, &mut state.totals);

        state.ring.push(json!({
            "turn": ctx.turn_id, "round": state.round,
            "finish": if outcome.tool_calls.is_empty() { "stop" } else { "tool_calls" },
            "finishReason": outcome.finish_reason,
            "usage": outcome.usage.as_ref().map(|usage| json!({
                "in": usage.prompt_tokens, "out": usage.completion_tokens,
            })),
            "provider": outcome.provider,
        }));

        let leaked = leak::has(&outcome.content);

        if outcome.tool_calls.is_empty() {
            if leaked && !state.wrap_up {
                state.messages.push(wrap_up_message());
                state.wrap_up = true;
                state.first = false;
                continue;
            }
            if leaked {
                delta(ctx, state.round, LEAK_NOTE);
            }
            if outcome.finish_reason.as_deref() == Some("length") {
                delta(ctx, state.round, LENGTH_NOTE);
            }
            if !outcome.content.trim().is_empty() {
                state.answer_chars = outcome.content.chars().count();
                state.finished = true;
            }
            // Empty prose with no tools: let the final pass handle it.
            break;
        }

        let calls: Vec<ToolCall> = outcome
            .tool_calls
            .into_iter()
            .filter(|call| !call.name.is_empty())
            .collect();
        state
            .messages
            .push(assistant_tool_message(&leak::strip(&outcome.content), &calls));

        let mut hit_limit = false;

        // A round is usually one search or a batch of page fetches; fetches are
        // independent, so run them together (max 3 in flight).
        if calls.len() > 1 && calls.iter().all(is_fetch) {
            for chunk in calls.chunks(3) {
                let mut handles = Vec::new();
                for call in chunk {
                    if !state.budget.allow(call) {
                        hit_limit = true;
                        emit_call(ctx.channel, call, true);
                        state
                            .messages
                            .push(tool_message(&call.id, &refuse(&state.budget, call)));
                        continue;
                    }
                    emit_call(ctx.channel, call, false);
                    let worker = ctx.app.clone();
                    let stop = ctx.cancel.clone();
                    let owned = call.clone();
                    let task_call = call.clone();
                    handles.push((
                        owned,
                        tauri::async_runtime::spawn(async move {
                            execute_tool(&worker, &task_call, 0, &stop).await
                        }),
                    ));
                }

                for (call, handle) in handles {
                    let result = handle.await.unwrap_or_else(|_| cancelled());
                    state.budget.add_fetched(result.content.chars().count() as u64);
                    state.totals.search_cost += result.cost;
                    trace_result(&mut state.ring, &ctx.turn_id, state.round, &call.name, &result);
                    send_result(ctx.channel, &call.id, &result);
                    push_result(state, &call, &result);
                }

                if ctx.cancel.is_cancelled() {
                    return Ok(());
                }
            }
        } else {
            for call in &calls {
                if !state.budget.allow(call) {
                    hit_limit = true;
                    emit_call(ctx.channel, call, true);
                    state
                        .messages
                        .push(tool_message(&call.id, &refuse(&state.budget, call)));
                    continue;
                }
                emit_call(ctx.channel, call, false);
                state.ring.push(json!({
                    "turn": ctx.turn_id, "round": state.round,
                    "call": { "name": call.name, "args": parse_args(&call.arguments) },
                }));

                let result = execute_tool(ctx.app, call, state.source_counter, ctx.cancel).await;
                if ctx.cancel.is_cancelled() {
                    return Ok(());
                }
                if is_fetch(call) {
                    state.budget.add_fetched(result.content.chars().count() as u64);
                }
                state.totals.search_cost += result.cost;
                state.source_counter += result.sources.len() as u32;
                trace_result(&mut state.ring, &ctx.turn_id, state.round, &call.name, &result);
                send_result(ctx.channel, &call.id, &result);
                push_result(state, call, &result);
            }
        }

        if hit_limit && !state.wrap_up {
            state.messages.push(wrap_up_message());
            state.wrap_up = true;
        }
        state.first = false;
    }
    Ok(())
}

fn tool_message(id: &str, content: &str) -> Value {
    json!({ "role": "tool", "tool_call_id": id, "content": content })
}

// Our own budget, not a provider failure: keep the model informed.
fn refuse(budget: &Budget, call: &ToolCall) -> String {
    format!("error: {}{}", stop_hint(&call.name), budget.footer())
}

fn push_result(state: &mut State, call: &ToolCall, result: &ToolOutput) {
    if let Some(note) = note(call, result) {
        state.notes.push(note);
    }
    let content = format!("{}{}", result.content, state.budget.footer());
    state.messages.push(tool_message(&call.id, &content));
}

fn cancelled() -> ToolOutput {
    ToolOutput {
        content: "error: cancelled".to_string(),
        summary: "cancelled".to_string(),
        sources: Vec::new(),
        cost: 0.0,
    }
}

fn delta(ctx: &Ctx<'_>, round: u32, content: &str) {
    let _ = ctx.channel.send(StreamEvent::Delta {
        content: content.to_string(),
        round,
    });
}
