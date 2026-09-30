pub mod chats;
pub mod messages;
pub mod search;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

pub use chats::{Chat, ChatView};
pub use messages::{Message, NewMessage};

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("creating {}: {error}", parent.display()))?;
        }
        let conn = Connection::open(path).map_err(|error| error.to_string())?;
        Self::prepare(conn)
    }

    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|error| error.to_string())?;
        Self::prepare(conn)
    }

    fn prepare(conn: Connection) -> Result<Self, String> {
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| error.to_string())?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
            .map_err(|error| error.to_string())?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("store lock poisoned")
    }
}

const SCHEMA_V1: &str = "
CREATE TABLE chats (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL DEFAULT '',
  profile TEXT NOT NULL DEFAULT '',
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  model TEXT,
  tokens_in INTEGER,
  tokens_out INTEGER,
  cost REAL,
  created_at INTEGER NOT NULL
);
CREATE INDEX messages_chat_idx ON messages(chat_id, created_at);
CREATE TABLE attachments (
  id TEXT PRIMARY KEY,
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  mime TEXT
);
CREATE VIRTUAL TABLE messages_fts USING fts5(content, content='messages', content_rowid='rowid');
CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
  INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
  INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
END;
CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
  INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
  INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
";

fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;

    if version < 1 {
        conn.execute_batch(SCHEMA_V1)
            .map_err(|error| format!("migrating schema: {error}"))?;
        conn.pragma_update(None, "user_version", 1)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

pub fn new_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}{:x}{counter:x}", now_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(chat_id: &str, role: &str, content: &str) -> NewMessage {
        NewMessage {
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            model: None,
            tokens_in: None,
            tokens_out: None,
            cost: None,
        }
    }

    #[test]
    fn migrations_create_a_working_schema() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM messages_fts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn appending_messages_indexes_them_for_search() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();
        messages::append(&conn, &message(&chat.id, "user", "hello world")).unwrap();

        assert_eq!(messages::list(&conn, &chat.id).unwrap().len(), 1);
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM messages_fts WHERE messages_fts MATCH 'hello'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1);

        let chat = chats::get(&conn, &chat.id).unwrap().unwrap();
        assert_eq!(chat.title, "hello world");
    }

    #[test]
    fn pinning_is_exclusive() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let first = chats::create(&conn, "default").unwrap();
        let second = chats::create(&conn, "default").unwrap();

        chats::toggle_pin(&conn, &first.id).unwrap();
        chats::toggle_pin(&conn, &second.id).unwrap();

        assert_eq!(chats::pinned(&conn).unwrap().unwrap().id, second.id);
        assert!(!chats::get(&conn, &first.id).unwrap().unwrap().pinned);
    }

    #[test]
    fn rewind_drops_later_messages() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();
        let first = messages::append(&conn, &message(&chat.id, "user", "one")).unwrap();
        messages::append(&conn, &message(&chat.id, "assistant", "two")).unwrap();
        messages::append(&conn, &message(&chat.id, "user", "three")).unwrap();

        messages::rewind(&conn, &chat.id, &first.id, "one edited").unwrap();

        let listed = messages::list(&conn, &chat.id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].content, "one edited");
    }
}
