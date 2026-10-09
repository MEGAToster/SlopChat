mod budget;
mod events;
mod exec;
mod final_pass;
mod finalize;
mod instructions;
mod research;
mod rounds;
mod state;
mod subagent;
mod turn;
mod turn_run;

use tauri::ipc::Channel;
use tauri::AppHandle;

use super::stream::Cancel;
use super::{ModelRef, StreamEvent};

use events::emit_error;
use turn::TurnError;
use turn_run::{run_turn, PreSearch};

pub async fn run(
    app: &AppHandle,
    request: super::ChatRequest,
    channel: Channel<StreamEvent>,
    cancel: Cancel,
) {
    let model = match ModelRef::parse(&request.model) {
        Ok(model) => model,
        Err(error) => return emit_error(&channel, error),
    };

    let mut pre_search: Option<PreSearch> = None;
    match run_turn(app, &request, &channel, &cancel, &model, &mut pre_search).await {
        Ok(()) => {}
        Err(TurnError::Unavailable(message)) => {
            let fallback = request
                .fallback
                .as_deref()
                .filter(|value| !value.is_empty());
            let Some(fallback) = fallback else {
                return emit_error(&channel, message);
            };

            let fallback_model = match ModelRef::parse(fallback) {
                Ok(model) => model,
                Err(error) => return emit_error(&channel, error),
            };

            let _ = channel.send(StreamEvent::Model {
                model: fallback.to_string(),
            });

            if let Err(error) = run_turn(
                app,
                &request,
                &channel,
                &cancel,
                &fallback_model,
                &mut pre_search,
            )
            .await
            {
                emit_error(&channel, error.message());
            }
        }
        Err(TurnError::Failed(message)) => emit_error(&channel, message),
    }
}
