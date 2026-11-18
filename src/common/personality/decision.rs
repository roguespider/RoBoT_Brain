//! Decision types used by the personality system.

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
