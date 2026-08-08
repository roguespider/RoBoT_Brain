// src/experience/integration/learning_coordinator/reinforcement.rs
//! Reinforcement learning functions for the learning coordinator

use anyhow::Result;
use std::sync::Arc;

use crate::experience::metrics::MetricsCollector;
use crate::experience::types::Experience;
use crate::experience::types::outcome::OutcomeKind;
use crate::knowledge::KnowledgeStore;
use crate::skills::registry::SkillRegistry;

use super::results::ReinforcementResult;

/// Reinforcement learning methods for LearningCoordinator
pub struct ReinforcementMethods<'a> {
    pub knowledge_store: &'a Arc<KnowledgeStore>,
    pub skill_registry: &'a Option<Arc<SkillRegistry>>,
    pub metrics: &'a Arc<MetricsCollector>,
}

impl<'a> ReinforcementMethods<'a> {
    /// Apply reinforcement learning from an experience outcome
    ///
    /// Per Architecture §9: "Reinforcement learning adjusts behavior based on rewards/penalties"
    pub async fn apply_reinforcement(
        &self,
        experience: &Experience,
    ) -> Result<ReinforcementResult> {
        let reward = self.calculate_reward(experience);

        let mut result = ReinforcementResult {
            experience_id: experience.id,
            reward,
            ..Default::default()
        };

        // Update knowledge based on reward
        let knowledge_updates = self
            .update_knowledge_from_reward(experience, reward)
            .await?;
        result.knowledge_updates = knowledge_updates;

        // Update skills based on reward
        let skill_updates = self.update_skills_from_reward(experience, reward).await?;
        result.skill_updates = skill_updates;

        // Update action values
        let action_value_update = self.update_action_values(experience, reward).await?;
        result.action_value_delta = action_value_update;

        self.metrics
            .increment("learning.reinforcement.applied")
            .await;

        tracing::info!(
            "Applied reinforcement learning for {}: reward={:.3}, knowledge_updates={}, skill_updates={}",
            experience.id, reward, knowledge_updates, skill_updates
        );

        Ok(result)
    }

    /// Calculate reward from experience outcome
    pub fn calculate_reward(&self, experience: &Experience) -> f64 {
        match experience.outcome.kind {
            OutcomeKind::Success => {
                // Positive reward scaled by confidence
                1.0 * (experience.confidence as f64)
            }
            OutcomeKind::Partial => {
                // Small positive reward for partial success
                0.3 * (experience.confidence as f64)
            }
            OutcomeKind::Failure => {
                // Negative reward for failure
                -1.0
            }
            OutcomeKind::Interrupted => {
                // Small negative reward for interruption
                -0.2
            }
            _ => 0.0,
        }
    }

    /// Update knowledge based on reinforcement reward
    pub async fn update_knowledge_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let mut updates = 0;

        // Increase confidence for positive reward
        if reward > 0.0 {
            // Find knowledge related to this experience
            let related_knowledge = self.knowledge_store.search(&experience.description).await;

            for knowledge in related_knowledge {
                if reward > 0.5 {
                    // High reward: boost confidence
                    self.knowledge_store.record_success(knowledge.id).await;
                    updates += 1;
                }
            }
        } else if reward < 0.0 {
            // Decrease confidence for negative reward
            let related_knowledge = self.knowledge_store.search(&experience.description).await;

            for knowledge in related_knowledge {
                self.knowledge_store.record_failure(knowledge.id).await;
                updates += 1;
            }
        }

        Ok(updates)
    }

    /// Update skills based on reinforcement reward
    pub async fn update_skills_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let mut updates = 0;

        // Only update if we have a skill registry
        if let Some(registry) = self.skill_registry {
            // Find skills related to this experience context
            let workflow_name = experience
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.clone())
                .unwrap_or_default();

            // Look for skills that match the workflow or experience type
            let skill_name = format!(
                "{}:{}",
                match experience.experience_type {
                    crate::experience::types::ExperienceType::ToolExecution => "tool",
                    crate::experience::types::ExperienceType::Planning => "planning",
                    crate::experience::types::ExperienceType::Workflow => &workflow_name,
                    _ => "general",
                },
                experience.title.replace(' ', "_").to_lowercase()
            );

            // Try to find and update the skill
            if let Some(skill) = registry.get_by_name(&skill_name).await {
                let success = reward > 0.0;
                let record_result = registry.record_usage(&skill.id, success).await;
                if record_result.is_ok() {
                    updates += 1;
                    tracing::debug!("Updated skill {} with success={}", skill_name, success);
                }
            }
        }

        tracing::debug!(
            "Skill update from reward {:.3}: {} ({} updates)",
            reward,
            experience.id,
            updates
        );
        Ok(updates)
    }

    /// Update action values for future decision making
    pub async fn update_action_values(&self, experience: &Experience, reward: f64) -> Result<f64> {
        // Store the reward for this action context
        // This could be used to build a Q-table or similar
        let action_key = format!(
            "{}:{}",
            experience
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.as_str())
                .unwrap_or("unknown"),
            experience.description.as_str()
        );

        // In a full implementation, this would update a Q-table or similar
        // For now, we just log the action value update
        tracing::debug!(
            "Action value update for '{}': reward={:.3}",
            action_key,
            reward
        );

        Ok(reward)
    }
}
