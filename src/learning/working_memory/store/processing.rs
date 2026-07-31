use std::collections::HashMap;
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

use super::WorkingMemory;
use crate::learning::working_memory::{WorkingMemoryItem, MemoryItemType};
use crate::learning::working_memory::memory_state::{StateTransition, MemoryState};

impl WorkingMemory {
    pub async fn clear_by_type(&self, item_type: MemoryItemType) -> usize {
        let mut items = self.items.write().await;
        let before = items.len();
        items.retain(|_, item| item.item_type != item_type);
        before - items.len()
    }

    pub async fn clear_by_state(&self, state: MemoryState) -> usize {
        let mut items = self.items.write().await;
        let before = items.len();
        items.retain(|_, item| item.state != state);
        before - items.len()
    }

    pub async fn clear_all(&self) {
        let mut items = self.items.write().await;
        items.clear();
    }

    pub async fn process_all(&self) -> usize {
        let mut items = self.items.write().await;
        let mut transitioned = 0;
        let now = Utc::now();

        for item in items.values_mut() {
            if let Some(ttl) = item.ttl_seconds {
                let age = now - item.created_at;
                if age > Duration::seconds(ttl as i64)
                    && item.transition(StateTransition::Timeout, Some("TTL expired".to_string())) {
                        transitioned += 1;
                    }
            }

            let eval = self.policy.evaluate(item);

            if eval.should_promote
                && item.transition(StateTransition::Promote, Some("Policy promotion".to_string())) {
                    item.confidence = self.policy.calculate_confidence(
                        item.access_count,
                        item.confirmation_count,
                    );
                    transitioned += 1;
                }
        }

        transitioned
    }

    pub async fn stats(&self) -> MemoryStats {
        let items = self.items.read().await;

        let mut by_type: HashMap<MemoryItemType, usize> = HashMap::new();
        let mut by_state: HashMap<MemoryState, usize> = HashMap::new();

        for item in items.values() {
            *by_type.entry(item.item_type).or_insert(0) += 1;
            *by_state.entry(item.state).or_insert(0) += 1;
        }

        let avg_importance = if items.is_empty() {
            0.0
        } else {
            items.values().map(|i| i.importance).sum::<f32>() / items.len() as f32
        };

        let avg_confidence = if items.is_empty() {
            0.0
        } else {
            items.values().map(|i| i.confidence).sum::<f32>() / items.len() as f32
        };

        let total_accesses: u32 = items.values().map(|i| i.access_count).sum();
        let promotable: usize = items.values()
            .filter(|i| i.should_promote(&self.policy))
            .count();

        MemoryStats {
            total_items: items.len(),
            max_items: self.max_items,
            by_type,
            by_state,
            avg_importance,
            avg_confidence,
            total_accesses,
            promotable,
        }
    }

    pub(crate) async fn evict_low_importance(&self, items: &mut HashMap<String, WorkingMemoryItem>) {
        let keys_to_remove: Vec<String> = {
            let mut sorted: Vec<_> = items.iter().collect();
            sorted.sort_by(|a, b| {
                a.1.importance.partial_cmp(&b.1.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let to_remove = (items.len() / 10).max(1);
            sorted.into_iter().take(to_remove).map(|(k, _)| k.clone()).collect()
        };

        for key in keys_to_remove {
            items.remove(&key);
        }
    }
}

/// Statistics about working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_items: usize,
    pub max_items: usize,
    pub by_type: HashMap<MemoryItemType, usize>,
    pub by_state: HashMap<MemoryState, usize>,
    pub avg_importance: f32,
    pub avg_confidence: f32,
    pub total_accesses: u32,
    pub promotable: usize,
}
