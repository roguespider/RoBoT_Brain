// /src/CoObOpLoop/research.rs
// Research system for the CoObOpLoop system.
//
// Per CoObOpLoop.md §11: research SHALL be a first-class objective that
// produces persistent knowledge/experience via the ResearchPipeline.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::decision::{CascadeDeps, ResearchCascade};
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::reflection::engine::ReflectionEngine;
use crate::experience::types::Experience;
use crate::knowledge::store::KnowledgeStore;
use crate::knowledge::world_model::store::WorldModel;
use crate::memory::permanent::PermanentMemory;
use crate::planning::engine::types::Plan;

/// Trigger for research (§11 / T8.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum ResearchTrigger {
    UnavailableInfo,
    HighUncertainty,
    CapabilityGap,
    TechnologyInvestigation,
    HardwareUpgrade,
    MultipleSolutions,
    PreviousFailure,
    ExternalOpportunityKnowledgeGap,
}

/// Persistence target for research results (§11 / T8.3).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum PersistenceTarget {
    KnowledgeBase,
    ExperienceLog,
    Both,
}

/// Research objective (§11 / T8.2).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchObjective {
    pub topic: String,
    pub trigger: ResearchTrigger,
    pub persistence_target: PersistenceTarget,
    pub expected_knowledge: String,
}

/// Research manager.
pub struct ResearchManager {
    objectives: Vec<ResearchObjective>,
    next_id: u64,
    /// Memory repository for persisting research findings (§11 / T8.3).
    memory_repository: Option<Arc<dyn crate::memory::repository::MemoryRepository>>,
    /// Optional research pipeline for dispatching research (§11).
    research_pipeline: Option<Arc<crate::research::pipeline::ResearchPipeline>>,
    /// Tokio runtime handle for spawning async research tasks.
    tokio_handle: Option<tokio::runtime::Handle>,
    /// 9-tier confidence cascade dependencies (research_engine.md §2).
    /// Note: CapabilityRegistry is excluded because it contains rusqlite::Connection
    /// which is not Sync (RefCell interior), so it cannot be stored behind Arc.
    knowledge_store: Option<Arc<KnowledgeStore>>,
    permanent_memory: Option<Arc<PermanentMemory>>,
    world_model: Option<Arc<WorldModel>>,
    hypothesis_engine: Option<Arc<HypothesisEngine>>,
    reflection_engine: Option<Arc<Mutex<ReflectionEngine>>>,
    current_plan: Option<Plan>,
    latest_experience: Option<Experience>,
}

impl ResearchManager {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            next_id: 1,
            memory_repository: None,
            research_pipeline: None,
            tokio_handle: None,
            knowledge_store: None,
            permanent_memory: None,
            world_model: None,
            hypothesis_engine: None,
            reflection_engine: None,
            current_plan: None,
            latest_experience: None,
        }
    }

    /// Set the research pipeline for dispatching research (§11).
    pub fn set_research_pipeline(
        &mut self,
        pipeline: Arc<crate::research::pipeline::ResearchPipeline>,
    ) {
        self.research_pipeline = Some(pipeline);
    }

    /// Set the Tokio runtime handle for spawning async tasks.
    pub fn set_tokio_handle(&mut self, handle: tokio::runtime::Handle) {
        self.tokio_handle = Some(handle);
    }

    /// Clone the research pipeline for dispatch.
    pub fn get_research_pipeline(
        &self,
    ) -> Option<Arc<crate::research::pipeline::ResearchPipeline>> {
        self.research_pipeline.clone()
    }

    /// Create a research objective, returns the assigned ID (§T8.4).
    pub fn create_objective(
        &mut self,
        topic: String,
        trigger: ResearchTrigger,
        persistence_target: PersistenceTarget,
        expected_knowledge: String,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.objectives.push(ResearchObjective {
            topic,
            trigger,
            persistence_target,
            expected_knowledge,
        });
        id
    }

    pub fn list(&self) -> &Vec<ResearchObjective> {
        &self.objectives
    }

    /// Set the memory repository for research result persistence (§11 / T8.3).
    pub fn set_memory_repository(
        &mut self,
        repo: Arc<dyn crate::memory::repository::MemoryRepository>,
    ) {
        self.memory_repository = Some(repo);
    }

    /// Get a reference to the configured memory repository (§11 / T8.3).
    pub fn memory_repository_ref(
        &self,
    ) -> Option<&Arc<dyn crate::memory::repository::MemoryRepository>> {
        self.memory_repository.as_ref()
    }

    /// Clone the configured memory repository for dispatch (§11 / T8.3).
    pub fn memory_repository_clone(
        &self,
    ) -> Option<Arc<dyn crate::memory::repository::MemoryRepository>> {
        self.memory_repository.clone()
    }

    // --- 9-tier confidence cascade setters (research_engine.md §2) ---

    pub fn set_knowledge_store(&mut self, store: Arc<KnowledgeStore>) {
        self.knowledge_store = Some(store);
    }

    pub fn set_permanent_memory(&mut self, memory: Arc<PermanentMemory>) {
        self.permanent_memory = Some(memory);
    }

    pub fn set_world_model(&mut self, model: Arc<WorldModel>) {
        self.world_model = Some(model);
    }

    pub fn set_hypothesis_engine(&mut self, engine: Arc<HypothesisEngine>) {
        self.hypothesis_engine = Some(engine);
    }

    pub fn set_reflection_engine(&mut self, engine: Arc<Mutex<ReflectionEngine>>) {
        self.reflection_engine = Some(engine);
    }

    pub fn set_current_plan(&mut self, plan: Option<Plan>) {
        self.current_plan = plan;
    }

    pub fn set_latest_experience(&mut self, experience: Option<Experience>) {
        self.latest_experience = experience;
    }

    /// Dispatch the next pending research objective (§11 / Architecture).
    /// Checks the 9-tier confidence cascade before dispatching to the
    /// ResearchPipeline. If any tier 1-8 can answer the query, research is
    /// skipped. Research is the last resort (tier 9).
    pub fn dispatch_next(
        &mut self,
        memory_repository: Option<&dyn crate::memory::repository::MemoryRepository>,
    ) -> Result<String, String> {
        // Get the first pending objective
        let objective = self
            .objectives
            .first()
            .cloned()
            .ok_or_else(|| "No pending research objectives".to_string())?;

        // Check if the topic is already known in the repository.
        let already_known = memory_repository
            .map(|repo| self.check_already_known(&objective.topic, repo))
            .unwrap_or(false);
        if already_known {
            tracing::debug!(
                "Research topic already known, removing objective: {}",
                objective.topic
            );
            self.objectives.remove(0);
            return Ok("already_known".to_string());
        }

        // Record a research experience to investigate.
        let experience_id = self.record_research_experience(&objective)?;
        self.objectives.remove(0);

        // Dispatch to the ResearchPipeline if available (§11).
        // The 9-tier cascade check runs inside the async task before the
        // pipeline is invoked. If any tier 1-8 can answer the query, research
        // is skipped and the pipeline is not called.
        if let (Some(pipeline), Some(handle)) = (&self.research_pipeline, &self.tokio_handle) {
            let pipeline_clone = pipeline.clone();
            let topic_clone = objective.topic.clone();
            let knowledge_store = self.knowledge_store.clone();
            let permanent_memory = self.permanent_memory.clone();
            let world_model = self.world_model.clone();
            let hypothesis_engine = self.hypothesis_engine.clone();
            let reflection_engine = self.reflection_engine.clone();
            let current_plan = self.current_plan.clone();
            let latest_experience = self.latest_experience.clone();

            handle.spawn(async move {
                // 9-tier confidence cascade gate (research_engine.md §2).
                // Research only fires when all tiers 1-8 fail.
                // CapabilityRegistry (tier 4) is not available because it is
                // not Sync (rusqlite::Connection interior RefCell), so tier 4
                // is skipped (check_skills returns false when registry is None).
                let should_research = match (
                    knowledge_store.as_deref(),
                    latest_experience.as_ref(),
                    reflection_engine.as_deref(),
                    permanent_memory.as_deref(),
                    world_model.as_deref(),
                    hypothesis_engine.as_deref(),
                ) {
                    (
                        Some(ks),
                        Some(le),
                        Some(re),
                        Some(pm),
                        Some(wm),
                        Some(he),
                    ) => {
                        let deps = CascadeDeps {
                            knowledge_store: ks,
                            latest_experience: Some(le),
                            reflection_engine: re,
                            current_plan: &current_plan,
                            permanent_memory: pm,
                            world_model: wm,
                            hypothesis_engine: he,
                        };
                        ResearchCascade::should_research(&topic_clone, &deps)
                        .await
                    }
                    _ => {
                        tracing::warn!(
                            "Cascade dependencies not fully configured, allowing research as fallback"
                        );
                        true
                    }
                };

                if !should_research {
                    tracing::debug!(
                        "Cascade skipped research for topic: {}",
                        topic_clone
                    );
                    return;
                }

                tracing::info!(topic = %topic_clone, "Dispatching research via ResearchPipeline");
                match pipeline_clone
                    .run_pipeline(&topic_clone, crate::research::pipeline::Mode::Auto)
                    .await
                {
                    Ok(result) => {
                        tracing::info!(
                            research_topic = %topic_clone,
                            sources = result.sources.len(),
                            findings = result.findings.len(),
                            "Research completed successfully"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            research_topic = %topic_clone,
                            error = %e,
                            "Research pipeline returned error"
                        );
                    }
                }
            });
        }

        Ok(experience_id)
    }

    /// Check if a research topic is already known in the memory repository.
    fn check_already_known(
        &self,
        topic: &str,
        repository: &dyn crate::memory::repository::MemoryRepository,
    ) -> bool {
        // Log the repository check for observability
        tracing::debug!(
            "Checking memory repository for existing knowledge about: {}",
            topic
        );
        // In production, this would query stored memories for relevant content
        // using semantic search. For now we return false to ensure research
        // gets dispatched when needed.
        let repository_type = std::any::type_name_of_val(repository);
        tracing::debug!(
            "Memory repository type: {} — checking topic: {}",
            repository_type,
            topic
        );
        false
    }

    /// Record a research experience for the given objective.
    /// Persists to memory repository when configured, returning a durable ID.
    fn record_research_experience(&self, objective: &ResearchObjective) -> Result<String, String> {
        let experience_uuid = uuid::Uuid::new_v4();
        let experience_id = format!("research_{:?}_{experience_uuid}", objective.trigger);

        // Persist to memory repository when configured (§11 / T8.3).
        if let Some(ref repo) = self.memory_repository {
            use crate::memory::types::{MemoryItem, MemoryLayer, MemoryType};
            let content = format!(
                "Research objective — topic: {}\nTrigger: {:?}\nExpected knowledge: {}",
                objective.topic, objective.trigger, objective.expected_knowledge
            );
            let mut item = MemoryItem::new(
                MemoryLayer::Working,
                MemoryType::Experience,
                content,
                format!("cooboploop_research_{:?}", objective.trigger),
            );
            item.id = experience_uuid;
            item.importance = 0.7;
            item.add_tag("research");
            item.add_tag(&objective.topic);
            if let Err(e) = repo.store(&item) {
                tracing::warn!("Failed to persist research experience: {e}");
            }
        }

        tracing::info!(
            "Recorded research experience: {} — topic: {} (target: {:?})",
            experience_id,
            objective.topic,
            objective.persistence_target
        );
        Ok(experience_id)
    }
}

impl Default for ResearchManager {
    fn default() -> Self {
        Self::new()
    }
}
