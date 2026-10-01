use rusqlite::{params, Connection, Row};
use serde::Serialize;

use super::Message;
use crate::config::Startup;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub profile: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatView {
    pub chat: Chat,
    pub messages: Vec<Message>,
}

pub(crate) fn row_to_chat(row: &Row<'_>) -> rusqlite::Result<Chat> {
    Ok(Chat {
        id: row.get(0)?,
        title: row.get(1)?,
        profile: row.get(2)?,
        pinned: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

const COLUMNS: &str = "id, title, profile, pinned, created_at, updated_at";

pub fn list(conn: &Connection) -> Result<Vec<Chat>, String> {
    let mut statement = conn
        .prepare(&format!(
            "SELECT {COLUMNS} FROM chats ORDER BY created_at ASC"
        ))
        .map_err(|error| error.to_string())?;
    let chats = statement
        .query_map([], row_to_chat)
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;
    Ok(chats)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Chat>, String> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM chats WHERE id = ?1"),
        params![id],
        row_to_chat,
    )
    .map(Some)
    .or_else(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(other.to_string()),
    })
}

pub fn most_recent(conn: &Connection) -> Result<Option<Chat>, String> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM chats ORDER BY updated_at DESC LIMIT 1"),
        [],
        row_to_chat,
    )
    .map(Some)
    .or_else(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(other.to_string()),
    })
}

pub fn pinned(conn: &Connection) -> Result<Option<Chat>, String> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM chats WHERE pinned = 1 LIMIT 1"),
        [],
        row_to_chat,
    )
    .map(Some)
    .or_else(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(other.to_string()),
    })
}

pub fn view(conn: &Connection, chat: Chat) -> Result<ChatView, String> {
    let messages = super::messages::list(conn, &chat.id)?;
    Ok(ChatView { chat, messages })
}

pub fn initial(
    conn: &Connection,
    mode: Startup,
    resume_minutes: u64,
) -> Result<Option<ChatView>, String> {
    let chat = match mode {
        Startup::New => None,
        Startup::Pinned => pinned(conn)?,
        Startup::Latest => match most_recent(conn)? {
            Some(chat) if within(&chat, resume_minutes) => Some(chat),
            _ => None,
        },
    };

    match chat {
        Some(chat) => view(conn, chat).map(Some),
        None => Ok(None),
    }
}

// Zero minutes means "always resume" — no staleness limit.
fn within(chat: &Chat, resume_minutes: u64) -> bool {
    resume_minutes == 0
        || super::now_millis() - chat.updated_at <= (resume_minutes as i64) * 60_000
}

pub fn create(conn: &Connection, profile: &str) -> Result<Chat, String> {
    let now = super::now_millis();
    let chat = Chat {
        id: super::new_id("c"),
        title: String::new(),
        profile: profile.to_string(),
        pinned: false,
        created_at: now,
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO chats (id, title, profile, pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, ?5)",
        params![chat.id, chat.title, chat.profile, chat.created_at, chat.updated_at],
    )
    .map_err(|error| error.to_string())?;
    Ok(chat)
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM chats WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_profile(conn: &Connection, id: &str, profile: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE chats SET profile = ?1 WHERE id = ?2",
        params![profile, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn toggle_pin(conn: &Connection, id: &str) -> Result<Chat, String> {
    let chat = get(conn, id)?.ok_or_else(|| format!("chat {id} not found"))?;
    let pinned = !chat.pinned;

    conn.execute("UPDATE chats SET pinned = 0", [])
        .map_err(|error| error.to_string())?;
    if pinned {
        conn.execute("UPDATE chats SET pinned = 1 WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
    }

    get(conn, id)?.ok_or_else(|| format!("chat {id} not found"))
}

pub(super) fn touch(conn: &Connection, id: &str, updated_at: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        params![updated_at, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn set_title_if_empty(conn: &Connection, id: &str, title: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE chats SET title = ?1 WHERE id = ?2 AND title = ''",
        params![title, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
