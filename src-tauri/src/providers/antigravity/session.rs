use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::prompt::system_text;
use super::super::{ChatMessage, ChatRequest};

// The resume pointer persisted per chat: which `agy` conversation and a
// fingerprint of the user turns already inside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stored {
    pub id: String,
    pub users: usize,
    pub hash: String,
}

pub fn encode(stored: &Stored) -> String {
    serde_json::to_string(stored).unwrap_or_default()
}

pub fn decode(value: &str) -> Option<Stored> {
    serde_json::from_str(value).ok()
}

pub enum Plan {
    Resume { id: String, prompt: String },
    Fresh { prompt: String },
}

pub fn plan(request: &ChatRequest, stored: Option<&Stored>) -> Plan {
    let users = user_texts(&request.messages);

    if let Some(stored) = stored {
        let grew_by_one = users.len() == stored.users + 1;
        let unchanged = fingerprint(&users[..stored.users.min(users.len())]) == stored.hash;
        if grew_by_one && unchanged {
            return Plan::Resume {
                id: stored.id.clone(),
                prompt: users.last().cloned().unwrap_or_default(),
            };
        }
    }

    Plan::Fresh {
        prompt: fresh_prompt(request),
    }
}

// Only user turns are fingerprinted: the app rewrites assistant text before
// saving it, so assistant content would never match on a reload.
pub fn fingerprint(texts: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    for text in texts {
        text.hash(&mut hasher);
        0u8.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub fn user_texts(messages: &[ChatMessage]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.role == "user")
        .map(|message| text_parts(&message.content, false))
        .collect()
}

pub fn fresh_prompt(request: &ChatRequest) -> String {
    let mut out = String::new();

    let system = system_text(request);
    if !system.trim().is_empty() {
        out.push_str(&system);
        out.push_str("\n\n");
    }

    let newest_user = request
        .messages
        .iter()
        .rposition(|message| message.role == "user");
    let history: Vec<String> = request
        .messages
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != newest_user)
        .map(|(_, message)| (message.role.to_string(), text_parts(&message.content, true)))
        .filter(|(_, text)| !text.trim().is_empty())
        .map(|(role, text)| format!("{}: {text}", label(&role)))
        .collect();

    if !history.is_empty() {
        out.push_str("Conversation so far:\n");
        out.push_str(&history.join("\n"));
        out.push_str("\n\n");
    }

    out.push_str("New message:\n");
    if let Some(index) = newest_user {
        out.push_str(&text_parts(&request.messages[index].content, true));
    }
    out
}

fn label(role: &str) -> String {
    match role {
        "assistant" => "Assistant".to_string(),
        "user" => "User".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn text_parts(content: &Value, images_as_placeholder: bool) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => {
            let mut chunks = Vec::new();
            for part in parts {
                match part.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        if let Some(text) = part.get("text").and_then(Value::as_str) {
                            chunks.push(text.to_string());
                        }
                    }
                    _ if images_as_placeholder => chunks.push("[image omitted]".to_string()),
                    _ => {}
                }
            }
            chunks.join("\n\n")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(messages: Vec<Value>) -> ChatRequest {
        serde_json::from_value(json!({
            "chatId": "c",
            "model": "antigravity/gemini-3.8-flash-medium",
            "useContext": false,
            "messages": messages,
        }))
        .unwrap()
    }

    fn turn(user: &str, assistant: &str) -> Vec<Value> {
        vec![
            json!({"role":"user","content":user}),
            json!({"role":"assistant","content":assistant}),
        ]
    }

    fn stored(users: usize, hash: String) -> Stored {
        Stored {
            id: "11111111-1111-1111-1111-111111111111".to_string(),
            users,
            hash,
        }
    }

    #[test]
    fn a_follow_up_resumes() {
        let stored = stored(1, fingerprint(&["hi".to_string()]));
        let request = request(
            [turn("hi", "hello"), vec![json!({"role":"user","content":"again"})]].concat(),
        );
        match plan(&request, Some(&stored)) {
            Plan::Resume { id, prompt } => {
                assert_eq!(id, stored.id);
                assert_eq!(prompt, "again");
            }
            Plan::Fresh { .. } => panic!("expected a resume"),
        }
    }

    #[test]
    fn an_edited_earlier_message_is_fresh() {
        let stored = stored(1, fingerprint(&["original".to_string()]));
        let request = request(
            [turn("edited", "ok"), vec![json!({"role":"user","content":"next"})]].concat(),
        );
        assert!(matches!(plan(&request, Some(&stored)), Plan::Fresh { .. }));
    }

    #[test]
    fn retrying_the_same_message_is_fresh() {
        let stored = stored(
            2,
            fingerprint(&["one".to_string(), "two".to_string()]),
        );
        let request = request(turn("one", "a").into_iter().chain(turn("two", "b")).collect());
        assert!(matches!(plan(&request, Some(&stored)), Plan::Fresh { .. }));
    }

    #[test]
    fn a_detour_through_another_model_is_fresh() {
        let stored = stored(1, fingerprint(&["one".to_string()]));
        let request = request(
            [
                turn("one", "a"),
                turn("two", "b"),
                vec![json!({"role":"user","content":"three"})],
            ]
            .concat(),
        );
        assert!(matches!(plan(&request, Some(&stored)), Plan::Fresh { .. }));
    }

    #[test]
    fn fresh_prompt_includes_system_history_and_new_message() {
        let mut request = request(
            turn("what is 2+2?", "4")
                .into_iter()
                .chain([json!({"role":"user","content":"and 3+3?"})])
                .collect(),
        );
        request.system = Some("Be terse.".to_string());
        let prompt = fresh_prompt(&request);
        assert!(prompt.starts_with("Be terse.\n\n"));
        assert!(prompt.contains("Conversation so far:\nUser: what is 2+2?\nAssistant: 4"));
        assert!(prompt.ends_with("New message:\nand 3+3?"));
    }

    #[test]
    fn fresh_prompt_marks_images() {
        let request = request(vec![json!({
            "role": "user",
            "content": [{"type": "image_url", "image_url": {"url": "data:x"}}],
        })]);
        assert!(fresh_prompt(&request).ends_with("New message:\n[image omitted]"));
    }

    #[test]
    fn stored_round_trips_through_json() {
        let value = stored(3, "deadbeefdeadbeef".to_string());
        assert_eq!(decode(&encode(&value)), Some(value));
    }
}
