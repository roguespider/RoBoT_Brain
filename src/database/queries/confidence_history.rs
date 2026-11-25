// src/database/queries/confidence_history.rs
//! Query functions for the confidence_history table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a confidence_history record.
pub fn insert_confidence_history(
    conn: &Connection,
    id: &str,
    item_id: &str,
    old_confidence: f64,
    new_confidence: f64,
    reason: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO confidence_history (id, item_id, old_confidence, new_confidence, reason) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, item_id, old_confidence, new_confidence, reason],
    )?;
    Ok(())
}

/// Query confidence history by item.
pub fn query_confidence_history_by_item(
    conn: &Connection,
    item_id: &str,
) -> Result<Vec<(f64, f64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT old_confidence, new_confidence, reason FROM confidence_history WHERE item_id = ?1 ORDER BY timestamp",
    )?;
    let rows = stmt.query_map([item_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query recent confidence changes.
pub fn query_recent_changes(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<(String, f64, f64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT item_id, old_confidence, new_confidence, reason FROM confidence_history ORDER BY timestamp DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i32], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
