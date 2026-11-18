//! LoopRunner — Main loop controller (Architecture Chapter 23).
//!
//! Autonomous operation is fully wired: objective generation, opportunity discovery,
//! prioritization, capability development, testing, optimization, hardware adaptation,
//! software maintenance, improvement proposal, and verification are all operational.
//!
//! The cognitive cycle and strategic learning logic have been extracted into
//! sibling modules: `cognitive_cycle` and `strategic`.

use super::super::capability::CapabilityRegistry;
use super::super::event_tracer::EventTracer;
use super::super::human::HumanActionHandler;
use super::super::idle::IdleState;
use super::super::post_task::PostTaskEvaluation;
use super::super::queue;
use super::super::queue::AgentGoal;
use super::super::research::ResearchManager;
use super::super::self_improvement::SelfImprovementPipeline;
use super::super::sources::ObjectiveSourceRegistry;
use super::super::strategic::StrategicObjectiveRegistry;

use crate::conversation::AutonomousConversation;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::types::Experience;
use crate::planning::engine::types::Plan;
use std::sync::{Arc, Mutex};

use super::config::LoopRunnerConfig;
use super::enums::{CognitiveCycleStage, CyclePhase, LoopStage};
use crate::api::bridge::app::agent::AutonomousAgentManager;
use crate::knowledge::store::KnowledgeStore;
use crate::learning::strategic::StrategicLearningEngine;
use crate::memory::MemoryHierarchy;
use crate::scheduler::cooboploop::Inspector;
use crate::scheduler::cooboploop::hardware::HardwareDiscovery;
use crate::workers::MaintenanceScheduler;

/// Strategy tracker: records reinforced and avoided strategies from learning feedback (Section 15).
pub struct StrategyTracker {
    pub reinforced: Vec<String>,
    pub avoided: Vec<String>,
}

impl StrategyTracker {
    pub fn new() -> Self {
        Self {
            reinforced: Vec::new(),
            avoided: Vec::new(),
        }
    }

    /// Add strategy refinements from learning update (Section 15).
    pub fn absorb_refinements(&mut self, refinements: &[String]) {
        for r in refinements {
            if r.starts_with("reinforce: ") {
                let strategy = r.strip_prefix("reinforce: ").unwrap_or(r);
                if !self.reinforced.contains(&strategy.to_string()) {
                    self.reinforced.push(strategy.to_string());
                }
            } else if r.starts_with("avoid: ") {
                let strategy = r.strip_prefix("avoid: ").unwrap_or(r);
                if !self.avoided.contains(&strategy.to_string()) {
                    self.avoided.push(strategy.to_string());
                }
            }
        }
    }
}

impl Default for StrategyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Main loop controller (Section 23 / T-COO-50).
pub struct LoopRunner {
    pub current_stage: LoopStage,
    pub(crate) cycle_count: u32,
    pub(crate) should_continue: bool,
    pub max_cycles: u32,
    pub(crate) autonomous_operation_enabled: bool,
    pub phase: CyclePhase,
    pub heartbeat_secs: u64,
    pub reevaluation_interval_secs: u64,
    pub task_completion_count: u32,
    pub cognitive_completion_count: u32,
    pub(crate) latest_experience: Option<Experience>,
    pub(crate) learning_history:
        Vec<crate::scheduler::cooboploop::learning_pipeline::LearningUpdate>,
    pub(crate) event_tracer: EventTracer,
    pub(crate) post_task_evaluation: PostTaskEvaluation,
    pub(crate) idle_state: IdleState,
    pub(crate) self_improvement_pipeline: Mutex<SelfImprovementPipeline>,
    pub(crate) research_manager: Mutex<ResearchManager>,
    pub(crate) capability_registry:
        Arc<Mutex<crate::scheduler::cooboploop::capability::CapabilityRegistry>>,
    pub(crate) objective_source_registry: ObjectiveSourceRegistry,
    pub experience_coordinator: Option<Arc<ExperienceCoordinator>>,
    pub objective_queue: queue::ObjectiveQueue,
    pub(crate) human_handler: HumanActionHandler,
    pub(crate) selected_goal: Option<AgentGoal>,
    pub current_plan: Option<Plan>,
    pub execution_result: Option<String>,
    /// Autonomous conversation handler (Architecture Section 6.30).
    pub(crate) autonomous_conversation: Mutex<AutonomousConversation>,
    /// Skill executor for autonomous skill refinement (Chapter 32).
    pub(crate) skill_executor: Option<Arc<crate::execution::skills::executor::SkillExecutor>>,
    pub(crate) strategic_registry:
        Arc<Mutex<crate::scheduler::cooboploop::strategic::StrategicObjectiveRegistry>>,
    pub(crate) reflection_engine: Arc<Mutex<ReflectionEngine>>,
    pub(crate) strategic_learning_engine: StrategicLearningEngine,
    pub(crate) mission: String,
    /// Autonomous background agent manager (Section 3.8 / Architecture).
    pub autonomous_agent_manager: Arc<Mutex<AutonomousAgentManager>>,
    /// Autonomous maintenance scheduler (Section 23.6.5 / Architecture Chapter 23).
    pub(crate) maintenance_scheduler: Mutex<MaintenanceScheduler>,
    /// Hardware discovery for adaptive capability assessment (Section 12 / Section 22).
    pub(crate) hardware_discovery: Mutex<HardwareDiscovery>,
    /// Persistent hardware snapshot registry for DB-backed profile storage (Section 12 / T8.9).
    pub(crate) hardware_registry:
        Option<Arc<Mutex<crate::scheduler::cooboploop::hardware::HardwareRegistry>>>,
    /// Software/system inspection engine (Section 13 / Architecture).
    pub(crate) inspector: Mutex<Inspector>,
    /// Knowledge store for learning feedback (Section 15: Knowledge Update).
    pub(crate) knowledge_store: KnowledgeStore,
    /// Memory hierarchy for autonomous promotion/compression/forgetting (Architecture §14).
    pub(crate) memory_hierarchy: MemoryHierarchy,
    /// Strategy tracker for learning feedback (Section 15: Strategy Refinement).
    pub(crate) strategy_tracker: Mutex<StrategyTracker>,
}

impl Default for LoopRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopRunner {
    pub fn new() -> Self {
        let config = LoopRunnerConfig::default();
        tracing::debug!(
            "LoopRunner config: heartbeat={}, interval={}",
            config.heartbeat_secs,
            config.reevaluation_interval_secs
        );
        let coordinator = Self::default_experience_coordinator();
        let mut capability_registry = CapabilityRegistry::new();
        super::super::capability::seed_default_capabilities(&mut capability_registry);
        let capability_registry_arc = Arc::new(Mutex::new(capability_registry));
        let mut research_manager = ResearchManager::new();
        research_manager.set_capability_registry(Arc::clone(&capability_registry_arc));
        let mut instance = Self {
            current_stage: LoopStage::ObserveState,
            cycle_count: 0,
            should_continue: true,
            max_cycles: 100,
            autonomous_operation_enabled: true,
            phase: CyclePhase::Observe,
            heartbeat_secs: 0,
            reevaluation_interval_secs: 60,
            task_completion_count: 0,
            cognitive_completion_count: 0,
            latest_experience: None,
            learning_history: Vec::new(),
            event_tracer: EventTracer::new(),
            experience_coordinator: coordinator,
            objective_queue: queue::ObjectiveQueue::new(),
            human_handler: HumanActionHandler::new(),
            selected_goal: None,
            current_plan: None,
            execution_result: None,
            post_task_evaluation: PostTaskEvaluation::new(),
            idle_state: IdleState::new(60),
            self_improvement_pipeline: Mutex::new(SelfImprovementPipeline::new()),
            research_manager: Mutex::new(research_manager),
            capability_registry: capability_registry_arc,
            objective_source_registry: ObjectiveSourceRegistry::init(),
            strategic_registry: Arc::new(Mutex::new(StrategicObjectiveRegistry::new())),
            reflection_engine: Arc::new(Mutex::new(ReflectionEngine::new())),
            strategic_learning_engine: StrategicLearningEngine::new(),
            mission: "Mission".to_string(),
            autonomous_conversation: Mutex::new(AutonomousConversation::new()),
            skill_executor: None,
            autonomous_agent_manager: Arc::new(Mutex::new(AutonomousAgentManager::new())),
            maintenance_scheduler: Mutex::new(MaintenanceScheduler::new()),
            hardware_discovery: Mutex::new(HardwareDiscovery::new()),
            hardware_registry: None,
            inspector: Mutex::new(Inspector::new()),
            knowledge_store: KnowledgeStore::new(10000),
            memory_hierarchy: MemoryHierarchy::new(),
            strategy_tracker: Mutex::new(StrategyTracker::new()),
        };
        config.apply_to(&mut instance);
        instance
    }

    fn default_experience_coordinator() -> Option<Arc<ExperienceCoordinator>> {
        let scorer = crate::experience::scorer::ExperienceScorer::new();
        let bus = Arc::new(crate::experience::bus::ExperienceBus::new());
        let metrics = Arc::new(crate::experience::metrics::MetricsCollector::new());
        Some(Arc::new(ExperienceCoordinator::new(scorer, bus, metrics)))
    }

    pub fn set_objective_queue(&mut self, queue: queue::ObjectiveQueue) {
        self.objective_queue = queue;
    }

    pub fn set_experience_coordinator(&mut self, coordinator: Arc<ExperienceCoordinator>) {
        self.experience_coordinator = Some(coordinator);
    }

    /// Set the hardware registry for DB-backed profile persistence (Section 12 / Section 22).
    pub fn set_hardware_registry(
        &mut self,
        registry: Arc<Mutex<crate::scheduler::cooboploop::hardware::HardwareRegistry>>,
    ) {
        self.hardware_registry = Some(registry);
    }

    /// Get a reference to the knowledge store (Section 15).
    pub fn knowledge_store(&self) -> &KnowledgeStore {
        &self.knowledge_store
    }

    /// Get strategy refinements from learning feedback (Section 15).
    pub fn get_strategy_refinements(&self) -> (Vec<String>, Vec<String>) {
        if let Ok(tracker) = self.strategy_tracker.lock() {
            (tracker.reinforced.clone(), tracker.avoided.clone())
        } else {
            (Vec::new(), Vec::new())
        }
    }

    pub fn start(&mut self) {
        self.should_continue = true;
    }

    pub fn stop(&mut self) {
        self.should_continue = false;
    }

    pub fn current_stage(&self) -> &LoopStage {
        &self.current_stage
    }

    pub fn current_cognitive_stage(&self) -> CognitiveCycleStage {
        self.current_stage.cognitive_cycle_stage()
    }

    pub fn should_continue(&self) -> bool {
        self.should_continue && self.cycle_count < self.max_cycles
    }

    pub fn set_max_cycles(&mut self, v: u32) {
        self.max_cycles = v;
    }

    /// Set the skill executor for autonomous skill refinement (Chapter 32).
    pub fn set_skill_executor(
        &mut self,
        executor: Arc<crate::execution::skills::executor::SkillExecutor>,
    ) {
        self.skill_executor = Some(executor);
    }

    /// Wire the memory repository into the research manager (Section 11 / T8.3).
    pub fn set_memory_repository_for_research(
        &mut self,
        repo: Arc<dyn crate::memory::repository::MemoryRepository>,
    ) {
        let mut manager = self
            .research_manager
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        manager.set_memory_repository(repo);
    }

    /// Wire the research pipeline for dispatching (§11).
    pub fn set_research_pipeline(
        &mut self,
        pipeline: Arc<crate::research::pipeline::ResearchPipeline>,
    ) {
        let mut manager = self
            .research_manager
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        manager.set_research_pipeline(pipeline);
    }

    /// Set the Tokio handle for spawning async research tasks.
    pub fn set_tokio_handle(&mut self, handle: tokio::runtime::Handle) {
        let mut manager = self
            .research_manager
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        manager.set_tokio_handle(handle);
    }

    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    pub fn max_cycles(&self) -> u32 {
        self.max_cycles
    }

    pub fn cycle_summary(&self) -> String {
        format!("cycles={}/{}", self.cycle_count, self.max_cycles)
    }

    pub fn learning_events(&self) -> &[String] {
        self.event_tracer.events()
    }

    pub fn is_autonomous(&self) -> bool {
        self.autonomous_operation_enabled
    }

    /// Get the autonomous agent manager for registering/managing agents (Section 3.8).
    pub fn autonomous_agent_manager(&self) -> &Arc<Mutex<AutonomousAgentManager>> {
        &self.autonomous_agent_manager
    }

    /// Set autonomous operation mode (Section 16 / Section 22 / T-COO-50).
    /// Fully operational: activates autonomous objective generation, opportunity
    /// discovery, prioritization, capability development, testing, optimization,
    /// hardware adaptation, software maintenance, improvement proposal, and verification.
    pub fn set_autonomous(&mut self, enabled: bool) {
        self.autonomous_operation_enabled = enabled;
        if enabled {
            tracing::debug!(
                "Autonomous operation fully activated: all cognitive cycle stages operational"
            );
        }
    }

    pub fn set_mission(&mut self, mission: String) {
        self.mission = mission;
    }

    pub fn mission(&self) -> &str {
        &self.mission
    }

    pub fn selected_goal(&self) -> Option<&AgentGoal> {
        self.selected_goal.as_ref()
    }

    pub fn current_plan(&self) -> Option<&Plan> {
        self.current_plan.as_ref()
    }

    pub fn post_task_evaluation(&self) -> &PostTaskEvaluation {
        &self.post_task_evaluation
    }

    pub fn tick_heartbeat(&mut self) -> bool {
        self.heartbeat_secs = self.heartbeat_secs.saturating_add(1);
        true
    }

    pub fn enter_wait(&mut self) {
        self.phase = CyclePhase::Wait;
        self.idle_state.enter_wait();
    }

    /// Return objectives back to the queue (Section 4 / T-COO-31).
    pub fn return_to_queue(&mut self, goals: &[AgentGoal]) -> u32 {
        let mut count = 0;
        for goal in goals {
            if self
                .objective_queue
                .transition(&goal.id, queue::GoalStatus::Discovered)
                .is_ok()
            {
                count += 1;
            }
        }
        count
    }

    /// Run continuous cognitive cycle (Section 20 / CoObOpLoop.md Section 20).
    ///
    /// This is the continuous sync mode that keeps the cognitive loop
    /// running autonomously between external objectives.
    /// Per Architecture Section 20: The loop SHALL NOT terminate when a task
    /// completes; it transitions to evaluation and continues cognitive activity.
    pub fn run_continuous_sync(&mut self) -> Result<(), String> {
        // Dispatch pending research objectives (Section 11 / T8.1-T8.4).
        // Research SHALL be first-class objective; SHALL produce persistent knowledge/experience.
        if let Err(e) = self.dispatch_research() {
            tracing::debug!("Research dispatch skipped: {e}");
        }
        // Run one full cycle
        match self.run_cycle() {
            Ok(()) => {
                tracing::debug!("Continuous sync: completed one cognitive cycle");
                // Transition to wait/idle state between cycles
                self.enter_wait();
            }
            Err(e) => {
                tracing::warn!("Continuous sync: cycle failed: {e}");
                // Don't fail the loop on individual cycle errors - enter wait state
                self.enter_wait();
            }
        }
        Ok(())
    }

    /// Dispatch the next pending research objective (Section 11 / T8.1-T8.4).
    /// Per CoObOpLoop.md §11: research SHALL be first-class objective;
    /// SHALL produce persistent knowledge/experience.
    /// Returns the experience ID if dispatched, or an error if no objectives exist.
    pub fn dispatch_research(&mut self) -> Result<String, String> {
        let mut manager = self
            .research_manager
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Clone the Arc to get an owned reference, avoiding borrow conflicts.
        let repo = manager.memory_repository_clone();
        manager.dispatch_next(repo.as_deref())
    }
}
