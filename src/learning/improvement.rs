//! Learning improvement - Per Architecture Section 10.5 "Skill improvement" (v0.0.2)

use serde::{Deserialize, Serialize};

/// Skill improvement record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillImprovement {
    /// Skill identifier.
    pub skill_id: String,
    /// Metric being improved.
    pub metric: String,
    /// Old value.
    pub old_value: f32,
    /// New value.
    pub new_value: f32,
    /// Delta (new - old).
    pub delta: f32,
}

/// Active reference to improvement contracts.
/// Compute improvement for a skill metric.
pub fn compute_improvement(skill_id: &str, metric: &str, old: f32, new: f32) -> SkillImprovement {
    SkillImprovement {
        skill_id: skill_id.to_string(),
        metric: metric.to_string(),
        old_value: old,
        new_value: new,
        delta: new - old,
    }
}

lazy_static::lazy_static! {
    /// Active contract reference per Architecture Section 10.5 hygiene rules.
    static ref IMPROVEMENT_CONTRACT_REF: SkillImprovement = {
        compute_improvement("ref-skill", "ref-metric", 0.0, 1.0)
    };
}
