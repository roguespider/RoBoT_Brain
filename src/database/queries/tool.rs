// src/database/queries/tool.rs
//! Query functions for the tool table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a tool record.
pub fn insert_tool(
    conn: &Connection,
    id: &str,
    name: &str,
    category: &str,
    description: &str,
    enabled: bool,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO tool (id, name, category, description, enabled) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, category, description, enabled as i32],
    )?;
    Ok(())
}

/// Query tool by name.
pub fn query_tool_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Option<(String, String, String)>> {
    let mut stmt =
        conn.prepare("SELECT category, description, enabled FROM tool WHERE name = ?1")?;
    let row = stmt.query_row([name], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)));
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List all enabled tools.
pub fn list_enabled_tools(conn: &Connection) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn
        .prepare("SELECT name, category, description FROM tool WHERE enabled = 1 ORDER BY name")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query tools by category.
pub fn query_tools_by_category(conn: &Connection, category: &str) -> Result<Vec<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT name, description FROM tool WHERE category = ?1 ORDER BY name")?;
    let rows = stmt.query_map([category], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
