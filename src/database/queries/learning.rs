// src/database/queries/learning.rs
//! Query functions for the learning table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a learning record.
pub fn insert_learning(
    conn: &Connection,
    id: &str,
    topic: &str,
    strategy: &str,
    outcome: &str,
    confidence: f64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO learning (id, topic, strategy, outcome, confidence) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, topic, strategy, outcome, confidence],
    )?;
    Ok(())
}

/// Query learning records by topic.
pub fn query_learning_by_topic(
    conn: &Connection,
    topic: &str,
) -> Result<Vec<(String, String, String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT id, strategy, outcome, confidence FROM learning WHERE topic = ?1 ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([topic], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query successful learning strategies.
pub fn query_successful_strategies(conn: &Connection) -> Result<Vec<(String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT topic, MAX(confidence) FROM learning WHERE outcome = 'success' GROUP BY topic",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query failed learning strategies.
pub fn query_failed_strategies(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT topic, strategy FROM learning WHERE outcome = 'failure'")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
