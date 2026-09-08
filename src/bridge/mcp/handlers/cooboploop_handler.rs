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
}

impl CooboploopToolsHandler {
    pub fn new() -> Result<Self, crate::bridge::mcp::handlers::HandlerInitError> {
        Ok(Self {
            queue: Arc::new(Mutex::new(ObjectiveQueue::new())),
            policy_registry: Arc::new(Mutex::new(PriorityPolicyRegistry::default())),
            capability_registry: Arc::new(Mutex::new(
                crate::cooboploop::capability::CapabilityRegistry::new(),
            )),
            loop_runner: Arc::new(Mutex::new(crate::cooboploop::loop_runner::LoopRunner::new())),
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
        true
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
            "cooboploop_list_capabilities" => Ok(
                cooboploop_mod::execute_cooboploop_list_capabilities(&self.capability_registry)
                    .await,
            ),
            "cooboploop_start_loop" => {
                let result = {
                    let mut runner = match self.loop_runner.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    runner.start();
                    let status_str = if runner.should_continue() {
                        "running"
                    } else {
                        "stopped"
                    };
                    ToolOutput::success(serde_json::json!({
                        "message": "Loop started",
                        "status": status_str,
                    }))
                };
                Ok(result)
            }
            "cooboploop_stop_loop" => {
                let result = {
                    let mut runner = match self.loop_runner.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    runner.stop();
                    let status_str = if runner.should_continue() {
                        "running"
                    } else {
                        "stopped"
                    };
                    ToolOutput::success(serde_json::json!({
                        "message": "Loop stopped",
                        "status": status_str,
                    }))
                };
                Ok(result)
            }
            "cooboploop_get_loop_status" => {
                let result = {
                    let runner = match self.loop_runner.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    let status_str = if runner.should_continue() {
                        "running"
                    } else {
                        "stopped"
                    };
                    let cycles = runner.cycle_count();
                    let stage = format!("{:?}", runner.current_stage());
                    ToolOutput::success(serde_json::json!({
                        "status": status_str,
                        "cycles_completed": cycles,
                        "current_stage": stage,
                    }))
                };
                Ok(result)
            }
            "cooboploop_run_single_cycle" => {
                let cycles = {
                    let mut runner = match self.loop_runner.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    runner.run_cycle().map_err(HandlerError::ExecutionFailed)?;
                    runner.cycle_count()
                };
                Ok(ToolOutput::success(serde_json::json!({
                    "message": "Cycle completed",
                    "status": "ok",
                    "cycles_completed": cycles,
                })))
            }
            "cooboploop_step_loop" => {
                let cycles = {
                    let mut runner = match self.loop_runner.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Err(e) = runner.run_cycle() {
                        return Ok(ToolOutput::success(serde_json::json!({
                            "message": "Cycle failed",
                            "status": "error",
                            "error": format!("{}", e),
                        })));
                    }
                    runner.cycle_count()
                };
                Ok(ToolOutput::success(serde_json::json!({
                    "message": "Step completed",
                    "status": "ok",
                    "cycles_completed": cycles,
                })))
            }
            "cooboploop_run_post_task_evaluation" => {
                let input: cooboploop_mod::CooboploopRunPostTaskEvaluationInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_post_task_evaluation(input).await)
            }
            "cooboploop_get_idle_state" => {
                Ok(cooboploop_mod::execute_cooboploop_get_idle_state().await)
            }
            "cooboploop_configure_idle_reevaluation_interval" => {
                let input: cooboploop_mod::CooboploopConfigureIdleReevaluationIntervalInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(
                    cooboploop_mod::execute_cooboploop_configure_idle_reevaluation_interval(input)
                        .await,
                )
            }
            "cooboploop_create_research_objective" => {
                let input: cooboploop_mod::CooboploopCreateResearchObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_create_research_objective(input).await)
            }
            "cooboploop_get_hardware_profile" => {
                Ok(cooboploop_mod::execute_cooboploop_get_hardware_profile().await)
            }
            "cooboploop_detect_hardware_changes" => {
                Ok(cooboploop_mod::execute_cooboploop_detect_hardware_changes().await)
            }
            "cooboploop_run_inspection" => {
                let input: cooboploop_mod::CooboploopRunInspectionInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_inspection(input).await)
            }
            "cooboploop_get_modification_boundary" => {
                Ok(cooboploop_mod::execute_cooboploop_get_modification_boundary().await)
            }
            "cooboploop_set_modification_boundary" => {
                let input: cooboploop_mod::CooboploopSetModificationBoundaryInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_set_modification_boundary(input).await)
            }
            "cooboploop_run_opportunity_intake" => {
                let input: cooboploop_mod::CooboploopRunOpportunityIntakeInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_run_opportunity_intake(input).await)
            }
            "cooboploop_get_pending_external_opportunities" => {
                Ok(cooboploop_mod::execute_cooboploop_get_pending_external_opportunities().await)
            }
            "cooboploop_set_autonomous_mode" => {
                let input: cooboploop_mod::CooboploopSetAutonomousModeInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_set_autonomous_mode(input).await)
            }
            "cooboploop_get_autonomous_mode" => {
                Ok(cooboploop_mod::execute_cooboploop_get_autonomous_mode().await)
            }
            "cooboploop_list_strategic_objectives" => {
                Ok(cooboploop_mod::execute_cooboploop_list_strategic_objectives().await)
            }
            "cooboploop_add_strategic_objective" => {
                let input: cooboploop_mod::CooboploopAddStrategicObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_add_strategic_objective(input).await)
            }
            "cooboploop_remove_strategic_objective" => {
                let input: cooboploop_mod::CooboploopRemoveStrategicObjectiveInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_remove_strategic_objective(input).await)
            }
            "cooboploop_get_objective_hierarchy" => {
                Ok(cooboploop_mod::execute_cooboploop_get_objective_hierarchy().await)
            }
            "cooboploop_set_mission" => {
                let input: cooboploop_mod::CooboploopSetMissionInput = serde_json::from_value(args)
                    .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_set_mission(input).await)
            }
            "cooboploop_get_autonomy_levels" => {
                Ok(cooboploop_mod::execute_cooboploop_get_autonomy_levels().await)
            }
            "cooboploop_promote_autonomy" => {
                let input: cooboploop_mod::CooboploopPromoteAutonomyInput =
                    serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                Ok(cooboploop_mod::execute_cooboploop_promote_autonomy(input).await)
            }
            _ => Err(HandlerError::ToolNotFound(name.to_string())),
        }
    }
}
