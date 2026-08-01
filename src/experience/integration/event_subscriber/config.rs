// src/experience/integration/event_subscriber/config.rs

//! Configuration for event subscription behavior

/// Configuration for event subscription behavior
#[derive(Debug, Clone)]
pub struct EventSubscriberConfig {
    /// Whether to auto-generate reflections
    pub auto_reflect: bool,
    /// Whether to auto-generate hypotheses
    pub auto_hypothesize: bool,
    /// Minimum score to trigger reflection
    pub reflection_threshold: f32,
    /// Whether to update knowledge from experiences
    pub auto_update_knowledge: bool,
}

impl Default for EventSubscriberConfig {
    fn default() -> Self {
        Self {
            auto_reflect: true,
            auto_hypothesize: true,
            reflection_threshold: 0.6,
            auto_update_knowledge: true,
        }
    }
}
