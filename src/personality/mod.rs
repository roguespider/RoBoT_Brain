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

pub mod adaptation;
pub mod communication;
pub mod decision;
pub mod decision_making;
pub mod emotional;
pub mod personality;
pub mod presets;
pub mod traits;

pub use communication::CommunicationStyle;
pub use decision::{Decision, DecisionApproach, DecisionContext};
pub use personality::Personality;
pub use traits::PersonalityTraits;

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
