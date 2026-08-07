// /src/experience/reflection/engine/reports.rs
//! Report types for the reflection engine

/// Report from analyzing experiences
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub patterns: Vec<String>,
    pub themes: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence: f32,
}

/// Report from validating a reflection
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub score: f32,
    pub issues: Vec<String>,
    pub quality_score: f32,
    pub suggestions: Vec<String>,
}

/// Statistics about the reflection engine
#[derive(Debug)]
pub struct EngineStats {
    pub total_reflections: usize,
    pub total_insights: usize,
    pub trusted_insights: usize,
    pub total_patterns: usize,
    pub mature_patterns: usize,
}
