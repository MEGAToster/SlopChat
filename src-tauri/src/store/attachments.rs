use std::collections::HashSet;
use std::path::PathBuf;

use base64::Engine;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::db_err;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub path: String,
    pub mime: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAttachment {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub mime: Option<String>,
}

pub fn save(name: &str, mime: &str, data: &str) -> Result<Attachment, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|error| format!("decoding attachment: {error}"))?;

    let id = super::new_id("a");
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    let dir = crate::paths::attachments_dir();
    std::fs::create_dir_all(&dir).map_err(db_err)?;
    let path = dir.join(format!("{id}{extension}"));
    std::fs::write(&path, bytes).map_err(db_err)?;

    Ok(Attachment {
        id,
        path: path.to_string_lossy().to_string(),
        mime: Some(mime.to_string()),
    })
}

pub fn read(path: &str) -> Result<String, String> {
    let dir = crate::paths::attachments_dir();
    let canonical = std::fs::canonicalize(path).map_err(db_err)?;
    let root = std::fs::canonicalize(&dir).unwrap_or(dir);
    if !canonical.starts_with(&root) {
        return Err("attachment path is outside the attachments directory".to_string());
    }

    let bytes = std::fs::read(&canonical).map_err(db_err)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn row_to_attachment(row: &Row<'_>) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: row.get(0)?,
        path: row.get(1)?,
        mime: row.get(2)?,
    })
}

pub fn insert(conn: &Connection, message_id: &str, attachment: &NewAttachment) -> Result<(), String> {
    conn.execute(
        "INSERT INTO attachments (id, message_id, path, mime) VALUES (?1, ?2, ?3, ?4)",
        params![
            attachment.id,
            message_id,
            attachment.path,
            attachment.mime
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

pub fn list_for_message(conn: &Connection, message_id: &str) -> Result<Vec<Attachment>, String> {
    let mut statement = conn
        .prepare("SELECT id, path, mime FROM attachments WHERE message_id = ?1")
        .map_err(db_err)?;
    let attachments = statement
        .query_map(params![message_id], row_to_attachment)
        .map_err(db_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_err)?;
    Ok(attachments)
}

// Collect file paths before their rows cascade away, so the files can be
// removed after the transaction commits.
pub fn paths_for_chat(conn: &Connection, chat_id: &str) -> Result<Vec<String>, String> {
    paths(
        conn,
        "SELECT a.path FROM attachments a
         JOIN messages m ON m.id = a.message_id
         WHERE m.chat_id = ?1",
        params![chat_id],
    )
}

pub fn paths_after(conn: &Connection, chat_id: &str, message_id: &str) -> Result<Vec<String>, String> {
    paths(
        conn,
        "SELECT a.path FROM attachments a
         JOIN messages m ON m.id = a.message_id
         WHERE m.chat_id = ?1
           AND m.rowid > (SELECT rowid FROM messages WHERE id = ?2)",
        params![chat_id, message_id],
    )
}

fn paths<P: rusqlite::Params>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<Vec<String>, String> {
    let mut statement = conn.prepare(sql).map_err(db_err)?;
    let rows = statement
        .query_map(params, |row| row.get::<_, String>(0))
        .map_err(db_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db_err)
}

// Best-effort: the rows are already gone, so a missing or locked file is not
// worth failing the delete over.
pub fn remove_files(paths: &[String]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

// Deletes attachment files that no row references, e.g. after a crash between
// writing a file and committing its row.
pub fn sweep_orphans(conn: &Connection) -> Result<(), String> {
    sweep_orphans_in(conn, &crate::paths::attachments_dir())
}

fn sweep_orphans_in(conn: &Connection, dir: &std::path::Path) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };

    let mut known: HashSet<PathBuf> = HashSet::new();
    let mut statement = conn.prepare("SELECT path FROM attachments").map_err(db_err)?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(db_err)?;
    for path in rows {
        if let Ok(canonical) = std::fs::canonicalize(path.map_err(db_err)?) {
            known.insert(canonical);
        }
    }

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !known.contains(&canonical) {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{messages, Store};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("slopchat-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn new_message(
        chat_id: &str,
        role: &str,
        content: &str,
        attachments: Vec<NewAttachment>,
    ) -> messages::NewMessage {
        messages::NewMessage {
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            model: None,
            tokens_in: None,
            tokens_out: None,
            cost: None,
            provider: None,
            sources: None,
            attachments,
        }
    }

    fn file_attachment(id: &str, path: &std::path::Path) -> NewAttachment {
        NewAttachment {
            id: id.to_string(),
            path: path.to_string_lossy().to_string(),
            mime: None,
        }
    }

    #[test]
    fn sweep_removes_files_without_a_row() {
        let dir = temp_dir("sweep");
        let kept = dir.join("kept.bin");
        let orphan = dir.join("orphan.bin");
        std::fs::write(&kept, b"x").unwrap();
        std::fs::write(&orphan, b"x").unwrap();

        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = crate::store::chats::create(&conn, "default").unwrap();
        let new = new_message(&chat.id, "user", "", vec![file_attachment("a", &kept)]);
        messages::append(&conn, &new).unwrap();

        sweep_orphans_in(&conn, &dir).unwrap();

        assert!(kept.exists());
        assert!(!orphan.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deleting_a_chat_removes_its_attachment_files() {
        let dir = temp_dir("attach-delete");
        let file = dir.join("a1.bin");
        std::fs::write(&file, b"x").unwrap();

        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = crate::store::chats::create(&conn, "default").unwrap();

        let new = new_message(
            &chat.id,
            "user",
            "with file",
            vec![file_attachment("a1", &file)],
        );
        messages::append(&conn, &new).unwrap();

        crate::store::chats::delete(&conn, &chat.id).unwrap();

        assert!(!file.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rewinding_removes_attachment_files_of_dropped_messages() {
        let dir = temp_dir("attach-rewind");
        let file = dir.join("b1.bin");
        std::fs::write(&file, b"x").unwrap();

        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = crate::store::chats::create(&conn, "default").unwrap();

        let first = messages::append(&conn, &new_message(&chat.id, "user", "one", vec![])).unwrap();
        messages::append(
            &conn,
            &new_message(
                &chat.id,
                "assistant",
                "two",
                vec![file_attachment("b1", &file)],
            ),
        )
        .unwrap();

        messages::rewind(&conn, &chat.id, &first.id, "one edited").unwrap();

        assert!(!file.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
