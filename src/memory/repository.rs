// src/memory/repository.rs
//! Memory Repository - Per Architecture §4.06 (Repository Pattern) / §22.14
//!
//! Isolates persistence behind a repository contract so the cognitive layer
//! (memory engine, handlers) never touches SQL or table structure directly.
//!
//! Correct flow (§22.14):
//!     Memory Engine → Memory Repository → Database Layer → SQLite

use anyhow::Result;

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::memory::types::MemoryItem;

/// Repository contract for memory persistence (Architecture §4.06).
///
/// The cognitive layer knows only this trait — not SQL, table names, or
/// connection handling. This lets storage be swapped or tested in isolation.
pub trait MemoryRepository: Send + Sync {
    /// Persist a memory item (including its tags and relationships).
    fn store(&self, item: &MemoryItem) -> Result<()>;
}

/// SQLite-backed implementation of [`MemoryRepository`].
pub struct SqliteMemoryRepository {
    db: SqliteDatabase,
}

impl SqliteMemoryRepository {
    /// Create a new repository wrapping a database handle.
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }
}

impl MemoryRepository for SqliteMemoryRepository {
    fn store(&self, item: &MemoryItem) -> Result<()> {
        let conn = self.db.connection()?;

        // Insert the memory row via the database query layer (single source
        // of truth for the memories table schema).
        let card = crate::database::models::MemoryCard::from(item.clone());
        queries::insert_memory(&conn, &card)?;

        // Persist tags (Architecture §6.3: Permanent Memory is "relationship aware").
        for tag in &item.tags {
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag) VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), tag],
            )?;
        }

        // Persist relationships to related memories.
        for related_id in &item.related_ids {
            conn.execute(
                "INSERT OR IGNORE INTO memory_relationships (memory_id, related_id) \
                 VALUES (?1, ?2)",
                rusqlite::params![item.id.to_string(), related_id.to_string()],
            )?;
        }

        Ok(())
    }
}
