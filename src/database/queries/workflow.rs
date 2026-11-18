// src/database/queries/workflow.rs
//! Query functions for the workflow table.

use anyhow::Result;
use rusqlite::Connection;

/// Insert a workflow record.
pub fn insert_workflow(
    conn: &Connection,
    id: &str,
    name: &str,
    steps: &str,
    dependencies: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO workflow_schema (id, name, steps, dependencies) \
         VALUES (?1, ?2, ?3, ?4)",
        [id, name, steps, dependencies],
    )?;
    Ok(())
}

/// Query workflow by name.
pub fn query_workflow_by_name(conn: &Connection, name: &str) -> Result<Option<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT steps, dependencies FROM workflow_schema WHERE name = ?1 LIMIT 1")?;
    let row = stmt.query_row([name], |row| Ok((row.get(0)?, row.get(1)?)));
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List all workflows.
pub fn list_workflows(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT name, dependencies FROM workflow_schema ORDER BY name")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query workflows with dependencies.
pub fn query_workflows_with_deps(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, dependencies FROM workflow_schema WHERE dependencies IS NOT NULL AND dependencies != '' ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
