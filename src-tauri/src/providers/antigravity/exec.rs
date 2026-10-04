use std::time::Duration;

use super::super::stream::Cancel;
use super::{binary, events::AgyEvent, process};

pub use super::events::AgyResult;

// `exec` is deliberately free of the store, the Tauri channel and `ChatRequest`
// so it can later back a non-chat research tool.
#[derive(Debug)]
pub enum AgyError {
    NotInstalled,
    TooOld(String),
    NotSignedIn,
    // The resume id was rejected; the caller re-seeds a fresh conversation once.
    UnknownConversation,
    UnknownModel(String),
    Cancelled,
    TimedOut,
    Failed(String),
}

impl AgyError {
    pub fn message(&self) -> String {
        match self {
            AgyError::NotInstalled => {
                "Antigravity isn't installed. Install `agy` and run it once to sign in.".to_string()
            }
            AgyError::TooOld(version) => {
                format!("Antigravity {version} is too old. Update `agy` and try again.")
            }
            AgyError::NotSignedIn => {
                "Antigravity isn't signed in. Run `agy` once in a terminal and sign in.".to_string()
            }
            AgyError::UnknownConversation => "Antigravity lost this conversation.".to_string(),
            AgyError::UnknownModel(message) => format!("Antigravity: {message}"),
            AgyError::Cancelled => "Antigravity was stopped.".to_string(),
            AgyError::TimedOut => {
                "Antigravity took longer than the turn limit (Advanced → max turn time).".to_string()
            }
            AgyError::Failed(message) => format!("Antigravity: {message}"),
        }
    }
}

pub struct ExecOptions {
    pub model: String,
    pub conversation: Option<String>,
    pub prompt: String,
    pub timeout: Duration,
}

pub async fn exec(
    opts: ExecOptions,
    cancel: &Cancel,
    on_event: impl FnMut(&AgyEvent),
) -> Result<AgyResult, AgyError> {
    let path = binary::check().await?;
    process::run(&path, &opts, cancel, on_event).await
}
