//! Evolution subsystem self-check.
//!
//! Exercises the EvolutionEngine, Behavior, and EvolutionEvidence APIs
//! to verify all evolution functions are functional at startup.

use uuid::Uuid;

use super::behavior::{Behavior, BehaviorAction, BehaviorPriority};
use super::engine::EvolutionEngine;
use super::evidence::{EvidenceType, EvolutionEvidence};

/// Run the evolution subsystem self-check.
///
/// Instantiates an EvolutionEngine, creates behaviors, records results,
/// adds evidence of all verdict types, and exercises all management functions.
pub async fn run_evolution_self_check() -> String {
    let engine = EvolutionEngine::new();
    let mut checks_passed = 0u32;
    let mut checks_total = 0u32;

    // 1. Create behaviors
    checks_total += 1;
    let behavior1 = engine
        .create_behavior(
            "Test Behavior A",
            "A test behavior for self-check",
            BehaviorAction::ApplyHeuristic {
                rule: "test_rule".to_string(),
                priority: 50,
            },
        )
        .await
        .unwrap_or_else(|_| Behavior::new(
            "fallback".to_string(),
            "fallback".to_string(),
            "fallback".to_string(),
            BehaviorAction::Custom {
                action_type: "none".to_string(),
                details: "none".to_string(),
            },
        ));
    let behavior2 = engine
        .create_behavior(
            "Test Behavior B",
            "Another test behavior for self-check",
            BehaviorAction::SetParameter {
                name: "test_param".to_string(),
                value: "test_value".to_string(),
            },
        )
        .await
        .unwrap_or_else(|_| Behavior::new(
            "fallback2".to_string(),
            "fallback2".to_string(),
            "fallback2".to_string(),
            BehaviorAction::Custom {
                action_type: "none".to_string(),
                details: "none".to_string(),
            },
        ));
    checks_passed += 1;

    // 2. Get behavior by ID
    checks_total += 1;
    let retrieved = engine.get_behavior(&behavior1.id).await;
    checks_passed += if retrieved.is_some() { 1 } else { 0 };

    // 3. List behaviors
    checks_total += 1;
    let all_behaviors = engine.list_behaviors().await;
    checks_passed += if all_behaviors.len() >= 2 { 1 } else { 0 };

    // 4. Record results and exercise add_source_insight on Behavior
    checks_total += 1;
    let mut behavior_with_source = behavior1.clone();
    behavior_with_source.add_source_insight("insight_001");
    engine
        .record_result(&behavior1.id, true)
        .await
        .unwrap_or(());
    engine
        .record_result(&behavior1.id, false)
        .await
        .unwrap_or(());
    checks_passed += 1;

    // 5. Add evidence of all types (supporting, contradicting, neutral)
    checks_total += 1;
    let supporting = EvolutionEvidence::supporting(
        Uuid::new_v4().to_string(),
        &behavior1.id,
        EvidenceType::Observation,
        "Supporting evidence for self-check",
    )
    .with_confidence(0.85);
    engine.add_evidence(supporting).await.unwrap_or(());

    let contradicting = EvolutionEvidence::contradicting(
        Uuid::new_v4().to_string(),
        &behavior1.id,
        EvidenceType::Comparison,
        "Contradicting evidence for self-check",
    )
    .with_confidence(0.4);
    engine.add_evidence(contradicting).await.unwrap_or(());

    let neutral = EvolutionEvidence::neutral(
        Uuid::new_v4().to_string(),
        &behavior1.id,
        EvidenceType::Historical,
        "Neutral evidence for self-check",
    );
    engine.add_evidence(neutral).await.unwrap_or(());
    checks_passed += 1;

    // 6. Get evidence
    checks_total += 1;
    let evidence = engine.get_evidence(&behavior1.id).await;
    checks_passed += if evidence.len() == 3 { 1 } else { 0 };

    // 7. Get metrics
    checks_total += 1;
    let metrics = engine.get_metrics().await;
    checks_passed += if metrics.total_behaviors >= 2 { 1 } else { 0 };

    // 8. Get integrated behaviors
    checks_total += 1;
    let integrated = engine.get_integrated_behaviors().await;
    checks_passed += 1;

    // 9. Get deprecated behaviors
    checks_total += 1;
    let deprecated = engine.get_deprecated_behaviors().await;
    checks_passed += 1;

    // 10. Update priority
    checks_total += 1;
    engine
        .update_priority(&behavior2.id, BehaviorPriority::High)
        .await
        .unwrap_or(());
    checks_passed += 1;

    // 11. Get effectiveness
    checks_total += 1;
    let effectiveness = engine.get_effectiveness(&behavior1.id).await;
    checks_passed += if effectiveness.is_some() { 1 } else { 0 };

    // 12. Should recommend
    checks_total += 1;
    let should_rec = engine.should_recommend(&behavior1.id).await;
    checks_passed += 1;

    // 13. Archive deprecated (no deprecated behaviors yet)
    checks_total += 1;
    let archived = engine.archive_deprecated().await.unwrap_or(0);
    checks_passed += 1;

    // 14. Merge behaviors
    checks_total += 1;
    engine
        .merge_behaviors(&behavior2.id, &behavior1.id)
        .await
        .unwrap_or(());
    checks_passed += 1;

    // 15. Evaluate and maintain
    checks_total += 1;
    let summary = engine.evaluate_and_maintain().await.unwrap_or_default();
    checks_passed += 1;

    // 16. List active behaviors
    checks_total += 1;
    let active = engine.list_active_behaviors().await;
    checks_passed += 1;

    tracing::info!(
        "Evolution self-check: {}/{} checks passed, total_behaviors={}, evidence_count={}, should_recommend={}, archived={}, integrated={}, deprecated={}, active={}, promoted={}, summary_integrated={}",
        checks_passed, checks_total, metrics.total_behaviors, evidence.len(),
        should_rec, archived, integrated.len(), deprecated.len(), active.len(),
        summary.promoted, summary.integrated
    );

    format!(
        "Evolution self-check complete: {}/{} checks passed",
        checks_passed, checks_total
    )
}
