// src/memory/permanent/store.rs
//! PermanentMemory implementation

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::PermanentMemoryStats;
use crate::database::queries;
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
}

impl PermanentMemory {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            graph_index: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
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

    pub async fn link_related(&self, id1: &Uuid, id2: &Uuid) -> bool {
        let (item1, item2) = {
            let cache = self.cache.read().await;
            let i1 = cache.get(id1).cloned();
            let i2 = cache.get(id2).cloned();
            (i1, i2)
        };
        
        match (item1, item2) {
            (Some(mut i1), Some(mut i2)) => {
                i1.add_related(*id2);
                i2.add_related(*id1);
                
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(*id1, i1.clone());
                    cache.insert(*id2, i2.clone());
                }
                
                let mut graph_index = self.graph_index.write().await;
                graph_index.entry(*id1).or_insert_with(Vec::new).push(*id2);
                graph_index.entry(*id2).or_insert_with(Vec::new).push(*id1);
                
                true
            }
            _ => false,
        }
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

    pub async fn find_by_type(&self, memory_type: MemoryType) -> Vec<MemoryItem> {
        let type_index = self.type_index.read().await;
        let cache = self.cache.read().await;

        type_index.get(&memory_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| cache.get(id).cloned())
                    .filter(|item| item.status == MemoryStatus::Active)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn find_by_tag(&self, tag: &str) -> Vec<MemoryItem> {
        let tag_index = self.tag_index.read().await;
        let cache = self.cache.read().await;

        tag_index.get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| cache.get(id).cloned())
                    .filter(|item| item.status == MemoryStatus::Active)
                    .collect()
            })
            .unwrap_or_default()
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

    pub async fn find_confident(&self, min_confidence: f32) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        cache.values()
            .filter(|item| item.status == MemoryStatus::Active && item.confidence >= min_confidence)
            .cloned()
            .collect()
    }

    pub async fn get_related(&self, id: &Uuid) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        let graph_index = self.graph_index.read().await;

        if let Some(related_ids) = graph_index.get(id) {
            related_ids.iter()
                .filter_map(|rid| cache.get(rid).cloned())
                .filter(|item| item.status == MemoryStatus::Active)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_related_graph(&self, id: &Uuid, depth: usize) -> Vec<(Uuid, MemoryItem)> {
        let cache = self.cache.read().await;
        let graph_index = self.graph_index.read().await;
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        let mut queue = vec![(*id, 0)];

        while let Some((current_id, current_depth)) = queue.pop() {
            if visited.contains(&current_id) || current_depth > depth {
                continue;
            }
            visited.insert(current_id);

            if let Some(related_ids) = graph_index.get(&current_id) {
                for rid in related_ids {
                    if !visited.contains(rid) {
                        if let Some(item) = cache.get(rid) {
                            result.push((*rid, item.clone()));
                            if current_depth < depth {
                                queue.push((*rid, current_depth + 1));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    pub async fn add_relationship(&self, from_id: &Uuid, to_id: &Uuid) {
        let mut graph_index = self.graph_index.write().await;
        graph_index.entry(*from_id).or_insert_with(Vec::new).push(*to_id);
    }

    pub async fn update_confidence(&self, id: &Uuid, confidence: f32) -> bool {
        let mut cache = self.cache.write().await;
        if let Some(item) = cache.get_mut(id) {
            item.update_confidence(confidence);
            true
        } else {
            false
        }
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

    pub async fn get_all(&self) -> Vec<MemoryItem> {
        let cache = self.cache.read().await;
        cache.values()
            .filter(|item| item.status == MemoryStatus::Active)
            .cloned()
            .collect()
    }

    pub async fn stats(&self) -> PermanentMemoryStats {
        let cache = self.cache.read().await;
        let mut by_type = HashMap::new();
        let mut total_confidence = 0.0;
        let mut total_importance = 0.0;
        let mut count = 0;

        for item in cache.values().filter(|i| i.status == MemoryStatus::Active) {
            *by_type.entry(item.memory_type.to_string()).or_insert(0) += 1;
            total_confidence += item.confidence;
            total_importance += item.importance;
            count += 1;
        }

        PermanentMemoryStats {
            total_items: cache.len(),
            by_type,
            avg_confidence: if count > 0 { total_confidence / count as f32 } else { 0.0 },
            avg_importance: if count > 0 { total_importance / count as f32 } else { 0.0 },
        }
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut type_index = self.type_index.write().await;
        let mut tag_index = self.tag_index.write().await;
        let mut graph_index = self.graph_index.write().await;

        cache.clear();
        type_index.clear();
        tag_index.clear();
        graph_index.clear();
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
