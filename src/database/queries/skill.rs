// src/database/queries/skill.rs
//! Query functions for the skill table.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a skill record.
pub fn insert_skill(
    conn: &Connection,
    id: &str,
    name: &str,
    category: &str,
    description: &str,
    version: &str,
    enabled: bool,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO skill (id, name, category, description, version, enabled) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, category, description, version, enabled as i32],
    )?;
    Ok(())
}

/// Update a skill's mastery level.
pub fn update_skill_mastery(
    conn: &Connection,
    id: &str,
    mastery: f64,
    success: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE skill SET mastery = ?2, last_used = datetime('now'), \
         success_count = success_count + ?3, failure_count = failure_count + (1 - ?3) \
         WHERE id = ?1",
        params![id, mastery, success as i32],
    )?;
    Ok(())
}

/// Update a skill's last_used timestamp.
pub fn update_skill_last_used(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE skill SET last_used = datetime('now') WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

/// Skill record returned by queries.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub mastery: f64,
}

/// Query a skill by name.
pub fn query_skill_by_name(conn: &Connection, name: &str) -> Result<Option<SkillRecord>> {
    let mut stmt =
        conn.prepare("SELECT id, name, category, description, mastery FROM skill WHERE name = ?1")?;
    let row = stmt.query_row([name], |row| {
        Ok(SkillRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            description: row.get(3)?,
            mastery: row.get(4)?,
        })
    });
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// List all enabled skills.
pub fn list_enabled_skills(conn: &Connection) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn
        .prepare("SELECT name, category, description FROM skill WHERE enabled = 1 ORDER BY name")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query skills by category.
pub fn query_skills_by_category(
    conn: &Connection,
    category: &str,
) -> Result<Vec<(String, String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT name, description, mastery FROM skill WHERE category = ?1 ORDER BY mastery DESC",
    )?;
    let rows = stmt.query_map([category], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
