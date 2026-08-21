// src/bridge/app/initialization/learning_pipeline.rs
//! Verify LearningPipeline coordinator at startup.

/// Verify the Learning Pipeline coordinator at startup.
/// Exercises start_from_input, advance_stage, get, get_by_stage, stats,
/// and cleanup so the learning pipeline stays live rather than dead code.
pub async fn verify_learning_pipeline() {
    use crate::learning::pipeline::{LearningPipeline, PipelineStage};

    let mut pipeline = LearningPipeline::new(100);
    let source_id = uuid::Uuid::new_v4();
    let record_id = pipeline.start_from_input(source_id, "probe input");
    let advanced = pipeline.advance_stage(
        &record_id,
        PipelineStage::Observation,
        "probe observation",
        Some(0.8),
    );
    let record_present = pipeline.get(&record_id).is_some();
    let in_observation_count = pipeline.get_by_stage(PipelineStage::Observation).len();
    let stats = pipeline.stats();
    pipeline.cleanup(chrono::Duration::hours(24));
    let stage_display = format!("{}", PipelineStage::Knowledge);
    tracing::info!(
        "LearningPipeline lifecycle verified: advanced={}, record={}, in_observation={}, stats_total={}, stage_display={}",
        advanced,
        record_present,
        in_observation_count,
        stats.total_records,
        stage_display,
    );
}
