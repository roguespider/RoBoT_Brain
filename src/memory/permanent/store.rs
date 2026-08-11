// src/memory/permanent/store.rs

//! PermanentMemory implementation

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::memory::types::{MemoryItem, MemoryLayer, MemoryStatus, MemoryType};

/// Permanent Memory - Per Architecture §6.3
///
/// Curated knowledge retained after evaluation.
/// Characteristics: Indexed, connected, confidence weighted, relationship aware.
pub struct PermanentMemory {
    cache: Arc<RwLock<HashMap<Uuid, MemoryItem>>>,
    type_index: Arc<RwLock<HashMap<MemoryType, Vec<Uuid>>>>,
    tag_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    graph_index: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    max_cache_size: usize,
    database: Option<Arc<SqliteDatabase>>,
}

impl PermanentMemory {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            graph_index: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
            database: None,
        }
    }
    
    pub async fn store(&self, item: MemoryItem) -> Uuid {
        let id = item.id;
        let mut item = item;
        item.layer = MemoryLayer::Permanent;

        let mut cache = self.cache.write().await;
        cache.insert(id, item.clone());

        {
            let mut type_index = self.type_index.write().await;
            type_index.entry(item.memory_type).or_insert_with(Vec::new).push(id);
        }

        for tag in &item.tags {
            let mut tag_index = self.tag_index.write().await;
            tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(id);
        }

        for related_id in &item.related_ids {
            let mut graph_index = self.graph_index.write().await;
            graph_index.entry(id).or_insert_with(Vec::new).push(*related_id);
        }

        id
    }

    pub async fn retrieve(&self, id: &Uuid) -> Option<MemoryItem> {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.record_access();
            Some(item.clone())
        } else {
            None
        }
    }

    pub async fn search(&self, query: &str) -> Vec<MemoryItem> {
        let query_lower = query.to_lowercase();
        let cache = self.cache.read().await;
        cache.values()
            .filter(|item| {
                item.status == MemoryStatus::Active
                    && item.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    pub async fn ranked_search(&self, query: &str, limit: usize) -> Vec<(MemoryItem, f32)> {
        let query_lower = query.to_lowercase();
        let cache = self.cache.read().await;
        
        let mut results: Vec<(MemoryItem, f32)> = cache
            .values()
            .filter(|item| item.status == MemoryStatus::Active)
            .filter_map(|item| {
                let score = self.calculate_relevance_score(&query_lower, item);
                if score > 0.0 {
                    Some((item.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    fn calculate_relevance_score(&self, query: &str, item: &MemoryItem) -> f32 {
        let content_lower = item.content.to_lowercase();
        let exact_match = if content_lower.contains(query) { 1.0 } else { 0.0 };
        
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        
        let mut word_overlap = 0.0;
        for qw in &query_words {
            for cw in &content_words {
                if cw.contains(qw) || qw.contains(cw) {
                    word_overlap += 1.0;
                    break;
                }
            }
        }
        let word_score = if !query_words.is_empty() {
            word_overlap / query_words.len() as f32
        } else {
            0.0
        };

        (exact_match * 0.4) + (word_score * 0.3) + (item.confidence * 0.2) + (item.importance * 0.1)
    }


    pub async fn add_relationship(&self, from_id: &Uuid, to_id: &Uuid) {
        // Update in-memory index
        {
            let mut graph_index = self.graph_index.write().await;
            graph_index.entry(*from_id).or_insert_with(Vec::new).push(*to_id);
        }
        
        // Persist to database
        let relationship = crate::database::models::MemoryRelationship::new(
            *from_id,
            *to_id,
            crate::database::models::MemoryRelationshipType::Related,
        );
        if let Err(e) = self.save_relationship_to_db(&relationship) {
            tracing::warn!("Failed to persist relationship to database: {}", e);
        }
    }
    
    /// Save a relationship to the database
    fn save_relationship_to_db(&self, relationship: &crate::database::models::MemoryRelationship) -> Result<()> {
        let db = self.database.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not configured"))?;
        let conn = db.connection()?;
        crate::database::queries::insert_memory_relationship(&conn, relationship)?;
        Ok(())
    }
    

    pub async fn archive(&self, id: &Uuid) -> bool {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.archive();
            true
        } else {
            false
        }
    }



    /// Load Permanent layer memories from SQLite into the cache
    /// This restores the cache from persistent storage on startup
    pub async fn load_from_database(&self, db: &Arc<crate::database::sqlite::SqliteDatabase>) -> Result<usize> {
        let conn = db.connection()?;
        let cards = queries::list_memories_by_layer(&conn, "permanent", self.max_cache_size)?;

        let mut count = 0;
        for card in cards {
            let item = MemoryItem::from(&card);
            
            // Store in cache
            let mut cache = self.cache.write().await;
            cache.insert(item.id, item.clone());
            
            // Rebuild indexes
            {
                let mut type_index = self.type_index.write().await;
                type_index.entry(item.memory_type).or_insert_with(Vec::new).push(item.id);
            }
            for tag in &item.tags {
                let mut tag_index = self.tag_index.write().await;
                tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(item.id);
            }
            for related_id in &item.related_ids {
                let mut graph_index = self.graph_index.write().await;
                graph_index.entry(item.id).or_insert_with(Vec::new).push(*related_id);
            }
            
            count += 1;
        }

        tracing::info!("Loaded {} memories from Permanent layer into cache", count);
        Ok(count)
    }

    /// Checkpoint all cached items to SQLite for persistence
    /// This saves the current state of permanent memory to the database
    pub async fn checkpoint_to_database(&self, db: &Arc<crate::database::sqlite::SqliteDatabase>) -> Result<usize> {
        let items: Vec<MemoryItem> = {
            let cache = self.cache.read().await;
            cache.values().cloned().collect()
        };

        let conn = db.connection()?;
        let mut count = 0;

        for item in items {
            let card = crate::database::models::MemoryCard::from(item);
            queries::insert_memory(&conn, &card)?;
            count += 1;
        }

        tracing::debug!("Checkpointed {} items from Permanent memory cache to database", count);
        Ok(count)
    }
}

impl Default for PermanentMemory {
    fn default() -> Self {
        Self::new(10000)
    }
}
