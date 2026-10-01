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
