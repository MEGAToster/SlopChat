use serde::Deserialize;

use super::stream::ToolCall;

#[derive(Clone, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub cost: Option<f64>,
}

pub struct PartialCall {
    pub index: u32,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
}

pub struct Parsed {
    pub content: Option<String>,
    pub reasoning: bool,
    pub tool_calls: Vec<PartialCall>,
    pub usage: Option<Usage>,
    pub provider: Option<String>,
    pub finish_reason: Option<String>,
    pub error: Option<String>,
}

pub fn merge_call(calls: &mut Vec<ToolCall>, incoming: PartialCall) {
    // Providers may send sparse indices; back-fill so `calls[index]` is safe.
    let index = incoming.index as usize;
    while calls.len() <= index {
        calls.push(ToolCall {
            id: String::new(),
            name: String::new(),
            arguments: String::new(),
        });
    }
    let call = &mut calls[index];
    if let Some(id) = incoming.id {
        if !id.is_empty() {
            call.id = id;
        }
    }
    if let Some(name) = incoming.name {
        if !name.is_empty() {
            call.name = name;
        }
    }
    if let Some(arguments) = incoming.arguments {
        call.arguments.push_str(&arguments);
    }
}

#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    error: Option<ProviderError>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Option<Delta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Deserialize)]
struct ToolCallDelta {
    #[serde(default)]
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<FunctionDelta>,
}

#[derive(Deserialize)]
struct FunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct ProviderError {
    #[serde(default)]
    message: Option<String>,
}

pub fn parse_chunk(data: &str) -> Option<Parsed> {
    let chunk: Chunk = serde_json::from_str(data).ok()?;

    let error = chunk
        .error
        .map(|error| error.message.unwrap_or_else(|| "provider error".to_string()));

    let choice = chunk.choices.into_iter().next();
    let finish_reason = choice
        .as_ref()
        .and_then(|choice| choice.finish_reason.clone());
    let delta = choice.and_then(|choice| choice.delta);

    let (content, reasoning, tool_calls) = match delta {
        Some(delta) => {
            let content = delta.content.filter(|content| !content.is_empty());
            let reasoning = delta
                .reasoning
                .is_some_and(|reasoning| !reasoning.is_empty());
            let tool_calls = delta
                .tool_calls
                .into_iter()
                .map(|call| PartialCall {
                    index: call.index,
                    id: call.id,
                    name: call.function.as_ref().and_then(|f| f.name.clone()),
                    arguments: call.function.and_then(|f| f.arguments),
                })
                .collect();
            (content, reasoning, tool_calls)
        }
        None => (None, false, Vec::new()),
    };

    Some(Parsed {
        content,
        reasoning,
        tool_calls,
        usage: chunk.usage,
        provider: chunk.provider,
        finish_reason,
        error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(chunks: &[&str]) -> Vec<ToolCall> {
        let mut calls = Vec::new();
        for data in chunks {
            let parsed = parse_chunk(data).unwrap();
            for call in parsed.tool_calls {
                merge_call(&mut calls, call);
            }
        }
        calls
    }

    #[test]
    fn accumulates_tool_call_fragments() {
        let calls = collect(&[
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"web_search","arguments":"{\"qu"}}]}}]}"#,
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"ery\":\"hi\"}"}}]}}]}"#,
        ]);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].name, "web_search");
        assert_eq!(calls[0].arguments, r#"{"query":"hi"}"#);
    }

    #[test]
    fn handles_single_chunk_and_multiple_indexes() {
        let calls = collect(&[
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"web_search","arguments":"{}"}},{"index":1,"id":"b","function":{"name":"fetch_url","arguments":"{}"}}]}}]}"#,
        ]);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id, "a");
        assert_eq!(calls[1].name, "fetch_url");
    }
}
