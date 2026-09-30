use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn xdg_dir(variable: &str, fallback: &str) -> PathBuf {
    match std::env::var_os(variable) {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => home().join(fallback),
    }
}

pub fn config_dir() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config").join("SlopChat")
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn context_path() -> PathBuf {
    config_dir().join("context.md")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn data_dir() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share").join("SlopChat")
}

pub fn chats_db_path() -> PathBuf {
    data_dir().join("history").join("chats.db")
}
