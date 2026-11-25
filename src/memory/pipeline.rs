// src/memory/pipeline.rs
//! Memory Pipeline - Stores memories in Working layer for consolidation
//!
//! Per Architecture §6.3:
//! - Consolidates working memory into permanent storage
//! - Handles the memory ingestion cycle

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use crate::database::models::MemoryCard;
use crate::database::sqlite::SqliteDatabase;

use crate::database::queries;
use crate::memory::types::MemoryLayer;

/// Memory Pipeline - Coordinates memory storage in Working layer
pub struct MemoryPipeline {
    database: Arc<SqliteDatabase>,
}

impl MemoryPipeline {
    /// Create a new MemoryPipeline
    pub fn new(database: Arc<SqliteDatabase>) -> Self {
        Self { database }
    }

    /// Store a memory in Working layer
    pub fn store_working(&self, memory: &MemoryCard) -> Result<()> {
        let conn = self.database.connection()?;

        // Ensure it's marked as Working layer
        let mut memory = memory.clone();
        memory.layer = MemoryLayer::Working;

        info!("Storing memory in Working layer: {}", memory.id);
        queries::insert_memory(&conn, &memory)?;
        Ok(())
    }
}
