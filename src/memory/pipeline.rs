// src/memory/pipeline.rs
//! Memory Pipeline - Coordinates Working → Permanent memory flow
//! 
//! Per Architecture §6.3:
//! - Consolidates working memory into permanent storage
//! - Evaluates memories for promotion based on access patterns
//! - Handles the reflection and memory update cycle

#![allow(dead_code)]

use std::sync::Arc;
use chrono::{Duration, Utc};
use anyhow::Result;
use tracing::{info, debug};

use crate::database::sqlite::SqliteDatabase;
use crate::database::models::{MemoryCard, MemoryLayer as DbMemoryLayer};
use crate::database::queries;

/// Consolidation criteria for promoting memories to Permanent layer
#[derive(Debug, Clone)]
pub struct ConsolidationCriteria {
    /// Minimum access count to consider for promotion
    pub min_access_count: u32,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Minimum importance threshold  
    pub min_importance: f32,
    /// Maximum age in working memory before forced consolidation
    pub max_working_age: Duration,
}

impl Default for ConsolidationCriteria {
    fn default() -> Self {
        Self {
            min_access_count: 3,
            min_confidence: 0.6,
            min_importance: 0.5,
            max_working_age: Duration::hours(24),
        }
    }
}

/// Result of evaluating a memory for consolidation
#[derive(Debug, Clone)]
pub enum ConsolidationDecision {
    /// Promote to permanent layer
    Promote,
    /// Keep in working layer
    KeepWorking,
    /// Archive (remove from active use)
    Archive,
    /// Delete entirely
    Delete,
}

/// Memory Pipeline - Coordinates memory flow between layers
pub struct MemoryPipeline {
    database: Arc<SqliteDatabase>,
    criteria: ConsolidationCriteria,
}

impl MemoryPipeline {
    /// Create a new MemoryPipeline
    pub fn new(database: Arc<SqliteDatabase>) -> Self {
        Self {
            database,
            criteria: ConsolidationCriteria::default(),
        }
    }

    /// Create with custom consolidation criteria
    pub fn with_criteria(database: Arc<SqliteDatabase>, criteria: ConsolidationCriteria) -> Self {
        Self { database, criteria }
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

    /// Store a memory directly in Permanent layer (curated content)
    pub fn store_permanent(&self, memory: &MemoryCard) -> Result<()> {
        let conn = self.database.connection()?;
        
        let mut memory = memory.clone();
        memory.layer = DbMemoryLayer::Permanent;
        memory.confidence = (memory.confidence + 0.2).min(1.0); // Boost confidence
        
        info!("Storing memory in Permanent layer: {}", memory.id);
        queries::insert_memory(&conn, &memory)?;
        Ok(())
    }

    /// Evaluate a memory for consolidation
    pub fn evaluate_for_consolidation(&self, memory: &MemoryCard) -> ConsolidationDecision {
        let age = Utc::now() - memory.created_at;
        
        // Forced consolidation if too old
        if age > self.criteria.max_working_age {
            if memory.confidence >= self.criteria.min_confidence 
                && memory.importance >= self.criteria.min_importance 
                && memory.access_count >= self.criteria.min_access_count
            {
                debug!("Memory {} eligible for promotion (forced consolidation)", memory.id);
                return ConsolidationDecision::Promote;
            } else if memory.access_count == 0 {
                debug!("Memory {} being archived (never accessed)", memory.id);
                return ConsolidationDecision::Archive;
            } else {
                return ConsolidationDecision::KeepWorking;
            }
        }

        // High-value memories get promoted
        if memory.access_count >= self.criteria.min_access_count * 2
            && memory.confidence >= self.criteria.min_confidence
            && memory.importance >= self.criteria.min_importance
        {
            info!("Memory {} promoted to Permanent (high value)", memory.id);
            return ConsolidationDecision::Promote;
        }

        // Frequently accessed with good metrics
        if memory.access_count >= self.criteria.min_access_count
            && memory.confidence >= self.criteria.min_confidence
        {
            info!("Memory {} promoted to Permanent (frequently accessed)", memory.id);
            return ConsolidationDecision::Promote;
        }

        // Never accessed old memories get archived
        if memory.access_count == 0 && age > Duration::hours(12) {
            debug!("Memory {} being archived (never accessed)", memory.id);
            return ConsolidationDecision::Archive;
        }

        ConsolidationDecision::KeepWorking
    }

    /// Consolidate a memory based on evaluation
    pub fn consolidate_memory(&self, memory: &mut MemoryCard) -> Result<ConsolidationDecision> {
        let decision = self.evaluate_for_consolidation(memory);
        
        match decision {
            ConsolidationDecision::Promote => {
                memory.promote_to_permanent();
                let conn = self.database.connection()?;
                queries::insert_memory(&conn, memory)?;
                info!("Memory {} promoted to Permanent layer", memory.id);
            }
            ConsolidationDecision::Archive => {
                memory.layer = DbMemoryLayer::Permanent;
                memory.confidence *= 0.8; // Reduce confidence
                let conn = self.database.connection()?;
                queries::insert_memory(&conn, memory)?;
                debug!("Memory {} archived", memory.id);
            }
            ConsolidationDecision::KeepWorking => {
                debug!("Memory {} kept in Working layer", memory.id);
            }
            ConsolidationDecision::Delete => {
                let conn = self.database.connection()?;
                queries::delete_memories(&conn, &[memory.id])?;
                info!("Memory {} deleted", memory.id);
            }
        }

        Ok(decision)
    }

    /// Run consolidation on all working memories
    pub fn consolidate_all(&self) -> Result<ConsolidationStats> {
        let conn = self.database.connection()?;
        
        let working_memories = queries::list_memories_by_layer(&conn, "working", 1000)?;
        let mut stats = ConsolidationStats::default();
        
        for mut memory in working_memories {
            let decision = self.consolidate_memory(&mut memory)?;
            match decision {
                ConsolidationDecision::Promote => stats.promoted += 1,
                ConsolidationDecision::Archive => stats.archived += 1,
                ConsolidationDecision::Delete => stats.deleted += 1,
                ConsolidationDecision::KeepWorking => stats.kept += 1,
            }
        }
        
        info!("Consolidation complete: {} promoted, {} archived, {} kept", 
              stats.promoted, stats.archived, stats.kept);
        
        Ok(stats)
    }

    /// Record an access to a memory (updates access count)
    pub fn record_access(&self, memory_id: &str) -> Result<()> {
        let conn = self.database.connection()?;
        
        if let Some(mut memory) = queries::get_memory(&conn, uuid::Uuid::parse_str(memory_id)?)? {
            memory.record_access();
            queries::insert_memory(&conn, &memory)?;
        }
        
        Ok(())
    }

    /// Get statistics about memory layers
    pub fn get_stats(&self) -> Result<PipelineStats> {
        let conn = self.database.connection()?;
        
        let working = queries::list_memories_by_layer(&conn, "working", 10000)?;
        let permanent = queries::list_memories_by_layer(&conn, "permanent", 10000)?;
        
        Ok(PipelineStats {
            working_count: working.len(),
            permanent_count: permanent.len(),
            avg_working_confidence: if !working.is_empty() {
                working.iter().map(|m| m.confidence).sum::<f32>() / working.len() as f32
            } else { 0.0 },
            avg_permanent_confidence: if !permanent.is_empty() {
                permanent.iter().map(|m| m.confidence).sum::<f32>() / permanent.len() as f32
            } else { 0.0 },
        })
    }
    
    /// Run consolidation synchronously (for use in scheduled tasks)
    pub fn run_consolidation_sync(&self) -> Result<ConsolidationStats> {
        self.consolidate_all()
    }
}

/// Statistics from a consolidation run
#[derive(Debug, Default, Clone)]
pub struct ConsolidationStats {
    pub promoted: usize,
    pub archived: usize,
    pub deleted: usize,
    pub kept: usize,
}

/// Statistics about the memory pipeline
#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub working_count: usize,
    pub permanent_count: usize,
    pub avg_working_confidence: f32,
    pub avg_permanent_confidence: f32,
}

/// Create a task handler for memory consolidation
pub fn create_consolidation_task_handler(database: Arc<SqliteDatabase>) -> impl Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> + Send + Sync + 'static {
    move || {
        let db = database.clone();
        Box::pin(async move {
            let pipeline = MemoryPipeline::new(db);
            tracing::info!("Starting memory consolidation...");
            
            match pipeline.run_consolidation_sync() {
                Ok(stats) => {
                    tracing::info!(
                        "Memory consolidation complete: {} promoted, {} archived, {} deleted, {} kept",
                        stats.promoted, stats.archived, stats.deleted, stats.kept
                    );
                }
                Err(e) => {
                    tracing::error!("Memory consolidation failed: {}", e);
                    return Err(e);
                }
            }
            
            Ok(())
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{MemoryCard, MemoryType};

    #[test]
    fn test_consolidation_decision_promotion() {
        let pipeline = MemoryPipeline::new(Arc::new(SqliteDatabase::initialize().unwrap()));
        
        let memory = MemoryCard::new(
            "Important fact that was accessed multiple times".to_string(),
            MemoryType::Fact,
        );
        
        // High access count should promote
        let decision = pipeline.evaluate_for_consolidation(&memory);
        assert!(matches!(decision, ConsolidationDecision::KeepWorking));
    }
}
