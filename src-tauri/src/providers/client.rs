use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use super::kinds::Kind;
use super::sse::SseParser;
use super::{api_key, ChatRequest, ModelRef, StreamEvent};
use crate::config::ConfigState;
use crate::paths;

#[derive(Default)]
pub struct Streams {
    active: Mutex<HashMap<String, Cancel>>,
}

#[derive(Clone)]
struct Cancel {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl Streams {
    fn start(&self, chat_id: &str) -> Cancel {
        let cancel = Cancel {
            cancelled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        };
        self.active
            .lock()
            .expect("streams lock poisoned")
            .insert(chat_id.to_string(), cancel.clone());
        cancel
    }

    pub fn stop(&self, chat_id: &str) {
        if let Some(cancel) = self
            .active
            .lock()
            .expect("streams lock poisoned")
            .get(chat_id)
        {
            cancel.cancelled.store(true, Ordering::SeqCst);
            cancel.notify.notify_one();
        }
    }

    fn finish(&self, chat_id: &str, cancel: &Cancel) {
        let mut active = self.active.lock().expect("streams lock poisoned");
        if let Some(existing) = active.get(chat_id) {
            if Arc::ptr_eq(&existing.cancelled, &cancel.cancelled) {
                active.remove(chat_id);
            }
        }
    }
}

struct ProviderRequest {
    endpoint: String,
    body: serde_json::Value,
    key: Option<String>,
    zero_cost: bool,
    provider: String,
}

pub async fn stream(
    app: &AppHandle,
    mut request: ChatRequest,
    channel: Channel<StreamEvent>,
) -> Result<(), String> {
    let model = ModelRef::parse(&request.model)?;
    let kind = Kind::from_name(&model.provider)
        .ok_or_else(|| format!("provider {:?} is not supported", model.provider))?;
    let settings = app.state::<ConfigState>().get();

    if request.use_context {
        if let Some(context) = read_context() {
            request.system = Some(match request.system.take() {
                Some(system) if !system.is_empty() => format!("{context}\n\n{system}"),
                _ => context,
            });
        }
    }

    let key = if kind.needs_auth() {
        Some(
            api_key(&settings, kind.name())?
                .ok_or_else(|| format!("no API key stored for {}", kind.name()))?,
        )
    } else {
        None
    };

    let plan = ProviderRequest {
        endpoint: kind.endpoint(&settings),
        body: kind.body(&request, &model.model),
        key,
        zero_cost: kind.zero_cost(),
        provider: kind.name().to_string(),
    };

    let streams = app.state::<Streams>();
    let cancel = streams.start(&request.chat_id);
    let result = send(&plan, &channel, &cancel).await;
    streams.finish(&request.chat_id, &cancel);

    match result {
        Ok(()) | Err(StreamError::Cancelled) => {
            let _ = channel.send(StreamEvent::Done);
        }
        Err(StreamError::Failed(message)) => {
            let _ = channel.send(StreamEvent::Error { message });
        }
    }

    Ok(())
}

fn read_context() -> Option<String> {
    let text = std::fs::read_to_string(paths::context_path()).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

async fn send(
    plan: &ProviderRequest,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<(), StreamError> {
    let mut builder = super::client().post(&plan.endpoint).json(&plan.body);
    if let Some(key) = plan.key.as_deref() {
        builder = builder.bearer_auth(key);
    }

    let mut response = builder
        .send()
        .await
        .map_err(|error| StreamError::Failed(error.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(StreamError::Failed(format!("{status}: {body}")));
    }

    let mut parser = SseParser::default();
    let mut usage = None;
    let mut provider = None;
    let mut reasoned = false;

    loop {
        let chunk = tokio::select! {
            _ = cancel.notify.notified() => return Err(StreamError::Cancelled),
            chunk = response.chunk() => {
                chunk.map_err(|error| StreamError::Failed(error.to_string()))?
            }
        };

        let Some(bytes) = chunk else {
            break;
        };

        let mut done = false;
        for data in parser.push(&bytes) {
            if data.trim() == "[DONE]" {
                done = true;
                break;
            }

            let Some(parsed) = parse_chunk(&data) else {
                continue;
            };

            if let Some(message) = parsed.error {
                return Err(StreamError::Failed(message));
            }
            if let Some(content) = parsed.content {
                let _ = channel.send(StreamEvent::Delta { content });
            }
            if parsed.reasoning && !reasoned {
                reasoned = true;
                let _ = channel.send(StreamEvent::Reasoning);
            }
            if parsed.usage.is_some() {
                usage = parsed.usage;
            }
            if parsed.provider.is_some() {
                provider = parsed.provider;
            }
        }

        if done {
            break;
        }
    }

    if let Some(usage) = usage {
        let cost = if plan.zero_cost { Some(0.0) } else { usage.cost };
        let _ = channel.send(StreamEvent::Usage {
            tokens_in: usage.prompt_tokens,
            tokens_out: usage.completion_tokens,
            cost,
            provider: provider.or_else(|| Some(plan.provider.clone())),
        });
    }

    Ok(())
}

enum StreamError {
    Cancelled,
    Failed(String),
}

#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    error: Option<ProviderError>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Option<Delta>,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
}

#[derive(Deserialize)]
struct ProviderError {
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize, Clone, Copy)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
    #[serde(default)]
    cost: Option<f64>,
}

struct Parsed {
    content: Option<String>,
    reasoning: bool,
    usage: Option<Usage>,
    provider: Option<String>,
    error: Option<String>,
}

fn parse_chunk(data: &str) -> Option<Parsed> {
    let chunk: Chunk = serde_json::from_str(data).ok()?;

    let error = chunk.error.map(|error| {
        error
            .message
            .unwrap_or_else(|| "provider error".to_string())
    });

    let delta = chunk
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.delta);

    let content = delta
        .as_ref()
        .and_then(|delta| delta.content.clone())
        .filter(|content| !content.is_empty());
    let reasoning = delta
        .and_then(|delta| delta.reasoning)
        .is_some_and(|reasoning| !reasoning.is_empty());

    Some(Parsed {
        content,
        reasoning,
        usage: chunk.usage,
        provider: chunk.provider,
        error,
    })
}
