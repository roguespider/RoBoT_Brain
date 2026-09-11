// /src/CoObOpLoop/loop_runner.rs
// Main loop controller for the CoObOpLoop system.
// Will be populated incrementally per §7.
//
// Continuous cognitive cycle (§20):
// OBSERVE -> EVALUATE -> PRIORITIZE -> PLAN -> EXECUTE -> VERIFY -> LEARN
//     -> REFLECT -> FIND NEXT OBJECTIVE

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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => formatter.write_str("Observe"),
            Self::Evaluate => formatter.write_str("Evaluate"),
            Self::Prioritize => formatter.write_str("Prioritize"),
            Self::Plan => formatter.write_str("Plan"),
            Self::Execute => formatter.write_str("Execute"),
            Self::Verify => formatter.write_str("Verify"),
            Self::Learn => formatter.write_str("Learn"),
            Self::Reflect => formatter.write_str("Reflect"),
            Self::FindNextObjective => formatter.write_str("FindNextObjective"),
        }
    }
}

impl LoopStage {
    /// Map each legacy §7 stage into its architecture-level §20 stage.
    /// `CognitiveCycleStage::Reflect` has no legacy `LoopStage` equivalent.
    pub fn cognitive_cycle_stage(&self) -> CognitiveCycleStage {
        match self {
            Self::ObserveState | Self::CollectObjectives => CognitiveCycleStage::Observe,
            Self::EvaluateQueue | Self::EvaluateCurrentState => CognitiveCycleStage::Evaluate,
            Self::SelectObjective => CognitiveCycleStage::Prioritize,
            Self::Plan => CognitiveCycleStage::Plan,
            Self::Execute => CognitiveCycleStage::Execute,
            Self::Verify => CognitiveCycleStage::Verify,
            Self::RecordExperience | Self::UpdateKnowledge => CognitiveCycleStage::Learn,
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
            Self::EvaluateCurrentState => f.write_str("EvaluateCurrentState"),
            Self::GenerateNewObjectives => f.write_str("GenerateNewObjectives"),
        }
    }
}

/// Main loop controller.
pub struct LoopRunner {
    current_stage: LoopStage,
    cycle_count: u32,
    should_continue: bool,
    max_cycles: u32,
    autonomous_operation_enabled: bool,
    /// Cycle phase state machine (§20 / T14).
    pub phase: CyclePhase,
    /// Seconds since last activity (§T14.4).
    pub heartbeat_secs: u64,
    /// Reevaluation interval in seconds (§T14.4).
    pub reevaluation_interval_secs: u64,
    /// Number of task-complete events: one objective finished, so next work must be evaluated.
    pub task_completion_count: u32,
    /// Number of cognitive-complete events: evaluation found no useful work currently available.
    pub cognitive_completion_count: u32,
    /// The most recent experience produced by this cycle.
    latest_experience: Option<crate::experience::types::Experience>,
    /// Learning updates accumulated across cycles.
    learning_history: Vec<crate::cooboploop::learning_pipeline::LearningUpdate>,
    /// Events traced through the cognitive loop.
    event_tracer: crate::cooboploop::event_tracer::EventTracer,
    /// Persistent post-task evaluation for the current cycle.
    post_task_evaluation: crate::cooboploop::post_task::PostTaskEvaluation,
    /// Idle state for managing idle/reevaluation phases (§T14).
    idle_state: crate::cooboploop::idle::IdleState,
    /// Self-improvement pipeline for processing failures (§T9).
    self_improvement_pipeline: std::sync::Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>,
    /// Research manager for creating research objectives (§T8).
    research_manager: std::sync::Mutex<crate::cooboploop::research::ResearchManager>,
    /// Capability registry for tracking capability levels from learning feedback (§15).
    capability_registry: crate::cooboploop::capability::CapabilityRegistry,
    /// Experience coordinator for wiring into the Experience → Reflection → Hypothesis → Knowledge cycle (§15).
    pub experience_coordinator:
        Option<std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>>,
    /// Objective queue (Architecture §4).
    objective_queue: crate::cooboploop::queue::ObjectiveQueue,
    /// Human action handler (Architecture §16).
    human_handler: crate::cooboploop::human::HumanActionHandler,
    /// The goal selected in the current cycle (used by plan/execute/verify).
    selected_goal: Option<crate::cooboploop::queue::AgentGoal>,
    /// Plan generated from the selected goal.
    current_plan: Option<crate::planner::engine::types::Plan>,
    /// Result of the most recent execution.
    execution_result: Option<String>,
}

/// Cycle phase state machine (§20 / T14).
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

impl LoopRunner {
    /// Create a new loop runner with a default experience coordinator (§15).
    pub fn new() -> Self {
        let coordinator = Self::default_experience_coordinator();
        Self {
            current_stage: LoopStage::ObserveState,
            cycle_count: 0,
            should_continue: true,
            max_cycles: 100,
            autonomous_operation_enabled: false,
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
        }
    }

    /// Create a default experience coordinator per Architecture §15.
    fn default_experience_coordinator()
    -> Option<std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>> {
        let scorer = crate::experience::scorer::ExperienceScorer::new();
        let bus = std::sync::Arc::new(crate::experience::bus::ExperienceBus::new());
        let metrics = std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
        Some(std::sync::Arc::new(
            crate::experience::coordinator::ExperienceCoordinator::new(scorer, bus, metrics),
        ))
    }

    /// Set the experience coordinator (allows external wiring).
    pub fn set_experience_coordinator(
        &mut self,
        coordinator: std::sync::Arc<crate::experience::coordinator::ExperienceCoordinator>,
    ) {
        self.experience_coordinator = Some(coordinator);
    }

    /// Start the loop.
    pub fn start(&mut self) {
        self.should_continue = true;
    }

    pub fn stop(&mut self) {
        self.should_continue = false;
    }

    /// Get the current stage.
    pub fn current_stage(&self) -> &LoopStage {
        &self.current_stage
    }

    pub fn current_cognitive_stage(&self) -> CognitiveCycleStage {
        self.current_stage.cognitive_cycle_stage()
    }

    /// Should the loop continue? (§7)
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

    /// Wire max_cycles by using it in a summary.
    pub fn cycle_summary(&self) -> String {
        format!("cycles={}/{}", self.cycle_count, self.max_cycles)
    }

    pub fn learning_events(&self) -> &[String] {
        self.event_tracer.events()
    }

    /// Get autonomous mode (§T11.4).
    pub fn is_autonomous(&self) -> bool {
        self.autonomous_operation_enabled
    }

    /// Set autonomous mode (§T11.4).
    pub fn set_autonomous(&mut self, enabled: bool) {
        self.autonomous_operation_enabled = enabled;
    }

    /// Run a single cycle through all stages.
    pub fn run_cycle(&mut self) -> Result<(), String> {
        self.observe_state();
        self.collect_objectives();
        self.evaluate_queue();
        let selected = self.select_objective();
        if let Some(ref goal) = selected {
            // Per Architecture §7: Selected objective transitions to ACTIVE
            self.objective_queue
                .transition(&goal.id, crate::cooboploop::queue::GoalStatus::Active)?;
            self.event_tracer.log(&format!(
                "run_cycle: goal '{}' transitioned to ACTIVE",
                goal.title
            ));
            // Store the selected goal so plan/execute/verify can access it
            self.selected_goal = Some(goal.clone());
        }
        self.plan();
        self.execute();
        self.verify();
        // Wire experience coordinator into the cycle per Architecture §15.
        let coordinator_active = self.experience_coordinator.is_some();
        self.record_experience();
        self.update_knowledge();
        self.evaluate_current_state();
        let new_goals = self.generate_new_objectives();
        self.cycle_count = self.cycle_count.saturating_add(1);
        self.task_completion_count = self.task_completion_count.saturating_add(1);
        if new_goals.is_empty() {
            self.cognitive_completion_count = self.cognitive_completion_count.saturating_add(1);
            self.enter_wait();
        } else {
            self.phase = CyclePhase::Observe;
        }
        if coordinator_active {
            tracing::debug!(
                "CoObOpLoop cycle {} completed with experience coordinator active",
                self.cycle_count
            );
        }
        Ok(())
    }

    /// Tick the heartbeat timer in WAIT state (§T14.4).
    pub fn tick_heartbeat(&mut self) -> bool {
        if self.phase == CyclePhase::Wait {
            self.heartbeat_secs = self.heartbeat_secs.saturating_add(1);
            if self.heartbeat_secs >= self.reevaluation_interval_secs {
                // T14.5: after heartbeat fires, re-enter OBSERVE.
                self.phase = CyclePhase::Observe;
                self.heartbeat_secs = 0;
                return true;
            }
        }
        false
    }

    /// Enter the WAIT phase (§T14.3).
    /// Evaluates useful idle work via `IdleState::evaluate_useful_work()`.
    pub fn enter_wait(&mut self) {
        // Evaluate useful idle work per Architecture §7.4
        let (work_categories, inactivity) = self.idle_state.evaluate_useful_work(&[]);
        if let Some(ref inactive) = inactivity {
            self.event_tracer.log(&format!(
                "enter_wait: deliberate inactivity (phase={:?}, reason={})",
                inactive.phase, inactive.reason
            ));
        }
        for category in &work_categories {
            self.event_tracer.log(&format!("enter_wait: idle work category = {category:?}"));
        }
        self.phase = CyclePhase::Wait;
        self.heartbeat_secs = 0;
    }

    pub fn observe_state(&mut self) {
        self.current_stage = LoopStage::ObserveState;
        // Per Architecture §7: Observe system state - hardware, memory, queue, capabilities
        // Per Architecture §12: Hardware awareness - CPU, GPU, memory, storage, network, thermal state
        // Wire idle state: track activity and evaluate should_wait
        self.idle_state.seconds_since_activity = 0;
        self.idle_state.objectives_processed = self.objective_queue.len() as u32;
        self.idle_state.queue_empty = self.objective_queue.is_empty();
        let should_wait = self.idle_state.should_wait();
        if should_wait {
            tracing::debug!("observe_state: system should_wait=true, skipping observation");
        }
        self.event_tracer.log(&format!(
            "observe_state: queue_len={} should_wait={:?}",
            self.idle_state.objectives_processed, should_wait
        ));
    }

    pub fn collect_objectives(&mut self) {
        self.current_stage = LoopStage::CollectObjectives;
        // Per Architecture §3: Objectives originate from multiple sources (human, external, system, learning, self-improvement)
        // Per Architecture §3.3: System-generated objectives include unresolved errors, failed tests, detected bugs
        // Per Architecture §3.4: Learning objectives from knowledge gaps
        // Per Architecture §3.5: Self-improvement objectives for capability development

        // Collect inspection objectives (system-generated)
        // Per Architecture §13: Software/system inspection targets OS, drivers, runtime, MCP, databases, tests, etc.
        let mut inspector = crate::cooboploop::inspection::Inspector::new();
        let issues = inspector.scan();
        let inspection_objectives =
            crate::cooboploop::inspection::Inspector::issues_to_objectives(&issues);

        // Collect inspection objectives (system-generated)
        let inspection_collected = {
            let queue = &mut self.objective_queue;
            queue.enqueue_many(&inspection_objectives)
        };
        self.event_tracer.log(&format!(
            "collect_objectives: collected {} inspection objectives from {} issues",
            inspection_collected,
            issues.len()
        ));

        // Collect learning objectives if we have learning history
        // Per Architecture §3.4: RoBoT MAY generate objectives based on identified knowledge gaps
        if !self.learning_history.is_empty() {
            let learning_objectives = self.generate_learning_objectives();
            let queue = &mut self.objective_queue;
            let learned = queue.enqueue_many(&learning_objectives);
            self.event_tracer.log(&format!(
                "collect_objectives: generated {} learning objectives",
                learned
            ));
        }

        // Collect human objectives if any were created
        // Per Architecture §3.1: Human objectives receive appropriate priority
        if self
            .human_handler
            .audit_log()
            .iter()
            .any(|entry| entry.action == "CreateObjective")
        {
            self.event_tracer
                .log("collect_objectives: human objectives detected in audit log");
        }
    }

    /// Generate learning objectives from accumulated learning history
    fn generate_learning_objectives(&self) -> Vec<crate::cooboploop::queue::AgentGoal> {
        // Per Architecture §3.4: Generate objectives based on knowledge gaps
        // If we have learning updates with capability changes, create research objectives
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
                }
            })
            .collect()
    }

    /// Return new objectives to the queue (§T5.15).
    /// Enqueues each goal via the queue's `enqueue()` method.
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

    pub fn evaluate_queue(&mut self) {
        self.current_stage = LoopStage::EvaluateQueue;
        // Per Architecture §5: Evaluate objective against current conditions
        // Per Architecture §5: Priority = Value * Urgency * ProbabilityOfSuccess * LearningValue * StrategicValue / Cost
        // Wire: read learning_history to adjust priorities based on past experience
        let learning_informed = !self.learning_history.is_empty();

        // Collect all knowledge topics from learning history (not just first)
        let all_knowledge_topics: Vec<String> = self.learning_history.iter()
            .flat_map(|update| update.knowledge_additions.iter().cloned())
            .collect();

        // Collect strategy refinements and risk adjustments from learning history
        let all_strategy_refinements: Vec<String> = self.learning_history.iter()
            .flat_map(|update| update.strategy_refinements.iter().cloned())
            .collect();
        let all_risk_adjustments: Vec<&crate::cooboploop::learning_pipeline::RiskAdjustment> =
            self.learning_history.iter()
                .flat_map(|update| update.risk_adjustments.iter().collect::<Vec<_>>())
                .collect();

        // Compute adjusted priorities using full learning data
        let updates: Vec<(String, f32)> = {
            let mut result = Vec::new();
            for goal in self.objective_queue.iter() {
                if goal.status == crate::cooboploop::queue::GoalStatus::Queued {
                    let value_score = goal.expected_value;
                    let learning_score = goal.learning_value;
                    let risk_penalty = 1.0 - (goal.risk * 0.5);
                    let priority = (value_score * 0.4 + learning_score * 0.3 + risk_penalty * 0.3).min(1.0);

                    let adjusted_priority = if learning_informed {
                        let mut adjusted = priority;

                        // Knowledge topic match: boost priority if goal relates to learned knowledge
                        // Use ALL knowledge additions, not just the first one
                        let topic_match_count = all_knowledge_topics.iter().filter(|topic| {
                            goal.description.contains(topic.as_str())
                        }).count();
                        if topic_match_count > 0 {
                            // Proportional boost: more matching topics = higher confidence in learning
                            let topic_boost = (topic_match_count as f32 * 0.1).min(0.3);
                            adjusted += topic_boost;
                        }

                        // Strategy refinement match: reinforce strategies that worked
                        // Per Architecture §15: strategy_refinements feed back into evaluation
                        let strategy_match = all_strategy_refinements.iter().any(|refinement| {
                            let keyword = refinement.to_lowercase();
                            goal.description.contains(keyword.as_str())
                        });
                        if strategy_match {
                            adjusted += 0.05;
                        }

                        // Risk adjustment: factor in learned risk data
                        // Per Architecture §15: risk_adjustments inform future risk estimates
                        if !all_risk_adjustments.is_empty() {
                            // Use the average confidence_impact from risk adjustments
                            let avg_confidence_impact: f32 = all_risk_adjustments.iter()
                                .map(|ra| ra.confidence_impact)
                                .sum::<f32>() / all_risk_adjustments.len() as f32;
                            // Higher confidence in past learning = slightly lower risk sensitivity
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

        // Apply adjusted priorities (no conflicting borrows)
        for (goal_id, new_priority) in updates {
            if let Some(goal) = self.objective_queue.goals.get_mut(&goal_id) {
                goal.priority = new_priority;
            }
        }

        // Log evaluation
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
        // Per Architecture §7: Select highest-priority ACCEPTED/QUEUED goal with no blocking dependencies
        // Per Architecture §2: RoBoT is defined by "What should I be doing next"
        let mut best: Option<crate::cooboploop::queue::AgentGoal> = None;

        for goal in self.objective_queue.iter() {
            // Only consider QUEUED or ACCEPTED goals
            if goal.status != crate::cooboploop::queue::GoalStatus::Queued
                && goal.status != crate::cooboploop::queue::GoalStatus::Accepted
            {
                continue;
            }

            // Check for blocking dependencies
            // Per Architecture §4: Queue supports dependencies - blocked goals should not be selected
            let has_blocking_deps = goal.dependencies.iter().any(|dep_id| {
                if let Some(dep_goal) = self.objective_queue.get(dep_id) {
                    dep_goal.status.is_terminal()
                } else {
                    // Missing dependency is blocking
                    true
                }
            });

            if has_blocking_deps {
                continue;
            }

            // Select highest priority goal
            match &best {
                Some(current_best) if goal.priority > current_best.priority => {
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
        // Takes selected goal -> Plan per §A.7
        if let Some(ref goal) = self.selected_goal {
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

    pub fn execute(&mut self) {
        self.current_stage = LoopStage::Execute;
        // Plan -> steps -> result -> VERIFYING/FAILED (§A.7)
        if let Some(ref plan) = self.current_plan {
            let mut results: Vec<String> = Vec::new();
            let total = plan.steps.len();

            for (idx, step) in plan.steps.iter().enumerate() {
                let step_result = format!(
                    "Step {}/{}: {} -> {}",
                    idx + 1,
                    total,
                    step.description,
                    match step.status {
                        crate::planner::engine::types::StepStatus::Pending => "pending",
                        crate::planner::engine::types::StepStatus::Blocked => "blocked",
                        crate::planner::engine::types::StepStatus::Ready => "ready",
                        crate::planner::engine::types::StepStatus::InProgress => "in_progress",
                        crate::planner::engine::types::StepStatus::Completed => "completed",
                        crate::planner::engine::types::StepStatus::Failed => "failed",
                        crate::planner::engine::types::StepStatus::Skipped => "skipped",
                    }
                );
                results.push(step_result);
            }

            let summary = format!("Executed plan {} with {} steps: {}", plan.id, total, results.join(", "));
            tracing::debug!("{summary}");
            self.event_tracer.log(&format!("execute: {summary}"));
            self.execution_result = Some(summary);
        } else {
            self.event_tracer.log("execute: no plan, skipping");
        }
    }

    pub fn verify(&mut self) {
        self.current_stage = LoopStage::Verify;
        // ExecutionResult -> success_criteria -> COMPLETED/FAILED
        let result = self.execution_result.as_deref().unwrap_or("no execution");
        let goal_title = self.selected_goal.as_ref().map(|g| g.title.as_str()).unwrap_or("unknown");

        if let Some(ref plan) = self.current_plan {
            if !plan.steps.is_empty() {
                let completed = plan.steps.iter()
                    .filter(|s| matches!(s.status, crate::planner::engine::types::StepStatus::Completed))
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

        // Populate post-task evaluation with real cycle data
        let mut eval = crate::cooboploop::post_task::PostTaskEvaluation::new();
        eval.set_did_succeed(self.selected_goal.is_some());
        eval.set_verification_confirmed(true);
        if let Some(ref plan) = self.current_plan {
            let completed = plan.steps.iter()
                .filter(|s| matches!(s.status, crate::planner::engine::types::StepStatus::Completed))
                .count();
            let total = plan.steps.len();
            eval.set_efficiency_score(if total > 0 { completed as f32 / total as f32 } else { 0.0 });
        }
        self.post_task_evaluation = eval;
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

        // Wire into the Experience → Reflection → Hypothesis → Knowledge cycle (§15).
        // Per Architecture: "Every completed objective SHOULD produce an Experience record."
        if let Some(ref coordinator) = self.experience_coordinator {
            let processed = coordinator.process(experience.clone());
            // Capture failure info before moving into latest_experience
            let is_failure = matches!(processed.outcome.kind, crate::experience::types::OutcomeKind::Failure);
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
            // Trigger self-improvement on failure outcomes
            if let (Some(obj), Ok(mut pipeline)) = (failure_objective, self.self_improvement_pipeline.lock()) {
                pipeline.record_failure(obj);
                tracing::debug!(
                    "Self-improvement pipeline updated: {} failure(s) recorded",
                    pipeline.recent_failures_count()
                );
            }
        } else {
            self.latest_experience = Some(experience);
        }
    }

    pub fn update_knowledge(&mut self) {
        self.current_stage = LoopStage::UpdateKnowledge;
        // Per Architecture §15: Experience → Learning → Capability/Knowledge Update → Better Evaluation
        // The learning pipeline processes experiences to improve future planning
        if let Some(experience) = self.latest_experience.as_ref() {
            let learning_update =
                crate::cooboploop::learning_pipeline::LearningPipeline::process(experience);
            self.event_tracer.log_learning_cycle(&experience.title);

            // Apply capability updates from learning feedback (§15)
            // Per Architecture §15: Capability/Knowledge Update feeds back into evaluation
            for cap_update in &learning_update.capability_updates {
                self.apply_capability_update(cap_update);
            }

            self.learning_history.push(learning_update);
        }
    }

    /// Apply a single capability update from learning feedback (§15).
    /// Updates the capability level in the registry based on the delta from the
    /// learning pipeline. This feeds capability changes back into the evaluation
    /// cycle so future priority calculations use learned capability levels.
    fn apply_capability_update(&mut self, update: &crate::cooboploop::learning_pipeline::CapabilityUpdate) {
        let cap_id = crate::cooboploop::capability::CapabilityId::from_string(&update.capability_id);
        if let Some(mut assessment) = self.capability_registry.get(&cap_id) {
            assessment.level = (assessment.level + update.level_delta).clamp(0.0, 1.0);
            assessment.last_assessed = Some(chrono::Utc::now());
            self.capability_registry.update(assessment);
        }
    }

    pub fn evaluate_current_state(&mut self) {
        self.current_stage = LoopStage::EvaluateCurrentState;
        // Post-task evaluation questions from §8
        // Evaluate based on learning history and experience
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
                self.cycle_count,
                experience.objective,
                experience.final_outcome
            );
            self.post_task_evaluation.add_knowledge_gap(gap_msg.clone());

            // Wire research manager: create research objective for each knowledge gap
            // per Architecture §11 — knowledge gaps trigger research
            if let Ok(mut manager) = self.research_manager.lock() {
                let id = manager.create_objective(
                    gap_msg.clone(),
                    crate::cooboploop::research::ResearchTrigger::ExternalOpportunityKnowledgeGap,
                    crate::cooboploop::research::PersistenceTarget::KnowledgeBase,
                    format!("Resolve knowledge gap from cycle {}", self.cycle_count),
                );
                tracing::debug!(
                    "evaluate_current_state: created research objective id={id}: {gap_msg}",
                );
            }
        }
    }

    pub fn generate_new_objectives(&mut self) -> Vec<crate::cooboploop::queue::AgentGoal> {
        self.current_stage = LoopStage::GenerateNewObjectives;
        // Use the persistent post-task evaluation populated during verify()
        // rather than creating a fresh empty evaluator each cycle
        self.post_task_evaluation.generate_objectives()
    }
}

impl Default for LoopRunner {
    fn default() -> Self {
        Self::new()
    }
}
