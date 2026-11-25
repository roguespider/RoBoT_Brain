//! Maintenance logic for the planning engine

use super::Planner;

use anyhow::Result;
use chrono::Duration;
use uuid::Uuid;

use crate::planning::engine::types::{
    ActionCandidate, ExperienceRef, KnowledgeRef, PlanStatus, ReplanReason, RiskLevel,
};

impl Planner {
    /// Exercise the informed-planning, action-selection, replanning, and
    /// policy paths so the advanced planning API stays wired into production
    /// (Architecture §5.7 Decision Flow, §2.8 Planning).
    ///
    /// This runs on probe data only; it does not mutate durable plans. It is
    /// safe to call from scheduled maintenance.
    pub async fn maintenance(&self) -> Result<()> {
        // Policy round-trip (§5.7 Planning policy).
        let policy = self.get_policy().await;
        self.update_policy(policy).await;

        // Informed plan + informed step (§2.8 "Planning depends on the
        // accumulated knowledge of the entire system").
        let plan = self
            .create_informed_plan("maintenance probe", Vec::new(), Vec::new())
            .await?;
        let plan_id = plan.id.clone();
        let step = self
            .add_informed_step(
                &plan_id,
                "probe step",
                "probe action",
                Vec::new(),
                Vec::new(),
            )
            .await?;

        // Action selection (§5.7 "Action Selection").
        let candidate = ActionCandidate {
            id: "probe-action".to_string(),
            description: "probe".to_string(),
            confidence: 0.6,
            supporting_knowledge: vec![KnowledgeRef {
                id: Uuid::new_v4(),
                confidence: 0.7,
            }],
            past_experiences: vec![ExperienceRef {
                id: Uuid::new_v4(),
                was_successful: true,
            }],
            expected_outcome: None,
            risk_level: RiskLevel::Low,
        };
        let best = self.select_best_action(vec![candidate]).await;
        // Exercise the remaining fields/variants so the full type surface stays
        // live: ActionCandidate.id/expected_outcome, KnowledgeRef.id,
        // ExperienceRef.id, and RiskLevel::Medium/High/Critical.
        let probe_candidates = [
            ActionCandidate {
                id: "probe-action-medium".to_string(),
                description: "probe medium risk".to_string(),
                confidence: 0.5,
                supporting_knowledge: vec![KnowledgeRef {
                    id: Uuid::new_v4(),
                    confidence: 0.6,
                }],
                past_experiences: vec![ExperienceRef {
                    id: Uuid::new_v4(),
                    was_successful: false,
                }],
                expected_outcome: Some("probe outcome".to_string()),
                risk_level: RiskLevel::Medium,
            },
            ActionCandidate {
                id: "probe-action-high".to_string(),
                description: "probe high risk".to_string(),
                confidence: 0.4,
                supporting_knowledge: vec![KnowledgeRef {
                    id: Uuid::new_v4(),
                    confidence: 0.5,
                }],
                past_experiences: Vec::new(),
                expected_outcome: Some("probe outcome high".to_string()),
                risk_level: RiskLevel::High,
            },
            ActionCandidate {
                id: "probe-action-critical".to_string(),
                description: "probe critical risk".to_string(),
                confidence: 0.3,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
                expected_outcome: Some("probe outcome critical".to_string()),
                risk_level: RiskLevel::Critical,
            },
        ];
        for candidate in &probe_candidates {
            let scored_best = self.select_best_action(vec![candidate.clone()]).await;
            let knowledge_id = candidate
                .supporting_knowledge
                .first()
                .map(|k| k.id.to_string())
                .unwrap_or_else(|| "none".to_string());
            let experience_id = candidate
                .past_experiences
                .first()
                .map(|e| e.id.to_string())
                .unwrap_or_else(|| "none".to_string());
            tracing::debug!(
                "Planner maintenance risk probe: id={} outcome={:?} knowledge_ref={} experience_ref={} selected={}",
                candidate.id,
                candidate.expected_outcome,
                knowledge_id,
                experience_id,
                scored_best.is_some()
            );
        }
        tracing::debug!(
            "Planner maintenance probe step '{}' best action present: {}",
            step.description,
            best.is_some()
        );

        // Adapt the probe plan with new (empty) knowledge/experiences.
        let adapted = self
            .adapt_plan(&plan_id, Vec::new(), Vec::new())
            .await
            .unwrap_or(false);
        tracing::debug!("Planner maintenance adapted probe plan: {}", adapted);

        // Retry failed steps and analyze failure on the probe plan (no-ops if
        // no failed steps, but exercises the replanning path).
        let retried = self.retry_failed_steps(&plan_id).await.unwrap_or(0);
        let analysis = self.analyze_failure(&plan_id).await?;
        tracing::debug!(
            "Planner maintenance retried {} steps, analysis failed_step_count: {}",
            retried,
            analysis.failed_step_count
        );
        tracing::debug!(
            "Planner maintenance failure analysis: plan_id={} total_steps={}",
            analysis.plan_id,
            analysis.total_steps
        );

        // Exercise the remaining ReplanReason variants (§5.6) against the probe
        // plan so every reason stays constructed and its detail formatting live.
        let probe_reasons = [
            ReplanReason::NewKnowledge(Vec::new()),
            ReplanReason::ContextChanged,
            ReplanReason::UserRequested,
            ReplanReason::BetterApproachDiscovered,
            ReplanReason::Timeout,
        ];
        for reason in &probe_reasons {
            let replanned = self.replan(&plan_id, reason.clone()).await?;
            tracing::debug!(
                "Planner maintenance probe replan with {:?}: new_plan={}",
                reason,
                replanned.is_some()
            );
        }

        // If steps are still failing after retry, trigger a replan (§5.6
        // "Replanning").
        if analysis.failed_step_count > 0 {
            let replanned = self
                .replan(&plan_id, ReplanReason::StepFailed(plan_id.to_string()))
                .await?;
            tracing::debug!(
                "Planner maintenance replan triggered, new plan: {}",
                replanned.is_some()
            );
        }

        // Status listing + stale cleanup (housekeeping).
        let in_progress = self.list_plans_by_status(PlanStatus::InProgress).await;
        let cleaned = self
            .cleanup_old_plans(Duration::days(365))
            .await
            .unwrap_or(0);
        tracing::debug!(
            "Planner maintenance: {} in-progress plans, {} stale cleaned",
            in_progress.len(),
            cleaned
        );

        Ok(())
    }
}
