mod budget;
mod events;
mod exec;
mod final_pass;
mod instructions;
mod rounds;
mod state;
mod turn;

use std::time::Duration;

use serde_json::json;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

use super::capabilities;
use super::kinds::Kind;
use super::prompt::{build_messages, last_user_text, Guidance};
use super::stream::Cancel;
use super::trace::Ring;
use super::{ModelRef, StreamEvent};
use crate::config::{ConfigState, WebSearchMode};
use crate::tools;

use budget::Budget;
use events::emit_error;
use exec::run_web_search;
use state::{Ctx, State};
use turn::{Totals, Turn, TurnError};

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

    let mut pre_search: Option<PreSearch> = None;
    match run_turn(app, &request, &channel, &cancel, &model, &mut pre_search).await {
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

            if let Err(error) = run_turn(
                app,
                &request,
                &channel,
                &cancel,
                &fallback_model,
                &mut pre_search,
            )
            .await
            {
                emit_error(&channel, error.message());
            }
        }
        Err(TurnError::Failed(message)) => emit_error(&channel, message),
    }
}

// The paid pre-search is model-independent, so the fallback attempt reuses it
// instead of paying for the same search twice.
struct PreSearch {
    content: String,
    cost: f64,
    next: u32,
}

async fn run_turn(
    app: &AppHandle,
    request: &super::ChatRequest,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
    model: &ModelRef,
    pre_search: &mut Option<PreSearch>,
) -> Result<(), TurnError> {
    let kind = Kind::resolve(&model.provider).map_err(TurnError::Failed)?;
    let settings = app.state::<ConfigState>().get();
    let turn_id = format!("{}-{}", request.chat_id, now_millis());

    let max_turn_secs = settings.advanced.max_turn_secs.clamp(30, 3600);
    let reserve = Duration::from_secs((max_turn_secs / 4).clamp(15, 45) as u64);
    let inactivity = Duration::from_secs(settings.advanced.response_timeout_secs.clamp(5, 600) as u64);
    let turn_deadline = tokio::time::Instant::now() + Duration::from_secs(max_turn_secs as u64);

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
        capabilities::supports(kind, &model.model, &settings, key.as_deref()).await
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
        for tool in ["web_search", "web_fetch"] {
            if !tool_names.iter().any(|name| name == tool) {
                tool_names.push(tool.to_string());
            }
        }
    }

    let guidance = (supports && !tool_names.is_empty()).then(|| Guidance {
        max_searches: settings.advanced.max_searches.clamp(1, 20) as usize,
        max_rounds: settings.advanced.max_rounds.clamp(1, 10) as usize,
    });
    let web_offered =
        (supports && tool_names.iter().any(|name| name == "web_search")) || (forced && !supports);
    let mut messages = build_messages(request, guidance, web_offered);

    let mut source_counter = 0u32;
    let mut pre_search_cost = 0.0;

    if forced && !supports {
        if pre_search.is_none() {
            let query = last_user_text(request);
            let (content, cost, next) =
                run_web_search(app, &query, source_counter, channel, cancel).await;
            *pre_search = Some(PreSearch { content, cost, next });
        }
        let search = pre_search.as_ref().expect("pre-search set above");
        pre_search_cost += search.cost;
        source_counter = search.next;
        messages.push(json!({ "role": "system", "content": format!("Web results (cite as [n]):\n{}", search.content) }));
        tool_names.clear();
    } else if !supports {
        tool_names.clear();
    }

    // `base` is the prompt without tool traffic, reused for the synthesis pass.
    let base = messages.clone();

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
        && capabilities::reasoning(kind, &model.model, &settings, key.as_deref())
            .await
            .is_some();
    let temperature = request
        .temperature
        .or_else(|| (!tool_names.is_empty() && !reasoning_model).then_some(0.3));
    // Reasoning endpoints reject sampling penalties too; apply them only
    // elsewhere, where weak models tend to fall into repetition loops.
    let (frequency_penalty, presence_penalty) = if reasoning_model {
        (0.0, 0.0)
    } else {
        (
            settings.advanced.frequency_penalty.clamp(0.0, 2.0),
            settings.advanced.presence_penalty.clamp(0.0, 2.0),
        )
    };

    let turn = Turn {
        kind,
        settings: &settings,
        model: model.model.as_str(),
        temperature,
        reasoning: request.reasoning.as_deref(),
        reasoning_max_tokens: settings.advanced.reasoning_max_tokens,
        max_completion_tokens: request.max_completion_tokens,
        frequency_penalty,
        presence_penalty,
        inactivity,
        deadline: turn_deadline,
        key: key.as_deref(),
    };

    let ctx = Ctx {
        app,
        turn,
        base,
        tools: tools_value,
        tool_names,
        forced,
        reserve,
        turn_id: turn_id.clone(),
        channel,
        cancel,
    };
    let totals = Totals {
        search_cost: pre_search_cost,
        ..Totals::default()
    };
    let mut state = State {
        messages,
        budget: Budget::from_settings(&settings.advanced),
        totals,
        notes: Vec::new(),
        ring: Ring::new(settings.debug.agent_log),
        round: 0,
        source_counter,
        first: true,
        wrap_up: false,
        finished: false,
        answer_chars: 0,
    };

    rounds::run(&ctx, &mut state).await?;
    let report = if !state.finished && !cancel.is_cancelled() {
        Some(final_pass::run(&ctx, &mut state).await?)
    } else {
        None
    };

    let answer_chars = state
        .answer_chars
        .max(report.as_ref().map_or(0, |report| report.content_chars));
    let final_pass = report.as_ref().map(|report| {
        json!({
            "attempt": report.attempt,
            "toolCallsIgnored": report.tool_calls_ignored,
            "contentChars": report.content_chars,
        })
    });
    if answer_chars == 0 {
        state.ring.push(json!({
            "turn": turn_id, "emptyAnswer": true, "finalPass": final_pass,
        }));
    }

    let total_cost = Some(state.totals.cost + state.totals.search_cost);
    state.ring.push(json!({
        "turn": turn_id, "done": true,
        "tokensIn": state.totals.tokens_in, "tokensOut": state.totals.tokens_out,
        "cost": total_cost, "searches": state.budget.searches, "fetches": state.budget.fetches,
        "finalPass": final_pass,
    }));

    let lookup = if state.totals.provider.is_none()
        && kind == Kind::OpenRouter
        && !cancel.is_cancelled()
    {
        match (key.as_deref(), state.totals.generation_id.as_deref()) {
            (Some(key), Some(id)) => {
                // The host is cosmetic attribution; its retry chain must not hold
                // the turn open, so cap the lookup hard.
                tokio::time::timeout(
                    Duration::from_secs(3),
                    super::generation::host(&settings, key, id),
                )
                .await
                .ok()
                .flatten()
            }
            _ => None,
        }
    } else {
        None
    };
    let provider = final_provider(state.totals.provider.clone(), lookup, kind);
    if state.totals.provider.is_none() && kind == Kind::OpenRouter {
        state.ring.push(json!({
            "turn": turn_id, "generationId": state.totals.generation_id,
            "generationHost": provider,
        }));
    }
    let _ = channel.send(StreamEvent::Usage {
        tokens_in: state.totals.tokens_in,
        tokens_out: state.totals.tokens_out,
        cost: total_cost,
        provider,
    });
    let _ = channel.send(StreamEvent::Done);
    Ok(())
}

// Only OpenRouter can name its host, and only via a lookup; others never invent one.
fn final_provider(stream: Option<String>, lookup: Option<String>, kind: Kind) -> Option<String> {
    if stream.is_some() {
        return stream;
    }
    if kind == Kind::OpenRouter {
        lookup
    } else {
        None
    }
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_comes_from_the_stream_when_present() {
        assert_eq!(
            final_provider(Some("DeepInfra".to_string()), None, Kind::OpenRouter).as_deref(),
            Some("DeepInfra")
        );
    }

    #[test]
    fn openrouter_uses_the_generation_lookup() {
        assert_eq!(
            final_provider(None, Some("Chutes".to_string()), Kind::OpenRouter).as_deref(),
            Some("Chutes")
        );
    }

    #[test]
    fn other_providers_never_invent_a_host() {
        assert_eq!(final_provider(None, None, Kind::Ollama), None);
        assert_eq!(final_provider(None, None, Kind::Gemini), None);
    }
}
