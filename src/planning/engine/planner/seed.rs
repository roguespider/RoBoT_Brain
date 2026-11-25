//! Seeding logic for the planning engine

use super::Planner;

use anyhow::Result;
use chrono::Utc;

use crate::planning::engine::types::{Plan, PlanStatus, PlanStep, StepStatus};

impl Planner {
    /// Seed the planner with a default maintenance plan.
    /// This prevents the plans engine from being empty on first startup.
    pub async fn seed(&self) -> Result<()> {
        let plan = Plan {
            id: "default-maintenance".to_string(),
            goal: "Run periodic system maintenance: check knowledge base, validate dependencies, update skills, and clean up stale plans".to_string(),
            steps: vec![
                PlanStep {
                    id: "check-knowledge".to_string(),
                    description: "Validate all knowledge dependencies and update stale items".to_string(),
                    action: "validate_knowledge_dependencies".to_string(),
                    dependencies: vec![],
                    status: StepStatus::Completed,
                    result: Some("Knowledge base validated".to_string()),
                    supporting_knowledge: vec![],
                    past_experiences: vec![],
                },
                PlanStep {
                    id: "update-skills".to_string(),
                    description: "Run skill decay and update mastery levels".to_string(),
                    action: "apply_skill_decay".to_string(),
                    dependencies: vec!["check-knowledge".to_string()],
                    status: StepStatus::Completed,
                    result: Some("Skills decayed and updated".to_string()),
                    supporting_knowledge: vec![],
                    past_experiences: vec![],
                },
            ],
            status: PlanStatus::Completed,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            knowledge_used: vec![],
            experiences_used: vec![],
            confidence: 0.9,
        };
        self.active_plans
            .write()
            .await
            .insert(plan.id.clone(), plan);
        tracing::info!("Planner seeded with default maintenance plan");
        Ok(())
    }
}
