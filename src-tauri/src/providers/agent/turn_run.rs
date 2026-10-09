use std::time::Duration;

use serde_json::json;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::super::capabilities;
use super::super::kinds::Kind;
use super::super::prompt::{build_messages, last_user_text, Guidance};
use super::super::stream::Cancel;
use super::super::trace::Ring;
use super::super::{ModelRef, StreamEvent};
use crate::config::{ConfigState, WebSearchMode};
use crate::tools;

use super::budget::Budget;
use super::exec::run_web_search;
use super::state::{Ctx, State};
use super::turn::{Totals, Turn, TurnError};
use super::{final_pass, finalize, research::build_research, rounds};

// The paid pre-search is model-independent, so the fallback attempt reuses it
// instead of paying for the same search twice.
pub(super) struct PreSearch {
    content: String,
    cost: f64,
    next: u32,
}

pub(super) async fn run_turn(
    app: &AppHandle,
    request: &super::super::ChatRequest,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
    model: &ModelRef,
    pre_search: &mut Option<PreSearch>,
) -> Result<(), TurnError> {
    if model.provider == super::super::antigravity::PROVIDER {
        return match super::super::antigravity::run(app, request.clone(), channel.clone(), cancel).await
        {
            Ok(()) => Ok(()),
            Err(error) => Err(antigravity_error(error)),
        };
    }

    let kind = Kind::resolve(&model.provider).map_err(TurnError::Failed)?;
    let settings = app.state::<ConfigState>().get();
    crate::tools::search::reset_paid_notice();
    let turn_id = format!("{}-{}", request.chat_id, crate::store::now_millis());

    let deep = request.web_search == WebSearchMode::Deep;
    let max_turn_secs = if deep {
        settings.advanced.deep.max_turn_secs.clamp(30, 3600)
    } else {
        settings.advanced.max_turn_secs.clamp(30, 3600)
    };
    let reserve = Duration::from_secs((max_turn_secs / 4).clamp(15, 45) as u64);
    let inactivity =
        Duration::from_secs(settings.advanced.response_timeout_secs.clamp(5, 600) as u64);
    let turn_deadline = tokio::time::Instant::now() + Duration::from_secs(max_turn_secs as u64);

    let key = if kind.needs_auth() {
        Some(
            super::super::api_key(&settings, kind.name())
                .map_err(TurnError::Failed)?
                .ok_or_else(|| TurnError::Failed(format!("no API key stored for {}", kind.name())))?,
        )
    } else {
        None
    };

    // Tool calling is opt-in per profile; web search follows the mode, not the
    // tools list, so it works even when no tools are listed.
    let tool_calling = request.tool_calling;
    let web_enabled = tool_calling && !request.web_search.is_off();
    let forced = web_enabled && deep;
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
    // With a sub-agent model, the main model plans and delegates; it is not
    // offered the web tools itself.
    let research = if deep && supports {
        build_research(&settings).await
    } else {
        None
    };
    if research.is_some() {
        tool_names = vec!["research".to_string()];
    } else if web_enabled {
        for tool in ["web_search", "web_fetch"] {
            if !tool_names.iter().any(|name| name == tool) {
                tool_names.push(tool.to_string());
            }
        }
    }

    let budget = if deep {
        Budget::deep(&settings.advanced)
    } else {
        Budget::from_settings(&settings.advanced)
    };
    let guidance = (supports && !tool_names.is_empty()).then_some(Guidance {
        max_searches: budget.max_searches,
        max_rounds: budget.max_rounds,
        deep,
        research: research.is_some(),
    });
    let web_offered = (supports
        && tool_names
            .iter()
            .any(|name| name == "web_search" || name == "research"))
        || (forced && !supports);
    let mut messages = build_messages(request, guidance, web_offered);

    let mut source_counter = 0u32;
    let mut pre_search_cost = 0.0;

    if forced && !supports {
        if pre_search.is_none() {
            let query = last_user_text(request);
            let (content, cost, next) =
                run_web_search(app, &query, source_counter, deep, channel, cancel).await;
            *pre_search = Some(PreSearch { content, cost, next });
        }
        let search = pre_search.as_ref().expect("pre-search set above");
        pre_search_cost += search.cost;
        source_counter = search.next;
        messages.push(
            json!({ "role": "system", "content": format!("Web results (cite as [n]):\n{}", search.content) }),
        );
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
        routing: settings
            .models
            .iter()
            .find(|entry| entry.model == request.model)
            .and_then(|entry| entry.routing.as_ref()),
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
        deep,
        research,
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
        budget,
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

    finalize::report(
        &mut state,
        kind,
        &settings,
        key.as_deref(),
        cancel,
        channel,
        &turn_id,
        answer_chars,
        final_pass,
    )
    .await;
    Ok(())
}

fn antigravity_error(error: super::super::antigravity::exec::AgyError) -> TurnError {
    let unavailable = matches!(
        &error,
        super::super::antigravity::exec::AgyError::NotInstalled
            | super::super::antigravity::exec::AgyError::TooOld(_)
            | super::super::antigravity::exec::AgyError::UnknownModel(_)
    );
    let message = error.message();
    if unavailable {
        TurnError::Unavailable(message)
    } else {
        TurnError::Failed(message)
    }
}
