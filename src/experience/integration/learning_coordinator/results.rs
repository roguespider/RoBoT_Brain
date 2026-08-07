// src/experience/integration/learning_coordinator/results.rs
//! Result types for the learning coordinator

// ========================================================================
// Result Types
// ========================================================================

/// Result of processing an experience through the learning pipeline
#[derive(Debug, Default)]
pub struct LearningResult {
    pub experience_id: uuid::Uuid,
    pub score: f32,
    pub reflection_id: Option<String>,
    pub hypothesis_ids: Vec<String>,
    pub knowledge_id: Option<uuid::Uuid>,
}

/// Result of validating a hypothesis
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub hypothesis_id: String,
    pub is_valid: bool,
    pub confidence: f32,
    pub promoted_to_knowledge: bool,
}

/// Statistics from maintenance run
#[derive(Debug, Default)]
pub struct MaintenanceStats {
    pub hypotheses_decayed: usize,
    pub explorations_archived: usize,
    pub knowledge_consolidated: usize,
}

/// Statistics about the learning coordinator
#[derive(Debug)]
pub struct LearningCoordinatorStats {
    pub total_reflections: usize,
    pub total_insights: usize,
    pub trusted_insights: usize,
    pub total_patterns: usize,
    pub active_reputations: usize,
    pub active_explorations: usize,
}

// ========================================================================
// Learning Result Types
// ========================================================================

/// Result of reinforcement learning
#[derive(Debug, Default)]
pub struct ReinforcementResult {
    pub experience_id: uuid::Uuid,
    pub reward: f64,
    pub knowledge_updates: usize,
    pub skill_updates: usize,
    pub action_value_delta: f64,
}

/// Pattern extracted from generalization
#[derive(Debug, Clone)]
pub struct LearningPattern {
    pub description: String,
    pub confidence: f32,
    pub source_experience_count: usize,
    pub pattern_type: PatternKind,
}

/// Type of pattern for learning coordinator
#[derive(Debug, Clone)]
pub enum PatternKind {
    Contextual,
    Temporal,
    Causal,
    Sequential,
}

/// Result of generalization
#[derive(Debug, Default)]
pub struct GeneralizationResult {
    pub patterns: Vec<LearningPattern>,
    pub generalized_knowledge_count: usize,
}

/// Result of transfer learning
#[derive(Debug)]
pub struct TransferResult {
    pub source_domain: String,
    pub target_domain: String,
    pub transferred_count: usize,
    pub adapted_count: usize,
    pub failed_count: usize,
    pub new_knowledge_ids: Vec<uuid::Uuid>,
}
