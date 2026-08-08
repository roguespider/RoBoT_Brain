// src/database/queries/relationships.rs
//! Memory relationship database operations

use anyhow::Result;
use rusqlite::{Connection, params};

use crate::database::models::MemoryRelationship;

/// Insert a memory relationship
pub fn insert_memory_relationship(conn: &Connection, relationship: &MemoryRelationship) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO memory_relationships (id, memory_id, related_id, relationship_type)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            relationship.id.to_string(),
            relationship.memory_id.to_string(),
            relationship.related_id.to_string(),
            relationship.relationship_type.to_string(),
        ],
    )?;
    Ok(())
}
