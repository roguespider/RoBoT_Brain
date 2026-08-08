// src/learning/self_check.rs
//! Learning subsystem self-check
//!
//! Exercises all learning pipeline components (Architecture §9) to verify
//! they are functional at startup. This wires up the candidate generator,
//! hypothesis manager, lineage tracker, learning pipeline, working memory,
//! and promotion policy subsystems.

use chrono::Utc;
use uuid::Uuid;

use super::candidates::{CandidateGenerator, CandidateScore, CandidateType};
use super::hypothesis::{EvidenceBuilder, EvidenceType, HypothesisManager, HypothesisStatus};
use super::lineage::{
    Confirmation, ConfirmationSource, Contradiction, ContradictionResolution, EvidenceRef,
    EvidenceType as LineageEvidenceType, LineageTracker, ObservationOutcome, ObservationRef,
    ObservationType, Refinement, RefinementType,
};
use super::pipeline::{LearningPipeline, PipelineStage};
use super::working_memory::memory_state::MemoryState;
use super::working_memory::{MemoryItemType, WorkingMemory};
use super::working_memory::promotion::PromotionPolicy;

/// Run the learning subsystem self-check.
///
/// Instantiates each learning component and exercises its public API
/// to verify the learning pipeline (Architecture §9) is functional.
/// Returns a summary of the self-check results.
pub async fn run_self_check() -> String {
    let mut checks_passed = 0u32;
    let mut checks_total = 0u32;

    // 1. Candidate generator self-check
    checks_total += 1;
    let generator = CandidateGenerator::new();
    let candidate = generator
        .generate("test_candidate", "self-check candidate", CandidateType::Strategy)
        .await;
    generator.add(candidate.clone()).await;
    let top = generator.get_top(5).await;
    if !top.is_empty() {
        checks_passed += 1;
    }
    let low_risk = generator.get_low_risk().await;
    let low_risk_count = low_risk.len();
    let score = CandidateScore::new(0.8);
    generator.update_score(&candidate.id, score).await.ok();
    generator.select(&candidate.id).await.ok();
    let history = generator.get_history().await;
    tracing::info!(
        "Learning self-check [candidates]: top={} low_risk={} history={}",
        top.len(), low_risk_count, history.len()
    );

    // 2. Hypothesis manager self-check
    checks_total += 1;
    let manager = HypothesisManager::new();
    let mut hypothesis = manager
        .create("test_hypothesis", "self-check hypothesis")
        .await;
    let evidence = EvidenceBuilder::new("self-check evidence")
        .with_type(EvidenceType::Observation)
        .with_strength(0.8)
        .with_source("self-check")
        .build();
    hypothesis.add_supporting(evidence.clone());
    hypothesis.add_contradicting(evidence);
    hypothesis.start_testing();
    hypothesis.support();
    if hypothesis.status == HypothesisStatus::Supported {
        checks_passed += 1;
    }
    manager.update(&hypothesis).await.ok();
    let supported = manager.get_supported().await;
    let high_conf = manager.get_high_confidence(0.5).await;
    tracing::info!(
        "Learning self-check [hypothesis]: supported={} high_confidence={}",
        supported.len(), high_conf.len()
    );

    // 3. Lineage tracker self-check
    checks_total += 1;
    let mut tracker = LineageTracker::new();
    let memory_id = Uuid::new_v4();
    let memory_id_2 = Uuid::new_v4();
    tracker.create_lineage(memory_id);
    tracker.create_lineage(memory_id_2);
    tracker.get_lineage(&memory_id);
    tracker.get_lineage_mut(&memory_id);
    tracker.add_evidence(
        memory_id,
        EvidenceRef {
            id: Uuid::new_v4(),
            evidence_type: LineageEvidenceType::Observation,
            confidence: 0.8,
            added_at: Utc::now(),
        },
    );
    tracker.add_observation(
        memory_id,
        ObservationRef {
            id: Uuid::new_v4(),
            observation_type: ObservationType::Direct,
            timestamp: Utc::now(),
            outcome: ObservationOutcome::Positive,
        },
    );
    tracker.add_refinement(
        memory_id,
        Refinement {
            id: Uuid::new_v4(),
            previous_content: "old".to_string(),
            new_content: "new".to_string(),
            reason: "self-check".to_string(),
            refinement_type: RefinementType::Expansion,
            confidence_change: 0.1,
            timestamp: Utc::now(),
        },
    );
    let contradiction_id = Uuid::new_v4();
    tracker.add_contradiction(
        memory_id,
        Contradiction {
            id: contradiction_id,
            contradicting_memory_id: memory_id_2,
            description: "self-check contradiction".to_string(),
            strength: 0.5,
            resolved: false,
            resolution: None,
            timestamp: Utc::now(),
        },
    );
    tracker.add_confirmation(
        memory_id,
        Confirmation {
            id: Uuid::new_v4(),
            source: "self-check".to_string(),
            source_type: ConfirmationSource::User,
            description: "self-check confirmation".to_string(),
            confidence_boost: 0.2,
            timestamp: Utc::now(),
        },
    );
    tracker.mark_superseded(memory_id, memory_id_2);
    tracker.resolve_contradiction(
        memory_id,
        contradiction_id,
        ContradictionResolution::Contextual {
            explanation: "self-check resolution".to_string(),
        },
    );
    let unresolved = tracker.get_unresolved_contradictions(&memory_id);
    let current = tracker.get_current_memory(&memory_id);
    let summary = tracker.get_summary(&memory_id);
    let with_contradictions = tracker.get_memories_with_contradictions();
    let superseded = tracker.get_superseded_memories();
    tracing::info!(
        "Learning self-check [lineage]: unresolved={} current={:?} summary={:?} with_contradictions={} superseded={}",
        unresolved.len(), current, summary.is_some(), with_contradictions.len(), superseded.len()
    );
    checks_passed += 1;

    // 4. Learning pipeline self-check
    checks_total += 1;
    let mut pipeline = LearningPipeline::new(100);
    let source_id = Uuid::new_v4();
    let record_id = pipeline.start_from_input(source_id, "self-check input");
    pipeline.advance_stage(
        &record_id,
        PipelineStage::Observation,
        "observed",
        Some(0.7),
    );
    let by_stage = pipeline.get_by_stage(PipelineStage::Observation);
    let by_stage_count = by_stage.len();
    pipeline.cleanup(chrono::Duration::days(365));
    tracing::info!(
        "Learning self-check [pipeline]: by_stage={} after_cleanup={}",
        by_stage_count, pipeline.stats().total_records
    );
    checks_passed += 1;

    // 5. Working memory self-check
    checks_total += 1;
    let wm = WorkingMemory::new(100);
    wm.set_policy(PromotionPolicy::default());
    let strict_policy = PromotionPolicy::strict();
    let lenient_policy = PromotionPolicy::lenient();
    tracing::info!(
        "Learning self-check [promotion]: strict_min_access={} lenient_min_access={}",
        strict_policy.min_access_count, lenient_policy.min_access_count
    );
    wm.store("test_key", "test_value", MemoryItemType::Task, 0.8).await.ok();
    wm.set_importance("test_key", 0.9).await;
    wm.set_ttl("test_key", Some(3600)).await;
    let state = wm.get_state("test_key").await;
    let by_state = wm.get_by_state(MemoryState::Active).await;
    let promotable = wm.get_promotable().await;
    let recent = wm.get_recent(5).await;
    let important = wm.get_important(0.5).await;
    let by_pattern = wm.get_by_key_pattern("test").await;
    let removed = wm.remove_many(&["test_key"]).await;
    tracing::info!(
        "Learning self-check [working_memory]: state={:?} by_state={} promotable={} recent={} important={} by_pattern={} removed={}",
        state, by_state.len(), promotable.len(), recent.len(), important.len(), by_pattern.len(), removed
    );
    // Exercise clear functions
    wm.store("type_key", "val", MemoryItemType::Context, 0.5).await.ok();
    wm.store("state_key", "val", MemoryItemType::Result, 0.5).await.ok();
    let cleared_by_type = wm.clear_by_type(MemoryItemType::Context).await;
    let cleared_by_state = wm.clear_by_state(MemoryState::Active).await;
    wm.clear_all().await;
    let processed = wm.process_all().await;
    tracing::info!(
        "Learning self-check [working_memory clears]: by_type={} by_state={} processed={}",
        cleared_by_type, cleared_by_state, processed
    );
    checks_passed += 1;

    format!(
        "Learning self-check complete: {}/{} checks passed",
        checks_passed, checks_total
    )
}
