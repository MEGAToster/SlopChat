use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Deserialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use super::sse::SseParser;
use super::{api_key, openrouter, ChatRequest, ModelRef, StreamEvent};
use crate::config::{ConfigState, Settings};

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

    fn finish(&self, chat_id: &str) {
        self.active
            .lock()
            .expect("streams lock poisoned")
            .remove(chat_id);
    }
}

pub async fn stream(
    app: &AppHandle,
    request: ChatRequest,
    channel: Channel<StreamEvent>,
) -> Result<(), String> {
    let model = ModelRef::parse(&request.model)?;
    if model.provider != openrouter::PROVIDER {
        return Err(format!("provider {:?} is not supported yet", model.provider));
    }

    let settings = app.state::<ConfigState>().get();
    let key = api_key(&settings, &model.provider)?
        .ok_or_else(|| format!("no API key stored for {}", model.provider))?;

    let streams = app.state::<Streams>();
    let cancel = streams.start(&request.chat_id);

    let result = send(&request, &model.model, &key, &settings, &channel, &cancel).await;
    streams.finish(&request.chat_id);

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

async fn send(
    request: &ChatRequest,
    model: &str,
    key: &str,
    settings: &Settings,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<(), StreamError> {
    let mut response = client()
        .post(openrouter::endpoint(settings))
        .bearer_auth(key)
        .json(&openrouter::body(request, model))
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
            if let Some(parsed) = parse_chunk(&data) {
                if let Some(content) = parsed.content {
                    let _ = channel.send(StreamEvent::Delta { content });
                }
                if parsed.usage.is_some() {
                    usage = parsed.usage;
                }
            }
        }

        if done {
            break;
        }
    }

    if let Some(usage) = usage {
        let _ = channel.send(StreamEvent::Usage {
            tokens_in: usage.prompt_tokens,
            tokens_out: usage.completion_tokens,
            cost: usage.cost,
        });
    }

    Ok(())
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
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
    usage: Option<Usage>,
}

fn parse_chunk(data: &str) -> Option<Parsed> {
    let chunk: Chunk = serde_json::from_str(data).ok()?;
    let content = chunk
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.delta)
        .and_then(|delta| delta.content)
        .filter(|content| !content.is_empty());

    Some(Parsed {
        content,
        usage: chunk.usage,
    })
}
