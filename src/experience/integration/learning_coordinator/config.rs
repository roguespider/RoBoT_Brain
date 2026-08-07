// src/experience/integration/learning_coordinator/config.rs
//! Configuration for the learning coordinator

/// Configuration for the learning coordinator
#[derive(Debug, Clone)]
pub struct LearningCoordinatorConfig {
    /// Whether to auto-reflect on experiences
    pub auto_reflect: bool,
    /// Minimum score to trigger reflection
    pub reflection_threshold: f32,
    /// Whether to auto-generate hypotheses
    pub auto_hypothesize: bool,
    /// Whether to auto-explore hypotheses
    pub auto_explore: bool,
    /// Minimum confidence for hypothesis validation
    pub hypothesis_validation_threshold: f32,
    /// Whether to promote high-confidence hypotheses to knowledge
    pub auto_promote_to_knowledge: bool,
    /// Batch size for reflection processing
    pub reflection_batch_size: usize,
    /// How often to run maintenance (in seconds)
    pub maintenance_interval_secs: u64,
}

impl Default for LearningCoordinatorConfig {
    fn default() -> Self {
        Self {
            auto_reflect: true,
            reflection_threshold: 0.6,
            auto_hypothesize: true,
            auto_explore: false,
            hypothesis_validation_threshold: 0.75,
            auto_promote_to_knowledge: true,
            reflection_batch_size: 5,
            maintenance_interval_secs: 300, // 5 minutes
        }
    }
}
