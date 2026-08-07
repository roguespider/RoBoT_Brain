// /src/experience/reflection/engine/config.rs
//! Configuration for the reflection engine

/// Configuration for the reflection engine
#[derive(Debug, Clone)]
pub struct ReflectionEngineConfig {
    /// Minimum experiences before auto-generating reflection
    pub min_experiences_for_auto_reflection: usize,

    /// Minimum confidence for valid reflection
    pub min_confidence: f32,

    /// Auto-validate reflections above this confidence
    pub auto_validate_threshold: f32,

    /// Maximum reflections to keep in memory
    pub max_cached_reflections: usize,
}

impl Default for ReflectionEngineConfig {
    fn default() -> Self {
        Self {
            min_experiences_for_auto_reflection: 3,
            min_confidence: 0.5,
            auto_validate_threshold: 0.8,
            max_cached_reflections: 1000,
        }
    }
}
