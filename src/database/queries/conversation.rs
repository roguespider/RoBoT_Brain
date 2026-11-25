// src/database/queries/conversation.rs
//! Query functions for the conversation table.

use anyhow::Result;
use rusqlite::Connection;

/// Insert a conversation record.
pub fn insert_conversation(
    conn: &Connection,
    id: &str,
    session_id: &str,
    user_message: &str,
    assistant_message: &str,
    timestamp: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO conversation (id, session_id, user_message, assistant_message, timestamp) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [id, session_id, user_message, assistant_message, timestamp],
    )?;
    Ok(())
}

/// Query conversations by session.
pub fn query_conversations_by_session(conn: &Connection, session_id: &str) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT user_message, assistant_message, timestamp FROM conversation \
         WHERE session_id = ?1 ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map([session_id], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
        ))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query last conversation message for a session.
pub fn query_last_message(conn: &Connection, session_id: &str) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT user_message, assistant_message FROM conversation \
         WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT 1",
    )?;
    let row = stmt.query_row([session_id], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
        ))
    });
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Count conversations in a session.
pub fn count_conversations(conn: &Connection, session_id: &str) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversation WHERE session_id = ?1",
        [session_id],
        |row| row.get(0),
    )?;
    Ok(count)
}
