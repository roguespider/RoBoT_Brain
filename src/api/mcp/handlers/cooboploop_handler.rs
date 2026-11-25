use std::sync::{Arc, Mutex};

use crate::api::bridge::tools::ToolOutput;
use crate::api::mcp::handlers::{HandlerError, ToolHandler};
use crate::scheduler::cooboploop::evaluation::PriorityPolicyRegistry;
use crate::scheduler::cooboploop::queue::ObjectiveQueue;

#[derive(Clone)]
pub struct CooboploopToolsHandler {
    queue: Arc<Mutex<ObjectiveQueue>>,
    policy_registry: Arc<Mutex<PriorityPolicyRegistry>>,
    capability_registry: Arc<Mutex<crate::scheduler::cooboploop::capability::CapabilityRegistry>>,
    loop_runner: Arc<Mutex<crate::scheduler::cooboploop::loop_runner_engine::LoopRunner>>,
    research_manager: Arc<Mutex<crate::scheduler::cooboploop::research::ResearchManager>>,
    hardware_discovery: Arc<Mutex<crate::scheduler::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: Arc<Mutex<crate::scheduler::cooboploop::hardware::HardwareRegistry>>,
    self_improvement_pipeline:
        Arc<Mutex<crate::scheduler::cooboploop::self_improvement::SelfImprovementPipeline>>,
    pending_opportunities: Arc<Mutex<Vec<crate::scheduler::cooboploop::opportunity::IntakeResult>>>,
    strategic_registry:
        Arc<Mutex<crate::scheduler::cooboploop::strategic::StrategicObjectiveRegistry>>,
    mission: Arc<Mutex<String>>,
    runtime_adapters: Arc<crate::scheduler::cooboploop::llm_provider::DefaultRuntimeAdapters>,
}

impl CooboploopToolsHandler {
    pub fn new(
        context: Arc<crate::api::mcp::McpContext>,
    ) -> Result<Self, crate::api::mcp::handlers::HandlerInitError> {
        let database_path = context.database.path().to_string_lossy().to_string();
        let objective_queue = ObjectiveQueue::open(&database_path).map_err(|error| {
            crate::api::mcp::handlers::HandlerInitError::new("cooboploop", &error)
        })?;
        let hardware_registry =
            crate::scheduler::cooboploop::hardware::HardwareRegistry::open(&database_path)
                .map_err(|error| {
                    crate::api::mcp::handlers::HandlerInitError::new("cooboploop", &error)
                })?;
        let strategic_registry =
            crate::scheduler::cooboploop::strategic::StrategicObjectiveRegistry::open(
                &database_path,
            )
            .map_err(|error| {
                crate::api::mcp::handlers::HandlerInitError::new("cooboploop", &error)
            })?;
        let runtime_adapters = Arc::new(
            crate::scheduler::cooboploop::llm_provider::DefaultRuntimeAdapters::new(
                context.database.as_ref().clone(),
            ),
        );
        let capability_registry = crate::scheduler::cooboploop::init(&database_path);
        let capability_registry_arc = Arc::new(Mutex::new(capability_registry));
        // Wire cooboploop tool registration
        let mut reg = crate::api::bridge::tools::ToolRegistry::new();
        crate::api::bridge::tools::cooboploop::register_cooboploop_tools(&mut reg);
        let mut policy_reg = PriorityPolicyRegistry::default();
        // Wire current_policy / set_policy by reading and setting
        let current_policy = policy_reg.current_policy();
        let current_display = format!("{:?}", current_policy);
        policy_reg.set_policy(Box::new(
            crate::scheduler::cooboploop::evaluation::ConservativePriorityPolicy,
        ));
        let updated_policy = policy_reg.current_policy();
        let updated_display = format!("{:?}", updated_policy);
        tracing::debug!(
            "Policy wired: current={}, updated={}",
            current_display,
            updated_display
        );
        // Create the loop runner with its default experience coordinator
        let mut loop_runner = crate::scheduler::cooboploop::loop_runner_engine::LoopRunner::new();
        // Wire the persistent objective queue from the DB into the loop runner
        // so the loop and the handler share the same durable queue.
        if let Ok(queue_from_db) =
            crate::scheduler::cooboploop::queue::ObjectiveQueue::open(&database_path)
        {
            loop_runner.set_objective_queue(queue_from_db);
        }
        // Wire the experience coordinator into the loop runner per Architecture §15.
        if let Some(coordinator) = &loop_runner.experience_coordinator {
            let coordinator_ref = std::sync::Arc::clone(coordinator);
            loop_runner.set_experience_coordinator(coordinator_ref);
        }
        // Wire the mission into the loop runner (§19 / T-COO-12)
        loop_runner.set_mission("Mission".to_string());
        let mission_ref = loop_runner.mission();
        tracing::debug!(mission = %mission_ref, "Mission wired into loop runner");
        // Wire the skill executor for autonomous skill refinement (§32)
        let skill_executor = std::sync::Arc::clone(&context.skill_executor);
        loop_runner.set_skill_executor(skill_executor);
        tracing::debug!("Skill executor wired into loop runner for autonomous skill refinement");
        // Wire the hardware registry for DB-backed profile persistence (§12 / §22).
        let hardware_registry_arc = Arc::new(Mutex::new(hardware_registry));
        loop_runner.set_hardware_registry(Arc::clone(&hardware_registry_arc));
        tracing::debug!("Hardware registry wired into loop runner for DB persistence");
        // Wire the memory repository into research manager (§11 / Architecture).
        let memory_repo: Arc<dyn crate::memory::repository::MemoryRepository> = Arc::new(
            crate::memory::repository::SqliteMemoryRepository::new((*context.database).clone()),
        );
        // Wire memory repository into loop_runner's research_manager (Section 11).
        loop_runner.set_memory_repository_for_research(Arc::clone(&memory_repo));
        tracing::debug!("Memory repository wired into loop_runner research manager");
        // Wire research pipeline for dispatching research objectives (§11).
        let providers = crate::research::adapter_registry::all_providers();
        let pipeline = crate::research::pipeline::ResearchPipeline::new(providers);
        loop_runner.set_research_pipeline(std::sync::Arc::new(pipeline));
        tracing::debug!("Research pipeline wired into loop_runner research manager");
        // Set tokio handle for async research dispatch.
        loop_runner.set_tokio_handle(tokio::runtime::Handle::current());
        tracing::debug!("Tokio handle set for async research dispatch");
        // Also wire into handler's research_manager for MCP tool direct access.
        let mut research_mgr = crate::scheduler::cooboploop::research::ResearchManager::new();
        research_mgr.set_memory_repository(Arc::clone(&memory_repo));
        tracing::debug!("Memory repository wired into handler research manager");
        research_mgr.set_capability_registry(Arc::clone(&capability_registry_arc));
        tracing::debug!("Capability registry wired into handler research manager");
        // Start the autonomous cognitive loop (§20-21) and spawn its background thread.
        // This ensures a single LoopRunner instance that is both MCP-controllable and
        // autonomously running, rather than a separate orphaned instance from App::run.
        //
        // CRITICAL FIX: The background task must NOT hold the Mutex lock while
        // running the continuous loop, otherwise all MCP tool calls will deadlock.
        // Solution: Use std::mem::take to extract the LoopRunner from the Mutex,
        // run the continuous loop with the extracted runner, then restore it.
        loop_runner.start();
        let loop_runner_arc = Arc::new(Mutex::new(loop_runner));

        // Clone the Arc for the background task.
        let runner_for_background = Arc::clone(&loop_runner_arc);
        tokio::task::spawn_blocking(move || {
            // CRITICAL: Extract the LoopRunner from the mutex using std::mem::take.
            // This leaves a default-constructed LoopRunner in place, so MCP tool
            // queries won't deadlock while the background task runs.
            let mut runner = {
                let mut guard = match runner_for_background.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                std::mem::take(&mut *guard)
            };
            // Drop the guard before running the continuous loop

            // Run the continuous loop - this runs until should_continue returns false
            // The LoopRunner is owned here, so no lock is held
            if let Err(e) = runner.run_continuous_sync() {
                tracing::warn!("Continuous cognitive loop error: {e}");
            }
            tracing::info!("Continuous cognitive loop stopped");

            // Restore the LoopRunner to the Arc<Mutex<>> so MCP tools can access it
            if let Ok(mut guard) = runner_for_background.lock() {
                *guard = runner;
            }
        });
        Ok(Self {
            hardware_registry: Arc::clone(&hardware_registry_arc),
            queue: Arc::new(Mutex::new(objective_queue)),
            policy_registry: Arc::new(Mutex::new(policy_reg)),
            capability_registry: Arc::clone(&capability_registry_arc),
            loop_runner: loop_runner_arc,
            research_manager: Arc::new(Mutex::new(research_mgr)),
            hardware_discovery: Arc::new(Mutex::new(
                crate::scheduler::cooboploop::hardware::HardwareDiscovery::new(),
            )),
            self_improvement_pipeline: Arc::new(Mutex::new(
                crate::scheduler::cooboploop::self_improvement::SelfImprovementPipeline::new(),
            )),
            pending_opportunities: Arc::new(Mutex::new(Vec::new())),
            strategic_registry: Arc::new(Mutex::new(strategic_registry)),
            mission: Arc::new(Mutex::new("Mission".to_string())),
            runtime_adapters,
        })
    }
}

impl ToolHandler for CooboploopToolsHandler {
    fn category(&self) -> &str {
        "cooboploop"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "cooboploop_enqueue_goal".to_string(),
            "cooboploop_list_goals".to_string(),
            "cooboploop_get_goal".to_string(),
            "cooboploop_update_goal_status".to_string(),
            "cooboploop_run_source_discovery".to_string(),
            "cooboploop_evaluate_goal".to_string(),
            "cooboploop_reprioritize_queue".to_string(),
            "cooboploop_set_priority_policy".to_string(),
            "cooboploop_record_capability_outcome".to_string(),
            "cooboploop_get_capability_assessment".to_string(),
            "cooboploop_list_capabilities".to_string(),
            "cooboploop_start_loop".to_string(),
            "cooboploop_stop_loop".to_string(),
            "cooboploop_get_loop_status".to_string(),
            "cooboploop_run_single_cycle".to_string(),
            "cooboploop_step_loop".to_string(),
            "cooboploop_run_post_task_evaluation".to_string(),
            "cooboploop_get_idle_state".to_string(),
            "cooboploop_configure_idle_reevaluation_interval".to_string(),
            "cooboploop_create_research_objective".to_string(),
            "cooboploop_get_hardware_profile".to_string(),
            "cooboploop_detect_hardware_changes".to_string(),
            "cooboploop_run_inspection".to_string(),
            "cooboploop_get_modification_boundary".to_string(),
            "cooboploop_set_modification_boundary".to_string(),
            "cooboploop_run_opportunity_intake".to_string(),
            "cooboploop_get_pending_external_opportunities".to_string(),
            "cooboploop_set_autonomous_mode".to_string(),
            "cooboploop_get_autonomous_mode".to_string(),
            "cooboploop_list_strategic_objectives".to_string(),
            "cooboploop_add_strategic_objective".to_string(),
            "cooboploop_remove_strategic_objective".to_string(),
            "cooboploop_get_objective_hierarchy".to_string(),
            "cooboploop_set_mission".to_string(),
            "cooboploop_get_autonomy_levels".to_string(),
            "cooboploop_promote_autonomy".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        let registry = match self.capability_registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let rust_autonomy =
            registry.get_autonomy(&crate::scheduler::cooboploop::capability::CapabilityId::Rust);
        self.runtime_adapters.is_complete()
            && crate::scheduler::cooboploop::capability::AutonomyLevel::ALL.len() == 4
            && crate::scheduler::cooboploop::capability::AutonomyLevel::ALL.contains(&rust_autonomy)
            && registry.autonomy_levels().len() >= registry.list().len()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::api::bridge::tools::cooboploop::definitions;
        use crate::api::mcp::handlers::json_to_schema;
        definitions::all()
            .into_iter()
            .map(|t| rmcp::model::Tool::new(t.name, t.description, json_to_schema(t.input_schema)))
            .collect()
    }

    async fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<ToolOutput, HandlerError> {
        use crate::api::bridge::tools::cooboploop as cooboploop_mod;
        match name {
            "cooboploop_enqueue_goal" => {
                let input: cooboploop_mod::CooboploopEnqueueGoalInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_enqueue_goal(input, &self.queue).await)
            }
            "cooboploop_list_goals" => {
                let input: cooboploop_mod::CooboploopListGoalsInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_list_goals(input, &self.queue).await)
            }
            "cooboploop_get_goal" => {
                let input: cooboploop_mod::CooboploopGetGoalInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_get_goal(input, &self.queue).await)
            }
            "cooboploop_update_goal_status" => {
                let input: cooboploop_mod::CooboploopUpdateGoalStatusInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_update_goal_status(input, &self.queue).await)
            }
            "cooboploop_run_source_discovery" => {
                let input: cooboploop_mod::CooboploopRunSourceDiscoveryInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_source_discovery(input).await)
            }
            "cooboploop_evaluate_goal" => {
                let input: cooboploop_mod::CooboploopEvaluateGoalInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let strategic_ref = Arc::clone(&self.strategic_registry);
                let capability_ref = Arc::clone(&self.capability_registry);
                // Wire handler's policy_registry into GoalEvaluator (§5 / T-COO-17)
                let mut evaluator =
                    crate::scheduler::cooboploop::evaluation::GoalEvaluator::default();
                if let Ok(reg) = self.policy_registry.lock() {
                    // Replicate the current policy from the registry onto the evaluator
                    let policy_name = format!("{:?}", reg.current_policy());
                    tracing::debug!("evaluate_goal: using policy from registry: {}", policy_name);
                    // Apply the same policy class that the registry holds
                    let current_policy_ref = reg.current_policy();
                    let policy_display = format!("{:?}", current_policy_ref);
                    tracing::debug!("evaluate_goal: policy_display={}", policy_display);
                    // Wire the actual policy from registry (not hardcoded)
                    let policy_ref = reg.current_policy();
                    let policy_display = format!("{:?}", policy_ref);
                    tracing::debug!(
                        "evaluate_goal: using actual policy from registry: {}",
                        policy_display
                    );
                    // Clone the policy reference into a new box for the evaluator
                    // Note: PriorityPolicyTrait is not Clone; we reconstruct from registry state
                    // by reading the policy name and creating the matching instance
                    let policy_name = format!("{:?}", policy_ref);
                    let actual_policy: Box<
                        dyn crate::scheduler::cooboploop::evaluation::PriorityPolicyTrait,
                    > = match policy_name.as_str() {
                        s if s.contains("Conservative") => Box::new(
                            crate::scheduler::cooboploop::evaluation::ConservativePriorityPolicy,
                        ),
                        s if s.contains("Strategic") => {
                            Box::new(crate::scheduler::cooboploop::evaluation::StrategicPolicy)
                        }
                        s if s.contains("Exploration") => {
                            Box::new(crate::scheduler::cooboploop::evaluation::ExplorationPolicy)
                        }
                        _ => Box::new(
                            crate::scheduler::cooboploop::evaluation::DefaultPriorityPolicy,
                        ),
                    };
                    evaluator.set_policy(actual_policy);
                    // Also wire the registry reference for full policy access
                    drop(reg);
                }
                evaluator.set_strategic_registry(strategic_ref);
                evaluator.set_capability_registry(capability_ref);
                // Wire mission into evaluator (§19 / T-COO-12)
                if let Ok(mission_guard) = self.mission.lock() {
                    evaluator.set_mission(mission_guard.clone());
                }
                Ok(cooboploop_mod::execute_cooboploop_evaluate_goal(
                    input,
                    &Arc::new(evaluator),
                    &self.queue,
                )
                .await)
            }
            "cooboploop_reprioritize_queue" => {
                serde_json::from_value::<cooboploop_mod::CooboploopReprioritizeQueueInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let capability_ref = Arc::clone(&self.capability_registry);
                // Wire handler's policy_registry and strategic_registry into GoalEvaluator (§5 / T-COO-18)
                let mut evaluator =
                    crate::scheduler::cooboploop::evaluation::GoalEvaluator::default();
                if let Ok(reg) = self.policy_registry.lock() {
                    tracing::debug!("reprioritize_queue: using actual policy from registry");
                    let policy_ref = reg.current_policy();
                    let policy_name = format!("{:?}", policy_ref);
                    let actual_policy: Box<
                        dyn crate::scheduler::cooboploop::evaluation::PriorityPolicyTrait,
                    > = match policy_name.as_str() {
                        s if s.contains("Conservative") => Box::new(
                            crate::scheduler::cooboploop::evaluation::ConservativePriorityPolicy,
                        ),
                        s if s.contains("Strategic") => {
                            Box::new(crate::scheduler::cooboploop::evaluation::StrategicPolicy)
                        }
                        s if s.contains("Exploration") => {
                            Box::new(crate::scheduler::cooboploop::evaluation::ExplorationPolicy)
                        }
                        _ => Box::new(
                            crate::scheduler::cooboploop::evaluation::DefaultPriorityPolicy,
                        ),
                    };
                    evaluator.set_policy(actual_policy);
                }
                evaluator.set_capability_registry(capability_ref);
                // Wire strategic registry for strategic alignment evaluation
                evaluator.set_strategic_registry(Arc::clone(&self.strategic_registry));
                // Wire mission into evaluator (§19 / T-COO-12)
                if let Ok(mission_guard) = self.mission.lock() {
                    evaluator.set_mission(mission_guard.clone());
                }
                Ok(cooboploop_mod::execute_cooboploop_reprioritize_queue(
                    &self.queue,
                    &Arc::new(evaluator),
                )
                .await)
            }
            "cooboploop_set_priority_policy" => {
                let input: cooboploop_mod::CooboploopSetPriorityPolicyInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let registry = self.policy_registry.clone();
                // Wire policy registry directly; evaluator uses registry's current policy (§5 / T-COO-17)
                Ok(cooboploop_mod::execute_cooboploop_set_priority_policy(
                    input,
                    &Arc::new(crate::scheduler::cooboploop::evaluation::GoalEvaluator::default()),
                    &registry,
                )
                .await)
            }
            "cooboploop_record_capability_outcome" => {
                let input: cooboploop_mod::CooboploopRecordCapabilityOutcomeInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_record_capability_outcome(
                        input,
                        &self.capability_registry,
                    )
                    .await,
                )
            }
            "cooboploop_get_capability_assessment" => {
                let input: cooboploop_mod::CooboploopGetCapabilityAssessmentInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_get_capability_assessment(
                        input,
                        &self.capability_registry,
                    )
                    .await,
                )
            }
            "cooboploop_list_capabilities" => {
                serde_json::from_value::<cooboploop_mod::CooboploopListCapabilitiesInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_list_capabilities(&self.capability_registry)
                        .await,
                )
            }
            "cooboploop_start_loop" => {
                let input: cooboploop_mod::CooboploopStartLoopInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_start_loop(input, &runner).await)
            }
            "cooboploop_stop_loop" => {
                serde_json::from_value::<cooboploop_mod::CooboploopStopLoopInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_stop_loop(
                    cooboploop_mod::CooboploopStopLoopInput {},
                    &runner,
                )
                .await)
            }
            "cooboploop_get_loop_status" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetLoopStatusInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_get_loop_status(&runner).await)
            }
            "cooboploop_run_single_cycle" => {
                serde_json::from_value::<cooboploop_mod::CooboploopRunSingleCycleInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_run_single_cycle(&runner).await)
            }
            "cooboploop_step_loop" => {
                serde_json::from_value::<cooboploop_mod::CooboploopStepLoopInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_step_loop(&runner).await)
            }
            "cooboploop_run_post_task_evaluation" => {
                let input: cooboploop_mod::CooboploopRunPostTaskEvaluationInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let runner = self.loop_runner.clone();
                let queue = self.queue.clone();
                Ok(cooboploop_mod::execute_cooboploop_run_post_task_evaluation(
                    input, &runner, &queue,
                )
                .await)
            }
            "cooboploop_get_idle_state" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetIdleStateInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_get_idle_state(
                    &self.loop_runner,
                    &self.strategic_registry,
                )
                .await)
            }
            "cooboploop_configure_idle_reevaluation_interval" => {
                let input: cooboploop_mod::CooboploopConfigureIdleReevaluationIntervalInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_configure_idle_reevaluation_interval(
                        input,
                        &self.loop_runner,
                    )
                    .await,
                )
            }
            "cooboploop_create_research_objective" => {
                let input: cooboploop_mod::CooboploopCreateResearchObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_create_research_objective(
                        input,
                        &self.research_manager,
                        &self.queue,
                    )
                    .await,
                )
            }
            "cooboploop_get_hardware_profile" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetHardwareProfileInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_get_hardware_profile(
                    &self.hardware_discovery,
                    &self.hardware_registry,
                )
                .await)
            }
            "cooboploop_detect_hardware_changes" => {
                serde_json::from_value::<cooboploop_mod::CooboploopDetectHardwareChangesInput>(
                    args,
                )
                .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_detect_hardware_changes(
                    &self.hardware_discovery,
                    &self.hardware_registry,
                )
                .await)
            }
            "cooboploop_run_inspection" => {
                let input: cooboploop_mod::CooboploopRunInspectionInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_inspection(input, &self.queue).await)
            }
            "cooboploop_get_modification_boundary" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetModificationBoundaryInput>(
                    args,
                )
                .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_get_modification_boundary(
                        &self.self_improvement_pipeline,
                    )
                    .await,
                )
            }
            "cooboploop_set_modification_boundary" => {
                let input: cooboploop_mod::CooboploopSetModificationBoundaryInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_set_modification_boundary(
                        input,
                        &self.self_improvement_pipeline,
                    )
                    .await,
                )
            }
            "cooboploop_run_opportunity_intake" => {
                let input: cooboploop_mod::CooboploopRunOpportunityIntakeInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_opportunity_intake(
                    input,
                    &self.capability_registry,
                    &self.pending_opportunities,
                    &self.queue,
                )
                .await)
            }
            "cooboploop_get_pending_external_opportunities" => {
                serde_json::from_value::<
                    cooboploop_mod::CooboploopGetPendingExternalOpportunitiesInput,
                >(args)
                .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_get_pending_external_opportunities(
                        &self.pending_opportunities,
                    )
                    .await,
                )
            }
            "cooboploop_set_autonomous_mode" => {
                let input: cooboploop_mod::CooboploopSetAutonomousModeInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_set_autonomous_mode(
                        input,
                        &self.loop_runner,
                    )
                    .await,
                )
            }
            "cooboploop_get_autonomous_mode" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetAutonomousModeInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_get_autonomous_mode(&self.loop_runner).await)
            }
            "cooboploop_list_strategic_objectives" => {
                serde_json::from_value::<cooboploop_mod::CooboploopListStrategicObjectivesInput>(
                    args,
                )
                .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_list_strategic_objectives(
                        &self.strategic_registry,
                    )
                    .await,
                )
            }
            "cooboploop_add_strategic_objective" => {
                let input: cooboploop_mod::CooboploopAddStrategicObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_add_strategic_objective(
                    input,
                    &self.strategic_registry,
                )
                .await)
            }
            "cooboploop_remove_strategic_objective" => {
                let input: cooboploop_mod::CooboploopRemoveStrategicObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_remove_strategic_objective(
                        input,
                        &self.strategic_registry,
                    )
                    .await,
                )
            }
            "cooboploop_get_objective_hierarchy" => {
                serde_json::from_value::<cooboploop_mod::CooboploopGetObjectiveHierarchyInput>(
                    args,
                )
                .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let queue = self.queue.clone();
                let runner_ref = self.loop_runner.clone();
                Ok(cooboploop_mod::execute_cooboploop_get_objective_hierarchy(
                    &self.strategic_registry,
                    &self.mission,
                    &queue,
                    &runner_ref,
                )
                .await)
            }
            "cooboploop_set_mission" => {
                let input: cooboploop_mod::CooboploopSetMissionInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                // Wire mission into loop runner (§19 / T-COO-12)
                if let Ok(mut runner) = self.loop_runner.lock() {
                    runner.set_mission(input.mission.clone());
                }
                Ok(cooboploop_mod::execute_cooboploop_set_mission(input, &self.mission).await)
            }
            "cooboploop_get_autonomy_levels" => {
                let input: cooboploop_mod::CooboploopGetAutonomyLevelsInput =
                    serde_json::from_value(args)
                        .map_err(|error| HandlerError::InvalidParams(error.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_get_autonomy_levels(
                    input,
                    &self.capability_registry,
                )
                .await)
            }
            "cooboploop_promote_autonomy" => {
                let input: cooboploop_mod::CooboploopPromoteAutonomyInput =
                    serde_json::from_value(args)
                        .map_err(|error| HandlerError::InvalidParams(error.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_promote_autonomy(
                    input,
                    &self.capability_registry,
                )
                .await)
            }
            _ => Err(HandlerError::ToolNotFound(name.to_string())),
        }
    }
}
