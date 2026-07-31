use super::WorkingMemory;
use crate::learning::working_memory::{WorkingMemoryItem, MemoryItemType};
use crate::learning::working_memory::memory_state::MemoryState;

impl WorkingMemory {
    pub async fn len(&self) -> usize {
        let items = self.items.read().await;
        items.len()
    }

    pub async fn is_empty(&self) -> bool {
        let items = self.items.read().await;
        items.is_empty()
    }

    pub async fn keys(&self) -> Vec<String> {
        let items = self.items.read().await;
        items.keys().cloned().collect()
    }

    pub async fn values(&self) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }

    pub async fn items(&self) -> Vec<(String, WorkingMemoryItem)> {
        let items = self.items.read().await;
        items.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub async fn get_by_type(&self, item_type: MemoryItemType) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.item_type == item_type)
            .cloned()
            .collect()
    }

    pub async fn get_by_state(&self, state: MemoryState) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.state == state)
            .cloned()
            .collect()
    }

    pub async fn get_promotable(&self) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|i| i.should_promote(&self.policy))
            .cloned()
            .collect()
    }

    pub async fn get_recent(&self, limit: usize) -> Vec<WorkingMemoryItem> {
        let mut items: Vec<_> = {
            let items = self.items.read().await;
            items.values().cloned().collect()
        };

        items.sort_by_key(|b| std::cmp::Reverse(b.accessed_at));
        items.truncate(limit);
        items
    }

    pub async fn get_important(&self, threshold: f32) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        let mut result: Vec<_> = items.values()
            .filter(|i| i.importance >= threshold)
            .cloned()
            .collect();

        result.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    pub async fn get_by_key_pattern(&self, pattern: &str) -> Vec<WorkingMemoryItem> {
        let items = self.items.read().await;
        let pattern_lower = pattern.to_lowercase();
        items.values()
            .filter(|i| i.key.to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect()
    }
}
