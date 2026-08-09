// src/personality/mod.rs

//! Personality System
//!
//! Per Architecture: Defines the behavioral characteristics of the AI system,
//! influencing decision-making, communication style, and learning preferences.
//!
//! # Usage
//!
//! ```rust
//! use robot_brain::personality::{Personality, CommunicationStyle};
//!
//! let mut personality = Personality::new();
//! // Apply preset with proper error handling
//! let applied = personality.apply_preset("analytical");
//! if !applied {
//!     eprintln!("Warning: preset 'analytical' not found");
//! }
//!
//! // Use personality traits for decisions
//! if personality.should_explore(0.5) {
//!     // Explore new approaches
//! }
//!
//! // Get communication style
//! let style = personality.get_communication_style();
//! match style {
//!     CommunicationStyle::Concise => println!("Keep it brief"),
//!     CommunicationStyle::Balanced => println!("Medium detail"),
//!     CommunicationStyle::Detailed => println!("Full explanation"),
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Personality traits that influence behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    /// How curious the system is (0.0 - 1.0)
    pub curiosity: f32,

    /// How cautious the system is (0.0 - 1.0)
    pub caution: f32,

    /// How creative the system is (0.0 - 1.0)
    pub creativity: f32,

    /// How patient the system is (0.0 - 1.0)
    pub patience: f32,

    /// How thorough the system is (0.0 - 1.0)
    pub thoroughness: f32,

    /// Communication verbosity (0.0 = minimal, 1.0 = verbose)
    pub verbosity: f32,

    /// Risk tolerance (0.0 = risk-averse, 1.0 = risk-tolerant)
    pub risk_tolerance: f32,
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            curiosity: 0.7,
            caution: 0.5,
            creativity: 0.6,
            patience: 0.7,
            thoroughness: 0.8,
            verbosity: 0.5,
            risk_tolerance: 0.4,
        }
    }
}

/// Communication style preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CommunicationStyle {
    /// Minimal output, just essential information
    Concise,
    /// Balanced between brief and detailed
    #[default]
    Balanced,
    /// Full explanations with context
    Detailed,
}

impl CommunicationStyle {
    /// Get format string for response based on style
    pub fn format_response(&self, content: &str) -> String {
        match self {
            CommunicationStyle::Concise => {
                // Strip extra whitespace, take first paragraph
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            CommunicationStyle::Balanced => {
                // Take first few paragraphs
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .take(5)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            CommunicationStyle::Detailed => content.to_string(),
        }
    }
}

/// Decision context for personality-based choices
#[derive(Debug, Clone, Default)]
pub struct DecisionContext {
    /// Current confidence in the approach (0.0 - 1.0)
    pub confidence: f32,
    /// Potential gain from an action
    pub potential_gain: f32,
    /// Potential loss from an action
    pub potential_loss: f32,
    /// Whether we're dealing with an uncertain situation
    pub uncertainty: f32,
    /// Time available (in seconds)
    pub time_available: u64,
}

/// Decision made by personality system
#[derive(Debug, Clone)]
pub struct Decision {
    /// Whether to take the proposed action
    pub should_act: bool,
    /// Reasoning for the decision
    pub reason: String,
    /// Recommended approach
    pub approach: DecisionApproach,
    /// Confidence in this decision (0.0 - 1.0)
    pub confidence: f32,
}

/// Approach style for decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecisionApproach {
    /// Quick, minimal processing
    Fast,
    /// Standard processing with verification
    #[default]
    Standard,
    /// Thorough analysis with multiple passes
    Thorough,
}

/// Personality system that influences decision-making
pub struct Personality {
    /// Current personality traits
    traits: PersonalityTraits,

    /// Named personality presets
    presets: HashMap<String, PersonalityTraits>,

    /// Current active personality name
    current_preset: String,

    /// Experience history for learning
    experience_count: u32,
    success_count: u32,
}

impl Personality {
    /// Create a new personality with default traits
    pub fn new() -> Self {
        let mut presets = HashMap::new();

        // Define personality presets
        presets.insert("balanced".to_string(), PersonalityTraits::default());

        presets.insert(
            "analytical".to_string(),
            PersonalityTraits {
                curiosity: 0.8,
                caution: 0.8,
                creativity: 0.4,
                patience: 0.9,
                thoroughness: 0.95,
                verbosity: 0.4,
                risk_tolerance: 0.2,
            },
        );

        presets.insert(
            "creative".to_string(),
            PersonalityTraits {
                curiosity: 0.9,
                caution: 0.3,
                creativity: 0.95,
                patience: 0.5,
                thoroughness: 0.5,
                verbosity: 0.7,
                risk_tolerance: 0.7,
            },
        );

        presets.insert(
            "cautious".to_string(),
            PersonalityTraits {
                curiosity: 0.5,
                caution: 0.95,
                creativity: 0.3,
                patience: 0.8,
                thoroughness: 0.9,
                verbosity: 0.3,
                risk_tolerance: 0.1,
            },
        );

        presets.insert(
            "bold".to_string(),
            PersonalityTraits {
                curiosity: 0.7,
                caution: 0.2,
                creativity: 0.7,
                patience: 0.4,
                thoroughness: 0.6,
                verbosity: 0.6,
                risk_tolerance: 0.9,
            },
        );

        Self {
            traits: PersonalityTraits::default(),
            presets,
            current_preset: "balanced".to_string(),
            experience_count: 0,
            success_count: 0,
        }
    }

    /// Get current personality traits
    pub fn get_traits(&self) -> &PersonalityTraits {
        &self.traits
    }

    /// Get mutable reference to traits for direct modification
    pub fn traits_mut(&mut self) -> &mut PersonalityTraits {
        &mut self.traits
    }

    /// Set personality traits directly
    pub fn set_traits(&mut self, traits: PersonalityTraits) {
        self.traits = traits;
        self.current_preset = "custom".to_string();
    }

    /// Apply a personality preset
    pub fn apply_preset(&mut self, name: &str) -> bool {
        if let Some(traits) = self.presets.get(name) {
            self.traits = traits.clone();
            self.current_preset = name.to_string();
            true
        } else {
            false
        }
    }

    /// Get current preset name
    pub fn get_current_preset(&self) -> &str {
        &self.current_preset
    }

    /// List available presets
    pub fn list_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    /// Adjust a specific trait
    pub fn adjust_trait(&mut self, trait_name: &str, delta: f32) {
        match trait_name {
            "curiosity" => self.traits.curiosity = (self.traits.curiosity + delta).clamp(0.0, 1.0),
            "caution" => self.traits.caution = (self.traits.caution + delta).clamp(0.0, 1.0),
            "creativity" => {
                self.traits.creativity = (self.traits.creativity + delta).clamp(0.0, 1.0)
            }
            "patience" => self.traits.patience = (self.traits.patience + delta).clamp(0.0, 1.0),
            "thoroughness" => {
                self.traits.thoroughness = (self.traits.thoroughness + delta).clamp(0.0, 1.0)
            }
            "verbosity" => self.traits.verbosity = (self.traits.verbosity + delta).clamp(0.0, 1.0),
            "risk_tolerance" => {
                self.traits.risk_tolerance = (self.traits.risk_tolerance + delta).clamp(0.0, 1.0)
            }
            _ => {}
        }
        self.current_preset = "custom".to_string();
    }

    /// Adjust traits based on learning outcomes
    pub fn adapt_from_experience(&mut self, success: bool, risk_taken: bool) {
        self.experience_count += 1;

        if success {
            self.success_count += 1;
            if risk_taken {
                self.adjust_trait("creativity", 0.05);
                self.adjust_trait("risk_tolerance", 0.03);
            }
            self.adjust_trait("patience", 0.02);
            self.adjust_trait("curiosity", 0.01); // Success encourages more exploration
        } else {
            self.adjust_trait("caution", 0.05);
            if risk_taken {
                self.adjust_trait("risk_tolerance", -0.05);
                self.adjust_trait("creativity", -0.03);
            }
        }
    }

    /// Get success rate based on experience
    pub fn success_rate(&self) -> f32 {
        if self.experience_count == 0 {
            0.5
        } else {
            self.success_count as f32 / self.experience_count as f32
        }
    }

    /// Get communication style based on verbosity
    pub fn get_communication_style(&self) -> CommunicationStyle {
        if self.traits.verbosity < 0.3 {
            CommunicationStyle::Concise
        } else if self.traits.verbosity < 0.7 {
            CommunicationStyle::Balanced
        } else {
            CommunicationStyle::Detailed
        }
    }

    /// Decide if system should explore new approaches
    pub fn should_explore(&self, current_confidence: f32) -> bool {
        let exploration_tendency = self.traits.curiosity * 0.6 + (1.0 - self.traits.caution) * 0.4;
        let uncertainty_bonus = (1.0 - current_confidence) * 0.3;
        exploration_tendency + uncertainty_bonus > 0.5
    }

    /// Decide if system should take a risk
    pub fn should_take_risk(&self, potential_gain: f32, potential_loss: f32) -> bool {
        let risk_ratio = potential_gain / (potential_loss + 0.001);
        let adjusted_ratio = risk_ratio * (1.0 + self.traits.risk_tolerance);
        adjusted_ratio > 1.0
    }

    /// Get patience-based timeout recommendation
    pub fn get_timeout(&self, base_timeout_secs: u64) -> u64 {
        let patience_multiplier = 1.0 + self.traits.patience as f64;
        (base_timeout_secs as f64 * patience_multiplier) as u64
    }

    /// Make a decision based on context and personality
    pub fn decide(&self, context: &DecisionContext) -> Decision {
        let approach = self.determine_approach();
        // Time pressure and uncertainty tilt toward faster, more cautious
        // choices (Architecture: Personality System).
        let time_pressure = context.time_available < 5;
        let should_explore = self.should_explore(context.confidence);
        let mut should_act =
            self.should_take_risk(context.potential_gain, context.potential_loss);

        // High uncertainty with limited time favors caution.
        if context.uncertainty > 0.7 && time_pressure {
            should_act = false;
        }

        let reason = format!(
            "Based on {} personality (curiosity={:.2}, caution={:.2}, risk={:.2}, uncertainty={:.2}, time={}s): {}",
            self.current_preset,
            self.traits.curiosity,
            self.traits.caution,
            self.traits.risk_tolerance,
            context.uncertainty,
            context.time_available,
            if should_act {
                "choosing to act"
            } else if should_explore {
                "exploring cautiously"
            } else {
                "choosing caution"
            }
        );

        Decision {
            should_act,
            reason,
            approach,
            confidence: context.confidence,
        }
    }

    /// Determine processing approach based on thoroughness trait
    fn determine_approach(&self) -> DecisionApproach {
        if self.traits.thoroughness < 0.4 {
            DecisionApproach::Fast
        } else if self.traits.thoroughness < 0.8 {
            DecisionApproach::Standard
        } else {
            DecisionApproach::Thorough
        }
    }

    /// Format response according to personality's verbosity setting
    pub fn format_response(&self, content: &str) -> String {
        let style = self.get_communication_style();
        style.format_response(content)
    }

    /// Check if creative approach should be used
    pub fn should_use_creativity(&self, problem_complexity: f32) -> bool {
        // More creative personalities use creative approaches more often
        // But complexity increases need for creativity
        let creativity_threshold = 0.5 + (1.0 - problem_complexity) * 0.3;
        self.traits.creativity > creativity_threshold
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self::new()
    }
}

pub mod self_check;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let p = Personality::new();
        assert_eq!(p.get_current_preset(), "balanced");
        assert_eq!(p.get_traits().curiosity, 0.7);
    }

    #[test]
    fn test_apply_preset() {
        let mut p = Personality::new();
        assert!(p.apply_preset("analytical"));
        assert_eq!(p.get_current_preset(), "analytical");
        assert_eq!(p.get_traits().caution, 0.8);
        assert_eq!(p.get_traits().thoroughness, 0.95);
    }

    #[test]
    fn test_apply_invalid_preset() {
        let mut p = Personality::new();
        assert!(!p.apply_preset("nonexistent"));
        assert_eq!(p.get_current_preset(), "balanced");
    }

    #[test]
    fn test_list_presets() {
        let p = Personality::new();
        let presets = p.list_presets();
        assert!(presets.contains(&"balanced".to_string()));
        assert!(presets.contains(&"analytical".to_string()));
        assert!(presets.contains(&"creative".to_string()));
    }

    #[test]
    fn test_adjust_trait() {
        let mut p = Personality::new();
        p.adjust_trait("curiosity", 0.2);
        assert_eq!(p.get_traits().curiosity, 0.9);
        assert_eq!(p.get_current_preset(), "custom");
    }

    #[test]
    fn test_adjust_trait_clamping() {
        let mut p = Personality::new();
        p.adjust_trait("caution", 2.0); // Should clamp to 1.0
        assert_eq!(p.get_traits().caution, 1.0);
    }

    #[test]
    fn test_adapt_from_experience_success() {
        let mut p = Personality::new();
        let initial_creativity = p.get_traits().creativity;
        p.adapt_from_experience(true, true);
        assert!(p.get_traits().creativity > initial_creativity);
        assert_eq!(p.success_rate(), 1.0);
    }

    #[test]
    fn test_adapt_from_experience_failure() {
        let mut p = Personality::new();
        p.adjust_trait("risk_tolerance", 0.5); // Set high for test
        let initial_risk = p.get_traits().risk_tolerance;
        p.adjust_trait("caution", 0.0); // Reset caution for test
        p.adapt_from_experience(false, true);
        assert!(p.get_traits().risk_tolerance < initial_risk);
    }

    #[test]
    fn test_communication_style() {
        let mut p = Personality::new();

        p.traits_mut().verbosity = 0.2;
        assert_eq!(p.get_communication_style(), CommunicationStyle::Concise);

        p.traits_mut().verbosity = 0.5;
        assert_eq!(p.get_communication_style(), CommunicationStyle::Balanced);

        p.traits_mut().verbosity = 0.8;
        assert_eq!(p.get_communication_style(), CommunicationStyle::Detailed);
    }

    #[test]
    fn test_format_response() {
        let mut p = Personality::new();
        p.traits_mut().verbosity = 0.8;

        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let formatted = p.format_response(content);

        // Detailed style should keep most content
        assert!(formatted.len() > 10);

        // Concise style should truncate
        p.traits_mut().verbosity = 0.1;
        let concise = p.format_response(content);
        assert!(concise.len() < formatted.len());
    }

    #[test]
    fn test_should_explore() {
        let mut p = Personality::new();

        // High curiosity, low caution = explore
        p.traits_mut().curiosity = 0.9;
        p.traits_mut().caution = 0.2;
        assert!(p.should_explore(0.5));

        // Low curiosity, high caution = don't explore
        p.traits_mut().curiosity = 0.2;
        p.traits_mut().caution = 0.9;
        assert!(!p.should_explore(0.5));

        // Test with moderate traits and varying confidence
        p.traits_mut().curiosity = 0.4;
        p.traits_mut().caution = 0.6;
        // Exploration tendency = 0.4 * 0.6 + 0.4 * 0.4 = 0.24 + 0.16 = 0.40
        // With confidence 0.9: 0.40 + 0.1 * 0.3 = 0.43 < 0.5 = false
        assert!(!p.should_explore(0.9)); // High confidence
                                         // With confidence 0.5: 0.40 + 0.5 * 0.3 = 0.55 > 0.5 = true
        assert!(p.should_explore(0.5)); // Low confidence
    }

    #[test]
    fn test_should_take_risk() {
        let mut p = Personality::new();

        // Low risk tolerance = don't take risk
        p.traits_mut().risk_tolerance = 0.1;
        // risk_ratio = 0.6 / 0.401 = 1.5, adjusted = 1.5 * 1.1 = 1.65 > 1 = true
        // Need very low ratio
        assert!(!p.should_take_risk(0.3, 0.7)); // Bad ratio

        // High risk tolerance = take risk with decent ratio
        p.traits_mut().risk_tolerance = 0.9;
        // risk_ratio = 0.6 / 0.401 = 1.5, adjusted = 1.5 * 1.9 = 2.85 > 1 = true
        assert!(p.should_take_risk(0.6, 0.4));
    }

    #[test]
    fn test_get_timeout() {
        let mut p = Personality::new();

        p.traits_mut().patience = 0.5;
        let timeout = p.get_timeout(100);
        assert_eq!(timeout, 150); // 100 * (1 + 0.5)

        p.traits_mut().patience = 1.0;
        let timeout = p.get_timeout(100);
        assert_eq!(timeout, 200); // 100 * (1 + 1.0)
    }

    #[test]
    fn test_decide() {
        let mut p = Personality::new();
        assert!(p.apply_preset("cautious"));

        let context = DecisionContext {
            confidence: 0.3,
            potential_gain: 0.8,
            potential_loss: 0.2,
            uncertainty: 0.6,
            time_available: 60,
        };

        let decision = p.decide(&context);
        assert!(decision.reason.contains("cautious"));
        assert_eq!(decision.approach, DecisionApproach::Thorough);
    }

    #[test]
    fn test_should_use_creativity() {
        let mut p = Personality::new();

        p.traits_mut().creativity = 0.9;
        assert!(p.should_use_creativity(0.5));

        p.traits_mut().creativity = 0.2;
        assert!(!p.should_use_creativity(0.5));
    }

    #[test]
    fn test_success_rate() {
        let mut p = Personality::new();
        assert_eq!(p.success_rate(), 0.5); // No experience

        p.adapt_from_experience(true, false);
        assert_eq!(p.success_rate(), 1.0);

        p.adapt_from_experience(false, false);
        assert_eq!(p.success_rate(), 0.5);
    }
}
