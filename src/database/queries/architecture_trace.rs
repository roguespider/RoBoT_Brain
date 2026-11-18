// src/database/queries/architecture_trace.rs
//! Query functions for the architecture_trace table.

use anyhow::Result;
use rusqlite::Connection;

/// Insert an architecture_trace record.
pub fn insert_architecture_trace(
    conn: &Connection,
    id: &str,
    function_name: &str,
    callers: &str,
    dependencies: &str,
    data_flow: &str,
    trace_id: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO architecture_trace (id, function_name, callers, dependencies, data_flow, trace_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [id, function_name, callers, dependencies, data_flow, trace_id],
    )?;
    Ok(())
}

/// Query traces by function.
pub fn query_traces_by_function(conn: &Connection, function_name: &str) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT callers, data_flow FROM architecture_trace WHERE function_name = ?1",
    )?;
    let rows = stmt.query_map([function_name], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
        ))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query trace by trace_id.
pub fn query_trace_by_id(conn: &Connection, trace_id: &str) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT function_name, data_flow FROM architecture_trace WHERE trace_id = ?1 LIMIT 1",
    )?;
    let row = stmt.query_row([trace_id], |row| {
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
