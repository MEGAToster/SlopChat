use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::ipc::Channel;
use tokio::sync::Notify;

use super::chunks::{self, Usage};
use super::sse::SseParser;
use super::StreamEvent;

#[derive(Clone)]
pub struct Cancel {
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) notify: Arc<Notify>,
}

impl Cancel {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_one();
    }
}

pub struct RequestPlan {
    pub endpoint: String,
    pub body: serde_json::Value,
    pub key: Option<String>,
}

#[derive(Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub struct StepOutcome {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
    pub provider: Option<String>,
}

pub enum StreamError {
    Cancelled,
    Failed(String),
}

pub async fn run_step(
    plan: &RequestPlan,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<StepOutcome, StreamError> {
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
    let mut content = String::new();
    let mut usage = None;
    let mut provider = None;
    let mut reasoned = false;
    let mut calls: Vec<ToolCall> = Vec::new();

    loop {
        let chunk = tokio::select! {
            _ = cancel.notify.notified() => return Err(StreamError::Cancelled),
            result = tokio::time::timeout(Duration::from_secs(90), response.chunk()) => {
                match result {
                    Ok(chunk) => chunk.map_err(|error| StreamError::Failed(error.to_string()))?,
                    Err(_) => return Err(StreamError::Failed("provider timed out".to_string())),
                }
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
            let Some(parsed) = chunks::parse_chunk(&data) else {
                continue;
            };
            if let Some(message) = parsed.error {
                return Err(StreamError::Failed(message));
            }
            if let Some(part) = parsed.content {
                content.push_str(&part);
                let _ = channel.send(StreamEvent::Delta { content: part });
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
            for call in parsed.tool_calls {
                chunks::merge_call(&mut calls, call);
            }
        }

        if done {
            break;
        }
    }

    Ok(StepOutcome {
        content,
        tool_calls: calls,
        usage,
        provider,
    })
}
