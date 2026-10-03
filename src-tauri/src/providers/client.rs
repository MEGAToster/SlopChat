use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use super::agent;
use super::stream::Cancel;
use super::{ChatRequest, StreamEvent};

#[derive(Default)]
pub struct Streams {
    active: Mutex<HashMap<String, Cancel>>,
}

impl Streams {
    fn start(&self, chat_id: &str) -> Cancel {
        let cancel = Cancel::new();
        let mut active = self.active.lock().expect("streams lock poisoned");
        // A second stream on the same chat must stop the first, or the orphan
        // keeps running with no Cancel left to reach it.
        if let Some(previous) = active.insert(chat_id.to_string(), cancel.clone()) {
            previous.cancel();
        }
        cancel
    }

    pub fn stop(&self, chat_id: &str) {
        if let Some(cancel) = self
            .active
            .lock()
            .expect("streams lock poisoned")
            .get(chat_id)
        {
            cancel.cancel();
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

pub async fn stream(
    app: &AppHandle,
    request: ChatRequest,
    channel: Channel<StreamEvent>,
) -> Result<(), String> {
    let chat_id = request.chat_id.clone();
    let streams = app.state::<Streams>();
    let cancel = streams.start(&chat_id);
    agent::run(app, request, channel, cancel.clone()).await;
    streams.finish(&chat_id, &cancel);
    Ok(())
}
