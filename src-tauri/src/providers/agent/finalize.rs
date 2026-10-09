use std::time::Duration;

use serde_json::{json, Value};
use tauri::ipc::Channel;

use super::super::kinds::Kind;
use super::super::stream::Cancel;
use super::super::StreamEvent;
use super::state::State;
use crate::config::Settings;

// Ends the turn: traces it, attributes the provider, and emits usage + done.
#[allow(clippy::too_many_arguments)]
pub(super) async fn report(
    state: &mut State,
    kind: Kind,
    settings: &Settings,
    key: Option<&str>,
    cancel: &Cancel,
    channel: &Channel<StreamEvent>,
    turn_id: &str,
    answer_chars: usize,
    final_pass: Option<Value>,
) {
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
        match (key, state.totals.generation_id.as_deref()) {
            (Some(key), Some(id)) => {
                // The host is cosmetic attribution; its retry chain must not hold
                // the turn open, so cap the lookup hard.
                tokio::time::timeout(Duration::from_secs(3), super::super::generation::host(settings, key, id))
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

#[cfg(test)]mod tests {
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
