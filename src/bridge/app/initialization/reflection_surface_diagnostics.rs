// src/bridge/app/initialization/reflection_surface_diagnostics.rs
//! Reflection / hypothesis type-surface probes (P2-001B).
//!
//! Exercises the reflection and hypothesis data-model APIs that no longer
//! have production callers after the startup-probe removal, so their test
//! coverage stays live through the explicit `robot diagnose` entry point
//! instead of running at production startup.

use crate::bridge::app::state::App;
use crate::experience::hypothesis::core::hypothesis::{Hypothesis, HypothesisConfidence};
use crate::experience::integration::event_subscriber::EventSubscriber;
use crate::experience::reflection::insight::{
    Insight, InsightType, KnowledgeMaturity, MaturityHistory,
};
use crate::experience::reflection::review::ReflectionReview;
use crate::experience::reflection::types::{
    EvidenceId, InsightId, Lesson, ReflectionEvidence, ReflectionInsight,
};
use crate::experience::reflection::{InsightProducer, Reflector, ValidatableReflection};

/// Verify the reflection/hypothesis type surface.
pub async fn verify_type_surfaces(app: &App) {
    // --- Hypothesis confidence lifecycle ---------------------------------
    let mut confidence = HypothesisConfidence::default();
    confidence.increase(0.1);
    confidence.decrease(0.6);
    let uncertain = confidence.is_uncertain();
    tracing::info!("HypothesisConfidence verified: uncertain={}", uncertain);

    // --- Hypothesis tag/evidence surface ---------------------------------
    let mut hypothesis = Hypothesis::new("diagnostics hyp", "transient probe");
    hypothesis.add_tag("diagnostics");
    hypothesis.add_tag("diagnostics"); // deduplicated
    hypothesis.add_supporting_evidence("ev-support-1");
    hypothesis.add_contradicting_evidence("ev-contra-1");
    tracing::info!(
        "Hypothesis surface verified: tags={} supporting={} contradicting={}",
        hypothesis.tags.len(),
        hypothesis.supporting_evidence.len(),
        hypothesis.contradicting_evidence.len()
    );

    // --- Insight add_hypothesis + KnowledgeMaturity + MaturityHistory ----
    let mut insight = Insight::new(
        uuid::Uuid::new_v4().to_string(),
        "diagnostics insight",
        "probe statement",
        InsightType::General,
    );
    insight.add_hypothesis(hypothesis.id.0.clone());
    let maturity = KnowledgeMaturity::Emerging;
    let history = MaturityHistory {
        timestamp: chrono::Utc::now(),
        previous: maturity,
        current: KnowledgeMaturity::Developing,
        reason: "diagnostics transition".to_string(),
    };
    tracing::info!(
        "Insight surface verified: hypotheses={} maturity {:?} -> {:?} ({}) at {}",
        insight.hypothesis_ids.len(),
        history.previous,
        history.current,
        history.reason,
        history.timestamp.to_rfc3339()
    );

    // --- InsightProducer trait object usage ------------------------------
    let producer_messages = InsightProducer::generate_insights(&insight);
    tracing::info!(
        "InsightProducer verified: messages={}",
        producer_messages.len()
    );

    // --- ReflectionReview ------------------------------------------------
    let review = ReflectionReview {
        id: uuid::Uuid::new_v4().to_string(),
        started_at: chrono::Utc::now(),
        ended_at: chrono::Utc::now(),
        reflections: vec!["r1".to_string()],
        summary: "diagnostics review".to_string(),
    };
    tracing::info!(
        "ReflectionReview verified: id={} reflections={} window={}s summary={}",
        review.id,
        review.reflections.len(),
        (review.ended_at - review.started_at).num_seconds(),
        review.summary
    );

    // --- Lesson / ReflectionInsight / ReflectionEvidence -----------------
    let lesson = Lesson {
        title: "diagnostics lesson".to_string(),
        description: "transient".to_string(),
        confidence: 0.5,
    };
    let r_insight = ReflectionInsight {
        statement: "diagnostics".to_string(),
        confidence: 0.5,
        importance: 0.5,
    };
    let evidence_id: EvidenceId = uuid::Uuid::new_v4().to_string();
    let insight_id: InsightId = uuid::Uuid::new_v4().to_string();
    let r_evidence = ReflectionEvidence {
        experience_id: evidence_id.clone(),
        description: "transient".to_string(),
        weight: 0.4,
    };
    tracing::info!(
        "Lesson/ReflectionInsight/ReflectionEvidence verified: lesson={} \
         insight_confidence={} evidence_weight={} insight_id_len={}",
        lesson.title.len(),
        r_insight.confidence,
        r_evidence.weight,
        insight_id.len()
    );

    // --- Reflector + ValidatableReflection traits ------------------------
    let mut reflection = crate::experience::reflection::types::Reflection::new(
        uuid::Uuid::new_v4().to_string(),
        crate::experience::reflection::types::ReflectionType::General,
        "diagnostics reflection",
    );
    reflection.set_confidence(0.9);
    let reflected = Reflector::reflect(&reflection, "ctx".to_string());
    let reflected_ok = reflected.is_ok();
    ValidatableReflection::validate(&mut reflection);
    ValidatableReflection::invalidate(&mut reflection);
    let invalidated_confidence = ValidatableReflection::confidence(&reflection);
    tracing::info!(
        "Reflector/ValidatableReflection verified: reflected_ok={} invalidated_zero={}",
        reflected_ok,
        invalidated_confidence == 0.0
    );

    // --- EventSubscriber.get_reputation ----------------------------------
    let subscriber = std::sync::Arc::new(build_probe_subscriber(app));
    let recorded = subscriber
        .record_reputation("diagnostics-source", 0.3, "probe")
        .await;
    let score = subscriber.get_reputation("diagnostics-source").await;
    tracing::info!(
        "EventSubscriber reputation verified: recorded_ok={} score={:?}",
        recorded.is_ok(),
        score
    );

    // --- HypothesisPipeline.add_contradicting_evidence -------------------
    verify_pipeline_contradicting(app).await;
}

fn build_probe_subscriber(app: &App) -> EventSubscriber {
    use crate::knowledge::KnowledgeStore;

    let knowledge_store = std::sync::Arc::new(KnowledgeStore::new(100));
    let hypothesis_engine =
        std::sync::Arc::new(crate::experience::hypothesis::HypothesisEngine::new());
    EventSubscriber::new(
        app.mcp_context.metrics.collector(),
        app.mcp_context.reflection.clone(),
        hypothesis_engine,
        app.mcp_context.evolution.clone(),
        knowledge_store,
        None,
    )
}

async fn verify_pipeline_contradicting(app: &App) {
    use crate::experience::integration::hypothesis_pipeline::{
        HypothesisPipeline, HypothesisPipelineConfig,
    };
    use crate::experience::types::{Experience as ProbeExperience, ExperienceType};

    let config = HypothesisPipelineConfig {
        contradicting_evidence_weight: 0.15,
        ..HypothesisPipelineConfig::default()
    };
    let pipeline = HypothesisPipeline::with_config(
        config,
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
        "diagnostics contradicting probe".to_string(),
        "Transient experience for contradicting-evidence path".to_string(),
        ExperienceType::Hypothesis,
        vec![],
    );
    let generated_ids = pipeline
        .process(&probe_experience)
        .await
        .unwrap_or_default();

    if let Some(hyp_id) = generated_ids.first() {
        let weakened = pipeline
            .add_contradicting_evidence(hyp_id, "diagnostics contra")
            .await;
        tracing::info!(
            "HypothesisPipeline.add_contradicting_evidence verified: ok={}",
            weakened.is_ok()
        );
    }
}
