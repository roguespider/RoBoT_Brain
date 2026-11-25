// /src/scheduler/cooboploop/learning_feedback/mod.rs
// Learning feedback module — wires LearningUpdate into the CoObOpLoop subsystems.
// Per CoObOpLoop.md §15: Experience → Learning → Capability/Knowledge Update → Better Evaluation.

use crate::knowledge::store::KnowledgeStore;
use crate::scheduler::cooboploop::capability::{
    CapabilityAssessment, CapabilityId, CapabilityRegistry,
};
use crate::scheduler::cooboploop::loop_runner_engine::StrategyTracker;
use chrono::Utc;

/// Apply learning feedback to the knowledge store, capability registry, and strategy tracker.
/// Per CoObOpLoop.md §15: Experience → Learning → Capability/Knowledge Update → Better Evaluation.
pub fn apply_learning_feedback(
    knowledge_store: &KnowledgeStore,
    capability_registry: &mut CapabilityRegistry,
    strategy_tracker: &mut StrategyTracker,
    learning_update: &crate::scheduler::cooboploop::learning_pipeline::LearningUpdate,
    exp: &crate::experience::types::experience::Experience,
) {
    // Wire knowledge_additions into KnowledgeStore (§15: Knowledge Update).
    for lesson in &learning_update.knowledge_additions {
        if !lesson.is_empty() {
            let knowledge_confidence =
                if exp.outcome.kind == crate::experience::types::outcome::OutcomeKind::Success {
                    exp.confidence.max(0.6)
                } else {
                    exp.confidence * 0.5
                };
            let item = crate::knowledge::types::KnowledgeItem::from_reflection(
                lesson,
                knowledge_confidence,
                exp.id,
            );
            knowledge_store.add(item);
            tracing::debug!("Learning feedback: stored knowledge from experience");
        }
    }

    // Wire capability_updates into CapabilityRegistry (§15: Capability Update).
    for cap_update in &learning_update.capability_updates {
        let cap_id = CapabilityId::from_string(&cap_update.capability_id);
        let current = capability_registry
            .get(&cap_id)
            .unwrap_or_else(|| CapabilityAssessment {
                id: cap_id.clone(),
                name: cap_update.capability_id.clone(),
                level: 0.5,
                last_assessed: None,
                success_rate: 0.5,
            });
        let new_level = (current.level + cap_update.level_delta).clamp(0.0, 1.0);
        capability_registry.update(CapabilityAssessment {
            id: cap_id.clone(),
            name: cap_update.capability_id.clone(),
            level: new_level,
            last_assessed: Some(Utc::now()),
            success_rate: current.success_rate,
        });
        tracing::info!(
            "Learning feedback: capability {} level {} -> {} ({})",
            cap_update.capability_id,
            current.level,
            new_level,
            cap_update.reason
        );
    }

    // Wire strategy_refinements into strategy tracker (§15: Strategy Refinement).
    strategy_tracker.absorb_refinements(&learning_update.strategy_refinements);
    tracing::debug!(
        "Learning feedback: strategy_tracker has {} reinforced, {} avoided",
        strategy_tracker.reinforced.len(),
        strategy_tracker.avoided.len()
    );
}
