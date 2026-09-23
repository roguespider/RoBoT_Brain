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
pub fn reference_contract() {
    let improvement = compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    tracing::info!(
        skill_id = %improvement.skill_id,
        metric = %improvement.metric,
        old_value = improvement.old_value,
        new_value = improvement.new_value,
        delta = improvement.delta,
        "Improvement contract actively referenced"
    );
}

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

/// Active reference to improvement contracts.
pub fn reference_contract_active() {
    let improvement_ref_name = std::any::type_name_of_val(&*IMPROVEMENT_CONTRACT_REF);
    tracing::info!(
        improvement_ref_name,
        "Improvement contract actively referenced via lazy_static"
    );
}
