//! The personality system that influences decision-making.

use super::emotional;
use super::presets::default_presets;
use super::traits::PersonalityTraits;

/// Personality system that influences decision-making
#[derive(Clone)]
pub struct Personality {
    /// Current personality traits
    pub(crate) traits: PersonalityTraits,

    /// Named personality presets
    pub(crate) presets: std::collections::HashMap<String, PersonalityTraits>,

    /// Current active personality name
    pub(crate) current_preset: String,

    /// Experience history for learning
    pub(crate) experience_count: u32,
    pub(crate) success_count: u32,

    /// Emotional state — feeds confidence/decision scoring (Architecture §13,
    /// TASK-V2-08). Not just text style: emotion nudges confidence and the
    /// action threshold.
    pub(crate) emotional_state: emotional::EmotionalState,

    /// Stable humor trait (output style).
    pub(crate) humor: emotional::Humor,

    /// Interaction policy: how the agent prefers to engage.
    pub(crate) interaction_mode: emotional::InteractionMode,

    /// Action-selection preferences.
    pub(crate) preferences: emotional::Preferences,
}

impl Personality {
    /// Create a new personality with default traits
    pub fn new() -> Self {
        Self {
            traits: PersonalityTraits::default(),
            presets: default_presets(),
            current_preset: "balanced".to_string(),
            experience_count: 0,
            success_count: 0,
            emotional_state: emotional::EmotionalState::default(),
            humor: emotional::Humor::default(),
            interaction_mode: emotional::InteractionMode::default(),
            preferences: emotional::Preferences::default(),
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

    /// Capture a full snapshot of the personality state. Used by explicit
    /// diagnostics to run mutation probes without permanently altering the
    /// live shared personality.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    /// Restore a previously captured snapshot, undoing any mutations made
    /// after it was taken (traits, preset name, experience counters,
    /// emotional state, humor, preferences).
    pub fn restore(&mut self, snapshot: &Self) {
        *self = snapshot.clone();
    }

    /// Observe an outcome and update emotional state (Architecture §13).
    /// `effort` (0.0–1.0) reflects how much the agent invested in the action.
    pub fn observe_emotional_outcome(&mut self, success: bool, effort: f32) {
        self.emotional_state.observe(success, effort);
    }

    /// Current emotional weight (Architecture §13). Exposed so the agent loop
    /// can fold emotion into confidence scoring outside `decide()`.
    pub fn emotional_weight(&self) -> f32 {
        self.emotional_state.emotional_weight()
    }

    /// Current humor trait level (output style).
    pub fn humor_level(&self) -> f32 {
        self.humor.level
    }

    /// Set the humor trait level (0.0 = serious, 1.0 = playful).
    pub fn set_humor_level(&mut self, level: f32) {
        self.humor = emotional::Humor::new(level);
    }

    /// Current interaction mode.
    pub fn interaction_mode(&self) -> emotional::InteractionMode {
        self.interaction_mode
    }

    /// Current action-selection preferences.
    pub fn preferences(&self) -> emotional::Preferences {
        self.preferences
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self::new()
    }
}
