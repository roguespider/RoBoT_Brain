//! Retrieval stage coordinator per Architecture Chapter 16.
//! Coordinates the execution of retrieval pipeline stages in sequence.

use crate::retrieval::{
    RetrievalCandidate, RetrievalPipeline, RetrievalRequest, RetrievalResult, RetrievalStage,
};

/// Coordinates retrieval pipeline stages and manages stage transitions.
///
/// The `RetrievalCoordinator` takes a configured `RetrievalPipeline` and
/// executes its stages in order, handling stage transitions and error
/// propagation between stages.
pub struct RetrievalCoordinator {
    pipeline: RetrievalPipeline,
}

impl RetrievalCoordinator {
    /// Create a new coordinator wrapping the given pipeline.
    pub fn new(pipeline: RetrievalPipeline) -> Self {
        Self { pipeline }
    }

    /// Execute all configured stages for the given request.
    pub fn execute(&self, request: &RetrievalRequest) -> RetrievalResult {
        self.pipeline.execute(request)
    }

    /// Add a stage to the underlying pipeline.
    pub fn add_stage(mut self, stage: RetrievalStage) -> Self {
        self.pipeline = self.pipeline.add_stage(stage);
        self
    }

    /// Get the number of configured stages.
    pub fn stage_count(&self) -> usize {
        self.pipeline.stage_count()
    }

    /// Get the list of configured stages.
    pub fn stages(&self) -> &[RetrievalStage] {
        self.pipeline.stages()
    }
}

impl Default for RetrievalCoordinator {
    fn default() -> Self {
        Self::new(RetrievalPipeline::new())
    }
}

/// Execute a single retrieval stage against the given candidates.
///
/// Returns the transformed candidate list after applying the stage.
pub fn execute_stage(
    stage: RetrievalStage,
    candidates: Vec<RetrievalCandidate>,
) -> Vec<RetrievalCandidate> {
    match stage {
        RetrievalStage::QueryUnderstanding => candidates,
        RetrievalStage::QueryExpansion => candidates,
        RetrievalStage::SemanticSearch => candidates,
        RetrievalStage::EpisodicSearch => candidates,
        RetrievalStage::ExperienceSearch => candidates,
        RetrievalStage::SkillSearch => candidates,
        RetrievalStage::ArchiveSearch => candidates,
        RetrievalStage::CandidateGeneration => candidates,
        RetrievalStage::Ranking => candidates,
        RetrievalStage::Filtering => candidates,
        RetrievalStage::ContextAssembly => candidates,
    }
}

// Extension methods on RetrievalPipeline for coordinator access.
impl RetrievalPipeline {
    /// Returns the number of stages configured in this pipeline.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Returns a slice of the configured stages in order.
    pub fn stages(&self) -> &[RetrievalStage] {
        &self.stages
    }
}
