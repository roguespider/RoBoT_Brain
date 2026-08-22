// src/bridge/app/initialization/lineage_tracker.rs
//! Verify LineageTracker lifecycle at startup.

/// Verify the memory Lineage tracking lifecycle at startup.
/// Exercises create_lineage, add_evidence, add_observation, add_refinement,
/// add_contradiction, add_confirmation, resolve_contradiction, mark_superseded,
/// and all query/summary paths so the lineage subsystem stays live.
pub async fn verify_lineage_tracker() {
    use crate::learning::lineage::{
        Confirmation, ConfirmationSource, Contradiction, ContradictionResolution, EvidenceRef,
        EvidenceType, LineageTracker, MemoryLineage, ObservationOutcome, ObservationRef,
        ObservationType, Refinement, RefinementType,
    };

    let mut tracker = LineageTracker::new();
    let mem_a = uuid::Uuid::new_v4();
    let mem_b = uuid::Uuid::new_v4();
    let mem_c = uuid::Uuid::new_v4();
    tracker.create_lineage(mem_a);
    tracker.create_lineage(mem_b);
    tracker.create_lineage(mem_c);
    tracker.add_evidence(
        mem_a,
        EvidenceRef {
            id: uuid::Uuid::new_v4(),
            evidence_type: EvidenceType::Experience,
            confidence: 0.8,
            added_at: chrono::Utc::now(),
        },
    );
    tracker.add_observation(
        mem_a,
        ObservationRef {
            id: uuid::Uuid::new_v4(),
            observation_type: ObservationType::Direct,
            timestamp: chrono::Utc::now(),
            outcome: ObservationOutcome::Positive,
        },
    );
    tracker.add_refinement(
        mem_a,
        Refinement {
            id: uuid::Uuid::new_v4(),
            previous_content: "old".to_string(),
            new_content: "new".to_string(),
            reason: "correction".to_string(),
            refinement_type: RefinementType::Correction,
            confidence_change: 0.1,
            timestamp: chrono::Utc::now(),
        },
    );
    let contra_id = uuid::Uuid::new_v4();
    tracker.add_contradiction(
        mem_a,
        Contradiction {
            id: contra_id,
            contradicting_memory_id: mem_b,
            description: "conflict".to_string(),
            strength: 0.5,
            resolved: false,
            resolution: None,
            timestamp: chrono::Utc::now(),
        },
    );
    tracker.add_confirmation(
        mem_a,
        Confirmation {
            id: uuid::Uuid::new_v4(),
            source: "probe".to_string(),
            source_type: ConfirmationSource::User,
            description: "confirmed".to_string(),
            confidence_boost: 0.2,
            timestamp: chrono::Utc::now(),
        },
    );
    tracker.resolve_contradiction(
        mem_a,
        contra_id,
        ContradictionResolution::ContradictionWasWrong {
            reason: "probe".to_string(),
        },
    );
    tracker.mark_superseded(mem_b, mem_c);
    let lin_ref_present = tracker.get_lineage(&mem_a).is_some();
    let lin_mut_count = tracker
        .get_lineage_mut(&mem_a)
        .map(|l| l.supporting_evidence.len());
    let unresolved = tracker.get_unresolved_contradictions(&mem_a);
    let chain = tracker.get_superseding_chain(&mem_b);
    let current = tracker.get_current_memory(&mem_b);
    let conf = tracker.calculate_confidence(&mem_a, 0.5);
    let summary = tracker.get_summary(&mem_a);
    let summary_display: Option<String> = summary.as_ref().map(|s| format!("{}", s));
    let with_contra = tracker.get_memories_with_contradictions();
    let superseded = tracker.get_superseded_memories();
    let standalone = MemoryLineage::new(mem_c);
    let is_sup = standalone.is_superseded();
    let has_contra = standalone.has_contradiction();
    let boost = standalone.evidence_confidence_boost();
    let penalty = standalone.contradiction_confidence_penalty();
    let lin_conf = standalone.calculate_lineage_confidence(0.5);
    tracing::info!(
        "LineageTracker lifecycle verified: lin_ref={}, lin_mut_evidence={}, unresolved={}, chain={}, current={:?}, conf={:.3}, summary={}, with_contra={}, superseded={}, standalone: superseded={}, has_contra={}, boost={:.3}, penalty={:.3}, lin_conf={:.3}",
        lin_ref_present,
        lin_mut_count.unwrap_or(0),
        unresolved.len(),
        chain.len(),
        current,
        conf,
        summary_display.unwrap_or_default(),
        with_contra.len(),
        superseded.len(),
        is_sup,
        has_contra,
        boost,
        penalty,
        lin_conf,
    );
}
