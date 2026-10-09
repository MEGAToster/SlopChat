use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::attachments::{Attachment, NewAttachment};
use super::db_err;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub cost: Option<f64>,
    pub provider: Option<String>,
    pub created_at: i64,
    pub sources: Option<serde_json::Value>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessage {
    pub chat_id: String,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub tokens_in: Option<i64>,
    #[serde(default)]
    pub tokens_out: Option<i64>,
    #[serde(default)]
    pub cost: Option<f64>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub sources: Option<serde_json::Value>,
    #[serde(default)]
    pub attachments: Vec<NewAttachment>,
}

const COLUMNS: &str =
    "id, chat_id, role, content, model, tokens_in, tokens_out, cost, provider, created_at, sources";

fn row_to_message(row: &Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        chat_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        model: row.get(4)?,
        tokens_in: row.get(5)?,
        tokens_out: row.get(6)?,
        cost: row.get(7)?,
        provider: row.get(8)?,
        created_at: row.get(9)?,
        sources: row
            .get::<_, Option<String>>(10)?
            .and_then(|text| serde_json::from_str(&text).ok()),
        attachments: Vec::new(),
    })
}

pub fn list(conn: &Connection, chat_id: &str) -> Result<Vec<Message>, String> {
    let mut messages = {
        let mut statement = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM messages WHERE chat_id = ?1 ORDER BY created_at ASC, rowid ASC"
            ))
            .map_err(db_err)?;
        let rows = statement
            .query_map(params![chat_id], row_to_message)
            .map_err(db_err)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_err)?
    };

    let mut grouped = super::attachments::list_for_chat(conn, chat_id)?;
    for message in &mut messages {
        if let Some(attachments) = grouped.remove(&message.id) {
            message.attachments = attachments;
        }
    }

    Ok(messages)
}

pub fn append(conn: &Connection, new: &NewMessage) -> Result<Message, String> {
    let mut message = Message {
        id: super::new_id("m"),
        chat_id: new.chat_id.clone(),
        role: new.role.clone(),
        content: new.content.clone(),
        model: new.model.clone(),
        tokens_in: new.tokens_in,
        tokens_out: new.tokens_out,
        cost: new.cost,
        provider: new.provider.clone(),
        created_at: super::now_millis(),
        sources: new.sources.clone(),
        attachments: Vec::new(),
    };

    let sources_json = new.sources.as_ref().map(|value| value.to_string());

    let tx = conn.unchecked_transaction().map_err(db_err)?;

    tx.execute(
        "INSERT INTO messages (id, chat_id, role, content, model, tokens_in, tokens_out, cost, provider, created_at, sources)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            message.id,
            message.chat_id,
            message.role,
            message.content,
            message.model,
            message.tokens_in,
            message.tokens_out,
            message.cost,
            message.provider,
            message.created_at,
            sources_json,
        ],
    )
    .map_err(db_err)?;

    for attachment in &new.attachments {
        super::attachments::insert(&tx, &message.id, attachment)?;
    }
    message.attachments = new
        .attachments
        .iter()
        .map(|attachment| Attachment {
            id: attachment.id.clone(),
            path: attachment.path.clone(),
            mime: attachment.mime.clone(),
        })
        .collect();

    super::chats::touch(&tx, &message.chat_id, message.created_at)?;

    if message.role == "user" {
        super::chats::set_title_if_empty(&tx, &message.chat_id, &truncate(&message.content, 60))?;
    }

    tx.commit().map_err(db_err)?;

    Ok(message)
}

pub fn rewind(
    conn: &Connection,
    chat_id: &str,
    message_id: &str,
    content: &str,
) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(db_err)?;

    let updated = tx
        .execute(
            "UPDATE messages SET content = ?1 WHERE id = ?2 AND chat_id = ?3",
            params![content, message_id, chat_id],
        )
        .map_err(db_err)?;
    if updated == 0 {
        return Err(format!("message {message_id} not found in chat {chat_id}"));
    }

    let paths = super::attachments::paths_after(&tx, chat_id, message_id)?;

    tx.execute(
        "DELETE FROM messages
         WHERE chat_id = ?1
           AND rowid > (SELECT rowid FROM messages WHERE id = ?2)",
        params![chat_id, message_id],
    )
    .map_err(db_err)?;

    super::chats::touch(&tx, chat_id, super::now_millis())?;
    tx.commit().map_err(db_err)?;

    super::attachments::remove_files(&paths);
    Ok(())
}

fn truncate(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    let mut result: String = trimmed.chars().take(limit).collect();
    if trimmed.chars().count() > limit {
        result.push('…');
    }
    result
}
