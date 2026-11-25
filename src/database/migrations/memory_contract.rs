// src/database/migrations/memory_contract.rs
// Migration 017: add the memory title column.
//
// The MemoryRecord contract (Architecture §5.6) carries a `title` alongside
// `content`. The `memories` table only had `content`, so a title supplied by a
// caller was silently dropped. This migration adds the column so the title
// round-trips through storage.

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "memories", "title")? {
        conn.execute_batch("ALTER TABLE memories ADD COLUMN title TEXT;")?;
    }
    Ok(())
}

/// Check whether a column already exists on a table.
fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}
