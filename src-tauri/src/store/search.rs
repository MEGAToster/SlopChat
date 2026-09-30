use rusqlite::{params, Connection};

use super::chats::{self, Chat};

pub fn chats(conn: &Connection, query: &str, limit: i64) -> Result<Vec<Chat>, String> {
    let Some(match_query) = match_query(query) else {
        return Ok(Vec::new());
    };

    let mut statement = conn
        .prepare(
            "SELECT c.id, c.title, c.profile, c.pinned, c.created_at, c.updated_at
             FROM messages_fts
             JOIN messages m ON m.rowid = messages_fts.rowid
             JOIN chats c ON c.id = m.chat_id
             WHERE messages_fts MATCH ?1
             GROUP BY c.id
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let chats = statement
        .query_map(params![match_query, limit], chats::row_to_chat)
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;

    Ok(chats)
}

fn match_query(query: &str) -> Option<String> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| token.chars().filter(|c| c.is_alphanumeric()).collect())
        .filter(|token: &String| !token.is_empty())
        .collect();

    if tokens.is_empty() {
        return None;
    }

    Some(
        tokens
            .iter()
            .map(|token| format!("\"{token}\"*"))
            .collect::<Vec<_>>()
            .join(" OR "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{messages::NewMessage, Store};

    fn message(chat_id: &str, content: &str) -> NewMessage {
        NewMessage {
            chat_id: chat_id.to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            model: None,
            tokens_in: None,
            tokens_out: None,
            cost: None,
            attachments: Vec::new(),
        }
    }

    #[test]
    fn finds_chats_by_message_content() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = super::super::chats::create(&conn, "default").unwrap();
        super::super::messages::append(&conn, &message(&chat.id, "the quick brown fox")).unwrap();

        assert_eq!(chats(&conn, "brown", 10).unwrap().len(), 1);
        assert!(chats(&conn, "zzz", 10).unwrap().is_empty());
        assert!(chats(&conn, "!!!", 10).unwrap().is_empty());
    }
}
