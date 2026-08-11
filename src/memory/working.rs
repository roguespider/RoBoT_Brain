// src/memory/working.rs
//! Working Memory - Per Architecture §6.3
//!
//! Working Memory contains temporary information used during active tasks.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{MemoryItem, MemoryLayer, MemoryStatus};
use crate::database::queries;

/// Working memory - Per Architecture §6.3
pub struct WorkingMemory {
    items: Arc<RwLock<HashMap<Uuid, MemoryItem>>>,
    max_items: usize,
}

impl WorkingMemory {
    /// Create a new working memory
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            max_items,
        }
    }

    /// Store an item in working memory
    pub async fn store(&self, item: MemoryItem) -> Uuid {
        let id = item.id;
        let mut items = self.items.write().await;

        let mut item = item;
        item.layer = MemoryLayer::Working;

        if items.len() >= self.max_items {
            self.evict_lru(&mut items).await;
        }

        items.insert(id, item);
        id
    }

    /// Retrieve an item from working memory
    pub async fn retrieve(&self, id: &Uuid) -> Option<MemoryItem> {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(id) {
            item.record_access();
            Some(item.clone())
        } else {
            None
        }
    }

    /// Search working memory by content
    pub async fn search(&self, query: &str) -> Vec<MemoryItem> {
        let query_lower = query.to_lowercase();
        let items = self.items.read().await;
        items
            .values()
            .filter(|item| {
                item.status == MemoryStatus::Active
                    && item.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Get all active items
    pub async fn get_all(&self) -> Vec<MemoryItem> {
        let items = self.items.read().await;
        items
            .values()
            .filter(|item| item.status == MemoryStatus::Active)
            .cloned()
            .collect()
    }

    /// Remove an item from working memory
    pub async fn remove(&self, id: &Uuid) -> bool {
        let mut items = self.items.write().await;
        items.remove(id).is_some()
    }

    /// Evict least recently used items
    async fn evict_lru(&self, items: &mut HashMap<Uuid, MemoryItem>) {
        let remove_count = (items.len() / 10).max(1);
        let mut sorted: Vec<_> = items
            .iter()
            .map(|(id, item)| (*id, item.accessed_at))
            .collect();
        sorted.sort_by_key(|(_, accessed)| *accessed);

        let ids_to_remove: Vec<Uuid> = sorted
            .into_iter()
            .take(remove_count)
            .map(|(id, _)| id)
            .collect();

        for id in ids_to_remove {
            items.remove(&id);
        }
    }

    /// Load Working layer memories from SQLite into the cache
    pub async fn load_from_database(&self, db: &Arc<crate::database::sqlite::SqliteDatabase>) -> Result<usize> {
        let conn = db.connection()?;
        let cards = queries::list_memories_by_layer(&conn, "working", self.max_items)?;

        let mut count = 0;
        for card in cards {
            let item = MemoryItem::from(&card);
            let mut items = self.items.write().await;
            items.insert(item.id, item);
            count += 1;
        }

        tracing::info!("Loaded {} memories from Working layer into cache", count);
        Ok(count)
    }

    /// Checkpoint all cached items to SQLite
    pub async fn checkpoint_to_database(&self, db: &Arc<crate::database::sqlite::SqliteDatabase>) -> Result<usize> {
        let items: Vec<MemoryItem> = {
            let items = self.items.read().await;
            items.values().cloned().collect()
        };

        let conn = db.connection()?;
        let mut count = 0;

        for item in items {
            let card = crate::database::models::MemoryCard::from(item);
            queries::insert_memory(&conn, &card)?;
            count += 1;
        }

        tracing::debug!("Checkpointed {} items from Working memory cache to database", count);
        Ok(count)
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(1000)
    }
}
