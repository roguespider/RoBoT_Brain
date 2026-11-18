// src/database/queries/ai_model.rs
//! Query functions for the ai_model table.

use anyhow::Result;
use rusqlite::Connection;

/// Insert an ai_model record.
pub fn insert_ai_model(
    conn: &Connection,
    id: &str,
    name: &str,
    provider: &str,
    model_type: &str,
    version: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO ai_model (id, name, provider, model_type, version) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [id, name, provider, model_type, version],
    )?;
    Ok(())
}

/// Query ai_model by provider.
pub fn query_models_by_provider(conn: &Connection, provider: &str) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, model_type, version FROM ai_model WHERE provider = ?1",
    )?;
    let rows = stmt.query_map([provider], |row| {
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

/// Query default model.
pub fn query_default_model(conn: &Connection) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, provider FROM ai_model WHERE is_default = 1 LIMIT 1",
    )?;
    let row = stmt.query_row([], |row| {
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

/// List all available models.
pub fn list_models(conn: &Connection) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, provider, model_type FROM ai_model ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
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
