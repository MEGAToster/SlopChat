use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub path: String,
    pub mime: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAttachment {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub mime: Option<String>,
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
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_for_message(conn: &Connection, message_id: &str) -> Result<Vec<Attachment>, String> {
    let mut statement = conn
        .prepare("SELECT id, path, mime FROM attachments WHERE message_id = ?1")
        .map_err(|error| error.to_string())?;
    let attachments = statement
        .query_map(params![message_id], row_to_attachment)
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;
    Ok(attachments)
}
