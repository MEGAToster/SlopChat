pub mod binary;
pub mod events;
pub mod exec;
pub mod mapper;
pub mod models;
pub mod process;
pub mod session;

use std::time::Duration;

use serde_json::Value;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::stream::Cancel;
use super::trace::Ring;
use super::{ChatRequest, StreamEvent};
use self::events::AgyResult;
use self::exec::{AgyError, ExecOptions};
use self::mapper::Mapper;
use self::session::{Plan, Stored};
use crate::config::ConfigState;
use crate::store::{chats, Store};

pub const PROVIDER: &str = "antigravity";

// The provider glue: request -> exec options, resume, mapping, Usage/Done. Errors
// are returned so `agent::run` can reuse its fallback path.
pub async fn run(
    app: &AppHandle,
    request: ChatRequest,
    channel: Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<(), AgyError> {
    if newest_user_content(&request).is_some_and(has_non_text) {
        let _ = channel.send(StreamEvent::Error {
            message: "Antigravity accepts text only — remove image attachments".to_string(),
        });
        return Ok(());
    }

    let model = request
        .model
        .strip_prefix(&format!("{PROVIDER}/"))
        .unwrap_or(&request.model)
        .to_string();

    let stored = load_session(app, &request.chat_id);
    let settings = app.state::<ConfigState>().get();
    let users = session::user_texts(&request.messages);
    let timeout = Duration::from_secs(settings.advanced.max_turn_secs.clamp(30, 3600) as u64);

    let mut mapper = Mapper::new();
    let mut ring = Ring::new(settings.debug.agent_log);

    let first = match session::plan(&request, stored.as_ref()) {
        Plan::Resume { id, prompt } => ExecOptions {
            model: model.clone(),
            conversation: Some(id),
            prompt,
            timeout,
        },
        Plan::Fresh { prompt } => ExecOptions {
            model: model.clone(),
            conversation: None,
            prompt,
            timeout,
        },
    };

    let mut outcome = run_once(first, cancel, &channel, &mut mapper, &mut ring).await;
    // The resume id was rejected and `agy` silently started fresh; re-seed once.
    if matches!(outcome, Err(AgyError::UnknownConversation)) {
        let fresh = ExecOptions {
            model: model.clone(),
            conversation: None,
            prompt: session::fresh_prompt(&request),
            timeout,
        };
        outcome = run_once(fresh, cancel, &channel, &mut mapper, &mut ring).await;
    }

    match outcome {
        Ok(result) if result.is_success() => {
            if result.response.is_empty() && mapper.streamed_chars == 0 {
                let _ = channel.send(StreamEvent::Error {
                    message: "Antigravity returned an empty response".to_string(),
                });
                return Ok(());
            }
            if !result.response.is_empty() && mapper.streamed_chars == 0 {
                let _ = channel.send(StreamEvent::Delta {
                    content: result.response.clone(),
                    round: mapper.round().max(1),
                });
            }
            let stored = Stored {
                id: result.conversation_id.clone(),
                model,
                users: users.len(),
                hash: session::fingerprint(&users),
            };
            save_session(app, &request.chat_id, &session::encode(&stored));
            let _ = channel.send(StreamEvent::Usage {
                tokens_in: result.input_tokens,
                tokens_out: result.output_tokens + result.thinking_tokens,
                cost: None,
                provider: Some(PROVIDER.to_string()),
            });
            let _ = channel.send(StreamEvent::Done);
            Ok(())
        }
        Ok(_) => Ok(()),
        // An HTTP turn ends on Stop by flushing Usage/Done, not an error.
        Err(AgyError::Cancelled) => {
            let _ = channel.send(StreamEvent::Done);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

async fn run_once(
    options: ExecOptions,
    cancel: &Cancel,
    channel: &Channel<StreamEvent>,
    mapper: &mut Mapper,
    ring: &mut Ring,
) -> Result<AgyResult, AgyError> {
    exec::exec(options, cancel, |event| {
        if let Ok(value) = serde_json::to_value(event) {
            ring.push(value);
        }
        for stream_event in mapper.map(event) {
            let _ = channel.send(stream_event);
        }
    })
    .await
}

fn load_session(app: &AppHandle, chat_id: &str) -> Option<Stored> {
    let store = app.state::<Store>();
    let raw = {
        let conn = store.conn();
        chats::get_agy_session(&conn, chat_id).ok().flatten()
    };
    raw.and_then(|value| session::decode(&value))
}

fn save_session(app: &AppHandle, chat_id: &str, value: &str) {
    let store = app.state::<Store>();
    let conn = store.conn();
    let _ = chats::set_agy_session(&conn, chat_id, Some(value));
}

fn newest_user_content(request: &ChatRequest) -> Option<&Value> {
    request
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| &message.content)
}

fn has_non_text(content: &Value) -> bool {
    match content {
        Value::Array(parts) => parts
            .iter()
            .any(|part| part.get("type").and_then(Value::as_str) != Some("text")),
        _ => false,
    }
}

