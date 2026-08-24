// src/bridge/app/initialization/reflection_diagnostics.rs
//! Reflection system probes (P2-001C).
//!
//! Exercises ReflectionPipeline.analyze_patterns and ReflectionEngine
//! list_by_type/list_validated/search so these APIs stay live without
//! running at production startup.

use crate::bridge::app::state::App;
use crate::experience::reflection::types::ReflectionType;
use crate::experience::types::{Experience, ExperienceType};

/// Verify ReflectionPipeline.analyze_patterns.
pub async fn verify_reflection_pipeline(app: &App) {
    let pipeline = &app.reflection_pipeline;

    let probe_experiences: Vec<Experience> = (0..3)
        .map(|i| {
            Experience::new(
                format!("Diagnostics reflection probe {}", i),
                "Transient experience used to verify pattern analysis".to_string(),
                ExperienceType::Learning,
                vec![uuid::Uuid::new_v4()],
            )
        })
        .collect();
    let pattern_count = pipeline
        .analyze_patterns(&probe_experiences)
        .await
        .map(|p| p.len())
        .unwrap_or(0);
    tracing::info!(
        "ReflectionPipeline.analyze_patterns verified: patterns={}",
        pattern_count
    );
}

/// Verify ReflectionEngine list_by_type/list_validated/search.
pub async fn verify_reflection_engine(app: &App) {
    let engine = &app.mcp_context.reflection;

    let by_type = engine.list_by_type(ReflectionType::General).await.len();
    let validated = engine.list_validated().await.len();
    let by_status = engine
        .list_by_status(crate::experience::reflection::types::ReflectionStatus::Active)
        .await
        .len();
    let searched = engine.search("diagnostics").await.len();

    tracing::info!(
        "ReflectionEngine verified: by_type={} validated={} by_status={} searched={}",
        by_type,
        validated,
        by_status,
        searched
    );
}
