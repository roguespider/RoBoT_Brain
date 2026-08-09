// src/agent/decision.rs
//! Action selection and confidence evaluation (Architecture §5.7 Decision Flow).
//!
//! The decision stage gathers evidence from three retrieval channels — memory,
//! knowledge, and past experiences — and combines them into a single confidence
//! score. If confidence clears the goal's threshold, the best-supported action
//! candidate is selected for execution.

use crate::experience::types::{Experience, ExperienceScore};
use crate::knowledge::KnowledgeItem;
use crate::memory::retrieval::RetrievalResult;
use crate::planner::engine::types::PlanStep;

/// Confidence in a proposed action (0.0–1.0), with the goal's threshold for the
/// safety gate to compare against.
#[derive(Debug, Clone)]
pub struct ActionConfidence {
    pub value: f32,
    pub threshold: f32,
    /// Breakdown of how the score was composed (for uncertainty reporting).
    pub components: ConfidenceComponents,
}

/// The per-channel contributions that produced `ActionConfidence::value`.
#[derive(Debug, Clone, Default)]
pub struct ConfidenceComponents {
    pub memory_support: f32,
    pub knowledge_support: f32,
    pub experience_support: f32,
    pub plan_step_confidence: f32,
}

impl ActionConfidence {
    /// Weighted blend of the three retrieval channels plus the plan step's own
    /// confidence. Weights follow the architecture's trust ordering:
    /// knowledge (validated) > experience (observed) > memory (raw) > plan.
    pub fn blend(
        memory_support: f32,
        knowledge_support: f32,
        experience_support: f32,
        plan_step_confidence: f32,
        threshold: f32,
    ) -> Self {
        let value = 0.20 * memory_support
            + 0.40 * knowledge_support
            + 0.30 * experience_support
            + 0.10 * plan_step_confidence;
        let clamped = value.clamp(0.0, 1.0);
        Self {
            value: clamped,
            threshold,
            components: ConfidenceComponents {
                memory_support,
                knowledge_support,
                experience_support,
                plan_step_confidence,
            },
        }
    }
}

/// An action the agent has selected to execute, with the evidence that backed it.
#[derive(Debug, Clone)]
pub struct SelectedAction {
    /// The plan step whose `action` field names the tool to invoke.
    pub step: PlanStep,
    /// Aggregated confidence in this action.
    pub confidence: ActionConfidence,
    /// Memory items that supported the action.
    pub supporting_memory: Vec<RetrievalResult>,
    /// Knowledge items that supported the action.
    pub supporting_knowledge: Vec<KnowledgeItem>,
    /// Past experiences that informed the action.
    pub supporting_experiences: Vec<Experience>,
}

impl SelectedAction {
    /// Human-readable rationale for the decision, citing the evidence channels
    /// (Architecture §5.7 / §16 uncertainty reporting).
    pub fn rationale(&self) -> String {
        format!(
            "action='{}' confidence={:.2} (memory={:.2}, knowledge={:.2}, experience={:.2}, \
             plan={:.2}) backed by {} memory item(s), {} knowledge item(s), {} experience(s)",
            self.step.action,
            self.confidence.value,
            self.confidence.components.memory_support,
            self.confidence.components.knowledge_support,
            self.confidence.components.experience_support,
            self.confidence.components.plan_step_confidence,
            self.supporting_memory.len(),
            self.supporting_knowledge.len(),
            self.supporting_experiences.len(),
        )
    }
}

/// Selects the best-supported action from a plan using retrieved evidence.
pub struct ActionSelector;

impl ActionSelector {
    /// Compose confidence from the three retrieval channels and pick the plan
    /// step with the highest blended confidence.
    ///
    /// Per §5.7: if no step clears a meaningful floor, the agent abstains
    /// (returns `None`) so the loop can record a low-confidence experience
    /// rather than act blindly.
    pub fn select(
        steps: &[PlanStep],
        memory: &[RetrievalResult],
        knowledge: &[KnowledgeItem],
        experiences: &[Experience],
        threshold: f32,
    ) -> Option<SelectedAction> {
        if steps.is_empty() {
            return None;
        }

        let memory_support = Self::aggregate_memory(memory);
        let knowledge_support = Self::aggregate_knowledge(knowledge);
        let experience_support = Self::aggregate_experience(experiences);

        let mut best: Option<SelectedAction> = None;
        for step in steps {
            // Only consider steps that are not yet done.
            if matches!(
                step.status,
                crate::planner::engine::types::StepStatus::Completed
            ) {
                continue;
            }
            let plan_step_confidence = if step.supporting_knowledge.is_empty() {
                0.4
            } else {
                0.6
            };
            let confidence = ActionConfidence::blend(
                memory_support,
                knowledge_support,
                experience_support,
                plan_step_confidence,
                threshold,
            );
            let candidate = SelectedAction {
                step: step.clone(),
                confidence,
                supporting_memory: memory.to_vec(),
                supporting_knowledge: knowledge.to_vec(),
                supporting_experiences: experiences.to_vec(),
            };
            match &best {
                Some(current) if current.confidence.value >= candidate.confidence.value => {}
                _ => best = Some(candidate),
            }
        }

        best
    }

    /// Average retrieval relevance across supporting memory items.
    fn aggregate_memory(memory: &[RetrievalResult]) -> f32 {
        if memory.is_empty() {
            return 0.0;
        }
        let sum: f32 = memory.iter().map(|r| r.relevance_score).sum();
        (sum / memory.len() as f32).clamp(0.0, 1.0)
    }

    /// Average confidence across supporting knowledge items.
    fn aggregate_knowledge(knowledge: &[KnowledgeItem]) -> f32 {
        if knowledge.is_empty() {
            return 0.0;
        }
        let sum: f32 = knowledge.iter().map(|k| k.confidence.overall()).sum();
        (sum / knowledge.len() as f32).clamp(0.0, 1.0)
    }

    /// Average overall score across supporting past experiences.
    fn aggregate_experience(experiences: &[Experience]) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }
        let sum: f32 = experiences
            .iter()
            .map(|e| Self::experience_overall(e.score.as_ref()))
            .sum();
        (sum / experiences.len() as f32).clamp(0.0, 1.0)
    }

    /// Extract an overall confidence from an `ExperienceScore`, defaulting low.
    fn experience_overall(score: Option<&ExperienceScore>) -> f32 {
        match score {
            Some(s) => (s.confidence * 0.5 + s.reliability * 0.5).clamp(0.0, 1.0),
            None => 0.2,
        }
    }
}
