//! Action selection logic for the planning engine

use super::Planner;

use crate::planning::engine::actions::{score_action, select_best_scored};
use crate::planning::engine::types::ActionCandidate;
use crate::planning::policy::{PolicyContext, PolicyResult};

impl Planner {
    /// Select the best next action based on knowledge and confidence
    ///
    /// Per Architecture §5.7:
    /// "Action Selection"
    pub async fn select_best_action(
        &self,
        actions: Vec<ActionCandidate>,
    ) -> Option<ActionCandidate> {
        if actions.is_empty() {
            return None;
        }

        let policy = self.policy.read().await;

        // Evaluate policy rules against the first candidate to determine gating
        if let Some(ref engine) = self.policy_engine {
            let first = &actions[0];
            let context = PolicyContext {
                task_type: Some("action_selection".to_string()),
                task_description: Some(first.description.clone()),
                confidence: first.confidence,
                target: None,
                experience_count: first.past_experiences.len(),
                error_count: 0,
                is_exploration: false,
            };
            let result = engine.evaluate(&context).await;
            if matches!(result, PolicyResult::NoMatch) {
                tracing::debug!("No policy rules matched for action selection");
            }
        }

        let scored: Vec<(ActionCandidate, f32)> = actions
            .into_iter()
            .map(|action| {
                let score = score_action(&action, &policy);
                (action, score)
            })
            .collect();

        select_best_scored(scored)
    }
}
