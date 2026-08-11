// src/memory/repository.rs
//! Memory Repository - Per Architecture §6.3
//!
//! Provides persistence layer for memory items using SQLite.
//! Bridges in-memory structures with database storage.

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;
use crate::memory::types::{MemoryItem, MemoryType};

/// Repository trait for memory persistence (scaffolding for future use)
pub trait MemoryRepository: Send + Sync {
    /// Store a memory item
    fn store(&self, item: &MemoryItem) -> Result<()>;









}

/// Memory statistics
#[derive(Debug, Clone)]

/// SQLite implementation of MemoryRepository
pub struct SqliteMemoryRepository {
    db: SqliteDatabase,
}

impl SqliteMemoryRepository {
    /// Create a new SQLite memory repository
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// Create from database path
    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let db = SqliteDatabase::initialize_at(path)?;
        Ok(Self::new(db))
    }

}

impl MemoryRepository for SqliteMemoryRepository {
    fn store(&self, item: &MemoryItem) -> Result<()> {
        let conn = self.db.connection()?;
        conn.execute(
            "
            INSERT OR REPLACE INTO memories 
            (id, content, memory_type, confidence, importance, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            rusqlite::params![
                item.id.to_string(),
                item.content,
                memory_type_to_string(&item.memory_type),
                item.confidence,
                item.importance,
                item.created_at.to_rfc3339(),
                item.modified_at.to_rfc3339(),
            ],
        )?;

        // Store memory tags if any
        for tag in &item.tags {
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag) VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), tag],
            )?;
        }

        // Store relationships if any
        for related_id in &item.related_ids {
            conn.execute(
                "INSERT OR IGNORE INTO memory_relationships (memory_id, related_id) VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), related_id.to_string()],
            )?;
        }

        Ok(())
    }


}

// ==========================================================
// HELPERS
// ==========================================================

fn memory_type_to_string(mt: &MemoryType) -> String {
    mt.to_string()
}




