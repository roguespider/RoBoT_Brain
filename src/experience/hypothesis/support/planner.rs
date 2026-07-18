// robot/src/experience/hypothesis/support/planner.rs

//! ============================================================================
//! HYPOTHESIS PLANNER
//! ============================================================================
//!
//! Future decision-support layer.
//!
//! Converts trusted hypotheses into possible actions.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::HypothesisId;

#[derive(Debug, Clone, Default)]
pub struct HypothesisPlanner;

impl HypothesisPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn create_plan(&self, hypothesis_id: HypothesisId) -> PlanningResult {
        PlanningResult {
            hypothesis_id,

            actions: Vec::new(),

            notes: "Planning engine not implemented.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningResult {
    pub hypothesis_id: HypothesisId,

    pub actions: Vec<String>,

    pub notes: String,
}
