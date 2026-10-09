use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use base64::Engine;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::db_err;

// Imports larger than this are rejected before being read into memory.
const MAX_IMPORT_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub path: String,
    pub mime: Option<String>,
}

// A file already copied into the attachments directory. Returned instead of the
// bytes so large pastes and imports never cross the IPC boundary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    pub id: String,
    pub path: String,
    pub mime: String,
    pub name: String,
    pub size: u64,
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
    let file = save_bytes(name, mime, &bytes)?;
    Ok(Attachment {
        id: file.id,
        path: file.path,
        mime: Some(file.mime),
    })
}

pub fn save_bytes(name: &str, mime: &str, bytes: &[u8]) -> Result<SavedFile, String> {
    let id = super::new_id("a");
    let extension = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    let dir = crate::paths::attachments_dir();
    std::fs::create_dir_all(&dir).map_err(db_err)?;
    let path = dir.join(format!("{id}{extension}"));
    std::fs::write(&path, bytes).map_err(db_err)?;

    Ok(SavedFile {
        id,
        path: path.to_string_lossy().to_string(),
        mime: mime.to_string(),
        name: name.to_string(),
        size: bytes.len() as u64,
    })
}

// Copies a user-picked file into the attachments directory. Reading and writing
// happen here so the bytes never have to round-trip through the webview.
pub fn import(path: &Path) -> Result<SavedFile, String> {
    let metadata = std::fs::metadata(path).map_err(db_err)?;
    if metadata.len() > MAX_IMPORT_BYTES {
        return Err(format!("{} is too large to attach", path.display()));
    }
    let bytes = std::fs::read(path).map_err(db_err)?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let mime = mime_for(path);
    save_bytes(name, mime, &bytes)
}

fn mime_for(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "txt" | "md" | "log" | "text" => "text/plain",
        "json" => "application/json",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "xml" => "text/xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
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

// One query for the whole chat; joining avoids the per-message N+1 that made
// rapid history navigation expensive.
pub fn list_for_chat(
    conn: &Connection,
    chat_id: &str,
) -> Result<HashMap<String, Vec<Attachment>>, String> {
    let mut statement = conn
        .prepare(
            "SELECT a.message_id, a.id, a.path, a.mime FROM attachments a
             JOIN messages m ON m.id = a.message_id
             WHERE m.chat_id = ?1",
        )
        .map_err(db_err)?;
    let rows = statement
        .query_map(params![chat_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                Attachment {
                    id: row.get(1)?,
                    path: row.get(2)?,
                    mime: row.get(3)?,
                },
            ))
        })
        .map_err(db_err)?;

    let mut grouped: HashMap<String, Vec<Attachment>> = HashMap::new();
    for row in rows {
        let (message_id, attachment) = row.map_err(db_err)?;
        grouped.entry(message_id).or_default().push(attachment);
    }
    Ok(grouped)
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

    #[test]
    fn infers_mime_from_the_extension() {
        assert_eq!(mime_for(std::path::Path::new("a.png")), "image/png");
        assert_eq!(mime_for(std::path::Path::new("a.JPG")), "image/jpeg");
        assert_eq!(mime_for(std::path::Path::new("notes.md")), "text/plain");
        assert_eq!(mime_for(std::path::Path::new("noext")), "application/octet-stream");
    }

    #[test]
    fn list_for_chat_groups_attachments_by_message() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = crate::store::chats::create(&conn, "default").unwrap();
        let other = crate::store::chats::create(&conn, "default").unwrap();

        let first = messages::append(&conn, &new_message(&chat.id, "user", "one", vec![])).unwrap();
        let second = messages::append(
            &conn,
            &new_message(
                &chat.id,
                "user",
                "two",
                vec![file_attachment("a1", std::path::Path::new("/tmp/a1")), file_attachment("a2", std::path::Path::new("/tmp/a2"))],
            ),
        )
        .unwrap();
        messages::append(
            &conn,
            &new_message(
                &other.id,
                "user",
                "elsewhere",
                vec![file_attachment("a3", std::path::Path::new("/tmp/a3"))],
            ),
        )
        .unwrap();

        let grouped = list_for_chat(&conn, &chat.id).unwrap();
        assert!(!grouped.contains_key(&first.id));
        assert_eq!(grouped.get(&second.id).unwrap().len(), 2);
        assert!(!grouped.values().flatten().any(|a| a.id == "a3"));
    }
}
