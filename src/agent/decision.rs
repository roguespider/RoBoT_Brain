//! Research cascade checker per research_engine.md §2 (9-Tier Confidence Cascade).
//! Before research (tier 9) is dispatched, tiers 1-8 are checked in order:
//! Memory -> Knowledge -> Experience -> Skills -> Reflections -> Workflows ->
//! WorldModel -> Hypotheses. If any tier can answer the query, research is
//! skipped. Research is the last resort.

use tokio::sync::Mutex;

use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::types::Experience;
use crate::knowledge::store::KnowledgeStore;
use crate::knowledge::world_model::store::WorldModel;
use crate::memory::permanent::PermanentMemory;
use crate::planning::engine::types::Plan;
use crate::scheduler::cooboploop::capability::CapabilityRegistry;

/// Dependencies for the 9-tier confidence cascade check.
///
/// Groups all tier 1-8 dependencies into a single parameter to avoid
/// the clippy::too_many_arguments lint (limit is 7).
pub struct CascadeDeps<'a> {
    pub knowledge_store: &'a KnowledgeStore,
    pub latest_experience: Option<&'a Experience>,
    pub reflection_engine: &'a Mutex<crate::experience::reflection::engine::ReflectionEngine>,
    pub current_plan: &'a Option<Plan>,
    pub permanent_memory: &'a PermanentMemory,
    pub world_model: &'a WorldModel,
    pub hypothesis_engine: &'a HypothesisEngine,
}

/// 9-tier confidence cascade checker.
///
/// Checks tiers 1-8 before allowing research (tier 9) to proceed. Returns
/// `true` if research should proceed (all tiers 1-8 failed), `false` if a
/// tier 1-8 could answer the query.
pub struct ResearchCascade;

impl ResearchCascade {
    /// Run the full 9-tier cascade for a given topic.
    ///
    /// Returns `true` if research should proceed (all tiers 1-8 failed),
    /// `false` if a tier 1-8 could answer the query.
    pub async fn should_research(topic: &str, deps: &CascadeDeps<'_>) -> bool {
        // Tier 1: Memory — check permanent memory for high-confidence matches.
        if Self::check_memory(deps.permanent_memory, topic).await {
            tracing::debug!("Cascade tier 1 (Memory) passed for topic: {topic}");
            return false;
        }

        // Tier 2: Knowledge Base — query knowledge store for confident matches.
        if Self::check_knowledge(deps.knowledge_store, topic) {
            tracing::debug!("Cascade tier 2 (Knowledge) passed for topic: {topic}");
            return false;
        }

        // Tier 3: Experience — check latest experience for pattern match.
        if Self::check_experience(deps.latest_experience, topic) {
            tracing::debug!("Cascade tier 3 (Experience) passed for topic: {topic}");
            return false;
        }

        // Tier 4: Skills — check if a relevant capability exists with sufficient level.
        // CapabilityRegistry is not Sync (rusqlite::Connection interior RefCell),
        // so tier 4 is always skipped (check_skills returns false when registry is None).
        if Self::check_skills(None, topic) {
            tracing::debug!("Cascade tier 4 (Skills) passed for topic: {topic}");
            return false;
        }

        // Tier 5: Reflections — check for validated reflections.
        if Self::check_reflections(deps.reflection_engine, topic).await {
            tracing::debug!("Cascade tier 5 (Reflections) passed for topic: {topic}");
            return false;
        }

        // Tier 6: Workflows/Plans — check for existing plan.
        if Self::check_workflows(deps.current_plan, topic) {
            tracing::debug!("Cascade tier 6 (Workflows) passed for topic: {topic}");
            return false;
        }

        // Tier 7: World Model — check for relevant entity.
        if Self::check_world_model(deps.world_model, topic).await {
            tracing::debug!("Cascade tier 7 (World Model) passed for topic: {topic}");
            return false;
        }

        // Tier 8: Hypotheses — check for relevant hypothesis.
        if Self::check_hypotheses(deps.hypothesis_engine, topic) {
            tracing::debug!("Cascade tier 8 (Hypotheses) passed for topic: {topic}");
            return false;
        }

        // All tiers 1-8 failed — proceed to research (tier 9, last resort).
        tracing::debug!("Cascade: all tiers 1-8 failed for topic: {topic}, proceeding to research");
        true
    }

    /// Check tier 1: Memory.
    ///
    /// Searches permanent memory for items matching the topic with
    /// confidence >= 0.7. Returns `true` if any such item exists.
    async fn check_memory(store: &PermanentMemory, topic: &str) -> bool {
        let results = store.search(topic).await;
        results.iter().any(|item| item.confidence >= 0.7)
    }

    /// Check tier 2: Knowledge Base.
    ///
    /// Searches the knowledge store for items matching the topic with
    /// confidence >= 0.7. Returns `true` if any such item exists.
    fn check_knowledge(store: &KnowledgeStore, topic: &str) -> bool {
        let results = store.search(topic);
        results.iter().any(|item| item.overall_confidence() >= 0.7)
    }

    /// Check tier 3: Experience.
    ///
    /// Checks the latest experience for a pattern match against the topic.
    /// Returns `true` if the experience has a confidence score >= 0.7 and
    /// its outcome or description references the topic.
    fn check_experience(exp: Option<&Experience>, topic: &str) -> bool {
        let Some(exp) = exp else { return false };
        let topic_lower = topic.to_lowercase();
        let outcome_lower = exp.outcome.message.as_deref().unwrap_or("").to_lowercase();
        let description_lower = exp.description.to_lowercase();
        if (outcome_lower.contains(&topic_lower) || description_lower.contains(&topic_lower))
            && exp.score.as_ref().is_some_and(|score| {
                let overall = (score.confidence * 0.5 + score.reliability * 0.5).clamp(0.0, 1.0);
                overall >= 0.7
            })
        {
            return true;
        }
        false
    }

    /// Check tier 4: Skills.
    ///
    /// Checks the capability registry for a capability matching the topic
    /// with level >= 0.7. Returns `true` if such a capability exists.
    /// If no registry is provided, returns `false` (skill check does not pass).
    fn check_skills(registry: Option<&CapabilityRegistry>, topic: &str) -> bool {
        let Some(registry) = registry else {
            return false;
        };
        let topic_lower = topic.to_lowercase();
        for cap in registry.list() {
            if cap.level >= 0.7
                && (cap.name.to_lowercase().contains(&topic_lower)
                    || topic_lower.contains(&cap.name.to_lowercase()))
            {
                return true;
            }
        }
        false
    }

    /// Check tier 5: Reflections.
    ///
    /// Checks the reflection engine for validated reflections matching the topic.
    /// Returns `true` if a validated reflection with sufficient confidence exists.
    async fn check_reflections(
        engine: &Mutex<crate::experience::reflection::engine::ReflectionEngine>,
        topic: &str,
    ) -> bool {
        let guard = engine.lock().await;
        let topic_lower = topic.to_lowercase();
        let reflections = guard.list_validated().await;
        for r in reflections {
            let content = format!("{} {} {}", r.title, r.summary, r.description);
            let content_lower = content.to_lowercase();
            if content_lower.contains(&topic_lower) {
                return true;
            }
        }
        false
    }

    /// Check tier 6: Workflows/Plans.
    ///
    /// Checks if there is an existing plan that addresses the topic.
    /// Returns `true` if a plan exists.
    fn check_workflows(current_plan: &Option<Plan>, topic: &str) -> bool {
        let Some(plan) = current_plan else {
            return false;
        };
        let topic_lower = topic.to_lowercase();
        let plan_description = format!("{} {:?}", plan.goal, plan.steps);
        plan_description.to_lowercase().contains(&topic_lower)
    }

    /// Check tier 7: World Model.
    ///
    /// Checks the world model for a relevant entity matching the topic.
    /// Returns `true` if a relevant entity exists.
    async fn check_world_model(model: &WorldModel, topic: &str) -> bool {
        model.find_by_name(topic).await.is_some()
    }

    /// Check tier 8: Hypotheses.
    ///
    /// Checks the hypothesis engine for a relevant hypothesis matching the topic.
    /// Returns `true` if a relevant hypothesis exists.
    fn check_hypotheses(engine: &HypothesisEngine, topic: &str) -> bool {
        let graph = engine.get_graph();
        let Ok(guard) = graph.lock() else {
            return false;
        };
        let topic_lower = topic.to_lowercase();
        // Check if any hypothesis node ID references the topic.
        guard
            .nodes
            .iter()
            .any(|node| node.hypothesis_id.0.to_lowercase().contains(&topic_lower))
    }
}
