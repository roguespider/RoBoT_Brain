use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::learning::working_memory::WorkingMemoryItem;
use crate::learning::working_memory::promotion::PromotionPolicy;

/// Working memory for active context with state machine
pub struct WorkingMemory {
    pub(super) items: Arc<RwLock<HashMap<String, WorkingMemoryItem>>>,
    pub(super) max_items: usize,
    pub(super) policy: Arc<PromotionPolicy>,
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
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(1000)
    }
}
