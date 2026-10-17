// src/experience/pattern_discovery/mod.rs
//! Pattern discovery from recorded experiences.
//!
//! Groups experiences by type and context to identify recurring patterns
//! that can inform future planning and capability assessment.

/// A discovered pattern from experience analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredPattern {
    /// Pattern identifier.
    pub id: String,
    /// Category of pattern (e.g., "tool_failure", "success_pattern").
    pub category: String,
    /// Description of the pattern.
    pub description: String,
    /// Number of experiences matching this pattern.
    pub count: usize,
    /// Average confidence across matching experiences.
    pub avg_confidence: f32,
}

/// Discover patterns from a list of experiences.
///
/// Groups experiences by `experience_type` and `outcome.kind`, then returns
/// aggregated pattern summaries for each non-empty group.
pub fn discover_patterns(
    experiences: &[crate::experience::types::experience::Experience],
) -> Vec<DiscoveredPattern> {
    if experiences.is_empty() {
        return Vec::new();
    }

    // Group by experience type and outcome kind
    let mut groups: std::collections::HashMap<
        (String, String),
        Vec<&crate::experience::types::experience::Experience>,
    > = std::collections::HashMap::new();

    for exp in experiences {
        let key = (
            exp.experience_type.to_string(),
            exp.outcome.kind.to_string(),
        );
        groups.entry(key).or_default().push(exp);
    }

    // Build pattern summaries
    let mut patterns = Vec::new();
    for ((exp_type, outcome_kind), group) in &groups {
        if group.len() < 2 {
            // Skip patterns with fewer than 2 instances (not a pattern)
            continue;
        }
        let avg_confidence = if !group.is_empty() {
            let sum: f32 = group.iter().map(|e| e.confidence).sum();
            sum / group.len() as f32
        } else {
            0.0
        };

        patterns.push(DiscoveredPattern {
            id: format!("{exp_type}_{outcome_kind}"),
            category: format!("{exp_type}/{}", outcome_kind.to_lowercase()),
            description: format!("{exp_type} experiences with outcome {outcome_kind}"),
            count: group.len(),
            avg_confidence,
        });
    }

    patterns.sort_by_key(|p| std::cmp::Reverse(p.count));
    patterns
}
