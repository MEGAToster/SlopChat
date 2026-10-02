use std::time::Duration;

use serde_json::Value;
use tauri::ipc::Channel;
use tauri::AppHandle;

use super::super::stream::Cancel;
use super::super::trace::Ring;
use super::super::StreamEvent;
use super::budget::Budget;
use super::turn::{Totals, Turn};

// Immutable per-turn inputs shared by the round loop and the final pass.
pub(super) struct Ctx<'a> {
    pub app: &'a AppHandle,
    pub turn: Turn<'a>,
    pub base: Vec<Value>,
    pub tools: Option<Value>,
    pub tool_names: Vec<String>,
    pub forced: bool,
    pub reserve: Duration,
    pub turn_id: String,
    pub channel: &'a Channel<StreamEvent>,
    pub cancel: &'a Cancel,
}

// Mutable per-turn state owned by the orchestrator.
pub(super) struct State {
    pub messages: Vec<Value>,
    pub budget: Budget,
    pub totals: Totals,
    pub notes: Vec<String>,
    pub ring: Ring,
    pub round: u32,
    pub source_counter: u32,
    pub first: bool,
    pub wrap_up: bool,
    pub finished: bool,
    pub answer_chars: usize,
}
