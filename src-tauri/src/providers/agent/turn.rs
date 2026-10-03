use std::time::Duration;

use serde_json::Value;
use tauri::ipc::Channel;

use super::super::kinds::{Kind, Limits};
use super::super::stream::{self, Cancel, RequestPlan, StepOutcome, StreamError};
use super::super::StreamEvent;

// Token and cost accumulators for the whole turn.
#[derive(Default)]
pub(super) struct Totals {
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub cost: f64,
    pub search_cost: f64,
    pub provider: Option<String>,
    pub generation_id: Option<String>,
}

pub(super) enum TurnError {
    Unavailable(String),
    Failed(String),
}

impl TurnError {
    pub fn message(self) -> String {
        match self {
            TurnError::Unavailable(message) | TurnError::Failed(message) => message,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Turn<'a> {
    pub kind: Kind,
    pub settings: &'a crate::config::Settings,
    pub model: &'a str,
    pub temperature: Option<f32>,
    pub reasoning: Option<&'a str>,
    pub reasoning_max_tokens: u32,
    pub max_completion_tokens: Option<u32>,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub inactivity: Duration,
    pub deadline: tokio::time::Instant,
    pub key: Option<&'a str>,
}

impl<'a> Turn<'a> {
    // Field-wise copy with a fresh deadline, used to reserve time for the answer.
    pub fn with_deadline(&self, deadline: tokio::time::Instant) -> Turn<'a> {
        Turn { deadline, ..*self }
    }

    pub async fn step(
        &self,
        messages: &[Value],
        tools: Option<&Value>,
        tool_choice: &Value,
        round: u32,
        channel: &Channel<StreamEvent>,
        cancel: &Cancel,
    ) -> Result<StepOutcome, StreamError> {
        let mut body = self.kind.body(
            self.model,
            messages,
            self.temperature,
            self.reasoning,
            tools,
            tool_choice,
        );
        self.kind.apply_limits(
            &mut body,
            &Limits {
                reasoning: self.reasoning,
                reasoning_max_tokens: self.reasoning_max_tokens,
                max_completion_tokens: self.max_completion_tokens,
                frequency_penalty: self.frequency_penalty,
                presence_penalty: self.presence_penalty,
            },
        );

        let plan = RequestPlan {
            endpoint: self.kind.endpoint(self.settings),
            body,
            key: self.key,
        };
        stream::run_step(&plan, round, self.inactivity, self.deadline, channel, cancel).await
    }
}

pub(super) fn accumulate(outcome: &StepOutcome, kind: Kind, totals: &mut Totals) {
    if let Some(usage) = &outcome.usage {
        totals.tokens_in += usage.prompt_tokens;
        totals.tokens_out += usage.completion_tokens;
        if !kind.zero_cost() {
            totals.cost += usage.cost.unwrap_or(0.0);
        }
    }
    if outcome.provider.is_some() {
        totals.provider = outcome.provider.clone();
    }
    if outcome.generation_id.is_some() {
        totals.generation_id = outcome.generation_id.clone();
    }
}
