// src/agent/loop_runner.rs
//! The goal-driven cognitive loop (Architecture §5.7 Decision Flow, §5.8).
//!
//! Per TASK-V2-04: given a goal, the loop
//!   1. asks the Planner to decompose it into a plan (§2.8),
//!   2. retrieves supporting memory (§3), knowledge (§2.3) and past
//!      experiences (§07) for the goal,
//!   3. evaluates action confidence and selects the best-supported step
//!      (§5.7),
//!   4. asks the SafetyGate whether the action may execute (§16),
//!   5. executes the action (here: a constrained, in-process action that the
//!      agent can take without external actuators — currently "abstain and
//!      record", since RoBoT has no physical actuators; the *decision* and the
//!      *recorded outcome* are the valuable artifacts),
//!   6. records the outcome as a new experience via the ExperienceRecorder,
//!      which — through the coordinator's `process()` — publishes
//!      `ExperienceRecorded` and drives the full §4.04 learning spine wired in
//!      P0.
//!
//! This closes the loop: Act → New Experience → Learn.

use anyhow::Result;

use crate::experience::types::{ExperienceContext, ExperienceOutcome, ExperienceType, OutcomeKind};

use super::context::AgentDeps;
use super::decision::ActionSelector;
use super::safety_gate::SafetyDecision;
use super::types::{AgentGoal, GoalStatus};

/// What the loop produced for a goal.
#[derive(Debug, Clone)]
pub struct AgentLoopOutcome {
    pub goal_id: String,
    pub status: GoalStatus,
    /// The action that was selected (if any), with its confidence.
    pub action_description: Option<String>,
    pub confidence_value: Option<f32>,
    /// Why the loop abstained, when it did (safety block or low confidence).
    pub abstain_reason: Option<String>,
    /// The experience id recorded for this goal attempt.
    pub experience_id: Option<String>,
}

/// The goal-driven agent loop. Owns no business logic; composes `AgentDeps`.
pub struct AgentLoop {
    deps: AgentDeps,
}

impl AgentLoop {
    pub fn new(deps: AgentDeps) -> Self {
        Self { deps }
    }

    /// Run the cognitive loop for a single goal and record the outcome.
    ///
    /// This is the entry point described in Architecture §5.7. It never
    /// panics; every failure path is converted into an abstained/failed
    /// outcome that is itself recorded as a learning experience.
    pub async fn run(&self, mut goal: AgentGoal) -> Result<AgentLoopOutcome> {
        goal.status = GoalStatus::InProgress;

        // 1. Plan (§2.8). A planning failure is a learnable outcome.
        let plan = match self.deps.planner.create_plan(goal.description.clone()).await {
            Ok(plan) => plan,
            Err(e) => {
                let experience_id = self
                    .record_failure(&goal, format!("Planning failed: {}", e))
                    .await
                    .ok();
                goal.status = GoalStatus::Failed;
                goal.completed_at = Some(chrono::Utc::now());
                return Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: None,
                    confidence_value: None,
                    abstain_reason: Some(format!("Planning failed: {}", e)),
                    experience_id,
                });
            }
        };

        // 2. Retrieve supporting evidence (§3 memory, §2.3 knowledge, §07
        //    experiences). Experiences are memory items of type Experience.
        let memory = self.deps.memory_retrieval.retrieve(&goal.description).await;
        let knowledge = self
            .deps
            .knowledge_store
            .search(goal.description.as_str())
            .await;
        let experiences: Vec<_> = memory
            .iter()
            .filter(|r| {
                r.item.memory_type
                    == crate::memory::types::MemoryType::Experience
            })
            .map(|r| r.item.clone())
            .collect();

        // 3. Select the best-supported action (§5.7).
        let selected = ActionSelector::select(
            &plan.steps,
            &memory,
            &knowledge,
            // The selector expects past Experience records; we synthesize
            // lightweight ones from memory items so the confidence blend has a
            // real experience channel without a second DB round-trip.
            &experiences
                .iter()
                .map(|m| crate::experience::types::Experience {
                    id: m.id,
                    score: Some(crate::experience::types::ExperienceScore {
                        confidence: m.confidence,
                        importance: m.confidence,
                        novelty: 0.0,
                        reliability: m.confidence,
                    }),
                    ..crate::experience::types::Experience::new(
                        m.content.clone(),
                        m.content.clone(),
                        ExperienceType::MemoryLookup,
                        Vec::new(),
                    )
                })
                .collect::<Vec<_>>(),
            goal.confidence_threshold,
        );

        let Some(selected) = selected else {
            // No actionable step: abstain and record a low-confidence outcome.
            let experience_id = self
                .record_abstention(&goal, "No actionable plan step remained".to_string())
                .await
                .ok();
            goal.status = GoalStatus::Abstained;
            goal.completed_at = Some(chrono::Utc::now());
            return Ok(AgentLoopOutcome {
                goal_id: goal.id,
                status: goal.status,
                action_description: None,
                confidence_value: None,
                abstain_reason: Some("No actionable plan step remained".to_string()),
                experience_id,
            });
        };

        // 3b. Emotional weighting (Architecture §13, TASK-V2-08). The
        //     personality's emotional state nudges the action confidence:
        //     engagement/satisfaction raise it, frustration lowers it. Emotion
        //     biases rather than overrides evidence-based confidence.
        let emotional_weight = {
            let personality = self.deps.personality.lock();
            match personality {
                Ok(p) => p.emotional_weight(),
                Err(_) => 0.0,
            }
        };
        let emotion_adjusted = (selected.confidence.value + emotional_weight).clamp(0.0, 1.0);
        let mut selected = selected;
        selected.confidence.value = emotion_adjusted;

        // 4. Safety gate (§16). The gate may block destructive actions or
        //    sub-threshold confidence.
        match self
            .deps
            .safety_gate
            .evaluate(&selected.step.action, &selected.confidence)
        {
            SafetyDecision::Block { reason } => {
                // Observe the abstention emotionally: a blocked action is a
                // low-friction setback (no real failure), so record a mild
                // failure with low effort.
                self.observe_emotion(false, 0.3);
                let experience_id = self.record_abstention(&goal, reason.clone()).await.ok();
                goal.status = GoalStatus::Abstained;
                goal.completed_at = Some(chrono::Utc::now());
                return Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: Some(selected.step.action.clone()),
                    confidence_value: Some(selected.confidence.value),
                    abstain_reason: Some(reason),
                    experience_id,
                });
            }
            SafetyDecision::Allow => {
                // 5. Execute. RoBoT has no external actuators (TASK-V2-04
                //    note): the autonomous action today is the *decision* —
                //    selecting and committing to an action with recorded
                //    rationale. The valuable artifact is the recorded
                //    experience, which closes the loop via §4.04.
                let rationale = selected.rationale();
                tracing::info!("Agent loop proceeding: {}", rationale);
                let outcome_summary = format!(
                    "Selected action '{}' (confidence {:.2}, emotion {:+.2}) for goal '{}'",
                    selected.step.action, selected.confidence.value, emotional_weight, goal.description
                );

                // 6. Record the outcome as a new experience. The recorder
                //    stores it; the coordinator's `process()` publishes
                //    ExperienceRecorded, which (per P0 wiring) drives the
                //    full learning pipeline.
                let experience_id = self
                    .record_success(&goal, outcome_summary.clone())
                    .await
                    .ok();

                // Observe the successful action emotionally (Architecture §13).
                self.observe_emotion(true, 0.5);

                goal.status = GoalStatus::Achieved;
                goal.completed_at = Some(chrono::Utc::now());
                Ok(AgentLoopOutcome {
                    goal_id: goal.id,
                    status: goal.status,
                    action_description: Some(selected.step.action.clone()),
                    confidence_value: Some(selected.confidence.value),
                    abstain_reason: None,
                    experience_id,
                })
            }
        }
    }

    /// Update the personality's emotional state from an outcome (§13).
    fn observe_emotion(&self, success: bool, effort: f32) {
        let personality = self.deps.personality.lock();
        if let Ok(mut p) = personality {
            p.observe_emotional_outcome(success, effort);
        }
    }

    /// Record a successful action outcome and drive the learning spine.
    ///
    /// The experience is created, processed through the coordinator (which
    /// scores it and publishes `Scored` + `ExperienceRecorded`), and stored.
    async fn record_success(&self, goal: &AgentGoal, summary: String) -> Result<String> {
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent action: {}", goal.description),
            summary,
            ExperienceType::Planning,
            Vec::new(),
        );
        experience.context = ExperienceContext {
            source: Some("agent-loop".to_string()),
            ..ExperienceContext::default()
        };
        experience.outcome = ExperienceOutcome::success();

        // process() scores + publishes ExperienceRecorded once (P0 V2-02).
        let processed = self.deps.coordinator.process(experience);

        // Persist so the experience is retrievable by future loops.
        let conn = self.deps.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;

        Ok(processed.id.to_string())
    }

    /// Record an abstention (safety block / low confidence) as a low-confidence
    /// experience so the system learns when it should *not* act.
    async fn record_abstention(&self, goal: &AgentGoal, reason: String) -> Result<String> {
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent abstained: {}", goal.description),
            reason.clone(),
            ExperienceType::Planning,
            Vec::new(),
        );
        experience.outcome = ExperienceOutcome {
            kind: OutcomeKind::Interrupted,
            message: Some("abstained".to_string()),
            error: Some(reason),
            duration_ms: None,
        };
        let processed = self.deps.coordinator.process(experience);
        let conn = self.deps.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;
        Ok(processed.id.to_string())
    }

    /// Record a failure (e.g. planning error) as a failed experience.
    async fn record_failure(&self, goal: &AgentGoal, reason: String) -> Result<String> {
        let mut experience = crate::experience::types::Experience::new(
            format!("Agent goal failed: {}", goal.description),
            reason.clone(),
            ExperienceType::Error,
            Vec::new(),
        );
        experience.outcome = ExperienceOutcome::failure(reason);
        let processed = self.deps.coordinator.process(experience);
        let conn = self.deps.database.connection()?;
        let memory = crate::database::models::MemoryCard::from_experience(&processed);
        crate::database::queries::insert_memory(&conn, &memory)?;
        Ok(processed.id.to_string())
    }
}
