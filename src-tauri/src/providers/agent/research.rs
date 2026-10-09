use super::super::capabilities;
use super::super::kinds::Kind;
use super::super::ModelRef;
use crate::config::Settings;

use super::budget::Budget;
use super::subagent;

// Builds the shared sub-agent config only when `advanced.deep.subagentModel` is
// set, resolvable, and capable of tool calling. Any failure falls back to the
// single-loop Deep research.
pub(super) async fn build_research(settings: &Settings) -> Option<subagent::Research> {
    let reference = settings.advanced.deep.subagent_model.trim();
    if reference.is_empty() {
        return None;
    }
    let parsed = ModelRef::parse(reference).ok()?;
    if parsed.provider == super::super::antigravity::PROVIDER {
        return None;
    }
    let kind = Kind::resolve(&parsed.provider).ok()?;
    let key = if kind.needs_auth() {
        Some(super::super::api_key(settings, kind.name()).ok().flatten()?)
    } else {
        None
    };
    if !capabilities::supports(kind, &parsed.model, settings, key.as_deref()).await {
        return None;
    }
    Some(subagent::Research {
        kind,
        model: parsed.model,
        key,
        settings: settings.clone(),
        budget: std::sync::Arc::new(tokio::sync::Mutex::new(Budget::deep(
            &settings.advanced,
        ))),
        sources: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
    })
}
