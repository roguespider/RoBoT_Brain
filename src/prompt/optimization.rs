//! Prompt optimization per Architecture Chapter 17.
//! Suggests improvements to prompts based on feedback and outcomes.

/// A suggested optimization for a prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptOptimization {
    /// The suggested change.
    pub suggestion: String,
    /// Expected improvement (0.0 - 1.0).
    pub expected_improvement: f32,
    /// Reason for the suggestion.
    pub reason: String,
}

/// Analyze prompt quality and suggest optimizations.
pub fn suggest_optimizations(text: &str, quality_score: f32) -> Vec<PromptOptimization> {
    let mut suggestions = Vec::new();

    if quality_score < 0.7 {
        suggestions.push(PromptOptimization {
            suggestion: "Add more specific context to the prompt".to_string(),
            expected_improvement: 0.2,
            reason: "Low quality score indicates insufficient context".to_string(),
        });
    }

    if text.len() < 100 {
        suggestions.push(PromptOptimization {
            suggestion: "Expand the prompt with more detail".to_string(),
            expected_improvement: 0.15,
            reason: "Short prompts may lack necessary guidance".to_string(),
        });
    }

    if text.contains("??") || text.contains("...") {
        suggestions.push(PromptOptimization {
            suggestion: "Replace ambiguous placeholders with specific instructions".to_string(),
            expected_improvement: 0.1,
            reason: "Placeholders reduce prompt clarity".to_string(),
        });
    }

    suggestions
}

/// Apply an optimization to a prompt.
pub fn apply_optimization(text: &str, optimization: &PromptOptimization) -> String {
    // In production, this would apply the specific optimization.
    // For now, return the original with an annotation.
    format!(
        "{text}\n\n[Optimization applied: {}]",
        optimization.suggestion
    )
}
