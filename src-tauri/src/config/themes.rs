use std::collections::BTreeMap;

use serde::Serialize;

use crate::paths;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTheme {
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}

pub fn load_user() -> Vec<UserTheme> {
    let Ok(entries) = std::fs::read_dir(paths::themes_dir()) else {
        return Vec::new();
    };

    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(tokens) = serde_json::from_str::<BTreeMap<String, String>>(&contents) else {
            continue;
        };
        themes.push(UserTheme {
            name: name.to_string(),
            tokens,
        });
    }

    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes
}
