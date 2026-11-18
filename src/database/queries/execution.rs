// src/database/queries/execution.rs
//! Query functions for the execution table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert an execution record.
pub fn insert_execution(
    conn: &Connection,
    id: &str,
    plan_id: &str,
    action: &str,
    result: &str,
    success: bool,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO execution (id, plan_id, action, result, success) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, plan_id, action, result, success as i32],
    )?;
    Ok(())
}

/// Query executions by plan.
pub fn query_executions_by_plan(
    conn: &Connection,
    plan_id: &str,
) -> Result<Vec<(String, String, bool)>> {
    let mut stmt = conn
        .prepare("SELECT action, result, success FROM execution WHERE plan_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([plan_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get::<_, i32>(2)? != 0))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query failed executions for diagnostics.
pub fn query_failed_executions(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT plan_id, action FROM execution WHERE success = 0")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Count executions by plan.
pub fn count_executions_by_plan(conn: &Connection, plan_id: &str) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM execution WHERE plan_id = ?1",
        [plan_id],
        |row| row.get(0),
    )?;
    Ok(count)
}
