use std::sync::{Arc, Mutex};

use crate::bridge::mcp::handlers::{HandlerError, ToolHandler};
use crate::bridge::tools::ToolOutput;
use crate::cooboploop::evaluation::PriorityPolicyRegistry;
use crate::cooboploop::queue::ObjectiveQueue;

#[derive(Clone)]
pub struct CooboploopToolsHandler {
    queue: Arc<Mutex<ObjectiveQueue>>,
    policy_registry: Arc<Mutex<PriorityPolicyRegistry>>,
    capability_registry: Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
    loop_runner: Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
    research_manager: Arc<Mutex<crate::cooboploop::research::ResearchManager>>,
    hardware_discovery: Arc<Mutex<crate::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: Arc<Mutex<crate::cooboploop::hardware::HardwareRegistry>>,
    self_improvement_pipeline:
        Arc<Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>>,
    pending_opportunities: Arc<Mutex<Vec<crate::cooboploop::opportunity::IntakeResult>>>,
    strategic_registry: Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
    mission: Arc<Mutex<String>>,
    runtime_adapters: Arc<crate::cooboploop::llm_provider::DefaultRuntimeAdapters>,
}

impl CooboploopToolsHandler {
    pub fn new(
        context: Arc<crate::bridge::mcp::McpContext>,
    ) -> Result<Self, crate::bridge::mcp::handlers::HandlerInitError> {
        let database_path = context.database.path().to_string_lossy().to_string();
        let objective_queue = ObjectiveQueue::open(&database_path).map_err(|error| {
            crate::bridge::mcp::handlers::HandlerInitError::new("cooboploop", &error)
        })?;
        let hardware_registry = crate::cooboploop::hardware::HardwareRegistry::open(&database_path)
            .map_err(|error| {
                crate::bridge::mcp::handlers::HandlerInitError::new("cooboploop", &error)
            })?;
        let strategic_registry =
            crate::cooboploop::strategic::StrategicObjectiveRegistry::open(&database_path)
                .map_err(|error| {
                    crate::bridge::mcp::handlers::HandlerInitError::new("cooboploop", &error)
                })?;
        let runtime_adapters = Arc::new(
            crate::cooboploop::llm_provider::DefaultRuntimeAdapters::new(
                context.database.as_ref().clone(),
            ),
        );
        crate::cooboploop::init();
        let mut capability_registry =
            crate::cooboploop::capability::CapabilityRegistry::open(&database_path)
                .unwrap_or_else(|_| crate::cooboploop::capability::CapabilityRegistry::new());
        crate::cooboploop::capability::seed_default_capabilities(&mut capability_registry);
        // Wire cooboploop tool registration
        let mut reg = crate::bridge::tools::ToolRegistry::new();
        crate::bridge::tools::cooboploop::register_cooboploop_tools(&mut reg);
        let mut policy_reg = PriorityPolicyRegistry::default();
        // Wire current_policy / set_policy by reading and setting
        let current_policy = policy_reg.current_policy();
        let current_display = format!("{:?}", current_policy);
        policy_reg.set_policy(Box::new(
            crate::cooboploop::evaluation::ConservativePriorityPolicy,
        ));
        let updated_policy = policy_reg.current_policy();
        let updated_display = format!("{:?}", updated_policy);
        tracing::debug!(
            "Policy wired: current={}, updated={}",
            current_display,
            updated_display
        );
        // Create the loop runner with its default experience coordinator
        let mut loop_runner = crate::cooboploop::loop_runner::LoopRunner::new();
        // Wire the experience coordinator into the loop runner per Architecture §15.
        if let Some(coordinator) = &loop_runner.experience_coordinator {
            let coordinator_ref = std::sync::Arc::clone(coordinator);
            loop_runner.set_experience_coordinator(coordinator_ref);
        }
        Ok(Self {
            queue: Arc::new(Mutex::new(objective_queue)),
            policy_registry: Arc::new(Mutex::new(policy_reg)),
            capability_registry: Arc::new(Mutex::new(capability_registry)),
            loop_runner: Arc::new(Mutex::new(loop_runner)),
            research_manager: Arc::new(Mutex::new(
                crate::cooboploop::research::ResearchManager::new(),
            )),
            hardware_discovery: Arc::new(Mutex::new(
                crate::cooboploop::hardware::HardwareDiscovery::new(),
            )),
            hardware_registry: Arc::new(Mutex::new(hardware_registry)),
            self_improvement_pipeline: Arc::new(Mutex::new(
                crate::cooboploop::self_improvement::SelfImprovementPipeline::new(),
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
            registry.get_autonomy(&crate::cooboploop::capability::CapabilityId::Rust);
        self.runtime_adapters.is_complete()
            && crate::cooboploop::capability::AutonomyLevel::ALL.len() == 4
            && crate::cooboploop::capability::AutonomyLevel::ALL.contains(&rust_autonomy)
            && registry.autonomy_levels().len() >= registry.list().len()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        use crate::bridge::tools::cooboploop::definitions;
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
        use crate::bridge::tools::cooboploop as cooboploop_mod;
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
                Ok(cooboploop_mod::execute_cooboploop_evaluate_goal(
                    input,
                    &Arc::new(crate::cooboploop::evaluation::GoalEvaluator::default()),
                    &self.queue,
                )
                .await)
            }
            "cooboploop_reprioritize_queue" => {
                serde_json::from_value::<cooboploop_mod::CooboploopReprioritizeQueueInput>(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_reprioritize_queue(
                    &self.queue,
                    &Arc::new(crate::cooboploop::evaluation::GoalEvaluator::default()),
                )
                .await)
            }
            "cooboploop_set_priority_policy" => {
                let input: cooboploop_mod::CooboploopSetPriorityPolicyInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                let registry = self.policy_registry.clone();
                Ok(cooboploop_mod::execute_cooboploop_set_priority_policy(
                    input,
                    &Arc::new(crate::cooboploop::evaluation::GoalEvaluator::default()),
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
                Ok(cooboploop_mod::execute_cooboploop_run_post_task_evaluation(input).await)
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
                Ok(cooboploop_mod::execute_cooboploop_get_objective_hierarchy(
                    &self.strategic_registry,
                    &self.mission,
                )
                .await)
            }
            "cooboploop_set_mission" => {
                let input: cooboploop_mod::CooboploopSetMissionInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
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
