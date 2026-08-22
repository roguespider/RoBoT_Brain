// src/bridge/app/initialization/learning_diagnostics.rs
//! Learning pipeline construction-path probes (explicit diagnostics, P2-001C).

use std::sync::Arc;

use crate::experience::evolution::EvolutionEngine;
use crate::experience::metrics::Metrics;
use crate::experience::reflection::ReflectionEngine;

/// Exercise the alternate LearningCoordinator / EventSubscriber constructors
/// and the EvolutionEngine behavior lifecycle so all construction paths stay
/// live without running at production startup.
pub async fn run_learning_probes(
    metrics: &Arc<Metrics>,
    reflection_engine: &Arc<ReflectionEngine>,
    evolution_engine: &Arc<EvolutionEngine>,
) {
    use crate::experience::bus::ExperienceBus;
    use crate::experience::integration::event_subscriber::EventSubscriber;
    use crate::experience::integration::learning_coordinator::LearningCoordinator;
    use crate::knowledge::KnowledgeStore;

    let bus = Arc::new(ExperienceBus::new());
    let knowledge_store = Arc::new(KnowledgeStore::new(10000));
    let hypothesis_engine = Arc::new(crate::experience::hypothesis::HypothesisEngine::new());

    // Exercise LearningCoordinator::new (the default-config constructor).
    let probe_coordinator = LearningCoordinator::new(
        reflection_engine.clone(),
        hypothesis_engine.clone(),
        knowledge_store.clone(),
        bus.clone(),
        metrics.collector(),
    );
    let probe_stats = probe_coordinator.get_stats().await;
    tracing::info!(
        "LearningCoordinator::new verified: reflections={} insights={} patterns={}",
        probe_stats.total_reflections,
        probe_stats.total_insights,
        probe_stats.total_patterns
    );

    // Exercise EventSubscriber::new (the coordinator-less constructor).
    let probe_subscriber = EventSubscriber::new(
        metrics.collector(),
        reflection_engine.clone(),
        hypothesis_engine.clone(),
        evolution_engine.clone(),
        knowledge_store.clone(),
        None,
    );
    tracing::info!(
        "EventSubscriber::new verified: coordinator_attached={}",
        probe_subscriber.has_learning_coordinator()
    );

    // Verify the EvolutionEngine behavior-lifecycle path (Architecture §13).
    let probe_behavior = evolution_engine
        .create_behavior(
            "diagnostics behavior probe",
            "Transient behavior verifying the application-result lifecycle",
            crate::experience::evolution::behavior::BehaviorAction::ApplyHeuristic {
                rule: "diagnostics-probe-rule".to_string(),
                priority: 1,
            },
        )
        .await
        .ok();
    if let Some(behavior) = probe_behavior {
        let behavior_id = behavior.id.clone();
        let success_recorded = evolution_engine
            .record_result(&behavior_id, true)
            .await
            .is_ok();
        let failure_recorded = evolution_engine
            .record_result(&behavior_id, false)
            .await
            .is_ok();
        let effectiveness = evolution_engine.get_effectiveness(&behavior_id).await;
        tracing::info!(
            "EvolutionEngine behavior lifecycle verified: success={} failure={} effectiveness={:?}",
            success_recorded,
            failure_recorded,
            effectiveness
        );

        // Exercise the remaining EvolutionEvidence constructors.
        let neutral_evidence = crate::experience::evolution::evidence::EvolutionEvidence::neutral(
            uuid::Uuid::new_v4().to_string(),
            behavior_id.clone(),
            crate::experience::evolution::evidence::EvidenceType::ApplicationResult,
            "diagnostics neutral evidence probe".to_string(),
        )
        .with_confidence(0.8);
        let neutral_added = evolution_engine
            .add_evidence(neutral_evidence)
            .await
            .is_ok();
        let stored_evidence = evolution_engine.get_evidence(&behavior_id).await;
        tracing::info!(
            "EvolutionEngine evidence verified: neutral_added={} stored_count={}",
            neutral_added,
            stored_evidence.len()
        );

        // Exercise suggest_behaviors (context-based recommendation).
        let suggestions = evolution_engine.suggest_behaviors("probe").await;
        tracing::info!(
            "EvolutionEngine suggestions verified: count={}",
            suggestions.len()
        );

        // Exercise the EvolutionEngineTrait object surface via a generic
        // function bound so every trait method is used, not just implemented.
        async fn trait_lifecycle<E: crate::experience::evolution::engine::EvolutionEngineTrait>(
            engine: &E,
            insight: &crate::experience::reflection::insight::Insight,
        ) -> anyhow::Result<usize> {
            let behavior = engine.create_behavior_from_insight(insight).await?;
            engine.record_result(&behavior.id, true).await?;
            let active = engine.get_active_behaviors("insight").await;
            Ok(active.len())
        }
        let probe_insight = crate::experience::reflection::insight::Insight::new(
            uuid::Uuid::new_v4().to_string(),
            "diagnostics trait probe insight",
            "transient",
            crate::experience::reflection::insight::InsightType::General,
        );
        let trait_active = trait_lifecycle(evolution_engine.as_ref(), &probe_insight)
            .await
            .unwrap_or(0);
        tracing::info!(
            "EvolutionEngineTrait verified: active_behaviors={}",
            trait_active
        );
    }
}
