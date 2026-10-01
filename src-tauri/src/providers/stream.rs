use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::ipc::Channel;
use tokio::sync::Notify;

use super::chunks::{self, Usage};
use super::leak;
use super::sse::SseParser;
use super::StreamEvent;

#[derive(Clone)]
pub struct Cancel {
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) notify: Arc<Notify>,
}

impl Default for Cancel {
    fn default() -> Self {
        Self::new()
    }
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
    // The model itself is unavailable (gone, rate-limited, provider down);
    // the caller may retry with a fallback model.
    Unavailable(String),
    Failed(String),
}

// Statuses that mean the model, not the request, is the problem.
fn unavailable(status: u16) -> bool {
    matches!(status, 402 | 404 | 429 | 503)
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
        let code = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        let message = format!("HTTP {code}: {body}");
        return Err(if unavailable(code) {
            StreamError::Unavailable(message)
        } else {
            StreamError::Failed(message)
        });
    }

    let mut parser = SseParser::default();
    let mut content = String::new();
    let mut forwarded = 0usize;
    let mut leaked = false;
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
                if !leaked {
                    forward(&content, &mut forwarded, &mut leaked, channel);
                }
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

    if !leaked && forwarded < content.len() {
        let _ = channel.send(StreamEvent::Delta {
            content: content[forwarded..].to_string(),
        });
    }

    Ok(StepOutcome {
        content,
        tool_calls: calls,
        usage,
        provider,
    })
}

// Streams the text up to a leak marker, holding back a tail so a marker split
// across two deltas is still caught. Once a marker is seen, markup is dropped.
fn forward(content: &str, forwarded: &mut usize, leaked: &mut bool, channel: &Channel<StreamEvent>) {
    if let Some(index) = leak::find(content) {
        *leaked = true;
        if index > *forwarded {
            let _ = channel.send(StreamEvent::Delta {
                content: content[*forwarded..index].to_string(),
            });
        }
        *forwarded = index;
        return;
    }

    let mut safe = content.len().saturating_sub(leak::MAX_MARKER);
    while safe > *forwarded && !content.is_char_boundary(safe) {
        safe -= 1;
    }
    if safe > *forwarded {
        let _ = channel.send(StreamEvent::Delta {
            content: content[*forwarded..safe].to_string(),
        });
        *forwarded = safe;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_outage_statuses_are_unavailable() {
        assert!(unavailable(404));
        assert!(unavailable(429));
        assert!(unavailable(402));
        assert!(unavailable(503));
        assert!(!unavailable(400));
        assert!(!unavailable(401));
        assert!(!unavailable(500));
    }
}
