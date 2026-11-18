// src/database/queries/diagnostics.rs
//! Query functions for the diagnostics table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a diagnostics record.
pub fn insert_diagnostic(
    conn: &Connection,
    id: &str,
    severity: &str,
    category: &str,
    message: &str,
    file: &str,
    line: u32,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO diagnostics (id, severity, category, message, file, line) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, severity, category, message, file, line as i64],
    )?;
    Ok(())
}

/// Query diagnostics by severity.
pub fn query_diagnostics_by_severity(
    conn: &Connection,
    severity: &str,
) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT category, message, file FROM diagnostics WHERE severity = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([severity], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query all diagnostics.
pub fn list_diagnostics(conn: &Connection) -> Result<Vec<(String, String, String, u32)>> {
    let mut stmt = conn.prepare(
        "SELECT severity, category, message, line FROM diagnostics ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Count diagnostics by severity.
pub fn count_diagnostics_by_severity(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare("SELECT severity, COUNT(*) FROM diagnostics GROUP BY severity")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
