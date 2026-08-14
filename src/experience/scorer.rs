// robot_mcp/src/experience/scorer.rs

use anyhow::Result;

use crate::experience::{
    events::ExperienceEvent,
    observer::ExperienceObserver,
    types::{Experience, ExperienceScore, ExperienceType, OutcomeKind},
};

/// Calculates learning signals for experiences.
///
/// The scorer does not decide what is true.
/// It only evaluates the usefulness and quality
/// of recorded experiences.
#[derive(Clone)]
pub struct ExperienceScorer;

impl ExperienceScorer {
    pub fn new() -> Self {
        Self
    }

    /// Generate a score for an experience.
    pub fn score(&self, experience: &Experience) -> ExperienceScore {
        ExperienceScore {
            importance: self.calculate_importance(experience),
            confidence: self.calculate_confidence(experience),
            novelty: self.calculate_novelty(experience),
            reliability: self.calculate_reliability(experience),
        }
    }

    fn calculate_importance(&self, experience: &Experience) -> f32 {
        let mut score: f32 = 0.5;

        // Errors and failures are valuable learning events.
        match experience.outcome.kind {
            OutcomeKind::Failure | OutcomeKind::Timeout => {
                score += 0.25;
            }

            OutcomeKind::Success => {
                score += 0.10;
            }

            _ => {}
        }

        // User feedback is highly valuable.
        if matches!(
            experience.experience_type,
            crate::experience::types::ExperienceType::UserFeedback
        ) {
            score += 0.25;
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_confidence(&self, experience: &Experience) -> f32 {
        let mut score: f32 = 0.5;

        if experience.context.tool.is_some() {
            score += 0.1;
        }

        if experience.context.model.is_some() {
            score += 0.1;
        }

        if experience.outcome.error.is_some() {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_novelty(&self, experience: &Experience) -> f32 {
        // Base novelty from experience type diversity
        let type_novelty = match &experience.experience_type {
            ExperienceType::ToolExecution => 0.3,
            ExperienceType::Learning => 0.7,
            ExperienceType::Exploration => 0.8,
            ExperienceType::Hypothesis => 0.75,
            ExperienceType::Planning => 0.6,
            ExperienceType::Reflection => 0.65,
            _ => 0.5,
        };

        // Bonus for experiences with lessons learned
        let lesson_bonus = if experience.lessons.is_empty() {
            0.0
        } else {
            (experience.lessons.len() as f32).min(0.2)
        };

        // Adjust based on confidence spread from default
        let confidence_factor = (experience.confidence - 0.5).abs() * 0.1;

        (type_novelty + lesson_bonus + confidence_factor).clamp(0.0, 1.0)
    }

    fn calculate_reliability(&self, experience: &Experience) -> f32 {
        match experience.outcome.kind {
            OutcomeKind::Success => 0.8,
            OutcomeKind::Partial => 0.5,
            OutcomeKind::Failure => 0.2,
            OutcomeKind::Timeout => 0.1,
            OutcomeKind::Interrupted => 0.3,
        }
    }
}

impl ExperienceObserver for ExperienceScorer {
    fn name(&self) -> &'static str {
        "ExperienceScorer"
    }

    fn accepts(&self, event: &ExperienceEvent) -> bool {
        use crate::experience::events::ExperienceEventType;
        matches!(
            event.event_type,
            ExperienceEventType::ExperienceRecorded | ExperienceEventType::Scored
        )
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        use crate::experience::events::payload::EventPayload;

        match &event.payload {
            // Process Scored events - already has the score calculated
            EventPayload::Score {
                experience_id,
                score,
            } => {
                tracing::debug!(
                    "ExperienceScorer received Scored event: {} with score {:?}",
                    experience_id,
                    score
                );
            }
            // Process ExperienceRecorded events - score the experience
            EventPayload::ExperienceRecord { experience, .. } => {
                let score = self.score(experience);
                tracing::debug!(
                    "ExperienceScorer received ExperienceRecorded: {} scored {:?}",
                    experience.id,
                    score
                );
            }
            _ => {
                tracing::trace!(
                    "ExperienceScorer ignoring event type: {:?}",
                    event.event_type
                );
            }
        }
        Ok(())
    }
}
