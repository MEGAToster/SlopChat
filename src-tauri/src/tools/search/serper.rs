use serde_json::{json, Value};

use super::hit::{text, Batch, Hit};
use super::{BackendCtx, SearchError, TIMEOUT};

pub async fn search(query: &str, n: u32, ctx: &BackendCtx<'_>) -> Result<Batch, SearchError> {
    let body = json!({ "q": query, "num": n });

    let value = super::send_json(
        crate::providers::client()
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", ctx.key)
            .timeout(TIMEOUT)
            .json(&body),
    )
    .await?;

    Ok(parse(&value))
}

fn parse(value: &Value) -> Batch {
    let units = value
        .get("credits")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;

    let mut hits = Vec::new();

    if let Some(answer) = value.pointer("/answerBox/answer").and_then(Value::as_str) {
        hits.push(lead("Answer", answer));
    } else if let Some(snippet) = value.pointer("/answerBox/snippet").and_then(Value::as_str) {
        hits.push(lead("Answer", snippet));
    }
    if let Some(description) = value
        .pointer("/knowledgeGraph/description")
        .and_then(Value::as_str)
    {
        hits.push(lead("Knowledge graph", description));
    }

    if let Some(organic) = value.get("organic").and_then(Value::as_array) {
        for item in organic {
            hits.push(Hit {
                title: text(item, "title"),
                url: text(item, "link"),
                snippet: text(item, "snippet"),
                published: item.get("date").and_then(Value::as_str).map(str::to_string),
            });
        }
    }

    Batch {
        hits,
        units,
        cost_usd: 0.0,
    }
}

fn lead(title: &str, snippet: &str) -> Hit {
    Hit {
        title: title.to_string(),
        url: String::new(),
        snippet: snippet.to_string(),
        published: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepends_the_answer_box_and_reads_credits() {
        let value: Value = serde_json::from_str(
            r#"{"credits":2,"answerBox":{"answer":"42"},"organic":[
                {"title":"One","link":"https://a.test","snippet":"first","date":"Jan 1"},
                {"title":"Two","link":"https://b.test","snippet":"second"}
            ]}"#,
        )
        .unwrap();

        let batch = parse(&value);
        assert_eq!(batch.units, 2);
        assert_eq!(batch.hits[0].title, "Answer");
        assert_eq!(batch.hits[0].snippet, "42");
        assert_eq!(batch.hits[1].title, "One");
        assert_eq!(batch.hits[1].url, "https://a.test");
        assert_eq!(batch.hits[1].published.as_deref(), Some("Jan 1"));
        assert_eq!(batch.hits[2].snippet, "second");
    }
}
