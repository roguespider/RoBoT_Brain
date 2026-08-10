//! Personality preset definitions and management (Architecture §13).

use std::collections::HashMap;

use super::traits::PersonalityTraits;
use super::personality::Personality;

/// Build the default set of personality presets.
pub fn default_presets() -> HashMap<String, PersonalityTraits> {
    let mut presets = HashMap::new();

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

    presets
}

impl Personality {
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
}
