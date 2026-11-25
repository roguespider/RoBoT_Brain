// src/planning/engine/planner/mod.rs
//! Core planning engine implementation
//!
//! Per Architecture §2.8:
//! Planning converts knowledge and goals into action.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::experience::metrics::MetricsCollector;
use crate::planning::engine::types::{Plan, PlannerPolicy, PlannerStatistics};
use crate::planning::policy::PolicyEngine;

/// Core planning engine
///
/// Per Architecture §2.8:
/// Planning converts knowledge and goals into action.
pub struct Planner {
    metrics: Arc<MetricsCollector>,
    active_plans: Arc<RwLock<HashMap<String, Plan>>>,
    policy: Arc<tokio::sync::RwLock<PlannerPolicy>>,
    policy_engine: Option<Arc<PolicyEngine>>,
    creativity_check: Option<Arc<dyn Fn(f32) -> bool + Send + Sync>>,
    stats: Arc<RwLock<PlannerStatistics>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
            policy_engine: None,
            creativity_check: None,
            stats: Arc::new(RwLock::new(PlannerStatistics::default())),
        }
    }

    /// Set the creativity decision callback.
    /// This connects the planner to the personality system so that
    /// creative personality traits can influence replanning behavior.
    pub fn set_creativity_check(&mut self, check: impl Fn(f32) -> bool + Send + Sync + 'static) {
        self.creativity_check = Some(Arc::new(check));
    }

    /// Check if a creative approach should be used for replanning.
    /// Returns true when the personality system determines creativity is warranted
    /// given the problem complexity.
    fn should_use_creativity(&self, problem_complexity: f32) -> bool {
        self.creativity_check
            .as_ref()
            .map(|check| check(problem_complexity))
            .unwrap_or(false)
    }

    /// Update planning policy
    pub async fn update_policy(&self, policy: PlannerPolicy) {
        let mut current = self.policy.write().await;
        *current = policy;
    }

    /// Get current planning policy
    pub async fn get_policy(&self) -> PlannerPolicy {
        self.policy.read().await.clone()
    }

    /// Attach the policy engine for policy-based decision gating
    pub fn set_policy_engine(&mut self, engine: Arc<PolicyEngine>) {
        self.policy_engine = Some(engine);
    }
}

// --- Submodules ---

pub mod conversion;
pub mod decompose;
pub mod lifecycle;
pub mod maintenance;
pub mod replanning;
pub mod seed;
pub mod selection;
