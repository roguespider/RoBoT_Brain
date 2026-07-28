// src/memory/pipeline.rs
//! Memory Pipeline - Stores memories in Working layer for consolidation
//! 
//! Per Architecture §6.3:
//! - Consolidates working memory into permanent storage
//! - Handles the memory ingestion cycle

use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use crate::database::sqlite::SqliteDatabase;
use crate::database::models::{MemoryCard, MemoryLayer as DbMemoryLayer};
use crate::database::queries;

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
        memory.layer = DbMemoryLayer::Working;
        
        info!("Storing memory in Working layer: {}", memory.id);
        queries::insert_memory(&conn, &memory)?;
        Ok(())
    }
}
