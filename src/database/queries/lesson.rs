// src/database/queries/lesson.rs
//! Query functions for the lesson table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a lesson record.
pub fn insert_lesson(
    conn: &Connection,
    id: &str,
    topic: &str,
    content: &str,
    source_experience: &str,
    confidence: f64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO lesson (id, topic, content, source_experience, confidence) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, topic, content, source_experience, confidence],
    )?;
    Ok(())
}

/// Query lessons by topic.
pub fn query_lessons_by_topic(
    conn: &Connection,
    topic: &str,
) -> Result<Vec<(String, String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, confidence FROM lesson WHERE topic = ?1 ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([topic], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query lessons by source experience.
pub fn query_lessons_by_experience(
    conn: &Connection,
    experience_id: &str,
) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT id, content FROM lesson WHERE source_experience = ?1")?;
    let rows = stmt.query_map([experience_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// List all lessons ordered by confidence.
pub fn list_lessons(conn: &Connection) -> Result<Vec<(String, String, f64)>> {
    let mut stmt =
        conn.prepare("SELECT topic, content, confidence FROM lesson ORDER BY confidence DESC")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
