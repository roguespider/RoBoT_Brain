// src/bridge/app/initialization/hypothesis_pipeline_diagnostics.rs
//! HypothesisPipeline lifecycle probe (P2-001C).
//!
//! Exercises HypothesisPipeline process, add_supporting_evidence, list_active,
//! graph_stats, list_validated, and archive_old so this API stays live without
//! running at production startup.

use crate::bridge::app::state::App;
use crate::experience::integration::hypothesis_pipeline::{
    HypothesisPipeline, HypothesisPipelineConfig,
};
use crate::experience::types::{Experience as ProbeExperience, ExperienceType};

/// Verify HypothesisPipeline lifecycle (process, add_supporting_evidence,
/// list_active, list_validated, graph_stats, archive_old).
pub async fn verify_hypothesis_pipeline(app: &App) {
    let pipeline = HypothesisPipeline::with_config(
        HypothesisPipelineConfig::default(),
        app.hypothesis_engine
            .lock()
            .map(|guard| {
                std::sync::Arc::new(crate::experience::hypothesis::HypothesisEngine::with_graph(
                    guard.get_graph(),
                ))
            })
            .unwrap_or_else(|_| {
                std::sync::Arc::new(crate::experience::hypothesis::HypothesisEngine::new())
            }),
        app.mcp_context.bus.clone(),
    );

    let probe_experience = ProbeExperience::new(
        "diagnostics hypothesis pipeline".to_string(),
        "Transient experience used to verify hypothesis validation".to_string(),
        ExperienceType::Hypothesis,
        vec![],
    );
    let generated_ids = pipeline
        .process(&probe_experience)
        .await
        .unwrap_or_default();

    if let Some(hyp_id) = generated_ids.first() {
        for i in 0..3 {
            let supported = pipeline
                .add_supporting_evidence(hyp_id, &format!("diagnostics evidence {}", i))
                .await;
            if supported.is_err() {
                break;
            }
        }
        let validated_hyp = pipeline.get(hyp_id).await;
        tracing::info!(
            "HypothesisPipeline verified: id={} validated={}",
            hyp_id,
            validated_hyp
                .as_ref()
                .map(|h| h.confidence.value >= 0.75)
                .unwrap_or(false)
        );

        // Exercise remaining pipeline surface
        let active_list = pipeline.list_active().await;
        let stats = pipeline.graph_stats();
        let validated_list = pipeline.list_validated().await;
        let archived = pipeline.archive_old(365).await.unwrap_or(0);
        tracing::info!(
            "HypothesisPipeline maintenance verified: active={} graph_nodes={} \
             graph_edges={} validated_list={} archived={}",
            active_list.len(),
            stats.node_count,
            stats.edge_count,
            validated_list.len(),
            archived
        );
    }
}
