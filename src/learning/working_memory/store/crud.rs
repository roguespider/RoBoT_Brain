use anyhow::Result;
use chrono::Utc;

use super::WorkingMemory;
use crate::learning::working_memory::{WorkingMemoryItem, MemoryItemType};

impl WorkingMemory {
    pub async fn store(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
        item_type: MemoryItemType,
        importance: f32,
    ) -> Result<String> {
        let key_str = key.into();

        {
            let items = self.items.read().await;
            if items.contains_key(&key_str) {
                drop(items);
                return self.update(&key_str, value).await;
            }
        }

        let item = WorkingMemoryItem::new(key_str.clone(), value.into(), item_type, importance);
        let mut items = self.items.write().await;

        if items.len() >= self.max_items {
            self.evict_low_importance(&mut items).await;
        }

        items.insert(key_str.clone(), item);
        Ok(key_str)
    }

    pub async fn update(&self, key: &str, value: impl Into<String>) -> Result<String> {
        let mut items = self.items.write().await;

        if let Some(item) = items.get_mut(key) {
            item.value = value.into();
            item.accessed_at = Utc::now();
            item.access_count += 1;
            Ok(item.id.clone())
        } else {
            anyhow::bail!("Item not found: {}", key)
        }
    }

    pub async fn get(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;

        if let Some(item) = items.get_mut(key) {
            item.record_access();
            return Some(item.clone());
        }

        None
    }

    pub async fn peek(&self, key: &str) -> Option<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.get(key).cloned()
    }

    pub async fn contains(&self, key: &str) -> bool {
        let items = self.items.read().await;
        items.contains_key(key)
    }

    pub async fn remove(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        items.remove(key)
    }

    pub async fn remove_many(&self, keys: &[&str]) -> usize {
        let mut items = self.items.write().await;
        let mut removed = 0;
        for key in keys {
            if items.remove(*key).is_some() {
                removed += 1;
            }
        }
        removed
    }
}
