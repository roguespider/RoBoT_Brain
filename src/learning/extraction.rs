//! Learning extraction - Per Architecture §10.3 "Knowledge extraction"

use serde::{Deserialize, Serialize};

/// Extracted knowledge from patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedKnowledge {
    /// Source pattern ID.
    pub pattern_id: String,
    /// The extracted rule.
    pub rule: String,
    /// Confidence in the rule.
    pub confidence: f32,
    /// Applicable context.
    pub applicable_context: String,
    /// History of confidence updates.
    pub confidence_history: Vec<(i64, f32)>,
}

/// Extract knowledge from patterns.
pub fn extract_knowledge(
    patterns: &[crate::learning::patterns::Pattern],
) -> Vec<ExtractedKnowledge> {
    patterns
        .iter()
        .map(|p| ExtractedKnowledge {
            pattern_id: p.id.clone(),
            rule: format!(
                "when context_signature={} with frequency>={}, expect success_rate>={}",
                p.context_signature, p.frequency, p.success_rate
            ),
            confidence: p.success_rate,
            applicable_context: p.context_signature.clone(),
            confidence_history: Vec::new(),
        })
        .collect()
}
