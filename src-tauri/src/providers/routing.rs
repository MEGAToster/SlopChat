use serde_json::{json, Map, Value};

use crate::config::{MaxPrice, Routing};

// OpenRouter-only. Merges into `provider` so the tool-routing
// `require_parameters` flag set by `kinds::body` survives.
pub fn apply(body: &mut Value, routing: &Routing) {
    let mut provider = Map::new();

    insert_list(&mut provider, "order", &routing.order);
    insert_list(&mut provider, "only", &routing.only);
    insert_list(&mut provider, "ignore", &routing.ignore);
    if let Some(allow_fallbacks) = routing.allow_fallbacks {
        provider.insert("allow_fallbacks".to_string(), json!(allow_fallbacks));
    }
    if let Some(sort) = routing.sort.as_deref().filter(|sort| !sort.is_empty()) {
        provider.insert("sort".to_string(), json!(sort));
    }
    if let Some(max_price) = &routing.max_price {
        let price = price_object(max_price);
        if !price.is_empty() {
            provider.insert("max_price".to_string(), Value::Object(price));
        }
    }

    if provider.is_empty() {
        return;
    }

    let Some(object) = body.as_object_mut() else {
        return;
    };
    let entry = object.entry("provider").or_insert_with(|| json!({}));
    if let Some(existing) = entry.as_object_mut() {
        existing.extend(provider);
    }
}

fn insert_list(provider: &mut Map<String, Value>, key: &str, values: &[String]) {
    if !values.is_empty() {
        provider.insert(key.to_string(), json!(values));
    }
}

fn price_object(price: &MaxPrice) -> Map<String, Value> {
    let mut object = Map::new();
    if let Some(prompt) = price.prompt {
        object.insert("prompt".to_string(), json!(prompt));
    }
    if let Some(completion) = price.completion {
        object.insert("completion".to_string(), json!(completion));
    }
    object
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::kinds::Kind;

    #[test]
    fn empty_routing_adds_no_provider_key() {
        let mut body = json!({ "model": "m" });
        apply(&mut body, &Routing::default());
        assert!(body.get("provider").is_none());
    }

    #[test]
    fn merges_with_tool_routing_and_keeps_require_parameters() {
        let tools = json!([{ "type": "function" }]);
        let mut body = Kind::OpenRouter.body("m", &[], None, None, Some(&tools), &json!("auto"));
        apply(
            &mut body,
            &Routing {
                only: vec!["anthropic".to_string()],
                allow_fallbacks: Some(false),
                sort: Some("price".to_string()),
                max_price: Some(MaxPrice {
                    prompt: Some(1.0),
                    completion: None,
                }),
                ..Routing::default()
            },
        );

        assert_eq!(body["provider"]["require_parameters"], json!(true));
        assert_eq!(body["provider"]["only"], json!(["anthropic"]));
        assert_eq!(body["provider"]["allow_fallbacks"], json!(false));
        assert_eq!(body["provider"]["sort"], json!("price"));
        assert_eq!(body["provider"]["max_price"]["prompt"], json!(1.0));
        assert!(body["provider"]["max_price"].get("completion").is_none());
    }

    #[test]
    fn routing_alone_creates_the_provider_key() {
        let mut body = json!({ "model": "m" });
        apply(
            &mut body,
            &Routing {
                order: vec!["openai".to_string()],
                ..Routing::default()
            },
        );
        assert_eq!(body["provider"]["order"], json!(["openai"]));
        assert!(body["provider"].get("require_parameters").is_none());
    }
}
