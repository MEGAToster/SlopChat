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
        // Wake every waiter, not just one: a tool round can have several in
        // flight, and the flag makes cancellation stick for later waits too.
        if !self.cancelled.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    // Resolves immediately if already cancelled; otherwise waits for `cancel`.
    pub async fn cancelled(&self) {
        let notified = self.notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

pub struct RequestPlan<'a> {
    pub endpoint: String,
    pub body: serde_json::Value,
    pub key: Option<&'a str>,
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
    pub generation_id: Option<String>,
    pub finish_reason: Option<String>,
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
    plan: &RequestPlan<'_>,
    round: u32,
    inactivity: Duration,
    hard_deadline: tokio::time::Instant,
    channel: &Channel<StreamEvent>,
    cancel: &Cancel,
) -> Result<StepOutcome, StreamError> {
    let mut builder = super::client().post(&plan.endpoint).json(&plan.body);
    if let Some(key) = plan.key {
        builder = builder.bearer_auth(key);
    }

    // Waiting for headers is part of the turn too: Stop and the deadline must
    // reach it, or the user is stuck for the whole request timeout.
    let mut response = tokio::select! {
        _ = cancel.cancelled() => return Err(StreamError::Cancelled),
        _ = tokio::time::sleep_until(hard_deadline) => {
            return Err(StreamError::Failed("provider stalled".to_string()));
        }
        result = builder.send() => result.map_err(|error| StreamError::Failed(error.to_string()))?,
    };

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
    let mut generation_id: Option<String> = None;
    let mut reasoned = false;
    let mut finish_reason: Option<String> = None;
    let mut calls: Vec<ToolCall> = Vec::new();

    // Idle deadline advances only on real events; keep-alive comments (and other
    // empty chunks) don't count, so a provider that stops producing content is
    // caught instead of hanging forever.
    let mut idle_deadline = tokio::time::Instant::now() + inactivity;

    loop {
        let chunk = tokio::select! {
            _ = cancel.cancelled() => return Err(StreamError::Cancelled),
            _ = tokio::time::sleep_until(idle_deadline.min(hard_deadline)) => {
                return Err(StreamError::Failed("provider stalled".to_string()));
            }
            result = response.chunk() => result.map_err(|error| StreamError::Failed(error.to_string()))?,
        };

        let (events, eof) = match chunk {
            Some(bytes) => (parser.push(&bytes), false),
            None => (parser.finish(), true),
        };

        let mut done = false;
        let mut activity = false;
        for data in events {
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
                activity = true;
                content.push_str(&part);
                if !leaked {
                    forward(&content, &mut forwarded, &mut leaked, round, channel);
                }
            }
            if parsed.reasoning {
                activity = true;
                if !reasoned {
                    reasoned = true;
                    let _ = channel.send(StreamEvent::Reasoning);
                }
            }
            if parsed.usage.is_some() {
                activity = true;
                usage = parsed.usage;
            }
            if parsed.provider.is_some() {
                provider = parsed.provider;
            }
            if generation_id.is_none() {
                generation_id = parsed.id.filter(|id| !id.is_empty());
            }
            if parsed.finish_reason.is_some() {
                finish_reason = parsed.finish_reason;
            }
            if !parsed.tool_calls.is_empty() {
                activity = true;
                for call in parsed.tool_calls {
                    chunks::merge_call(&mut calls, call);
                }
            }
        }

        if activity {
            idle_deadline = tokio::time::Instant::now() + inactivity;
        }
        if done || eof {
            break;
        }
    }

    if !leaked && forwarded < content.len() {
        let _ = channel.send(StreamEvent::Delta {
            content: content[forwarded..].to_string(),
            round,
        });
    }

    Ok(StepOutcome {
        content,
        tool_calls: calls,
        usage,
        provider,
        generation_id,
        finish_reason,
    })
}

// Streams the text up to a leak marker, holding back a tail so a marker split
// across two deltas is still caught. Once a marker is seen, markup is dropped.
fn forward(
    content: &str,
    forwarded: &mut usize,
    leaked: &mut bool,
    round: u32,
    channel: &Channel<StreamEvent>,
) {
    if let Some(index) = leak::find(content) {
        *leaked = true;
        if index > *forwarded {
            let _ = channel.send(StreamEvent::Delta {
                content: content[*forwarded..index].to_string(),
                round,
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
            round,
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
