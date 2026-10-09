pub mod attachments;
pub mod chats;
pub mod messages;
pub mod search;

mod migrations;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

pub use attachments::Attachment;
pub use chats::{Chat, ChatView};
pub use messages::{Message, NewMessage};

pub(crate) fn db_err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("creating {}: {error}", parent.display()))?;
        }
        let conn = Connection::open(path).map_err(db_err)?;
        let store = Self::prepare(conn)?;
        // Best-effort cleanup of files left by an earlier interrupted delete.
        let _ = attachments::sweep_orphans(&store.conn());
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(db_err)?;
        Self::prepare(conn)
    }

    fn prepare(mut conn: Connection) -> Result<Self, String> {
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(db_err)?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
            .map_err(db_err)?;
        migrations::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("store lock poisoned")
    }
}

pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

// Time plus a process counter keeps ids sortable and unique within a run.
pub fn new_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}{:x}{counter:x}", now_millis())
}


#[cfg(test)]
mod tests;
