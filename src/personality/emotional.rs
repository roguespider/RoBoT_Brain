// src/personality/emotional.rs
//! Emotional weighting subsystem (Architecture §13, TASK-V2-08).
//!
//! The architecture calls for personality to go *beyond* communication style:
//! emotional state should feed confidence and decision scoring, not just text
//! formatting. This module models a small set of emotional dimensions and
//! produces a numeric weight that adjusts how confident the agent should be in
//! an action, and how willing it is to act at all.
//!
//! Emotional dimensions (each 0.0–1.0):
//!
//!   * `engagement`  — how invested/energized the agent is. High engagement
//!     *raises* confidence in familiar actions and lowers the action threshold.
//!   * `frustration` — accumulated friction from recent failures. High
//!     frustration *lowers* confidence and raises the action threshold (the
//!     agent becomes more cautious, mirroring human "I keep hitting walls"
//!     behavior).
//!   * `satisfaction` — accumulated success. High satisfaction slightly raises
//!     confidence but, paradoxically, also raises exploration willingness.
//!   * `humor` — a stable trait (not a state): how much the agent values
//!     levity in communication. Pure output style; does not affect decisions.
//!
//! The combined `emotional_weight()` is added to the decision confidence and
//! biases `should_act`. This is the "emotional weighting feeds confidence /
//! decision scoring" requirement from the roadmap.

use serde::{Deserialize, Serialize};

/// A stable personality trait for humor (output style only).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Humor {
    /// 0.0 = serious, 1.0 = playful.
    pub level: f32,
}

impl Default for Humor {
    fn default() -> Self {
        Self { level: 0.3 }
    }
}

impl Humor {
    pub fn new(level: f32) -> Self {
        Self {
            level: level.clamp(0.0, 1.0),
        }
    }
}

/// The agent's current emotional state across four dimensions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Investment / energy (0.0–1.0).
    pub engagement: f32,
    /// Accumulated friction from recent failures (0.0–1.0).
    pub frustration: f32,
    /// Accumulated success signal (0.0–1.0).
    pub satisfaction: f32,
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self {
            engagement: 0.5,
            frustration: 0.0,
            satisfaction: 0.5,
        }
    }
}

impl EmotionalState {
    /// Update state from a single outcome (Architecture §13 emotional loop).
    pub fn observe(&mut self, success: bool, effort: f32) {
        let effort = effort.clamp(0.0, 1.0);
        if success {
            self.satisfaction = (self.satisfaction + 0.1 * effort).min(1.0);
            self.frustration = (self.frustration - 0.15).max(0.0);
            self.engagement = (self.engagement + 0.05).min(1.0);
        } else {
            self.frustration = (self.frustration + 0.2 * effort).min(1.0);
            self.satisfaction = (self.satisfaction - 0.05).max(0.0);
            self.engagement = (self.engagement - 0.03).max(0.0);
        }
    }

    /// Produce a combined emotional weight in [-0.3, +0.3] that adjusts a
    /// decision confidence. Engagement and satisfaction raise confidence;
    /// frustration lowers it. The magnitude is deliberately small so emotion
    /// *nudges* rather than overrides evidence-based confidence (§13).
    pub fn emotional_weight(&self) -> f32 {
        let positive = self.engagement * 0.15 + self.satisfaction * 0.15;
        let negative = self.frustration * 0.30;
        (positive - negative).clamp(-0.3, 0.3)
    }

    /// Bias on the action threshold: high frustration raises the bar to act
    /// (more caution); high engagement lowers it. Range [-0.15, +0.15].
    pub fn action_threshold_bias(&self) -> f32 {
        (self.frustration * 0.15 - self.engagement * 0.05).clamp(-0.15, 0.15)
    }
}

/// Interaction policy: how the agent prefers to engage (Architecture §13).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InteractionMode {
    /// Proactively offers information and suggestions.
    #[default]
    Proactive,
    /// Waits to be asked, then answers.
    Reactive,
    /// Minimal, only surfaces critical information.
    Minimal,
}

/// Preferences that bias action selection (Architecture §13 "preferences").
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Preferences {
    /// Prefer shorter plans over longer ones (0.0–1.0).
    pub brevity: f32,
    /// Prefer well-trodden paths over novel ones (0.0–1.0).
    pub familiarity: f32,
    /// How much to value reversibility of actions (0.0–1.0).
    pub reversibility: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            brevity: 0.5,
            familiarity: 0.6,
            reversibility: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_lowers_frustration() {
        let mut state = EmotionalState {
            frustration: 0.5,
            ..EmotionalState::default()
        };
        state.observe(true, 1.0);
        assert!(state.frustration < 0.5);
        assert!(state.satisfaction > 0.5);
    }

    #[test]
    fn failure_raises_frustration_and_lowers_weight() {
        let mut state = EmotionalState::default();
        let before = state.emotional_weight();
        state.observe(false, 1.0);
        let after = state.emotional_weight();
        assert!(after < before);
        assert!(state.frustration > 0.0);
    }

    #[test]
    fn emotional_weight_is_bounded() {
        let mut state = EmotionalState {
            engagement: 1.0,
            frustration: 1.0,
            satisfaction: 1.0,
        };
        let w = state.emotional_weight();
        assert!(w >= -0.3 && w <= 0.3);

        state.frustration = 0.0;
        let w_pos = state.emotional_weight();
        assert!(w_pos > w);
    }
}
