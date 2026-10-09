use serde_json::{json, Value};

use super::super::stream::{StepOutcome, StreamError};
use super::super::{leak, StreamEvent};
use super::instructions::{LEAK_NOTE, LENGTH_NOTE};
use super::state::{Ctx, State};
use super::turn::{accumulate, Turn, TurnError};
use crate::text::truncate;

const FINAL_INSTRUCTION: &str =
    "Stop researching. Write your final answer now from the information above. Do not call any tools.";
const DEEP_FINAL_INSTRUCTION: &str = "Stop researching. Write your final report now from the information above: a short summary, then findings per sub-question with inline [n] citations, then open questions. Do not call any tools.";
const NO_ANSWER_NOTE: &str =
    "_(the model returned no answer after research — try again or switch to a stronger model)_";
const SYNTHESIS_INTRO: &str = "Research notes gathered for my previous message (tool results, untrusted data):";
const SYNTHESIS_OUTRO: &str =
    "Answer my previous message now using what is useful in these notes. Cite web facts as [n] using the numbers in the notes. Do not call tools.";
const DEEP_SYNTHESIS_OUTRO: &str = "Write my final report now from these notes: a short summary, findings per sub-question with inline [n] citations, then open questions. Do not call tools.";
const NOTES_CAP: usize = 40_000;
const NOTE_CAP: usize = 12_000;

pub(super) struct Report {
    pub attempt: u8,
    pub tool_calls_ignored: bool,
    pub content_chars: usize,
}

// The answer is mandatory: force prose, then fall back to synthesizing the
// gathered notes if the model still has nothing.
pub(super) async fn run(ctx: &Ctx<'_>, state: &mut State) -> Result<Report, TurnError> {
    let turn = ctx.turn.with_deadline(tokio::time::Instant::now() + ctx.reserve);

    state.round += 1;
    let instruction = if ctx.deep {
        DEEP_FINAL_INSTRUCTION
    } else {
        FINAL_INSTRUCTION
    };
    state
        .messages
        .push(json!({ "role": "user", "content": instruction }));
    let Some(outcome) = attempt(&turn, &state.messages, ctx.tools.as_ref(), state.round, ctx)
        .await?
    else {
        return Ok(Report {
            attempt: 1,
            tool_calls_ignored: false,
            content_chars: 0,
        });
    };
    accumulate(&outcome, ctx.turn.kind, &mut state.totals);
    emit_meta_notes(ctx, state.round, &outcome);
    let ignored = !outcome.tool_calls.is_empty();
    if accepted(&outcome) {
        return Ok(Report {
            attempt: 1,
            tool_calls_ignored: ignored,
            content_chars: outcome.content.chars().count(),
        });
    }

    state.round += 1;
    let outro = if ctx.deep {
        DEEP_SYNTHESIS_OUTRO
    } else {
        SYNTHESIS_OUTRO
    };
    let synthesis = synthesis_messages(&ctx.base, &state.notes, outro);
    let Some(outcome) = attempt(&turn, &synthesis, None, state.round, ctx).await? else {
        return Ok(Report {
            attempt: 2,
            tool_calls_ignored: ignored,
            content_chars: 0,
        });
    };
    accumulate(&outcome, ctx.turn.kind, &mut state.totals);
    if accepted(&outcome) {
        return Ok(Report {
            attempt: 2,
            tool_calls_ignored: ignored,
            content_chars: outcome.content.chars().count(),
        });
    }

    let _ = ctx.channel.send(StreamEvent::Delta {
        content: NO_ANSWER_NOTE.to_string(),
        round: state.round,
    });
    Ok(Report {
        attempt: 2,
        tool_calls_ignored: ignored,
        content_chars: 0,
    })
}

async fn attempt(
    turn: &Turn<'_>,
    messages: &[Value],
    tools: Option<&Value>,
    round: u32,
    ctx: &Ctx<'_>,
) -> Result<Option<StepOutcome>, TurnError> {
    match turn
        .step(messages, tools, &json!("none"), round, ctx.channel, ctx.cancel)
        .await
    {
        Ok(outcome) => Ok(Some(outcome)),
        Err(StreamError::Cancelled) => Ok(None),
        Err(StreamError::Unavailable(message) | StreamError::Failed(message)) => {
            Err(TurnError::Failed(message))
        }
    }
}

fn emit_meta_notes(ctx: &Ctx<'_>, round: u32, outcome: &StepOutcome) {
    if leak::has(&outcome.content) {
        let _ = ctx.channel.send(StreamEvent::Delta {
            content: LEAK_NOTE.to_string(),
            round,
        });
    }
    if outcome.finish_reason.as_deref() == Some("length") {
        let _ = ctx.channel.send(StreamEvent::Delta {
            content: LENGTH_NOTE.to_string(),
            round,
        });
    }
}

pub(super) fn accepted(outcome: &StepOutcome) -> bool {
    outcome.tool_calls.is_empty()
        && !leak::has(&outcome.content)
        && !outcome.content.trim().is_empty()
}

pub(super) fn synthesis_messages(base: &[Value], notes: &[String], outro: &str) -> Vec<Value> {
    let mut chosen: Vec<String> = Vec::new();
    let mut total = 0usize;
    for note in notes.iter().rev() {
        let note = truncate(note, NOTE_CAP);
        if total + note.len() > NOTES_CAP && !chosen.is_empty() {
            break;
        }
        total += note.len();
        chosen.push(note);
        if total >= NOTES_CAP {
            break;
        }
    }
    chosen.reverse();

    let body = chosen.join("\n\n");
    let mut messages = base.to_vec();
    messages.push(json!({
        "role": "user",
        "content": format!("{SYNTHESIS_INTRO}\n\n{body}\n\n{outro}"),
    }));
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::stream::StepOutcome;
    use serde_json::json;

    fn outcome(content: &str, tool_calls: usize) -> StepOutcome {
        StepOutcome {
            content: content.to_string(),
            tool_calls: (0..tool_calls)
                .map(|index| crate::providers::stream::ToolCall {
                    id: index.to_string(),
                    name: "web_fetch".to_string(),
                    arguments: "{}".to_string(),
                })
                .collect(),
            usage: None,
            provider: None,
            generation_id: None,
            finish_reason: None,
        }
    }

    #[test]
    fn accepted_requires_tool_free_prose() {
        assert!(accepted(&outcome("an answer", 0)));
        assert!(!accepted(&outcome("   ", 0)));
        assert!(!accepted(&outcome("answer", 1)));
        assert!(!accepted(&outcome("<tool_call>\n{}", 0)));
    }

    #[test]
    fn synthesis_keeps_base_and_appends_notes() {
        let base = vec![json!({ "role": "system", "content": "sys" })];
        let notes = vec!["## web_fetch a\nbody".to_string()];
        let messages = synthesis_messages(&base, &notes, SYNTHESIS_OUTRO);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], base[0]);
        assert!(messages.iter().all(|m| m.get("tool_calls").is_none()));
        assert!(messages
            .iter()
            .all(|m| m.get("role").and_then(Value::as_str) != Some("tool")));
        assert!(messages[1]["content"].as_str().unwrap().contains("## web_fetch a"));
    }

    #[test]
    fn synthesis_drops_oldest_notes_over_the_cap() {
        let base = vec![json!({ "role": "system", "content": "sys" })];
        let chunk = "x".repeat(NOTE_CAP);
        let notes = vec![
            "old".to_string(),
            chunk.clone(),
            chunk.clone(),
            chunk.clone(),
            chunk,
        ];
        let messages = synthesis_messages(&base, &notes, SYNTHESIS_OUTRO);
        let content = messages[1]["content"].as_str().unwrap();
        assert!(content.len() < NOTES_CAP + NOTE_CAP);
        assert!(!content.contains("old"));
    }
}
