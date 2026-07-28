// src/personality/mod.rs

//! Personality System
//!
//! Per Architecture: Defines the behavioral characteristics of the AI system,
//! influencing decision-making, communication style, and learning preferences.


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

/// Personality system that influences decision-making
pub struct Personality {
    /// Current personality traits
    traits: PersonalityTraits,
    
    /// Named personality presets
    presets: HashMap<String, PersonalityTraits>,
    
    /// Current active personality name
    current_preset: String,
}

impl Personality {
    /// Create a new personality with default traits
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        
        // Define personality presets
        presets.insert("balanced".to_string(), PersonalityTraits::default());
        
        presets.insert("analytical".to_string(), PersonalityTraits {
            curiosity: 0.8,
            caution: 0.8,
            creativity: 0.4,
            patience: 0.9,
            thoroughness: 0.95,
            verbosity: 0.4,
            risk_tolerance: 0.2,
        });
        
        presets.insert("creative".to_string(), PersonalityTraits {
            curiosity: 0.9,
            caution: 0.3,
            creativity: 0.95,
            patience: 0.5,
            thoroughness: 0.5,
            verbosity: 0.7,
            risk_tolerance: 0.7,
        });
        
        presets.insert("cautious".to_string(), PersonalityTraits {
            curiosity: 0.5,
            caution: 0.95,
            creativity: 0.3,
            patience: 0.8,
            thoroughness: 0.9,
            verbosity: 0.3,
            risk_tolerance: 0.1,
        });
        
        presets.insert("bold".to_string(), PersonalityTraits {
            curiosity: 0.7,
            caution: 0.2,
            creativity: 0.7,
            patience: 0.4,
            thoroughness: 0.6,
            verbosity: 0.6,
            risk_tolerance: 0.9,
        });
        
        Self {
            traits: PersonalityTraits::default(),
            presets,
            current_preset: "balanced".to_string(),
        }
    }
    
    /// Get current personality traits
    pub fn get_traits(&self) -> &PersonalityTraits {
        &self.traits
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
            "creativity" => self.traits.creativity = (self.traits.creativity + delta).clamp(0.0, 1.0),
            "patience" => self.traits.patience = (self.traits.patience + delta).clamp(0.0, 1.0),
            "thoroughness" => self.traits.thoroughness = (self.traits.thoroughness + delta).clamp(0.0, 1.0),
            "verbosity" => self.traits.verbosity = (self.traits.verbosity + delta).clamp(0.0, 1.0),
            "risk_tolerance" => self.traits.risk_tolerance = (self.traits.risk_tolerance + delta).clamp(0.0, 1.0),
            _ => {}
        }
        self.current_preset = "custom".to_string();
    }
    
    /// Adjust traits based on learning outcomes
    pub fn adapt_from_experience(&mut self, success: bool, risk_taken: bool) {
        if success {
            if risk_taken {
                self.adjust_trait("creativity", 0.05);
                self.adjust_trait("risk_tolerance", 0.03);
            }
            self.adjust_trait("patience", 0.02);
        } else {
            self.adjust_trait("caution", 0.05);
            if risk_taken {
                self.adjust_trait("risk_tolerance", -0.05);
                self.adjust_trait("creativity", -0.03);
            }
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
}

/// Communication style preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationStyle {
    Concise,
    Balanced,
    Detailed,
}

impl Default for Personality {
    fn default() -> Self {
        Self::new()
    }
}
