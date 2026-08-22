use super::WorkingMemory;
use crate::learning::working_memory::WorkingMemoryItem;
use crate::learning::working_memory::memory_state::{StateTransition, StateTransitionRecord, MemoryState};

impl WorkingMemory {
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
        if let Some(item) = items.get_mut(key)
            && item.transition(StateTransition::Promote, Some("Manual promotion".to_string())) {
                item.confidence = self.policy.calculate_confidence(
                    item.access_count,
                    item.confirmation_count,
                );
                return Some(item.clone());
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
}
