// robot_mcp/src/experience/scorer.rs


use anyhow::Result;

use crate::experience::{
    events::ExperienceEvent,
    observer::ExperienceObserver,
    types::{Experience, ExperienceScore, OutcomeKind},
};

/// Calculates learning signals for experiences.
///
/// The scorer does not decide what is true.
/// It only evaluates the usefulness and quality
/// of recorded experiences.
#[derive(Clone)]
pub struct ExperienceScorer;

/// Scores individual encounters.
///
/// This provides granular scoring for each encounter within an experience,
/// complementing the ExperienceScore which scores the overall experience.
#[derive(Debug, Clone)]

pub struct EncounterScore {
    /// Success indicator (0.0-1.0)
    pub success: f32,
    /// Quality of the encounter (0.0-1.0)
    pub quality: f32,
    /// Reliability of the result (0.0-1.0)
    pub reliability: f32,
}


impl EncounterScore {
    /// Create a new encounter score with default values
    pub fn new() -> Self {
        Self {
            success: 0.5,
            quality: 0.5,
            reliability: 0.5,
        }
    }

    /// Create from an encounter result
    pub fn from_result(result: &super::types::EncounterResult) -> Self {
        match result {
            super::types::EncounterResult::Success => Self {
                success: 1.0,
                quality: 0.8,
                reliability: 0.9,
            },
            super::types::EncounterResult::Failure => Self {
                success: 0.0,
                quality: 0.2,
                reliability: 0.8,
            },
            super::types::EncounterResult::Partial(_) => Self {
                success: 0.5,
                quality: 0.6,
                reliability: 0.6,
            },
            super::types::EncounterResult::Error(_) => Self {
                success: 0.0,
                quality: 0.1,
                reliability: 0.5,
            },
            super::types::EncounterResult::Timeout => Self {
                success: 0.0,
                quality: 0.3,
                reliability: 0.3,
            },
        }
    }

    /// Calculate the overall score as a weighted average
    pub fn overall(&self) -> f32 {
        (self.success * 0.5) + (self.quality * 0.3) + (self.reliability * 0.2)
    }
}

impl Default for EncounterScore {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Score an individual encounter
    pub fn score_encounter(&self, result: &super::types::EncounterResult) -> EncounterScore {
        EncounterScore::from_result(result)
    }

    /// Aggregate scores from multiple encounters
    pub fn aggregate_encounter_scores(&self, scores: &[EncounterScore]) -> EncounterScore {
        if scores.is_empty() {
            return EncounterScore::new();
        }

        let sum_success = scores.iter().map(|s| s.success).sum::<f32>() / scores.len() as f32;
        let sum_quality = scores.iter().map(|s| s.quality).sum::<f32>() / scores.len() as f32;
        let sum_reliability = scores.iter().map(|s| s.reliability).sum::<f32>() / scores.len() as f32;

        EncounterScore {
            success: sum_success,
            quality: sum_quality,
            reliability: sum_reliability,
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

    #[allow(unused)]
    fn calculate_novelty(&self, _experience: &Experience) -> f32 {
        // Future:
        // Compare embeddings against previous experiences.
        //
        // This will eventually use memory/vector search.

        0.5
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
            EventPayload::Score { experience_id, score } => {
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
                tracing::trace!("ExperienceScorer ignoring event type: {:?}", event.event_type);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encounter_score_from_success() {
        let score = EncounterScore::from_result(&super::super::types::EncounterResult::Success);
        assert_eq!(score.success, 1.0);
        assert_eq!(score.quality, 0.8);
        assert_eq!(score.reliability, 0.9);
    }

    #[test]
    fn test_encounter_score_from_failure() {
        let score = EncounterScore::from_result(&super::super::types::EncounterResult::Failure);
        assert_eq!(score.success, 0.0);
        assert_eq!(score.quality, 0.2);
    }

    #[test]
    fn test_encounter_score_overall() {
        let score = EncounterScore::from_result(&super::super::types::EncounterResult::Success);
        // 1.0 * 0.5 + 0.8 * 0.3 + 0.9 * 0.2 = 0.5 + 0.24 + 0.18 = 0.92
        let overall = score.overall();
        assert!((overall - 0.92).abs() < 0.001);
    }

    #[test]
    fn test_scorer_score_encounter() {
        let scorer = ExperienceScorer::new();
        let score = scorer.score_encounter(&super::super::types::EncounterResult::Success);
        assert_eq!(score.success, 1.0);
    }

    #[test]
    fn test_aggregate_encounter_scores() {
        let scorer = ExperienceScorer::new();
        let scores = vec![
            EncounterScore::from_result(&super::super::types::EncounterResult::Success),
            EncounterScore::from_result(&super::super::types::EncounterResult::Failure),
        ];
        let aggregated = scorer.aggregate_encounter_scores(&scores);
        assert!((aggregated.success - 0.5).abs() < 0.001); // (1.0 + 0.0) / 2
    }
}
