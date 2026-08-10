//! Trait adjustment and experience-based adaptation (Architecture §13).

use super::personality::Personality;

impl Personality {
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
            self.adjust_trait("curiosity", 0.01);
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
}
