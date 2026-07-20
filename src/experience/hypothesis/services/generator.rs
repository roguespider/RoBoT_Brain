// robot/src/experience/hypothesis/services/generator.rs

//! ============================================================================
//! HYPOTHESIS GENERATOR
//! ============================================================================
//!
//! Creates new hypothesis candidates from observations and patterns.
//!
//! The generator does not decide whether a hypothesis is correct.
//! It only creates possible beliefs that can later be evaluated.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::{Hypothesis, HypothesisCategory, HypothesisPriority};

use crate::experience::types::Experience;

/// ============================================================================
/// GENERATOR
/// ============================================================================

#[derive(Debug, Clone)]
pub struct HypothesisGenerator {
    /// Minimum confidence required before generating.
    pub generation_threshold: f32,
}

impl HypothesisGenerator {
    pub fn new() -> Self {
        Self {
            generation_threshold: 0.60,
        }
    }

    /// Generate a hypothesis from an experience.
    ///
    /// This is intentionally simple.
    ///
    /// Future versions may:
    /// - analyze experience clusters
    /// - use embeddings
    /// - use an LLM
    /// - detect repeated behavior
    pub fn generate(&self, experience: &Experience) -> Result<Option<Hypothesis>> {
        if experience.title.is_empty() {
            return Ok(None);
        }

        let mut hypothesis = Hypothesis::new(
            format!("Pattern observed: {}", experience.title),
            format!("Generated from experience: {}", experience.description),
        );

        hypothesis.category = HypothesisCategory::Behavioral;

        hypothesis.priority = HypothesisPriority::Normal;

        hypothesis.metadata.source = "experience_generator".to_string();

        Ok(Some(hypothesis))
    }

    /// Generate a hypothesis from repeated observations.
    ///
    /// Placeholder for future pattern detection.
    pub fn generate_from_pattern(&self, pattern: &str) -> Result<Option<Hypothesis>> {
        if pattern.trim().is_empty() {
            return Ok(None);
        }

        let mut hypothesis = Hypothesis::new("Detected pattern", pattern);

        hypothesis.category = HypothesisCategory::Prediction;

        hypothesis.priority = HypothesisPriority::High;

        hypothesis.metadata.source = "pattern_generator".to_string();

        Ok(Some(hypothesis))
    }
}

impl Default for HypothesisGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// GENERATION RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub generated: bool,

    pub hypothesis_id: Option<String>,

    pub reason: String,
}
