// src/learning/working_memory/store.rs
#![allow(dead_code)]

//! WorkingMemory store implementation

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{Utc, Duration};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::learning::working_memory::{WorkingMemoryItem, MemoryItemType};
use crate::learning::working_memory::memory_state::{StateTransition, StateTransitionRecord, MemoryState};
use crate::learning::working_memory::promotion::PromotionPolicy;

/// Working memory for active context with state machine
pub struct WorkingMemory {
    items: Arc<RwLock<HashMap<String, WorkingMemoryItem>>>,
    max_items: usize,
    policy: Arc<PromotionPolicy>,
}

impl WorkingMemory {
    pub fn new(max_items: usize) -> Self {
        Self::with_policy(max_items, PromotionPolicy::default())
    }
    
    pub fn with_policy(max_items: usize, policy: PromotionPolicy) -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            max_items,
            policy: Arc::new(policy),
        }
    }
    
    pub fn policy(&self) -> &PromotionPolicy {
        &self.policy
    }
    
    pub fn set_policy(&self, policy: PromotionPolicy) {
        *Arc::make_mut(&mut self.policy.clone()) = policy;
    }

    pub async fn store(&self, key: impl Into<String>, value: impl Into<String>, item_type: MemoryItemType, importance: f32) -> Result<String> {
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
    
    pub async fn confirm(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.record_confirmation();
            return true;
        }
        false
    }
    
    pub async fn contradict(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.record_contradiction();
            return true;
        }
        false
    }
    
    pub async fn promote(&self, key: &str) -> Option<WorkingMemoryItem> {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            if item.transition(StateTransition::Promote, Some("Manual promotion".to_string())) {
                item.confidence = self.policy.calculate_confidence(
                    item.access_count,
                    item.confirmation_count,
                );
                return Some(item.clone());
            }
        }
        None
    }
    
    pub async fn reject(&self, key: &str) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            return item.transition(StateTransition::Reject, Some("Manual rejection".to_string()));
        }
        false
    }

    pub async fn set_importance(&self, key: &str, importance: f32) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.importance = importance.clamp(0.0, 1.0);
            return true;
        }
        false
    }
    
    pub async fn set_ttl(&self, key: &str, ttl_seconds: Option<u64>) -> bool {
        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(key) {
            item.ttl_seconds = ttl_seconds;
            return true;
        }
        false
    }
    
    pub async fn get_state(&self, key: &str) -> Option<MemoryState> {
        let items = self.items.read().await;
        items.get(key).map(|i| i.state)
    }
    
    pub async fn get_history(&self, key: &str) -> Option<Vec<StateTransitionRecord>> {
        let items = self.items.read().await;
        items.get(key).map(|i| i.transition_history.clone())
    }

    async fn evict_low_importance(&self, items: &mut HashMap<String, WorkingMemoryItem>) {
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
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(1000)
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
