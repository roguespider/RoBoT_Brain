//! Learning generalization - Per Architecture §10.2 "Pattern discovery" (generalization)

use serde::{Deserialize, Serialize};

/// A generalization rule over patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralizationRule {
    /// The specific pattern this generalizes from.
    pub specific_pattern: String,
    /// The generalized pattern.
    pub general_pattern: String,
    /// Confidence in the generalization.
    pub confidence: f32,
    /// Supporting experiences.
    pub supporting_experiences: Vec<String>,
}

/// Detect generalizations from patterns (cluster by first token of context_signature).
pub fn detect_generalizations(
    patterns: &[crate::learning::patterns::Pattern],
    min_support: u32,
) -> Vec<GeneralizationRule> {
    use std::collections::HashMap;
    let mut clusters: HashMap<String, Vec<String>> = HashMap::new();
    for p in patterns {
        let first_token = p
            .context_signature
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        clusters
            .entry(first_token.clone())
            .or_default()
            .push(p.id.clone());
    }
    clusters
        .into_iter()
        .filter(|(_, ids)| ids.len() as u32 >= min_support)
        .map(|(token, ids)| GeneralizationRule {
            specific_pattern: token.clone(),
            general_pattern: token,
            confidence: 0.6,
            supporting_experiences: ids,
        })
        .collect()
}

/// Apply a generalization rule to a context (substring match for now).
pub fn apply_generalization(rule: &GeneralizationRule, context: &str) -> bool {
    context.contains(&rule.general_pattern)
}

