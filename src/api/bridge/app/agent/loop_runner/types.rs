// src/api/bridge/app/agent/loop_runner/types.rs
//! Agent loop types and outcomes

/// What the loop produced for a goal.
#[derive(Clone)]
pub struct AgentLoopOutcome {
    pub goal_id: String,
    pub status: super::super::types::GoalStatus,
    /// The action that was selected (if any), with its confidence.
    pub action_description: Option<String>,
    pub confidence_value: Option<f32>,
    /// Why the loop abstained, when it did (safety block or low confidence).
    pub abstain_reason: Option<String>,
    /// The experience id recorded for this goal attempt.
    pub experience_id: Option<String>,
}

impl std::fmt::Debug for AgentLoopOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentLoopOutcome")
            .field("goal_id", &self.goal_id)
            .field("status", &self.status)
            .field("action_description", &self.action_description)
            .field("confidence_value", &self.confidence_value)
            .field("abstain_reason", &self.abstain_reason)
            .field("experience_id", &self.experience_id)
            .finish()
    }
}
