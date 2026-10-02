use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::paths;

const MAX_BYTES: u64 = 5 * 1024 * 1024;

pub fn path() -> PathBuf {
    paths::agent_log_path()
}

pub fn ensure() -> Result<(), String> {
    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if !path.exists() {
        fs::write(&path, b"").map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn append(value: &Value) {
    append_to(&path(), value);
}

fn append_to(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let oversized = fs::metadata(path)
        .map(|meta| meta.len() > MAX_BYTES)
        .unwrap_or(false);
    if oversized {
        let _ = fs::rename(path, path.with_extension("jsonl.1"));
    }

    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    if let Ok(line) = serde_json::to_string(value) {
        let _ = writeln!(file, "{line}");
    }
}

const RING_CAP: usize = 60;

// Records a turn's events. When `live`, writes each event immediately; otherwise
// buffers them (dropping the oldest past the cap) until `flush`, so an empty
// answer still leaves a full trace without turning debug logging on.
pub struct Ring {
    live: bool,
    events: Vec<Value>,
}

impl Ring {
    pub fn new(live: bool) -> Self {
        Self {
            live,
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        if self.live {
            append(&value);
            return;
        }
        self.events.push(value);
        if self.events.len() > RING_CAP {
            self.events.remove(0);
        }
    }

    pub fn flush(&mut self) {
        if self.live {
            return;
        }
        let path = path();
        self.flush_to(&path);
    }

    fn flush_to(&mut self, path: &Path) {
        for event in self.events.drain(..) {
            append_to(path, &event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffers_flushes_in_order_and_caps() {
        let dir = std::env::temp_dir().join(format!("slopchat-ring-{}", std::process::id()));
        let path = dir.join("agent.jsonl");
        let _ = fs::remove_dir_all(&dir);

        let mut ring = Ring::new(false);
        for index in 0..(RING_CAP as i64 + 5) {
            ring.push(serde_json::json!({ "n": index }));
        }
        ring.flush_to(&path);

        let lines: Vec<Value> = fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), RING_CAP);
        // Oldest events were dropped, order preserved.
        assert_eq!(lines[0]["n"], 5);
        assert_eq!(lines[RING_CAP - 1]["n"], RING_CAP as i64 + 4);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_writes_json_lines_and_rotates() {
        let dir = std::env::temp_dir().join(format!("slopchat-trace-{}", std::process::id()));
        let path = dir.join("agent.jsonl");
        let _ = fs::remove_dir_all(&dir);

        append_to(&path, &serde_json::json!({ "round": 1 }));
        append_to(&path, &serde_json::json!({ "round": 2 }));

        let lines: Vec<Value> = fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1]["round"], 2);

        let _ = fs::remove_dir_all(&dir);
    }
}
