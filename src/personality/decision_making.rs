//! Decision-making logic driven by personality traits and emotional state
//! (Architecture §13).

use super::communication::CommunicationStyle;
use super::decision::{Decision, DecisionApproach, DecisionContext};
use super::personality::Personality;

impl Personality {
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
        let time_pressure = context.time_available < 5;
        let should_explore = self.should_explore(context.confidence);

        // Emotional weighting feeds the decision (Architecture §13,
        // TASK-V2-08): adjust confidence by emotional weight, and bias the
        // action threshold by frustration/engagement.
        let emotional_weight = self.emotional_state.emotional_weight();
        let emotion_adjusted_confidence =
            (context.confidence + emotional_weight).clamp(0.0, 1.0);
        let threshold_bias = self.emotional_state.action_threshold_bias();

        let mut should_act =
            self.should_take_risk(context.potential_gain, context.potential_loss);
        if threshold_bias > 0.0 {
            let gain_margin = context.potential_gain - context.potential_loss;
            if gain_margin < threshold_bias {
                should_act = false;
            }
        }

        if context.uncertainty > 0.7 && time_pressure {
            should_act = false;
        }

        let reason = format!(
            "Based on {} personality (curiosity={:.2}, caution={:.2}, risk={:.2}, \
             uncertainty={:.2}, time={}s, emotion_weight={:+.2}, interaction={:?}): {}",
            self.current_preset,
            self.traits.curiosity,
            self.traits.caution,
            self.traits.risk_tolerance,
            context.uncertainty,
            context.time_available,
            emotional_weight,
            self.interaction_mode,
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
            confidence: emotion_adjusted_confidence,
        }
    }

    /// Determine processing approach based on thoroughness trait
    pub(super) fn determine_approach(&self) -> DecisionApproach {
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
        let creativity_threshold = 0.5 + (1.0 - problem_complexity) * 0.3;
        self.traits.creativity > creativity_threshold
    }
}
