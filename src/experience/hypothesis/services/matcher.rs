// robot/src/experience/hypothesis/services/matcher.rs

//! ============================================================================
//! HYPOTHESIS MATCHER
//! ============================================================================
//!
//! Finds hypotheses related to incoming experiences or search terms.
//!
//! The matcher does not determine whether a hypothesis is correct.
//! It only finds possible relationships.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::Hypothesis;

use crate::experience::types::Experience;

/// ============================================================================
/// MATCHER
/// ============================================================================

#[derive(Debug, Clone)]
pub struct HypothesisMatcher {
    /// Minimum similarity score required.
    pub minimum_score: f32,
}

impl HypothesisMatcher {
    pub fn new() -> Self {
        Self {
            minimum_score: 0.30,
        }
    }

    /// Find hypotheses related to an experience.
    pub fn match_experience(
        &self,
        experience: &Experience,
        hypotheses: &[Hypothesis],
    ) -> Vec<HypothesisMatch> {
        let mut matches = Vec::new();

        for hypothesis in hypotheses {
            let score = self.calculate_score(
                experience.title.as_str(),
                experience.description.as_str(),
                hypothesis,
            );

            if score >= self.minimum_score {
                matches.push(HypothesisMatch {
                    hypothesis_id: hypothesis.id.clone(),

                    score,

                    reason: "Keyword similarity".to_string(),
                });
            }
        }

        matches
    }

    /// Find hypotheses matching text.
    pub fn match_text(&self, text: &str, hypotheses: &[Hypothesis]) -> Vec<HypothesisMatch> {
        let mut results = Vec::new();

        for hypothesis in hypotheses {
            let score = self.text_similarity(
                text,
                &format!("{} {}", hypothesis.title, hypothesis.description),
            );

            if score >= self.minimum_score {
                results.push(HypothesisMatch {
                    hypothesis_id: hypothesis.id.clone(),

                    score,

                    reason: "Text similarity".to_string(),
                });
            }
        }

        results
    }

    fn calculate_score(&self, title: &str, description: &str, hypothesis: &Hypothesis) -> f32 {
        let input = format!("{} {}", title, description);

        let target = format!("{} {}", hypothesis.title, hypothesis.description);

        self.text_similarity(&input, &target)
    }

    /// Basic word overlap scoring.
    ///
    /// This is intentionally simple.
    ///
    /// Future versions:
    /// - embeddings
    /// - vector database search
    /// - semantic similarity
    fn text_similarity(&self, left: &str, right: &str) -> f32 {
        let left_words = Self::words(left);

        let right_words = Self::words(right);

        if left_words.is_empty() || right_words.is_empty() {
            return 0.0;
        }

        let shared = left_words.intersection(&right_words).count();

        let total = left_words.union(&right_words).count();

        shared as f32 / total as f32
    }

    fn words(text: &str) -> std::collections::HashSet<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}

impl Default for HypothesisMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// MATCH RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisMatch {
    pub hypothesis_id: crate::experience::hypothesis::core::HypothesisId,

    /// Similarity score 0.0 - 1.0.
    pub score: f32,

    pub reason: String,
}

