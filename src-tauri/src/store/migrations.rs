use rusqlite::Connection;

use super::db_err;

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

const SCHEMA_V2: &str = "
ALTER TABLE messages ADD COLUMN sources TEXT;
CREATE TABLE search_usage (
  month TEXT PRIMARY KEY,
  requests INTEGER NOT NULL DEFAULT 0,
  cost REAL NOT NULL DEFAULT 0
);
";

const SCHEMA_V3: &str = "
CREATE TABLE search_usage_v3 (
  month TEXT NOT NULL,
  provider TEXT NOT NULL,
  requests INTEGER NOT NULL DEFAULT 0,
  units INTEGER NOT NULL DEFAULT 0,
  cost REAL NOT NULL DEFAULT 0,
  PRIMARY KEY (month, provider)
);
INSERT INTO search_usage_v3 (month, provider, requests, units, cost)
  SELECT month, 'exa', requests, requests, cost FROM search_usage;
DROP TABLE search_usage;
ALTER TABLE search_usage_v3 RENAME TO search_usage;
CREATE TABLE search_exhausted (provider TEXT PRIMARY KEY, month TEXT NOT NULL);
";

const SCHEMA_V4: &str = "ALTER TABLE messages ADD COLUMN provider TEXT;";

pub(crate) fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(db_err)?;

    let migrations: [(i64, &str); 4] = [
        (1, SCHEMA_V1),
        (2, SCHEMA_V2),
        (3, SCHEMA_V3),
        (4, SCHEMA_V4),
    ];
    for (target, schema) in migrations {
        if version < target {
            conn.execute_batch(schema)
                .map_err(|error| format!("migrating schema: {error}"))?;
            conn.pragma_update(None, "user_version", target)
                .map_err(db_err)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v3_preserves_exa_usage_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA_V1).unwrap();
        conn.execute_batch(SCHEMA_V2).unwrap();
        conn.pragma_update(None, "user_version", 2).unwrap();
        conn.execute(
            "INSERT INTO search_usage (month, requests, cost) VALUES ('2024-01', 7, 1.5)",
            [],
        )
        .unwrap();

        migrate(&conn).unwrap();

        let (provider, requests, units, cost): (String, i64, i64, f64) = conn
            .query_row(
                "SELECT provider, requests, units, cost FROM search_usage WHERE month = '2024-01'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(provider, "exa");
        assert_eq!((requests, units, cost), (7, 7, 1.5));

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 4);
    }

    #[test]
    fn v4_adds_provider_without_touching_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA_V1).unwrap();
        conn.execute_batch(SCHEMA_V2).unwrap();
        conn.execute_batch(SCHEMA_V3).unwrap();
        conn.pragma_update(None, "user_version", 3).unwrap();
        conn.execute(
            "INSERT INTO chats (id, title, profile, pinned, created_at, updated_at)
             VALUES ('c', 't', 'p', 0, 0, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, chat_id, role, content, created_at)
             VALUES ('m', 'c', 'assistant', 'hi', 1)",
            [],
        )
        .unwrap();

        migrate(&conn).unwrap();

        let provider: Option<String> = conn
            .query_row("SELECT provider FROM messages WHERE id = 'm'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(provider, None);

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 4);
    }
}
