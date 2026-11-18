// src/database/queries/planning.rs
//! Query functions for the planning table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a planning record.
pub fn insert_planning(
    conn: &Connection,
    id: &str,
    plan_id: &str,
    goal: &str,
    status: &str,
    confidence: f64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO planning (id, plan_id, goal, status, confidence) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, plan_id, goal, status, confidence],
    )?;
    Ok(())
}

/// Query planning records by plan.
pub fn query_planning_by_plan(
    conn: &Connection,
    plan_id: &str,
) -> Result<Vec<(String, String, String, f64)>> {
    let mut stmt = conn
        .prepare("SELECT goal, status, confidence FROM planning WHERE plan_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([plan_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query pending plans.
pub fn query_pending_plans(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT plan_id, goal FROM planning WHERE status = 'pending' ORDER BY id")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query plan by ID.
pub fn query_plan_by_id(conn: &Connection, plan_id: &str) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare("SELECT goal, status FROM planning WHERE plan_id = ?1 LIMIT 1")?;
    let row = stmt.query_row([plan_id], |row| Ok((row.get(0)?, row.get(1)?)));
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
