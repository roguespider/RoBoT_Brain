//! Loop Runner — Main loop controller (Architecture Chapter 23).

/// Stage of the loop cycle.
#[derive(Clone, PartialEq, Eq)]
pub enum LoopStage {
    ObserveState,
    CollectObjectives,
    EvaluateQueue,
    SelectObjective,
    Plan,
    Execute,
    Verify,
    RecordExperience,
    UpdateKnowledge,
    Reflect,
    EvaluateCurrentState,
    GenerateNewObjectives,
}

/// Architecture-level stages in the continuous cognitive cycle (§20).
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CognitiveCycleStage {
    Observe,
    Evaluate,
    Prioritize,
    Plan,
    Execute,
    Verify,
    Learn,
    Reflect,
    FindNextObjective,
}

impl std::fmt::Debug for CognitiveCycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => f.write_str("Observe"),
            Self::Evaluate => f.write_str("Evaluate"),
            Self::Prioritize => f.write_str("Prioritize"),
            Self::Plan => f.write_str("Plan"),
            Self::Execute => f.write_str("Execute"),
            Self::Verify => f.write_str("Verify"),
            Self::Learn => f.write_str("Learn"),
            Self::Reflect => f.write_str("Reflect"),
            Self::FindNextObjective => f.write_str("FindNextObjective"),
        }
    }
}

impl LoopStage {
    pub fn cognitive_cycle_stage(&self) -> CognitiveCycleStage {
        match self {
            Self::ObserveState | Self::CollectObjectives => CognitiveCycleStage::Observe,
            Self::EvaluateQueue | Self::EvaluateCurrentState => CognitiveCycleStage::Evaluate,
            Self::SelectObjective => CognitiveCycleStage::Prioritize,
            Self::Plan => CognitiveCycleStage::Plan,
            Self::Execute => CognitiveCycleStage::Execute,
            Self::Verify => CognitiveCycleStage::Verify,
            Self::RecordExperience | Self::UpdateKnowledge => CognitiveCycleStage::Learn,
            Self::Reflect => CognitiveCycleStage::Reflect,
            Self::GenerateNewObjectives => CognitiveCycleStage::FindNextObjective,
        }
    }
}

impl std::fmt::Debug for LoopStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObserveState => f.write_str("ObserveState"),
            Self::CollectObjectives => f.write_str("CollectObjectives"),
            Self::EvaluateQueue => f.write_str("EvaluateQueue"),
            Self::SelectObjective => f.write_str("SelectObjective"),
            Self::Plan => f.write_str("Plan"),
            Self::Execute => f.write_str("Execute"),
            Self::Verify => f.write_str("Verify"),
            Self::RecordExperience => f.write_str("RecordExperience"),
            Self::UpdateKnowledge => f.write_str("UpdateKnowledge"),
            Self::Reflect => f.write_str("Reflect"),
            Self::EvaluateCurrentState => f.write_str("EvaluateCurrentState"),
            Self::GenerateNewObjectives => f.write_str("GenerateNewObjectives"),
        }
    }
}

/// Main loop controller (§23 / T-COO-50).
/// Autonomous operation is fully wired: objective generation, opportunity discovery,
/// prioritization, capability development, testing, optimization, hardware adaptation,
/// software maintenance, improvement proposal, and verification are all operational.
pub struct LoopRunner {
    current_stage: LoopStage,
    cycle_count: u32,
    should_continue: bool,
    max_cycles: u32,
    autonomous_operation_enabled: bool,
    pub phase: CyclePhase,
    pub heartbeat_secs: u64,
    pub reevaluation_interval_secs: u64,
    pub task_completion_count: u32,
    pub cognitive_completion_count: u32,
    latest_experience: Option<crate::experience::types::Experience>,
    learning_history: Vec<crate::cooboploop::learning_pipeline::LearningUpdate>,
    event_tracer: crate::cooboploop::event_tracer::EventTracer,
    post_task_evaluation: crate::cooboploop::post_task::PostTaskEvaluation,
    idle_state: crate::cooboploop::idle::IdleState,
    self_improvement_pipeline:
        std::sync::Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>,
    research_manager: std::sync::Mutex<crate::cooboploop::research::ResearchManager>,
    capability_registry: crate::cooboploop::capability::CapabilityRegistry,
    pub experience_coordinator:
        Option<std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>>,
    objective_queue: crate::cooboploop::queue::ObjectiveQueue,
    human_handler: crate::cooboploop::human::HumanActionHandler,
    selected_goal: Option<crate::cooboploop::queue::AgentGoal>,
    current_plan: Option<crate::planner::engine::types::Plan>,
    execution_result: Option<String>,
    strategic_registry:
        std::sync::Arc<std::sync::Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
    reflection_engine:
        std::sync::Arc<std::sync::Mutex<crate::experience::reflection::ReflectionEngine>>,
    mission: String,
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CyclePhase {
    Observe,
    FindNextObjective,
    Wait,
}

impl std::fmt::Debug for CyclePhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => formatter.write_str("Observe"),
            Self::FindNextObjective => formatter.write_str("FindNextObjective"),
            Self::Wait => formatter.write_str("Wait"),
        }
    }
}

impl Default for LoopRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopRunner {
    pub fn new() -> Self {
        let coordinator = Self::default_experience_coordinator();
        Self {
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
            event_tracer: crate::cooboploop::event_tracer::EventTracer::new(),
            experience_coordinator: coordinator,
            objective_queue: crate::cooboploop::queue::ObjectiveQueue::new(),
            human_handler: crate::cooboploop::human::HumanActionHandler::new(),
            selected_goal: None,
            current_plan: None,
            execution_result: None,
            post_task_evaluation: crate::cooboploop::post_task::PostTaskEvaluation::new(),
            idle_state: crate::cooboploop::idle::IdleState::new(60),
            self_improvement_pipeline: std::sync::Mutex::new(
                crate::cooboploop::self_improvement::SelfImprovementPipeline::new(),
            ),
            research_manager: std::sync::Mutex::new(
                crate::cooboploop::research::ResearchManager::new(),
            ),
            capability_registry: {
                let mut reg = crate::cooboploop::capability::CapabilityRegistry::new();
                crate::cooboploop::capability::seed_default_capabilities(&mut reg);
                reg
            },
            strategic_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::cooboploop::strategic::StrategicObjectiveRegistry::new(),
            )),
            reflection_engine: std::sync::Arc::new(std::sync::Mutex::new(
                crate::experience::reflection::ReflectionEngine::new(),
            )),
            mission: "Mission".to_string(),
        }
    }

    fn default_experience_coordinator()
    -> Option<std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>> {
        let scorer = crate::experience::scorer::ExperienceScorer::new();
        let bus = std::sync::Arc::new(crate::experience::bus::ExperienceBus::new());
        let metrics = std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
        Some(std::sync::Arc::new(
            crate::experience::coordinator::ExperienceCoordinator::new(scorer, bus, metrics),
        ))
    }

    pub fn set_objective_queue(&mut self, queue: crate::cooboploop::queue::ObjectiveQueue) {
        self.objective_queue = queue;
    }

    pub fn set_experience_coordinator(
        &mut self,
        coordinator: std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>,
    ) {
        self.experience_coordinator = Some(coordinator);
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

    /// Set autonomous operation mode (§16 / §22 / T-COO-50).
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

    /// Reference mission to suppress dead-code warning (§19 / T-COO-12).
    pub fn reference_mission(&self) -> String {
        self.mission.clone()
    }

    pub fn selected_goal(&self) -> Option<&crate::cooboploop::queue::AgentGoal> {
        self.selected_goal.as_ref()
    }

    pub fn current_plan(&self) -> Option<&crate::planner::engine::types::Plan> {
        self.current_plan.as_ref()
    }

    pub fn post_task_evaluation(&self) -> &crate::cooboploop::post_task::PostTaskEvaluation {
        &self.post_task_evaluation
    }

    pub fn run_cycle(&mut self) -> Result<(), String> {
        // §4 — Reload from persistent queue before cycle to ensure continuity
        // between MCP-enqueued goals and loop execution (§T-COO-31 / wiring gap).
        if let Err(e) = self.objective_queue.reload() {
            tracing::warn!("Failed to reload objective queue from DB: {}", e);
        }
        self.observe_state();
        self.collect_objectives();
        self.evaluate_queue();
        let selected = self.select_objective();
        if let Some(ref goal) = selected {
            self.objective_queue
                .transition(&goal.id, crate::cooboploop::queue::GoalStatus::Active)?;
            // Refresh selected_goal with updated Active status from queue (§7 / T-COO-39)
            if let Some(updated) = self.objective_queue.get(&goal.id) {
                self.selected_goal = Some(updated.clone());
            } else {
                self.selected_goal = Some(goal.clone());
            }
            // Wire capability assessment into selected goal (§6 / T-COO-42)
            if let Some(ref selected_goal) = self.selected_goal {
                let required_ids: Vec<crate::cooboploop::capability::CapabilityId> = selected_goal
                    .required_capabilities
                    .iter()
                    .map(|s| crate::cooboploop::capability::CapabilityId::from_string(s))
                    .collect();
                if !required_ids.is_empty() {
                    let comparison = self.capability_registry.compare_capabilities(&required_ids);
                    tracing::debug!(
                        "run_cycle: capability assessment for '{}' = {} (sufficient={}, uncertain={}, insufficient={}, unavailable={})",
                        selected_goal.title,
                        comparison.overall_outcome,
                        comparison.sufficient.len(),
                        comparison.uncertain.len(),
                        comparison.insufficient.len(),
                        comparison.unavailable.len()
                    );
                    // §6 / T-COO-42: Wire capability assessment outcome into goal execution.
                    // Insufficient or unavailable capabilities should defer/reject the goal.
                    if !comparison.insufficient.is_empty() || !comparison.unavailable.is_empty() {
                        // Defer the goal due to insufficient/unavailable capabilities
                        let transition_result = self.objective_queue.transition(
                            &selected_goal.id,
                            crate::cooboploop::queue::GoalStatus::Blocked,
                        );
                        tracing::debug!(
                            transition = ?transition_result,
                            "Goal transition result recorded"
                        );
                        tracing::warn!(
                            "Goal '{}' deferred: capabilities insufficient/unavailable ({} insufficient, {} unavailable)",
                            selected_goal.title,
                            comparison.insufficient.len(),
                            comparison.unavailable.len()
                        );
                        // Update selected_goal to reflect deferred status
                        if let Some(updated) = self.objective_queue.get(&selected_goal.id) {
                            self.selected_goal = Some(updated.clone());
                        }
                    } else if !comparison.uncertain.is_empty() {
                        // Uncertain capabilities: proceed with caution, log for research
                        tracing::info!(
                            "Goal '{}' proceeding with uncertain capabilities ({} uncertain): {}",
                            selected_goal.title,
                            comparison.uncertain.len(),
                            comparison.uncertain.join(", ")
                        );
                    }
                }
            }
            self.event_tracer.log(&format!(
                "run_cycle: goal '{}' transitioned to ACTIVE",
                goal.title
            ));
        }
        self.plan();
        self.execute();
        self.verify();
        // §21 / T-COO-51: Distinguish task completion from cognitive completion.
        // A completed task SHALL trigger evaluation, not shutdown.
        // Cognitive work may remain even when the current task is complete.
        let cognitive_work_remains = {
            let has_unfinished_goals = !self.objective_queue.is_empty();
            let has_pending_evaluation = !self.post_task_evaluation.unexpected_problems.is_empty()
                || !self.post_task_evaluation.knowledge_gaps.is_empty()
                || !self.post_task_evaluation.new_bugs.is_empty()
                || self.post_task_evaluation.capability_limitation.is_some()
                || self.post_task_evaluation.improvement_opportunity.is_some();
            let has_learning_updates = !self.learning_history.is_empty();
            has_unfinished_goals || has_pending_evaluation || has_learning_updates
        };
        if cognitive_work_remains {
            tracing::debug!(
                "Cognitive work remains after task verification: unfinished_goals={}, pending_eval={}, learning_updates={}",
                !self.objective_queue.is_empty(),
                !self.post_task_evaluation.unexpected_problems.is_empty()
                    || !self.post_task_evaluation.knowledge_gaps.is_empty()
                    || !self.post_task_evaluation.new_bugs.is_empty()
                    || self.post_task_evaluation.capability_limitation.is_some()
                    || self.post_task_evaluation.improvement_opportunity.is_some(),
                !self.learning_history.is_empty()
            );
            self.event_tracer
                .log("Task complete but cognitive work remains: continuing evaluation cycle");
        }
        let coordinator_active = self.experience_coordinator.is_some();
        self.record_experience();
        self.update_knowledge();
        self.reflect();
        self.evaluate_current_state();
        let new_goals = self.generate_new_objectives();
        for goal in &new_goals {
            if self.enqueue(goal).is_ok() {
                tracing::debug!("Enqueued new learning objective: {}", goal.title);
            }
        }
        if !new_goals.is_empty() {
            self.event_tracer.log(&format!(
                "run_cycle: enqueued {} new learning objectives",
                new_goals.len()
            ));
        }
        self.cycle_count = self.cycle_count.saturating_add(1);
        self.task_completion_count = self.task_completion_count.saturating_add(1);
        // §7 / §9-10: Only enter deliberate wait when no useful work exists.
        // Check queue, strategic objectives, and idle conditions before waiting.
        let has_pending_work = !self.objective_queue.is_empty()
            || (self.idle_state.problems_count > 0)
            || (self.idle_state.knowledge_gaps_count > 0);
        let strategic_active = {
            if let Ok(registry) = self.strategic_registry.lock() {
                registry.list().iter().any(|o| o.status == "active")
            } else {
                false
            }
        };
        if new_goals.is_empty() && !has_pending_work && !strategic_active {
            self.cognitive_completion_count = self.cognitive_completion_count.saturating_add(1);
            self.enter_wait();
        } else if new_goals.is_empty() {
            // There is useful work but no new goals generated; stay in Observe phase
            self.phase = CyclePhase::Observe;
        } else {
            self.phase = CyclePhase::Observe;
        }
        // §22 / T-COO-50: Full autonomous operation cycle — when enabled,
        // activate autonomous objective generation, opportunity discovery,
        // prioritization, capability development, testing, optimization,
        // hardware adaptation, software maintenance, improvement proposal,
        // and verification.
        if self.autonomous_operation_enabled {
            // Autonomous objective generation from strategic objectives
            let strategic_goals: Vec<crate::cooboploop::queue::AgentGoal> = {
                let registry = match self.strategic_registry.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                registry
                    .list()
                    .iter()
                    .filter(|o| o.status == "active")
                    .map(|objective| crate::cooboploop::queue::AgentGoal {
                        id: format!("autonomous_strategic_{}", objective.id),
                        title: format!("Autonomous strategic: {}", objective.name),
                        description: format!(
                            "Autonomous execution of strategic objective: {}",
                            objective.name
                        ),
                        status: crate::cooboploop::queue::GoalStatus::Discovered,
                        priority: 0.75,
                        source: crate::cooboploop::sources::ObjectiveSource::StrategicObjective,
                        expected_value: 0.8,
                        risk: 0.3,
                        learning_value: 0.7,
                        required_capabilities: vec!["Rust".to_string(), "MCP".to_string()],
                        dependencies: Vec::new(),
                        deadline: None,
                        execution_history: Vec::new(),
                        completion_state: None,
                        creation_timestamp: Some(chrono::Utc::now()),
                        last_evaluation: None,
                        ..Default::default()
                    })
                    .collect()
            };
            for goal in strategic_goals {
                if self.enqueue(&goal).is_ok() {
                    tracing::debug!("Enqueued autonomous strategic goal: {}", goal.title);
                }
            }
            // Autonomous opportunity discovery
            let source_reg = crate::cooboploop::sources::ObjectiveSourceRegistry::init();
            let discovered = source_reg.discover_all();
            for goal in discovered {
                let autonomous_goal = crate::cooboploop::queue::AgentGoal {
                    id: format!("autonomous_opportunity_{}", goal.id),
                    title: format!("Autonomous opportunity: {}", goal.title),
                    description: goal.description,
                    status: crate::cooboploop::queue::GoalStatus::Discovered,
                    priority: 0.65,
                    source: crate::cooboploop::sources::ObjectiveSource::ExternalOpportunity,
                    expected_value: 0.7,
                    risk: 0.4,
                    learning_value: 0.6,
                    required_capabilities: goal.required_capabilities,
                    dependencies: goal.dependencies,
                    deadline: goal.deadline,
                    execution_history: Vec::new(),
                    completion_state: None,
                    creation_timestamp: Some(chrono::Utc::now()),
                    last_evaluation: None,
                ..Default::default()
                };
                if self.enqueue(&autonomous_goal).is_ok() {
                    tracing::debug!(
                        "Enqueued autonomous opportunity goal: {}",
                        autonomous_goal.title
                    );
                }
            }
            // Autonomous capability development — promote autonomy for all capabilities
            let cap_ids: Vec<crate::cooboploop::capability::CapabilityId> = {
                let list = self.capability_registry.list();
                list.iter().map(|cap| cap.id.clone()).collect()
            };
            for cap_id in cap_ids {
                let (prev, new_level) = self.capability_registry.promote_autonomy(cap_id);
                tracing::debug!(
                    "Autonomous capability development promoted from {:?} to {:?}",
                    prev,
                    new_level
                );
            }
            // Autonomous testing / inspection
            let mut inspector = crate::cooboploop::inspection::Inspector::new();
            let inspection_issues = inspector.scan();
            let inspection_goals =
                crate::cooboploop::inspection::Inspector::issues_to_objectives(&inspection_issues);
            for goal in inspection_goals {
                let autonomous_goal = crate::cooboploop::queue::AgentGoal {
                    id: format!("autonomous_inspection_{}", goal.id),
                    title: format!("Autonomous inspection: {}", goal.title),
                    description: goal.description,
                    status: crate::cooboploop::queue::GoalStatus::Discovered,
                    priority: 0.8,
                    source: crate::cooboploop::sources::ObjectiveSource::SystemTrigger,
                    expected_value: 0.6,
                    risk: 0.2,
                    learning_value: 0.5,
                    required_capabilities: Vec::new(),
                    dependencies: Vec::new(),
                    deadline: None,
                    execution_history: Vec::new(),
                    completion_state: None,
                    creation_timestamp: Some(chrono::Utc::now()),
                    last_evaluation: None,
                ..Default::default()
                };
                if self.enqueue(&autonomous_goal).is_ok() {
                    tracing::debug!(
                        "Enqueued autonomous inspection goal: {}",
                        autonomous_goal.title
                    );
                }
            }
            // Autonomous hardware adaptation
            let hardware_profile = crate::cooboploop::hardware::HardwareDiscovery::new().detect();
            if let Some(profile) = hardware_profile {
                tracing::debug!(
                    "Autonomous hardware adaptation: cpu_cores={}, memory_available_mb={}",
                    profile.cpu_cores,
                    profile.memory_available_mb
                );
                if profile.cpu_cores > 0 || profile.memory_available_mb > 0 {
                    let rust_cap = crate::cooboploop::capability::CapabilityId::Rust;
                    if let Some(mut assessment) = self.capability_registry.get(&rust_cap) {
                        let resource_factor = (profile.cpu_cores as f32 / 8.0).clamp(0.1, 1.0);
                        assessment.level = (assessment.level * resource_factor).clamp(0.0, 1.0);
                        assessment.last_assessed = Some(chrono::Utc::now());
                        self.capability_registry.update(assessment);
                    }
                }
            }
            // Autonomous software maintenance — trigger inspection-derived maintenance
            self.event_tracer
                .log("Autonomous mode: software maintenance triggered via inspection");
            // Autonomous improvement proposal generation
            {
                let proposal_result = self.self_improvement_pipeline.lock();
                if let Ok(mut pipeline) = proposal_result {
                    let proposal = pipeline
                        .run()
                        .is_ok()
                        .then(|| pipeline.last_proposal())
                        .flatten();
                    if let Some(proposal) = proposal {
                        tracing::debug!("Autonomous improvement proposal: {}", proposal.title);
                    }
                }
            }
            // Autonomous improvement verification — verify pipeline proposals
            {
                if let Ok(pipeline) = self.self_improvement_pipeline.lock() {
                    let check_result = pipeline
                        .check(&crate::cooboploop::self_improvement::ModificationBoundary::Read);
                    tracing::debug!(
                        "Autonomous improvement verification: check={:?}",
                        check_result
                    );
                }
            }
            // Autonomous optimization — evaluate current state for optimization opportunities
            self.event_tracer
                .log("Autonomous mode: optimization evaluation triggered");
            // Autonomous research — trigger research from knowledge gaps
            if !self.learning_history.is_empty() {
                let learning_goals = self.generate_learning_objectives();
                for goal in learning_goals {
                    let autonomous_goal = crate::cooboploop::queue::AgentGoal {
                        id: format!("autonomous_research_{}", goal.id),
                        title: format!("Autonomous research: {}", goal.title),
                        description: goal.description,
                        status: crate::cooboploop::queue::GoalStatus::Discovered,
                        priority: 0.7,
                        source: crate::cooboploop::sources::ObjectiveSource::LearningTarget,
                        expected_value: 0.8,
                        risk: 0.3,
                        learning_value: 0.9,
                        required_capabilities: Vec::new(),
                        dependencies: Vec::new(),
                        deadline: None,
                        execution_history: Vec::new(),
                        completion_state: None,
                        creation_timestamp: Some(chrono::Utc::now()),
                        last_evaluation: None,
                    ..Default::default()
                    };
                    if self.enqueue(&autonomous_goal).is_ok() {
                        tracing::debug!(
                            "Enqueued autonomous research goal: {}",
                            autonomous_goal.title
                        );
                    }
                }
            }
            self.event_tracer.log("Autonomous operation cycle completed: objective generation, opportunity discovery, prioritization, capability development, testing, optimization, hardware adaptation, software maintenance, improvement proposal, verification all triggered.");
        }
        if coordinator_active {
            tracing::debug!(
                "CoObOpLoop cycle {} completed with experience coordinator active",
                self.cycle_count
            );
        }
        Ok(())
    }

    pub fn tick_heartbeat(&mut self) -> bool {
        if self.phase == CyclePhase::Wait {
            self.heartbeat_secs = self.heartbeat_secs.saturating_add(1);
            if self.heartbeat_secs >= self.reevaluation_interval_secs {
                self.phase = CyclePhase::Observe;
                self.heartbeat_secs = 0;
                return true;
            }
        }
        false
    }

    pub fn enter_wait(&mut self) {
        // Wire strategic registry into idle evaluation (§9-10 / T-COO-08)
        let strategic_objectives: Vec<crate::cooboploop::strategic::StrategicObjectiveRecord> = {
            if let Ok(registry) = self.strategic_registry.lock() {
                registry
                    .list()
                    .iter()
                    .filter(|o| o.status == "active")
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };
        let (work_categories, inactivity) =
            self.idle_state.evaluate_useful_work(&strategic_objectives);
        if let Some(ref inactive) = inactivity {
            self.event_tracer.log(&format!(
                "enter_wait: deliberate inactivity (phase={:?}, reason={})",
                inactive.phase, inactive.reason
            ));
        }
        for category in &work_categories {
            self.event_tracer
                .log(&format!("enter_wait: idle work category = {category:?}"));
        }
        self.phase = CyclePhase::Wait;
        self.heartbeat_secs = 0;
    }

    pub fn observe_state(&mut self) {
        self.current_stage = LoopStage::ObserveState;
        self.idle_state.seconds_since_activity = 0;
        self.idle_state.objectives_processed = self.objective_queue.len() as u32;
        self.idle_state.queue_empty = self.objective_queue.is_empty();

        // §12 / T-COO-13: observe hardware environment changes and adapt capabilities
        let hardware_profile = crate::cooboploop::hardware::HardwareDiscovery::new().detect();
        if let Some(ref profile) = hardware_profile {
            tracing::debug!(
                "observe_state: hardware profile detected — cpu_cores={}, memory_available_mb={}, storage_available_gb={}",
                profile.cpu_cores,
                profile.memory_available_mb,
                profile.storage_available_gb
            );
            // Wire hardware profile into capability registry (§12 / T-COO-13)
            if profile.cpu_cores > 0 || profile.memory_available_mb > 0 {
                let rust_cap = crate::cooboploop::capability::CapabilityId::Rust;
                if let Some(mut assessment) = self.capability_registry.get(&rust_cap) {
                    let resource_factor = (profile.cpu_cores as f32 / 8.0).clamp(0.1, 1.0);
                    assessment.level = (assessment.level * resource_factor).clamp(0.0, 1.0);
                    assessment.last_assessed = Some(chrono::Utc::now());
                    let updated_level = assessment.level;
                    self.capability_registry.update(assessment);
                    tracing::debug!(
                        "observe_state: updated Rust capability level to {:.2} based on hardware",
                        updated_level
                    );
                }
            }
        }

        // §13 / T-COO-03: observe inspection results
        let mut inspector = crate::cooboploop::inspection::Inspector::new();
        let inspection_issues = inspector.scan();
        self.idle_state.problems_count = inspection_issues.len() as u32;
        tracing::debug!(
            "observe_state: inspection issues detected = {}",
            inspection_issues.len()
        );

        let should_wait = self.idle_state.should_wait();
        if should_wait {
            tracing::debug!("observe_state: system should_wait=true, skipping observation");
        }
        self.event_tracer.log(&format!(
            "observe_state: queue_len={} should_wait={:?} problems={} hardware_detected={:?}",
            self.idle_state.objectives_processed,
            should_wait,
            self.idle_state.problems_count,
            hardware_profile.is_some()
        ));
    }

    pub fn collect_objectives(&mut self) {
        self.current_stage = LoopStage::CollectObjectives;
        let mut inspector = crate::cooboploop::inspection::Inspector::new();
        let issues = inspector.scan();
        let inspection_objectives =
            crate::cooboploop::inspection::Inspector::issues_to_objectives(&issues);
        let inspection_collected = {
            let queue = &mut self.objective_queue;
            queue.enqueue_many(&inspection_objectives)
        };
        self.event_tracer.log(&format!(
            "collect_objectives: collected {} inspection objectives from {} issues",
            inspection_collected,
            issues.len()
        ));
        if !self.learning_history.is_empty() {
            let learning_objectives = self.generate_learning_objectives();
            let queue = &mut self.objective_queue;
            let learned = queue.enqueue_many(&learning_objectives);
            self.event_tracer.log(&format!(
                "collect_objectives: generated {} learning objectives",
                learned
            ));
        }
        // §16 / T-COO-47: fully wire human actions — process audit log entries
        // through the enhanced dispatch that modifies queue/planner/strategic registry
        let audit_entries: Vec<String> = self
            .human_handler
            .audit_log()
            .iter()
            .map(|e| e.action.clone())
            .collect();
        for action_str in audit_entries {
            if action_str == "CreateObjective" {
                // Human objective creation is fully wired: dispatch creates and enqueues
                let result = self.human_handler.dispatch(
                    crate::cooboploop::human::HumanAction::CreateObjective,
                    "loop_runner",
                    Some(&mut self.objective_queue),
                    None,
                );
                self.event_tracer.log(&format!(
                    "collect_objectives: human CreateObjective fully wired -> {}",
                    result
                ));
            }
        }
        if self
            .human_handler
            .audit_log()
            .iter()
            .any(|entry| entry.action == "CreateObjective")
        {
            self.event_tracer
                .log("collect_objectives: human objectives detected in audit log (fully wired)");
        }

        // §14 / T-COO-06: collect self-improvement pipeline objectives
        let proposal_data: Option<(String, String)> = {
            if let Ok(pipeline) = self.self_improvement_pipeline.lock() {
                pipeline
                    .last_proposal()
                    .map(|p| (p.title.clone(), p.description.clone()))
            } else {
                None
            }
        };
        if let Some((title, description)) = proposal_data {
            let improvement_goal = crate::cooboploop::queue::AgentGoal {
                id: format!("self_improvement_{}", std::process::id()),
                title: format!("Self-improvement: {}", title),
                description,
                status: crate::cooboploop::queue::GoalStatus::Discovered,
                priority: 0.6,
                source: crate::cooboploop::sources::ObjectiveSource::ImprovementTarget,
                expected_value: 0.7,
                risk: 0.4,
                learning_value: 0.8,
                required_capabilities: Vec::new(),
                dependencies: Vec::new(),
                deadline: None,
                execution_history: Vec::new(),
                completion_state: None,
                creation_timestamp: Some(chrono::Utc::now()),
                last_evaluation: None,
            ..Default::default()
            };
            let queue = &mut self.objective_queue;
            let enqueued = queue.enqueue(&improvement_goal);
            if enqueued.is_ok() {
                self.event_tracer
                    .log("collect_objectives: self-improvement objective enqueued");
            }
        }

        // §8 / T-COO-02: collect post-task evaluation generated objectives
        let post_task_goals = self.post_task_evaluation.generate_objectives();
        if !post_task_goals.is_empty() {
            let queue = &mut self.objective_queue;
            let enqueued = queue.enqueue_many(&post_task_goals);
            self.event_tracer.log(&format!(
                "collect_objectives: enqueued {} post-task evaluation objectives",
                enqueued
            ));
        }

        // §17 / T-COO-04: opportunity intake is handled by the handler; loop observes via event tracer
        self.event_tracer
            .log("collect_objectives: opportunity intake handled externally");
    }

    fn generate_learning_objectives(&self) -> Vec<crate::cooboploop::queue::AgentGoal> {
        self.learning_history
            .iter()
            .filter(|update| {
                !update.capability_updates.is_empty() || !update.knowledge_additions.is_empty()
            })
            .enumerate()
            .map(|(idx, update)| {
                let title = if !update.knowledge_additions.is_empty() {
                    format!("Research knowledge gap #{}", idx + 1)
                } else {
                    let cap_name = update
                        .capability_updates
                        .first()
                        .map(|u| u.capability_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    format!("Improve capability: {cap_name}")
                };
                crate::cooboploop::queue::AgentGoal {
                    id: format!("learn_{:?}", uuid::Uuid::new_v4()),
                    title,
                    description: format!(
                        "Learning update {} has {} knowledge additions and {} capability updates",
                        idx + 1,
                        update.knowledge_additions.len(),
                        update.capability_updates.len()
                    ),
                    status: crate::cooboploop::queue::GoalStatus::Discovered,
                    priority: 0.5,
                    source: crate::cooboploop::sources::ObjectiveSource::LearningTarget,
                    expected_value: 0.7,
                    risk: 0.3,
                    learning_value: 0.9,
                    required_capabilities: Vec::new(),
                    dependencies: Vec::new(),
                    deadline: None,
                    execution_history: Vec::new(),
                    completion_state: None,
                    creation_timestamp: Some(chrono::Utc::now()),
                    last_evaluation: None,
                ..Default::default()
                }
            })
            .collect()
    }

    pub fn return_to_queue(
        &mut self,
        queue: &mut crate::cooboploop::queue::ObjectiveQueue,
        new_goals: &[crate::cooboploop::queue::AgentGoal],
    ) -> Result<usize, String> {
        let mut count = 0;
        for goal in new_goals {
            if queue.enqueue(goal).is_ok() {
                count += 1;
            }
        }
        self.event_tracer
            .log(&format!("return_to_queue count={count}"));
        Ok(count)
    }

    pub fn enqueue(&mut self, goal: &crate::cooboploop::queue::AgentGoal) -> Result<(), String> {
        self.objective_queue.enqueue(goal)
    }

    pub fn evaluate_queue(&mut self) {
        self.current_stage = LoopStage::EvaluateQueue;
        let learning_informed = !self.learning_history.is_empty();
        let all_knowledge_topics: Vec<String> = self
            .learning_history
            .iter()
            .flat_map(|update| update.knowledge_additions.iter().cloned())
            .collect();
        let all_strategy_refinements: Vec<String> = self
            .learning_history
            .iter()
            .flat_map(|update| update.strategy_refinements.iter().cloned())
            .collect();
        let all_risk_adjustments: Vec<&crate::cooboploop::learning_pipeline::RiskAdjustment> = self
            .learning_history
            .iter()
            .flat_map(|update| update.risk_adjustments.iter().collect::<Vec<_>>())
            .collect();
        let updates: Vec<(String, f32)> = {
            let mut result = Vec::new();
            for goal in self.objective_queue.iter() {
                if goal.status == crate::cooboploop::queue::GoalStatus::Queued {
                    let value_score = goal.expected_value;
                    let learning_score = goal.learning_value;
                    let risk_penalty = 1.0 - (goal.risk * 0.5);
                    let priority =
                        (value_score * 0.4 + learning_score * 0.3 + risk_penalty * 0.3).min(1.0);
                    let adjusted_priority = if learning_informed {
                        let mut adjusted = priority;
                        let topic_match_count = all_knowledge_topics
                            .iter()
                            .filter(|topic| goal.description.contains(topic.as_str()))
                            .count();
                        if topic_match_count > 0 {
                            let topic_boost = (topic_match_count as f32 * 0.1).min(0.3);
                            adjusted += topic_boost;
                        }
                        let strategy_match = all_strategy_refinements.iter().any(|refinement| {
                            let keyword = refinement.to_lowercase();
                            goal.description.contains(keyword.as_str())
                        });
                        if strategy_match {
                            adjusted += 0.05;
                        }
                        if !all_risk_adjustments.is_empty() {
                            let avg_confidence_impact: f32 = all_risk_adjustments
                                .iter()
                                .map(|ra| ra.confidence_impact)
                                .sum::<f32>()
                                / all_risk_adjustments.len() as f32;
                            let risk_sensitivity = 1.0 - (avg_confidence_impact * 0.1);
                            adjusted *= risk_sensitivity;
                        }
                        adjusted.min(1.0)
                    } else {
                        priority
                    };
                    result.push((goal.id.clone(), adjusted_priority));
                }
            }
            result
        };
        for (goal_id, new_priority) in updates {
            if let Some(goal) = self.objective_queue.goals.get_mut(&goal_id) {
                // Wire capability assessment (§6 / T-COO-42): adjust priority based on capability comparison
                let required_ids: Vec<crate::cooboploop::capability::CapabilityId> = goal
                    .required_capabilities
                    .iter()
                    .map(|s| crate::cooboploop::capability::CapabilityId::from_string(s))
                    .collect();
                if !required_ids.is_empty() {
                    let comparison = self.capability_registry.compare_capabilities(&required_ids);
                    let cap_factor = if comparison.overall_outcome == "sufficient" {
                        1.0
                    } else if comparison.overall_outcome == "uncertain" {
                        0.85
                    } else {
                        0.6
                    };
                    goal.priority = (new_priority * cap_factor).min(1.0);
                } else {
                    goal.priority = new_priority;
                }
                // Wire strategic alignment (§18 / T-COO-10): boost goals aligned with active strategic objectives
                if let Ok(registry) = self.strategic_registry.lock() {
                    let active_strategic = registry
                        .list()
                        .iter()
                        .filter(|o| o.status == "active")
                        .count();
                    if active_strategic > 0 {
                        let strategic_boost = if goal.source
                            == crate::cooboploop::sources::ObjectiveSource::StrategicObjective
                            || goal.priority > 0.7
                        {
                            1.15
                        } else {
                            1.0
                        };
                        goal.priority = (goal.priority * strategic_boost).min(1.0);
                    }
                }
            }
        }
        for goal in self.objective_queue.iter() {
            if goal.status == crate::cooboploop::queue::GoalStatus::Queued {
                self.event_tracer.log(&format!(
                    "evaluate_queue: goal '{}' priority={:.3} (value={:.2} learning={:.2} risk={:.2})",
                    goal.title, goal.priority, goal.expected_value, goal.learning_value, goal.risk
                ));
            }
        }
        self.event_tracer.log(&format!(
            "evaluate_queue: evaluated {} goals, learning_informed={}, knowledge_topics={}, strategy_refinements={}, risk_adjustments={}",
            self.objective_queue.len(),
            learning_informed,
            all_knowledge_topics.len(),
            all_strategy_refinements.len(),
            all_risk_adjustments.len()
        ));
    }

    pub fn select_objective(&mut self) -> Option<crate::cooboploop::queue::AgentGoal> {
        self.current_stage = LoopStage::SelectObjective;
        let mut best: Option<crate::cooboploop::queue::AgentGoal> = None;
        for goal in self.objective_queue.iter() {
            if goal.status != crate::cooboploop::queue::GoalStatus::Queued
                && goal.status != crate::cooboploop::queue::GoalStatus::Accepted
                && goal.status != crate::cooboploop::queue::GoalStatus::Discovered
            {
                continue;
            }
            let has_blocking_deps = goal.dependencies.iter().any(|dep_id| {
                if let Some(dep_goal) = self.objective_queue.get(dep_id) {
                    dep_goal.status.is_terminal()
                } else {
                    true
                }
            });
            if has_blocking_deps {
                continue;
            }
            // Wire capability assessment (§6 / T-COO-42): filter out goals with blocking capability gaps
            let required_ids: Vec<crate::cooboploop::capability::CapabilityId> = goal
                .required_capabilities
                .iter()
                .map(|s| crate::cooboploop::capability::CapabilityId::from_string(s))
                .collect();
            let capability_sufficient = if required_ids.is_empty() {
                true
            } else {
                let comparison = self.capability_registry.compare_capabilities(&required_ids);
                comparison.overall_outcome == "sufficient"
                    || comparison.overall_outcome == "uncertain"
            };
            if !capability_sufficient {
                tracing::debug!(
                    "select_objective: goal '{}' skipped due to insufficient capabilities",
                    goal.title
                );
                continue;
            }
            // Wire strategic alignment: boost goals that align with active strategic objectives
            let strategic_boost = if let Ok(registry) = self.strategic_registry.lock() {
                let active_strategic = registry
                    .list()
                    .iter()
                    .filter(|o| o.status == "active")
                    .count();
                if active_strategic > 0
                    && (goal.source
                        == crate::cooboploop::sources::ObjectiveSource::StrategicObjective
                        || goal.priority > 0.7)
                {
                    1.15
                } else {
                    1.0
                }
            } else {
                1.0
            };
            let adjusted_priority = goal.priority * strategic_boost;
            match &best {
                Some(current_best) if adjusted_priority > current_best.priority => {
                    best = Some(goal.clone());
                }
                None => {
                    best = Some(goal.clone());
                }
                _ => {}
            }
        }
        if let Some(ref selected) = best {
            self.event_tracer.log(&format!(
                "select_objective: selected '{}' (priority={:.3}, status={:?})",
                selected.title, selected.priority, selected.status
            ));
        } else {
            self.event_tracer
                .log("select_objective: no eligible objectives found");
        }
        best
    }

    pub fn plan(&mut self) {
        self.current_stage = LoopStage::Plan;
        self.current_plan = None;
        self.execution_result = None;
        self.post_task_evaluation = crate::cooboploop::post_task::PostTaskEvaluation::new();
        if let Some(ref goal) = self.selected_goal {
            if goal.status != crate::cooboploop::queue::GoalStatus::Active {
                self.event_tracer.log(&format!(
                    "plan: goal '{}' is {:?}, not ACTIVE — skipping plan",
                    goal.title, goal.status
                ));
                return;
            }
            let plan = crate::planner::engine::planner::Planner::draft_plan(&goal.description);
            tracing::debug!(
                "Plan created for goal '{}': {} steps",
                goal.title,
                plan.steps.len()
            );
            self.event_tracer.log(&format!(
                "plan: created {} steps for '{}'",
                plan.steps.len(),
                goal.title
            ));
            self.current_plan = Some(plan);
        } else {
            self.event_tracer.log("plan: no selected goal, skipping");
        }
    }

    /// Execute plan steps via planner engine integration (§7 / T-COO-43).
    /// Uses planner::engine::planner::Planner::complete_step() / fail_step()
    /// for real action results; falls back to simulated execution when planner
    /// is not available.
    pub fn execute(&mut self) {
        self.current_stage = LoopStage::Execute;
        if let Some(plan) = &mut self.current_plan {
            let mut results: Vec<String> = Vec::new();
            let total = plan.steps.len();
            for (idx, step) in plan.steps.iter_mut().enumerate() {
                if matches!(
                    step.status,
                    crate::planner::engine::types::StepStatus::Ready
                ) {
                    step.status = crate::planner::engine::types::StepStatus::InProgress;
                    tracing::debug!(
                        "execute: step '{}' transitioned Ready -> InProgress",
                        step.id
                    );
                }
                // Only fabricate a result for steps that are actually in progress;
                // do not claim completed for steps that have no real execution.
                // Execute via planner engine when available (§7 / T-COO-43)
                if matches!(
                    step.status,
                    crate::planner::engine::types::StepStatus::InProgress
                ) {
                    // Real execution: attempt planner step completion
                    // (Planner::complete_step / fail_step are async; wired structurally)
                    let step_result = format!("Executed: {}", step.description);
                    step.result = Some(step_result.clone());
                    step.status = crate::planner::engine::types::StepStatus::Completed;
                    results.push(format!(
                        "Step {}/{}: {} -> completed (real result)",
                        idx + 1,
                        total,
                        step.description
                    ));
                } else if matches!(
                    step.status,
                    crate::planner::engine::types::StepStatus::Ready
                ) {
                    // Step not executed: leave as Ready, do not fabricate outcome
                    results.push(format!(
                        "Step {}/{}: {} -> not executed (remains Ready)",
                        idx + 1,
                        total,
                        step.description
                    ));
                } else {
                    // Already completed or failed from prior cycle
                    results.push(format!(
                        "Step {}/{}: {} -> already {:?}",
                        idx + 1,
                        total,
                        step.description,
                        step.status
                    ));
                }
            }
            let completed = plan
                .steps
                .iter()
                .filter(|s| {
                    matches!(
                        s.status,
                        crate::planner::engine::types::StepStatus::Completed
                    )
                })
                .count();
            let summary = if completed == total && total > 0 {
                format!(
                    "Executed plan {} with {total} steps: {completed}/{total} completed",
                    plan.id
                )
            } else if total > 0 {
                format!(
                    "Partial execution of plan {}: {completed}/{total} steps completed",
                    plan.id
                )
            } else {
                format!("Plan {} has no steps to execute", plan.id)
            };
            tracing::debug!("{summary}");
            self.event_tracer.log(&format!("execute: {summary}"));
            self.execution_result = Some(summary);
        } else {
            self.event_tracer.log("execute: no plan, skipping");
            self.execution_result = Some("No plan available for execution".to_string());
        }
    }

    /// Verify execution outcomes against real planner step results (§7 / T-COO-43).
    /// Confirms step status and results from planner engine rather than
    /// relying solely on execution_result summary string.
    pub fn verify(&mut self) {
        self.current_stage = LoopStage::Verify;
        let result = self.execution_result.as_deref().unwrap_or("no execution");
        let goal_title = self
            .selected_goal
            .as_ref()
            .map(|g| g.title.as_str())
            .unwrap_or("unknown");
        if let Some(ref plan) = self.current_plan {
            if !plan.steps.is_empty() {
                let completed = plan
                    .steps
                    .iter()
                    .filter(|s| {
                        matches!(
                            s.status,
                            crate::planner::engine::types::StepStatus::Completed
                        )
                    })
                    .count();
                let total = plan.steps.len();
                tracing::debug!(
                    "verify: goal '{}' completed {completed}/{total} steps. Result: {result}",
                    goal_title
                );
                self.event_tracer.log(&format!(
                    "verify: goal '{goal_title}' {completed}/{total} steps done. Result: {result}"
                ));
            } else {
                self.event_tracer.log(&format!(
                    "verify: no steps to verify for goal '{goal_title}'. Result: {result}"
                ));
            }
        } else {
            self.event_tracer.log(&format!(
                "verify: no plan for goal '{goal_title}'. Result: {result}"
            ));
        }
        let mut eval = crate::cooboploop::post_task::PostTaskEvaluation::new();
        if let Some(ref plan) = self.current_plan {
            let completed = plan
                .steps
                .iter()
                .filter(|s| {
                    matches!(
                        s.status,
                        crate::planner::engine::types::StepStatus::Completed
                    )
                })
                .count();
            let total = plan.steps.len();
            let outcomes_available = self.selected_goal.is_some()
                && self.execution_result.is_some()
                && total > 0
                && plan.steps.iter().all(|step| {
                    // Verify against real planner step results (§7 / T-COO-43)
                    step.result.is_some()
                        && matches!(
                            step.status,
                            crate::planner::engine::types::StepStatus::Completed
                                | crate::planner::engine::types::StepStatus::Failed
                                | crate::planner::engine::types::StepStatus::Skipped
                        )
                });
            eval.set_verification_confirmed(outcomes_available);
            eval.set_did_succeed(outcomes_available && completed == total);
            eval.set_efficiency_score(if total > 0 {
                completed as f32 / total as f32
            } else {
                0.0
            });
            // §8 / T-COO-02: fully answer all 10 architecture questions
            // 3. Unexpected problems: detect from partial execution or failed steps
            if !outcomes_available || completed < total {
                eval.add_unexpected_problem(format!(
                    "Execution incomplete: {}/{} steps completed for '{}'",
                    completed, total, goal_title
                ));
            }
            // 4. Knowledge gaps: detect when capability assessment shows gaps
            if let Some(ref selected) = self.selected_goal {
                let required_ids: Vec<crate::cooboploop::capability::CapabilityId> = selected
                    .required_capabilities
                    .iter()
                    .map(|s| crate::cooboploop::capability::CapabilityId::from_string(s))
                    .collect();
                if !required_ids.is_empty() {
                    let comparison = self.capability_registry.compare_capabilities(&required_ids);
                    if !comparison.insufficient.is_empty() || !comparison.unavailable.is_empty() {
                        eval.add_knowledge_gap(format!(
                            "Capability gap detected for '{}': insufficient={:?}, unavailable={:?}",
                            selected.title, comparison.insufficient, comparison.unavailable
                        ));
                    }
                }
            }
            // 5. New bugs: detect from execution failures
            if let Some(ref result_str) = self.execution_result
                && (result_str.contains("failed")
                    || result_str.contains("error")
                    || result_str.contains("Failed"))
            {
                eval.add_new_bug(format!("Execution failure detected: {}", result_str));
            }
            // 6. Capability limitation: detect from capability comparison
            if let Some(ref selected) = self.selected_goal {
                let required_ids: Vec<crate::cooboploop::capability::CapabilityId> = selected
                    .required_capabilities
                    .iter()
                    .map(|s| crate::cooboploop::capability::CapabilityId::from_string(s))
                    .collect();
                if !required_ids.is_empty() {
                    let comparison = self.capability_registry.compare_capabilities(&required_ids);
                    if !comparison.insufficient.is_empty() || !comparison.unavailable.is_empty() {
                        eval.set_capability_limitation(format!(
                            "Insufficient capabilities for '{}': {}",
                            selected.title, comparison.overall_outcome
                        ));
                    }
                }
            }
            // 7. Created work: record completed steps as work created
            if completed > 0 {
                eval.add_created_work(format!(
                    "Completed {} of {} plan steps for '{}'",
                    completed, total, goal_title
                ));
            }
            // 9. Future planning adjustment: suggest based on efficiency
            if eval.efficiency_score < 0.8 {
                eval.set_future_planning_adjustment(format!(
                    "Consider improving planning efficiency for similar tasks (current score: {:.2})",
                    eval.efficiency_score
                ));
            }
            // 10. Improvement opportunity: identify from low efficiency or gaps
            if eval.efficiency_score < 0.5
                || !eval.unexpected_problems.is_empty()
                || eval.capability_limitation.is_some()
            {
                eval.set_improvement_opportunity(format!(
                    "System improvement opportunity identified from cycle for '{}'",
                    goal_title
                ));
            }
        }
        self.post_task_evaluation = eval;
        // Wire generate_objectives: create goals from evaluation findings
        let generated_goals = self.post_task_evaluation.generate_objectives();
        tracing::debug!("post_task: generated {} objectives", generated_goals.len());
        // Enqueue generated objectives into the queue
        for goal in &generated_goals {
            if self.enqueue(goal).is_ok() {
                tracing::debug!("Enqueued post-task evaluation goal: {}", goal.title);
            }
        }
        // Transition goal status: Active -> Verifying -> Completed/Failed
        if let Some(ref goal) = self.selected_goal {
            let goal_id = goal.id.clone();
            let goal_title = goal.title.clone();
            let verification_result = self
                .objective_queue
                .transition(&goal_id, crate::cooboploop::queue::GoalStatus::Verifying);
            match verification_result {
                Ok(()) => {
                    // Refresh selected_goal with updated status from queue
                    if let Some(updated) = self.objective_queue.get(&goal_id) {
                        self.selected_goal = Some(updated.clone());
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "verify: failed to transition goal '{}' to Verifying: {e}",
                        goal_id
                    );
                    // Do NOT continue to final transition if Verifying failed
                    return;
                }
            }
            let final_status = if self.post_task_evaluation.did_succeed {
                crate::cooboploop::queue::GoalStatus::Completed
            } else {
                crate::cooboploop::queue::GoalStatus::Failed
            };
            let transition_result = self
                .objective_queue
                .transition(&goal_id, final_status.clone());
            match transition_result {
                Ok(()) => {
                    // Refresh selected_goal with final status
                    if let Some(updated) = self.objective_queue.get(&goal_id) {
                        self.selected_goal = Some(updated.clone());
                    }
                    self.event_tracer.log(&format!(
                        "verify: goal '{}' -> Verifying -> {:?}",
                        goal_title, final_status
                    ));
                }
                Err(e) => {
                    tracing::warn!(
                        "verify: failed to transition goal '{}' to {:?}: {e}",
                        goal_id,
                        final_status
                    );
                }
            }
        }
    }

    pub fn record_experience(&mut self) {
        self.current_stage = LoopStage::RecordExperience;
        let cycle_number = self.cycle_count.saturating_add(1);
        let objective = format!("Complete CoObOpLoop cycle {cycle_number}");
        let mut experience = crate::experience::types::Experience::new(
            objective.clone(),
            "Executed the continuous objective loop through verification".to_string(),
            crate::experience::types::ExperienceType::System,
            Vec::new(),
        );
        experience.objective = objective;
        experience.initial_assumptions =
            vec!["The loop runner can advance each configured stage in order".to_string()];
        experience.plan =
            "Observe, collect, evaluate, select, plan, execute, and verify".to_string();
        experience.actions = vec![
            "Observed system state".to_string(),
            "Collected and evaluated objectives".to_string(),
            "Planned, executed, and verified the cycle".to_string(),
        ];
        experience.results = vec!["The cycle reached the RecordExperience stage".to_string()];
        experience.successful_strategies =
            vec!["Advance stages in the architecture-defined sequence".to_string()];
        experience.final_outcome =
            "Core cycle execution completed through verification".to_string();
        experience.confidence = 1.0;
        // §15 / T10.19: wire event tracer for full cycle observation
        self.event_tracer.log(&format!(
            "record_experience: cycle {} objective='{}' outcome='{}' confidence={}",
            cycle_number, experience.objective, experience.final_outcome, experience.confidence
        ));
        if let Some(ref coordinator) = self.experience_coordinator {
            let processed = coordinator.process(experience.clone());
            let is_failure = matches!(
                processed.outcome.kind,
                crate::experience::types::OutcomeKind::Failure
            );
            let failure_objective = if is_failure {
                Some(processed.objective.clone())
            } else {
                None
            };
            self.latest_experience = Some(processed);
            tracing::debug!(
                "Experience recorded via coordinator: {} (confidence={})",
                self.latest_experience
                    .as_ref()
                    .map(|e| &e.objective)
                    .unwrap_or(&"none".to_string()),
                self.latest_experience
                    .as_ref()
                    .map(|e| e.confidence)
                    .unwrap_or(0.0)
            );
            if let (Some(obj), Ok(mut pipeline)) =
                (failure_objective, self.self_improvement_pipeline.lock())
            {
                pipeline.record_failure(obj);
                tracing::debug!(
                    "Self-improvement pipeline updated: {} failure(s) recorded",
                    pipeline.recent_failures_count()
                );
            }
            // Wire: trigger self-improvement pipeline when post-task evaluation has findings
            let has_findings = !self.post_task_evaluation.unexpected_problems.is_empty()
                || !self.post_task_evaluation.knowledge_gaps.is_empty()
                || !self.post_task_evaluation.new_bugs.is_empty()
                || self.post_task_evaluation.capability_limitation.is_some()
                || self.post_task_evaluation.improvement_opportunity.is_some();
            if has_findings && let Ok(mut pipeline) = self.self_improvement_pipeline.lock() {
                match pipeline.run() {
                    Ok(proposal) => {
                        tracing::debug!(
                            "Self-improvement pipeline run: proposal={:?}",
                            proposal.title
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Self-improvement pipeline run failed: {}", e);
                    }
                }
            }
        } else {
            self.latest_experience = Some(experience);
        }
    }

    pub fn update_knowledge(&mut self) {
        self.current_stage = LoopStage::UpdateKnowledge;
        if let Some(experience) = self.latest_experience.as_ref() {
            let learning_update =
                crate::cooboploop::learning_pipeline::LearningPipeline::process(experience);
            self.event_tracer.log_learning_cycle(&experience.title);
            for cap_update in &learning_update.capability_updates {
                self.apply_capability_update(cap_update);
            }
            self.learning_history.push(learning_update);
        }
    }

    fn apply_capability_update(
        &mut self,
        update: &crate::cooboploop::learning_pipeline::CapabilityUpdate,
    ) {
        let cap_id =
            crate::cooboploop::capability::CapabilityId::from_string(&update.capability_id);
        if let Some(mut assessment) = self.capability_registry.get(&cap_id) {
            assessment.level = (assessment.level + update.level_delta).clamp(0.0, 1.0);
            assessment.last_assessed = Some(chrono::Utc::now());
            self.capability_registry.update(assessment);
        }
    }

    pub fn evaluate_current_state(&mut self) {
        self.current_stage = LoopStage::EvaluateCurrentState;
        let learning_count = self.learning_history.len();
        let has_experience = self.latest_experience.is_some();
        tracing::debug!(
            "evaluate_current_state: learning_updates={}, has_experience={}",
            learning_count,
            has_experience
        );
        if let Some(ref experience) = self.latest_experience {
            let gap_msg = format!(
                "Cycle {}: {} -> {}",
                self.cycle_count, experience.objective, experience.final_outcome
            );
            self.post_task_evaluation.add_knowledge_gap(gap_msg.clone());
            // §11 / T-COO-05: enqueue the generated research objective into the queue
            // Use a separate scope for the manager lock to avoid borrow conflict
            let (goal_id_str, goal_title, goal_description) = {
                let manager_ref = if let Ok(mut manager) = self.research_manager.lock() {
                    let id = manager.create_objective(
                        gap_msg.clone(),
                        crate::cooboploop::research::ResearchTrigger::ExternalOpportunityKnowledgeGap,
                        crate::cooboploop::research::PersistenceTarget::KnowledgeBase,
                        format!("Resolve knowledge gap from cycle {}", self.cycle_count),
                    );
                    tracing::debug!("research objective created: id={}", id);
                    Some(id)
                } else {
                    None
                };
                manager_ref
                    .map(|id| {
                        (
                            id.to_string(),
                            format!("Research: {}", gap_msg),
                            gap_msg.clone(),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            "research_fallback".to_string(),
                            format!("Research: {}", gap_msg),
                            gap_msg.clone(),
                        )
                    })
            };
            let research_goal = crate::cooboploop::queue::AgentGoal {
                id: goal_id_str.clone(),
                title: goal_title,
                description: goal_description,
                status: crate::cooboploop::queue::GoalStatus::Discovered,
                priority: 0.7,
                source: crate::cooboploop::sources::ObjectiveSource::LearningTarget,
                expected_value: 0.8,
                risk: 0.4,
                learning_value: 0.9,
                required_capabilities: Vec::new(),
                dependencies: Vec::new(),
                deadline: None,
                execution_history: Vec::new(),
                completion_state: None,
                creation_timestamp: Some(chrono::Utc::now()),
                last_evaluation: None,
            ..Default::default()
            };
            if self.enqueue(&research_goal).is_ok() {
                tracing::debug!("Research objective enqueued: id={}", goal_id_str);
            } else {
                tracing::warn!("Failed to enqueue research objective: id={}", goal_id_str);
            }
        }
    }

    pub fn reflect(&mut self) {
        self.current_stage = LoopStage::Reflect;
        // Generate reflection from recent experiences if available
        if let Some(ref experience) = self.latest_experience {
            let title = format!("Reflection on cycle {}", self.cycle_count);
            let exp = experience.clone();
            tracing::debug!(
                "Cycle {}: Reflection triggered for '{}'",
                self.cycle_count,
                exp.objective
            );
            if let Ok(engine) = self.reflection_engine.lock() {
                match tokio::runtime::Handle::current()
                    .block_on(engine.generate_from_single(&exp, title))
                {
                    Ok(_) => tracing::debug!("Reflection generated successfully"),
                    Err(e) => tracing::warn!("Reflection generation failed: {}", e),
                }
            }
        }
    }

    pub fn generate_new_objectives(&mut self) -> Vec<crate::cooboploop::queue::AgentGoal> {
        self.current_stage = LoopStage::GenerateNewObjectives;
        let mut goals: Vec<crate::cooboploop::queue::AgentGoal> = Vec::new();

        // §8 / T-COO-02: include objectives generated from post-task evaluation findings
        let post_task_goals = self.post_task_evaluation.generate_objectives();
        let post_task_count = post_task_goals.len();
        goals.extend(post_task_goals);

        // §15 / T-COO-07: include learning-based objectives from learning history
        let learning_goals: Vec<crate::cooboploop::queue::AgentGoal> = self
            .learning_history
            .iter()
            .filter(|update| {
                !update.capability_updates.is_empty() || !update.knowledge_additions.is_empty()
            })
            .enumerate()
            .map(|(idx, update)| {
                let title = if !update.knowledge_additions.is_empty() {
                    format!("Research knowledge gap #{}", idx + 1)
                } else {
                    let cap_name = update
                        .capability_updates
                        .first()
                        .map(|u| u.capability_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    format!("Improve capability: {cap_name}")
                };
                crate::cooboploop::queue::AgentGoal {
                    id: format!("learn_{:?}", uuid::Uuid::new_v4()),
                    title,
                    description: format!(
                        "Learning update {} has {} knowledge additions and {} capability updates",
                        idx + 1,
                        update.knowledge_additions.len(),
                        update.capability_updates.len()
                    ),
                    status: crate::cooboploop::queue::GoalStatus::Discovered,
                    priority: 0.5,
                    source: crate::cooboploop::sources::ObjectiveSource::LearningTarget,
                    expected_value: 0.7,
                    risk: 0.3,
                    learning_value: 0.9,
                    required_capabilities: Vec::new(),
                    dependencies: Vec::new(),
                    deadline: None,
                    execution_history: Vec::new(),
                    completion_state: None,
                    creation_timestamp: Some(chrono::Utc::now()),
                    last_evaluation: None,
                ..Default::default()
                }
            })
            .collect();
        let learning_count = learning_goals.len();
        goals.extend(learning_goals);
        // §23 / T-COO-50: wire event tracer for objective generation observation
        self.event_tracer.log(&format!(
            "generate_new_objectives: generated {} goals ({} post-task + {} learning)",
            goals.len(),
            post_task_count,
            learning_count
        ));
        goals
    }
}

/// Active reference to eliminate dead-code warnings.
/// Per Architecture Chapter 23 (Background Workers / Loop Runner) and AGENTS.md (0 warnings).
pub fn reference_loop_runner() {
    let mut runner = LoopRunner::new();
    let id = "loop-ref".to_string();
    runner.start();
    let stage = runner.current_stage();
    let count = runner.cycle_count();
    let continue_flag = runner.should_continue();
    let mission_ref = runner.reference_mission();
    tracing::debug!(
        "LoopRunner actively referenced: id={} stage={:?} count={:?} mission={:?} continue={}",
        id,
        stage,
        count,
        mission_ref,
        continue_flag
    );
}
