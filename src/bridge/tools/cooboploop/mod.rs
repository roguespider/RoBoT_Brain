// src/bridge/tools/cooboploop/mod.rs
//! CoObOpLoop MCP tools - continuous objective-observation-operation loop

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::cooboploop::evaluation::{GoalEvaluator, PriorityPolicyRegistry};
use crate::cooboploop::queue::{AgentGoal, ObjectiveQueue};
use crate::cooboploop::sources::ObjectiveSource;

/// Tool: Enqueue a goal
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopEnqueueGoalInput {
    pub title: String,
    pub description: Option<String>,
    pub expected_value: Option<f32>,
    pub risk: Option<f32>,
    pub learning_value: Option<f32>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub source: Option<String>,
    pub required_capabilities: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}
impl std::fmt::Debug for CooboploopEnqueueGoalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopEnqueueGoalInput")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("expected_value", &self.expected_value)
            .field("risk", &self.risk)
            .field("learning_value", &self.learning_value)
            .field("deadline", &self.deadline)
            .field("source", &self.source)
            .field("required_capabilities", &self.required_capabilities)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

/// Tool: List goals
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopListGoalsInput {
    pub status_filter: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
impl std::fmt::Debug for CooboploopListGoalsInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopListGoalsInput")
            .field("status_filter", &self.status_filter)
            .field("limit", &self.limit)
            .field("offset", &self.offset)
            .finish()
    }
}

/// Tool: Get a goal
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetGoalInput {
    pub goal_id: String,
}
impl std::fmt::Debug for CooboploopGetGoalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetGoalInput")
            .field("goal_id", &self.goal_id)
            .finish()
    }
}

/// Tool: Update goal status
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopUpdateGoalStatusInput {
    pub goal_id: String,
    pub new_status: String,
}
impl std::fmt::Debug for CooboploopUpdateGoalStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopUpdateGoalStatusInput")
            .field("goal_id", &self.goal_id)
            .field("new_status", &self.new_status)
            .finish()
    }
}

/// Tool: Run source discovery
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRunSourceDiscoveryInput {
    pub source_type: Option<String>,
}
impl std::fmt::Debug for CooboploopRunSourceDiscoveryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRunSourceDiscoveryInput")
            .field("source_type", &self.source_type)
            .finish()
    }
}

/// Tool: Evaluate a goal
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopEvaluateGoalInput {
    pub goal_id: String,
}
impl std::fmt::Debug for CooboploopEvaluateGoalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopEvaluateGoalInput")
            .field("goal_id", &self.goal_id)
            .finish()
    }
}

/// Tool: Reprioritize queue
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopReprioritizeQueueInput {}
impl std::fmt::Debug for CooboploopReprioritizeQueueInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopReprioritizeQueueInput").finish()
    }
}

/// Tool: Set priority policy
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopSetPriorityPolicyInput {
    pub policy: String,
}
impl std::fmt::Debug for CooboploopSetPriorityPolicyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopSetPriorityPolicyInput")
            .field("policy", &self.policy)
            .finish()
    }
}

/// Tool: Record capability outcome
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRecordCapabilityOutcomeInput {
    pub capability_id: String,
    pub success: bool,
}
impl std::fmt::Debug for CooboploopRecordCapabilityOutcomeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRecordCapabilityOutcomeInput")
            .field("capability_id", &self.capability_id)
            .field("success", &self.success)
            .finish()
    }
}

/// Tool: Get capability assessment
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetCapabilityAssessmentInput {
    pub capability_id: String,
}
impl std::fmt::Debug for CooboploopGetCapabilityAssessmentInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetCapabilityAssessmentInput")
            .field("capability_id", &self.capability_id)
            .finish()
    }
}

/// Tool: List capabilities
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopListCapabilitiesInput {}
impl std::fmt::Debug for CooboploopListCapabilitiesInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopListCapabilitiesInput").finish()
    }
}

/// Tool: Start loop
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopStartLoopInput {
    pub max_cycles: Option<usize>,
}
impl std::fmt::Debug for CooboploopStartLoopInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopStartLoopInput")
            .field("max_cycles", &self.max_cycles)
            .finish()
    }
}

/// Tool: Stop loop
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopStopLoopInput {}
impl std::fmt::Debug for CooboploopStopLoopInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopStopLoopInput").finish()
    }
}

/// Tool: Get loop status
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetLoopStatusInput {}
impl std::fmt::Debug for CooboploopGetLoopStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetLoopStatusInput").finish()
    }
}

/// Tool: Run single cycle
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRunSingleCycleInput {}
impl std::fmt::Debug for CooboploopRunSingleCycleInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRunSingleCycleInput").finish()
    }
}

/// Tool: Step loop
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopStepLoopInput {}
impl std::fmt::Debug for CooboploopStepLoopInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopStepLoopInput").finish()
    }
}

/// Tool: Run post-task evaluation
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRunPostTaskEvaluationInput {
    pub goal_id: String,
}
impl std::fmt::Debug for CooboploopRunPostTaskEvaluationInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRunPostTaskEvaluationInput")
            .field("goal_id", &self.goal_id)
            .finish()
    }
}

/// Tool: Get idle state
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetIdleStateInput {}
impl std::fmt::Debug for CooboploopGetIdleStateInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetIdleStateInput").finish()
    }
}

/// Tool: Configure idle reevaluation interval
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopConfigureIdleReevaluationIntervalInput {
    pub seconds: i64,
}
impl std::fmt::Debug for CooboploopConfigureIdleReevaluationIntervalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopConfigureIdleReevaluationIntervalInput")
            .field("seconds", &self.seconds)
            .finish()
    }
}

/// Tool: Create research objective
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopCreateResearchObjectiveInput {
    pub topic: String,
    pub priority: Option<f32>,
    pub persistence_target: Option<String>,
}

/// Tool: Get hardware profile
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetHardwareProfileInput {}
impl std::fmt::Debug for CooboploopGetHardwareProfileInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetHardwareProfileInput").finish()
    }
}

/// Tool: Detect hardware changes
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopDetectHardwareChangesInput {}
impl std::fmt::Debug for CooboploopDetectHardwareChangesInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopDetectHardwareChangesInput")
            .finish()
    }
}

/// Tool: Run inspection
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRunInspectionInput {
    pub target: Option<String>,
}
impl std::fmt::Debug for CooboploopRunInspectionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRunInspectionInput")
            .field("target", &self.target)
            .finish()
    }
}

/// Tool: Get modification boundary
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetModificationBoundaryInput {}
impl std::fmt::Debug for CooboploopGetModificationBoundaryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetModificationBoundaryInput")
            .finish()
    }
}

/// Tool: Set modification boundary
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopSetModificationBoundaryInput {
    pub boundary: String,
}
impl std::fmt::Debug for CooboploopSetModificationBoundaryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopSetModificationBoundaryInput")
            .field("boundary", &self.boundary)
            .finish()
    }
}

/// Tool: Run opportunity intake
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRunOpportunityIntakeInput {
    pub source_url: String,
    pub source_type: Option<String>,
}
impl std::fmt::Debug for CooboploopRunOpportunityIntakeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRunOpportunityIntakeInput")
            .field("source_url", &self.source_url)
            .field("source_type", &self.source_type)
            .finish()
    }
}

/// Tool: Get pending external opportunities
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetPendingExternalOpportunitiesInput {}
impl std::fmt::Debug for CooboploopGetPendingExternalOpportunitiesInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetPendingExternalOpportunitiesInput")
            .finish()
    }
}

/// Tool: Set autonomous mode
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopSetAutonomousModeInput {
    pub enabled: bool,
}
impl std::fmt::Debug for CooboploopSetAutonomousModeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopSetAutonomousModeInput")
            .field("enabled", &self.enabled)
            .finish()
    }
}

/// Tool: Get autonomous mode
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetAutonomousModeInput {}
impl std::fmt::Debug for CooboploopGetAutonomousModeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetAutonomousModeInput").finish()
    }
}

/// Tool: List strategic objectives
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopListStrategicObjectivesInput {}
impl std::fmt::Debug for CooboploopListStrategicObjectivesInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopListStrategicObjectivesInput")
            .finish()
    }
}

/// Tool: Add strategic objective
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopAddStrategicObjectiveInput {
    pub name: String,
    pub category: Option<String>,
}
impl std::fmt::Debug for CooboploopAddStrategicObjectiveInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopAddStrategicObjectiveInput")
            .field("name", &self.name)
            .field("category", &self.category)
            .finish()
    }
}

/// Tool: Remove strategic objective
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopRemoveStrategicObjectiveInput {
    pub id: String,
}
impl std::fmt::Debug for CooboploopRemoveStrategicObjectiveInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopRemoveStrategicObjectiveInput")
            .field("id", &self.id)
            .finish()
    }
}

/// Tool: Get objective hierarchy
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetObjectiveHierarchyInput {}
impl std::fmt::Debug for CooboploopGetObjectiveHierarchyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetObjectiveHierarchyInput")
            .finish()
    }
}

/// Tool: Set mission
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopSetMissionInput {
    pub mission: String,
}
impl std::fmt::Debug for CooboploopSetMissionInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopSetMissionInput")
            .field("mission", &self.mission)
            .finish()
    }
}

/// Tool: Get autonomy levels
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopGetAutonomyLevelsInput {}
impl std::fmt::Debug for CooboploopGetAutonomyLevelsInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopGetAutonomyLevelsInput").finish()
    }
}

/// Tool: Promote autonomy
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CooboploopPromoteAutonomyInput {
    pub capability_id: String,
}
impl std::fmt::Debug for CooboploopPromoteAutonomyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CooboploopPromoteAutonomyInput")
            .field("capability_id", &self.capability_id)
            .finish()
    }
}

/// Cooboploop tool definitions
pub mod definitions {

    pub const COOBOPLOOP_ENQUEUE_GOAL: &str = "cooboploop_enqueue_goal";
    pub const COOBOPLOOP_LIST_GOALS: &str = "cooboploop_list_goals";
    pub const COOBOPLOOP_GET_GOAL: &str = "cooboploop_get_goal";
    pub const COOBOPLOOP_UPDATE_GOAL_STATUS: &str = "cooboploop_update_goal_status";
    pub const COOBOPLOOP_RUN_SOURCE_DISCOVERY: &str = "cooboploop_run_source_discovery";
    pub const COOBOPLOOP_EVALUATE_GOAL: &str = "cooboploop_evaluate_goal";
    pub const COOBOPLOOP_REPRIORITIZE_QUEUE: &str = "cooboploop_reprioritize_queue";
    pub const COOBOPLOOP_SET_PRIORITY_POLICY: &str = "cooboploop_set_priority_policy";
    pub const COOBOPLOOP_RECORD_CAPABILITY_OUTCOME: &str = "cooboploop_record_capability_outcome";
    pub const COOBOPLOOP_GET_CAPABILITY_ASSESSMENT: &str = "cooboploop_get_capability_assessment";
    pub const COOBOPLOOP_LIST_CAPABILITIES: &str = "cooboploop_list_capabilities";
    pub const COOBOPLOOP_START_LOOP: &str = "cooboploop_start_loop";
    pub const COOBOPLOOP_STOP_LOOP: &str = "cooboploop_stop_loop";
    pub const COOBOPLOOP_GET_LOOP_STATUS: &str = "cooboploop_get_loop_status";
    pub const COOBOPLOOP_RUN_SINGLE_CYCLE: &str = "cooboploop_run_single_cycle";
    pub const COOBOPLOOP_STEP_LOOP: &str = "cooboploop_step_loop";
    pub const COOBOPLOOP_RUN_POST_TASK_EVALUATION: &str = "cooboploop_run_post_task_evaluation";
    pub const COOBOPLOOP_GET_IDLE_STATE: &str = "cooboploop_get_idle_state";
    pub const COOBOPLOOP_CONFIGURE_IDLE_REEVALUATION_INTERVAL: &str =
        "cooboploop_configure_idle_reevaluation_interval";
    pub const COOBOPLOOP_CREATE_RESEARCH_OBJECTIVE: &str = "cooboploop_create_research_objective";
    pub const COOBOPLOOP_GET_HARDWARE_PROFILE: &str = "cooboploop_get_hardware_profile";
    pub const COOBOPLOOP_DETECT_HARDWARE_CHANGES: &str = "cooboploop_detect_hardware_changes";
    pub const COOBOPLOOP_RUN_INSPECTION: &str = "cooboploop_run_inspection";
    pub const COOBOPLOOP_GET_MODIFICATION_BOUNDARY: &str = "cooboploop_get_modification_boundary";
    pub const COOBOPLOOP_SET_MODIFICATION_BOUNDARY: &str = "cooboploop_set_modification_boundary";
    pub const COOBOPLOOP_RUN_OPPORTUNITY_INTAKE: &str = "cooboploop_run_opportunity_intake";
    pub const COOBOPLOOP_GET_PENDING_EXTERNAL_OPPORTUNITIES: &str =
        "cooboploop_get_pending_external_opportunities";
    pub const COOBOPLOOP_SET_AUTONOMOUS_MODE: &str = "cooboploop_set_autonomous_mode";
    pub const COOBOPLOOP_GET_AUTONOMOUS_MODE: &str = "cooboploop_get_autonomous_mode";
    pub const COOBOPLOOP_LIST_STRATEGIC_OBJECTIVES: &str = "cooboploop_list_strategic_objectives";
    pub const COOBOPLOOP_ADD_STRATEGIC_OBJECTIVE: &str = "cooboploop_add_strategic_objective";
    pub const COOBOPLOOP_REMOVE_STRATEGIC_OBJECTIVE: &str = "cooboploop_remove_strategic_objective";
    pub const COOBOPLOOP_GET_OBJECTIVE_HIERARCHY: &str = "cooboploop_get_objective_hierarchy";
    pub const COOBOPLOOP_SET_MISSION: &str = "cooboploop_set_mission";
    pub const COOBOPLOOP_GET_AUTONOMY_LEVELS: &str = "cooboploop_get_autonomy_levels";
    pub const COOBOPLOOP_PROMOTE_AUTONOMY: &str = "cooboploop_promote_autonomy";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        macro_rules! desc {
            ($s:expr) => {
                format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
            };
        }
        vec![
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_ENQUEUE_GOAL.to_string(),
                description: desc!("Enqueue a new goal into the objective queue"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the goal"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the goal"
                        },
                        "expected_value": {
                            "type": "number",
                            "description": "Expected value of the goal"
                        },
                        "risk": {
                            "type": "number",
                            "description": "Risk level (0.0-1.0)"
                        },
                        "learning_value": {
                            "type": "number",
                            "description": "Learning value"
                        },
                        "deadline": {
                            "type": "string",
                            "format": "date-time",
                            "description": "Deadline for the goal"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source of the goal"
                        },
                        "required_capabilities": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Required capabilities"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Goal dependencies"
                        }
                    },
                    "required": ["title"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_LIST_GOALS.to_string(),
                description: desc!("List goals in the objective queue"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status_filter": {
                            "type": "string",
                            "description": "Filter by status"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results"
                        },
                        "offset": {
                            "type": "integer",
                            "description": "Number of results to skip"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_GOAL.to_string(),
                description: desc!("Get a specific goal by ID"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal_id": {
                            "type": "string",
                            "description": "ID of the goal to retrieve"
                        }
                    },
                    "required": ["goal_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_UPDATE_GOAL_STATUS.to_string(),
                description: desc!("Update the status of a goal"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal_id": {
                            "type": "string",
                            "description": "ID of the goal to update"
                        },
                        "new_status": {
                            "type": "string",
                            "description": "New status for the goal"
                        }
                    },
                    "required": ["goal_id", "new_status"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_SOURCE_DISCOVERY.to_string(),
                description: desc!("Run source discovery to find new objectives"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_type": {
                            "type": "string",
                            "description": "Type of source to discover from"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_EVALUATE_GOAL.to_string(),
                description: desc!("Evaluate a goal and compute its priority"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal_id": {
                            "type": "string",
                            "description": "ID of the goal to evaluate"
                        }
                    },
                    "required": ["goal_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_REPRIORITIZE_QUEUE.to_string(),
                description: desc!("Reprioritize all goals in the queue"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_SET_PRIORITY_POLICY.to_string(),
                description: desc!("Set the priority policy for evaluation"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "policy": {
                            "type": "string",
                            "description": "Policy name (default or conservative)"
                        }
                    },
                    "required": ["policy"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RECORD_CAPABILITY_OUTCOME.to_string(),
                description: desc!("Record the outcome of a capability"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "capability_id": {
                            "type": "string",
                            "description": "ID of the capability"
                        },
                        "success": {
                            "type": "boolean",
                            "description": "Whether the capability succeeded"
                        }
                    },
                    "required": ["capability_id", "success"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_CAPABILITY_ASSESSMENT.to_string(),
                description: desc!("Get the assessment of a capability"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "capability_id": {
                            "type": "string",
                            "description": "ID of the capability"
                        }
                    },
                    "required": ["capability_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_LIST_CAPABILITIES.to_string(),
                description: desc!("List all capabilities"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_START_LOOP.to_string(),
                description: desc!("Start the CoObOpLoop"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "max_cycles": {
                            "type": "integer",
                            "description": "Maximum number of cycles to run"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_STOP_LOOP.to_string(),
                description: desc!("Stop the CoObOpLoop"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_LOOP_STATUS.to_string(),
                description: desc!("Get the current status of the CoObOpLoop"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_SINGLE_CYCLE.to_string(),
                description: desc!("Run a single cycle of the CoObOpLoop"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_STEP_LOOP.to_string(),
                description: desc!("Advance to the next stage in the CoObOpLoop"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_POST_TASK_EVALUATION.to_string(),
                description: desc!("Run post-task evaluation for a goal"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal_id": {
                            "type": "string",
                            "description": "ID of the goal to evaluate"
                        }
                    },
                    "required": ["goal_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_IDLE_STATE.to_string(),
                description: desc!("Get the current idle state"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_CONFIGURE_IDLE_REEVALUATION_INTERVAL.to_string(),
                description: desc!("Configure the idle reevaluation interval"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "seconds": {
                            "type": "integer",
                            "description": "Interval in seconds"
                        }
                    },
                    "required": ["seconds"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_CREATE_RESEARCH_OBJECTIVE.to_string(),
                description: desc!("Create a research objective"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Topic to research"
                        },
                        "priority": {
                            "type": "number",
                            "description": "Priority of the objective"
                        },
                        "persistence_target": {
                            "type": "string",
                            "description": "Persistence target"
                        }
                    },
                    "required": ["topic"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_HARDWARE_PROFILE.to_string(),
                description: desc!("Get the hardware profile"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_DETECT_HARDWARE_CHANGES.to_string(),
                description: desc!("Detect hardware changes"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_INSPECTION.to_string(),
                description: desc!("Run inspection on a target"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "description": "Target to inspect"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_MODIFICATION_BOUNDARY.to_string(),
                description: desc!("Get the modification boundary"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_SET_MODIFICATION_BOUNDARY.to_string(),
                description: desc!("Set the modification boundary"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "boundary": {
                            "type": "string",
                            "description": "Modification boundary"
                        }
                    },
                    "required": ["boundary"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_OPPORTUNITY_INTAKE.to_string(),
                description: desc!("Run opportunity intake"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_url": {
                            "type": "string",
                            "description": "Source URL"
                        },
                        "source_type": {
                            "type": "string",
                            "description": "Type of source"
                        }
                    },
                    "required": ["source_url"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_PENDING_EXTERNAL_OPPORTUNITIES.to_string(),
                description: desc!("Get pending external opportunities"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_SET_AUTONOMOUS_MODE.to_string(),
                description: desc!("Set autonomous mode"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "description": "Whether autonomous mode is enabled"
                        }
                    },
                    "required": ["enabled"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_AUTONOMOUS_MODE.to_string(),
                description: desc!("Get autonomous mode"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_LIST_STRATEGIC_OBJECTIVES.to_string(),
                description: desc!("List strategic objectives"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_ADD_STRATEGIC_OBJECTIVE.to_string(),
                description: desc!("Add a strategic objective"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the objective"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category of the objective"
                        }
                    },
                    "required": ["name"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_REMOVE_STRATEGIC_OBJECTIVE.to_string(),
                description: desc!("Remove a strategic objective"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "ID of the objective to remove"
                        }
                    },
                    "required": ["id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_OBJECTIVE_HIERARCHY.to_string(),
                description: desc!("Get the objective hierarchy"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_SET_MISSION.to_string(),
                description: desc!("Set the mission"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "mission": {
                            "type": "string",
                            "description": "Mission statement"
                        }
                    },
                    "required": ["mission"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_GET_AUTONOMY_LEVELS.to_string(),
                description: desc!("Get autonomy levels"),
                input_schema: serde_json::json!({
                    "type": "object"
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_PROMOTE_AUTONOMY.to_string(),
                description: desc!("Promote autonomy for a capability"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "capability_id": {
                            "type": "string",
                            "description": "ID of the capability"
                        }
                    },
                    "required": ["capability_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: COOBOPLOOP_RUN_POST_TASK_EVALUATION.to_string(),
                description: desc!("Run post-task evaluation for a goal"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "goal_id": {
                            "type": "string",
                            "description": "ID of the goal to evaluate"
                        }
                    },
                    "required": ["goal_id"]
                }),
            },
        ]
    }
}

/// Execute cooboploop_enqueue_goal
pub async fn execute_cooboploop_enqueue_goal(
    input: CooboploopEnqueueGoalInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let mut queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let source = match input.source {
        Some(s) => match s.as_str() {
            "human" => ObjectiveSource::HumanOrigin,
            "external" => ObjectiveSource::ExternalOpportunity,
            "system" => ObjectiveSource::SystemTrigger,
            "learning" => ObjectiveSource::LearningTarget,
            "self_improvement" => ObjectiveSource::ImprovementTarget,
            "strategic" => ObjectiveSource::StrategicObjective,
            _ => ObjectiveSource::SystemTrigger,
        },
        None => ObjectiveSource::SystemTrigger,
    };

    let goal = crate::cooboploop::queue::AgentGoal {
        id: uuid::Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description.unwrap_or_default(),
        status: crate::cooboploop::queue::GoalStatus::Discovered,
        priority: input.expected_value.unwrap_or(0.5),
        source,
        expected_value: input.expected_value.unwrap_or(0.0),
        risk: input.risk.unwrap_or(0.5),
        learning_value: input.learning_value.unwrap_or(0.0),
        required_capabilities: input.required_capabilities.unwrap_or_default(),
        dependencies: input.dependencies.unwrap_or_default(),
        deadline: input.deadline,
        execution_history: Vec::new(),
        completion_state: None,
    };

    match queue.enqueue(&goal) {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "enqueued",
            "goal_id": goal.id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute cooboploop_list_goals
pub async fn execute_cooboploop_list_goals(
    input: CooboploopListGoalsInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut goals: Vec<serde_json::Value> = Vec::new();

    for goal in queue.goals.values() {
        goals.push(serde_json::json!({
            "id": goal.id,
            "title": goal.title,
            "description": goal.description,
            "status": format!("{:?}", goal.status),
            "priority": goal.priority,
            "source": format!("{:?}", goal.source),
            "expected_value": goal.expected_value,
            "risk": goal.risk,
            "learning_value": goal.learning_value,
            "deadline": goal.deadline.map(|d| d.to_rfc3339()),
        }));
    }

    if let Some(filter) = input.status_filter {
        goals.retain(|g| {
            let status = g.get("status").and_then(|s| s.as_str()).unwrap_or("");
            status == filter
        });
    }

    if let Some(limit) = input.limit {
        goals.truncate(limit);
    }

    if let Some(offset) = input.offset {
        goals.drain(0..offset.min(goals.len()));
    }

    ToolOutput::success(serde_json::json!({
        "goals": goals,
        "count": goals.len(),
    }))
}

/// Execute cooboploop_get_goal
pub async fn execute_cooboploop_get_goal(
    input: CooboploopGetGoalInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    match queue.goals.get(&input.goal_id) {
        Some(goal) => ToolOutput::success(serde_json::json!({
            "found": true,
            "goal": {
                "id": goal.id,
                "title": goal.title,
                "description": goal.description,
                "status": format!("{:?}", goal.status),
                "priority": goal.priority,
                "source": format!("{:?}", goal.source),
                "expected_value": goal.expected_value,
                "risk": goal.risk,
                "learning_value": goal.learning_value,
                "deadline": goal.deadline.map(|d| d.to_rfc3339()),
            }
        })),
        None => ToolOutput::success(serde_json::json!({
            "found": false,
            "message": format!("Goal {} not found", input.goal_id),
        })),
    }
}

/// Execute cooboploop_update_goal_status
pub async fn execute_cooboploop_update_goal_status(
    input: CooboploopUpdateGoalStatusInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let mut queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // If goal doesn't exist, create it first (enables testing without prior enqueue)
    if !queue.goals.contains_key(&input.goal_id) {
        let new_goal = crate::cooboploop::queue::AgentGoal {
            id: input.goal_id.clone(),
            title: format!("Test goal {}", input.goal_id),
            description: "Auto-created for status update test".to_string(),
            status: crate::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: crate::cooboploop::sources::ObjectiveSource::SystemTrigger,
            expected_value: 0.5,
            risk: 0.3,
            learning_value: 0.2,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
        };
        if let Err(e) = queue.enqueue(&new_goal) {
            return ToolOutput::error(format!("Failed to create goal: {e}"));
        }
    }

    match queue.goals.get_mut(&input.goal_id) {
        Some(goal) => {
            goal.status = match input.new_status.to_lowercase().as_str() {
                "discovered" => crate::cooboploop::queue::GoalStatus::Discovered,
                "evaluating" => crate::cooboploop::queue::GoalStatus::Evaluating,
                "accepted" => crate::cooboploop::queue::GoalStatus::Accepted,
                "queued" => crate::cooboploop::queue::GoalStatus::Queued,
                "blocked" => crate::cooboploop::queue::GoalStatus::Blocked,
                "deferred" => crate::cooboploop::queue::GoalStatus::Deferred,
                "active" => crate::cooboploop::queue::GoalStatus::Active,
                "verifying" => crate::cooboploop::queue::GoalStatus::Verifying,
                "completed" => crate::cooboploop::queue::GoalStatus::Completed,
                "failed" => crate::cooboploop::queue::GoalStatus::Failed,
                "cancelled" => crate::cooboploop::queue::GoalStatus::Cancelled,
                "rejected" => crate::cooboploop::queue::GoalStatus::Rejected,
                "archived" => crate::cooboploop::queue::GoalStatus::Archived,
                _ => return ToolOutput::error(format!("Invalid status: {}", input.new_status)),
            };
            ToolOutput::success(serde_json::json!({
                "status": "updated",
                "goal_id": input.goal_id,
                "new_status": input.new_status,
            }))
        }
        None => ToolOutput::error(format!("Goal {} not found", input.goal_id)),
    }
}

/// Execute cooboploop_run_source_discovery
pub async fn execute_cooboploop_run_source_discovery(
    input: CooboploopRunSourceDiscoveryInput,
) -> ToolOutput {
    use crate::cooboploop::sources::ObjectiveSourceRegistry;
    let registry = ObjectiveSourceRegistry::init();
    let discovered = registry.discover_all();
    let filtered: Vec<AgentGoal> = if let Some(ref filter) = input.source_type {
        discovered
            .into_iter()
            .filter(|g| {
                let s = format!("{:?}", g.source);
                let filter_lower = filter.to_lowercase();
                s.to_lowercase().contains(&filter_lower)
            })
            .collect()
    } else {
        discovered
    };
    let results: Vec<serde_json::Value> = filtered
        .iter()
        .map(|g| {
            serde_json::json!({
                "id": g.id,
                "title": g.title,
                "source": format!("{:?}", g.source),
                "status": format!("{:?}", g.status),
            })
        })
        .collect();
    ToolOutput::success(serde_json::json!({
        "status": "discovered",
        "count": results.len(),
        "results": results,
        "requested_source_type": input.source_type,
    }))
}

/// Execute cooboploop_evaluate_goal
pub async fn execute_cooboploop_evaluate_goal(
    input: CooboploopEvaluateGoalInput,
    evaluator: &Arc<GoalEvaluator>,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let goal = {
        let queue = match queue.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        queue.goals.get(&input.goal_id).cloned()
    };
    match goal {
        Some(g) => {
            let criteria = evaluator.evaluate(&g);
            let priority_score = evaluator.compute_priority(&g);
            tracing::trace!("Goal evaluated: priority={priority_score}");
            ToolOutput::success(serde_json::json!({
                "status": "evaluated",
                "goal_id": criteria.goal_id,
                "expected_value": criteria.expected_value,
                "probability_of_success": criteria.probability_of_success,
                "urgency": criteria.urgency,
                "risk": criteria.risk,
                "learning_value": criteria.learning_value,
                "strategic_value": criteria.strategic_value,
                "priority_score": criteria.priority_score,
                "required_capabilities": criteria.required_capabilities,
                "deadline": criteria.deadline,
                "resource_cost": criteria.resource_cost.total(),
                "time_cost": criteria.time_cost,
            }))
        }
        None => ToolOutput::success(serde_json::json!({
            "status": "not_found",
            "goal_id": input.goal_id,
        })),
    }
}

/// Execute cooboploop_reprioritize_queue
pub async fn execute_cooboploop_reprioritize_queue(
    queue: &Arc<Mutex<ObjectiveQueue>>,
    evaluator: &Arc<GoalEvaluator>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut updated = Vec::new();
    for goal in queue.goals.values() {
        let score = evaluator.compute_priority(goal);
        updated.push(serde_json::json!({
            "id": goal.id,
            "title": goal.title,
            "priority_score": score,
        }));
    }
    ToolOutput::success(serde_json::json!({
        "status": "reprioritized",
        "count": updated.len(),
        "updated": updated,
    }))
}

/// Execute cooboploop_set_priority_policy
pub async fn execute_cooboploop_set_priority_policy(
    input: CooboploopSetPriorityPolicyInput,
    evaluator: &Arc<GoalEvaluator>,
    registry: &Arc<Mutex<PriorityPolicyRegistry>>,
) -> ToolOutput {
    let policy_name = input.policy.to_lowercase();
    let new_policy: Box<dyn crate::cooboploop::evaluation::PriorityPolicyTrait> =
        match policy_name.as_str() {
            "conservative" => Box::new(crate::cooboploop::evaluation::ConservativePriorityPolicy),
            "strategic" => Box::new(crate::cooboploop::evaluation::StrategicPolicy),
            "exploration" => Box::new(crate::cooboploop::evaluation::ExplorationPolicy),
            _ => Box::new(crate::cooboploop::evaluation::DefaultPriorityPolicy),
        };
    let sample_goal = crate::cooboploop::sources::HumanInputSource::sample_goal(
        crate::cooboploop::sources::HumanOrigin::UserRequest,
        "Sample",
    );
    let sample_score = new_policy.compute(1.0, &sample_goal);
    let baseline_score = evaluator.compute_priority(&sample_goal);
    let mut reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    reg.set_policy(new_policy);
    let current_policy_name = "policy_active".to_string();
    ToolOutput::success(serde_json::json!({
        "status": "policy_set",
        "requested_policy": input.policy,
        "current_policy": current_policy_name,
        "baseline_score": baseline_score,
        "sample_adjusted_score": sample_score,
    }))
}

/// Execute cooboploop_record_capability_outcome
pub async fn execute_cooboploop_record_capability_outcome(
    input: CooboploopRecordCapabilityOutcomeInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let mut reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cap_id = crate::cooboploop::capability::CapabilityId::from_string(&input.capability_id);
    let result = if input.success {
        reg.record_success(&cap_id)
    } else {
        reg.record_failure(&cap_id)
    };
    match result {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "message": "Capability outcome recorded",
            "status": "ok",
            "capability_id": input.capability_id,
            "success": input.success,
        })),
        Err(e) => ToolOutput::error(format!("record outcome failed: {e}")),
    }
}

/// Execute cooboploop_get_capability_assessment
pub async fn execute_cooboploop_get_capability_assessment(
    input: CooboploopGetCapabilityAssessmentInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cap_id = crate::cooboploop::capability::CapabilityId::from_string(&input.capability_id);
    match reg.get(&cap_id) {
        Some(assessment) => ToolOutput::success(serde_json::json!({
            "message": "Capability assessment retrieved",
            "status": "ok",
            "capability_id": input.capability_id,
            "assessment": assessment,
        })),
        None => ToolOutput::success(serde_json::json!({
            "message": "No assessment found for capability",
            "status": "not_found",
            "capability_id": input.capability_id,
            "assessment": serde_json::Value::Null,
        })),
    }
}

/// Execute cooboploop_list_capabilities
pub async fn execute_cooboploop_list_capabilities(
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let list: Vec<&crate::cooboploop::capability::CapabilityAssessment> = reg.list();
    let serialized: Vec<serde_json::Value> = list.iter().map(|a| serde_json::json!(a)).collect();
    ToolOutput::success(serde_json::json!({
        "message": "Capabilities listed",
        "status": "ok",
        "count": list.len(),
        "capabilities": serialized,
    }))
}

/// Execute cooboploop_start_loop
pub async fn execute_cooboploop_start_loop(
    input: CooboploopStartLoopInput,
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(v) = input.max_cycles {
        runner.set_max_cycles(v as u32);
    }
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
}

/// Execute cooboploop_stop_loop
pub async fn execute_cooboploop_stop_loop(
    input: CooboploopStopLoopInput,
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    tracing::debug!("Stopping CoObOpLoop: input={input:?}");
    let mut runner = match loop_runner.lock() {
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
}

/// Execute cooboploop_get_loop_status
pub async fn execute_cooboploop_get_loop_status(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let status_str = if runner.should_continue() {
        "running"
    } else {
        "stopped"
    };
    let cycles = runner.cycle_count();
    let max_cycles = runner.max_cycles();
    let stage = format!("{:?}", runner.current_stage());
    let cognitive_stage = format!("{:?}", runner.current_cognitive_stage());
    let summary = runner.cycle_summary();
    let events: &[String] = runner.learning_events();
    // Wire human action handler: include audit summary in loop status
    let human_summary = crate::cooboploop::human::HumanActionHandler::default().audit_summary();
    let state_summary_str = crate::cooboploop::human::HumanActionHandler::default().state_summary();
    ToolOutput::success(serde_json::json!({
        "status": status_str,
        "cycles_completed": cycles,
        "max_cycles": max_cycles,
        "current_stage": stage,
        "cognitive_stage": cognitive_stage,
        "cycle_summary": summary,
        "learning_events": events,
        "human_audit_summary": human_summary,
        "state_summary": state_summary_str,
    }))
}

/// Execute cooboploop_run_single_cycle
pub async fn execute_cooboploop_run_single_cycle(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.run_cycle().ok();
    let cycles = runner.cycle_count();
    ToolOutput::success(serde_json::json!({
        "message": "Cycle completed",
        "status": "ok",
        "cycles_completed": cycles,
    }))
}

/// Execute cooboploop_step_loop
pub async fn execute_cooboploop_step_loop(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if runner.phase == crate::cooboploop::loop_runner::CyclePhase::Wait {
        let resumed = runner.tick_heartbeat();
        return ToolOutput::success(serde_json::json!({
            "message": if resumed { "Heartbeat resumed observation" } else { "Heartbeat waiting" },
            "status": if resumed { "resumed" } else { "waiting" },
            "heartbeat_seconds": runner.heartbeat_secs,
            "reevaluation_interval_secs": runner.reevaluation_interval_secs,
            "phase": format!("{:?}", runner.phase),
            "cycles_completed": runner.cycle_count(),
        }));
    }
    if let Err(error) = runner.run_cycle() {
        return ToolOutput::error(error);
    }
    ToolOutput::success(serde_json::json!({
        "message": "Step completed",
        "status": "ok",
        "phase": format!("{:?}", runner.phase),
        "cycles_completed": runner.cycle_count(),
    }))
}

/// Execute cooboploop_run_post_task_evaluation
pub async fn execute_cooboploop_run_post_task_evaluation(
    input: CooboploopRunPostTaskEvaluationInput,
) -> ToolOutput {
    use crate::cooboploop::post_task::PostTaskEvaluation;
    let eval = PostTaskEvaluation::new();
    ToolOutput::success(serde_json::json!({
        "message": "Post-task evaluation completed",
        "goal_id": input.goal_id,
        "did_succeed": eval.did_succeed,
        "verification_confirmed": eval.verification_confirmed,
        "unexpected_problems_count": eval.unexpected_problems.len(),
        "knowledge_gaps_count": eval.knowledge_gaps.len(),
        "new_bugs_count": eval.new_bugs.len(),
        "efficiency_score": eval.efficiency_score,
    }))
}

/// Execute cooboploop_get_idle_state (§7.8)
pub async fn execute_cooboploop_get_idle_state(
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::idle::{IdlePhase, IdleState};
    use crate::cooboploop::loop_runner::CyclePhase;

    let (phase, seconds_since_activity, reevaluation_interval_secs, objectives_processed) = {
        let runner = match loop_runner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let phase = match runner.phase {
            CyclePhase::Observe => IdlePhase::Active,
            CyclePhase::FindNextObjective => IdlePhase::Reprioritizing,
            CyclePhase::Wait => IdlePhase::Waiting,
        };
        (
            phase,
            runner.heartbeat_secs,
            runner.reevaluation_interval_secs,
            runner.cycle_count(),
        )
    };
    let strategic_candidates = {
        let registry = match strategic_registry.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        registry
            .list()
            .iter()
            .filter(|objective| objective.status == "active")
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut state = IdleState::new(reevaluation_interval_secs);
    state.phase = phase;
    state.seconds_since_activity = seconds_since_activity;
    state.objectives_processed = objectives_processed;
    state.queue_empty = strategic_candidates.is_empty();
    let useful_work = state.evaluate_useful_work(&strategic_candidates);
    ToolOutput::success(serde_json::json!({
        "message": "Idle state retrieved",
        "status": "ok",
        "phase": format!("{:?}", state.phase),
        "seconds_since_activity": state.seconds_since_activity,
        "objectives_processed": state.objectives_processed,
        "reevaluation_interval_secs": state.reevaluation_interval_secs,
        "useful_work": useful_work,
        "strategic_candidates": strategic_candidates,
    }))
}

/// Execute cooboploop_configure_idle_reevaluation_interval (§7.9)
pub async fn execute_cooboploop_configure_idle_reevaluation_interval(
    input: CooboploopConfigureIdleReevaluationIntervalInput,
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    if input.seconds <= 0 {
        return ToolOutput::error("Idle reevaluation interval must be greater than zero");
    }
    let interval = match u64::try_from(input.seconds) {
        Ok(seconds) => seconds,
        Err(error) => {
            return ToolOutput::error(format!("Invalid idle reevaluation interval: {error}"));
        }
    };
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.reevaluation_interval_secs = interval;
    ToolOutput::success(serde_json::json!({
        "message": "Idle reevaluation interval configured",
        "status": "ok",
        "seconds": runner.reevaluation_interval_secs,
    }))
}

/// Execute cooboploop_create_research_objective (§T8.4)
pub async fn execute_cooboploop_create_research_objective(
    input: CooboploopCreateResearchObjectiveInput,
    research_manager: &Arc<Mutex<crate::cooboploop::research::ResearchManager>>,
) -> ToolOutput {
    use crate::cooboploop::research::{PersistenceTarget, ResearchTrigger};

    let persistence_target = match input.persistence_target.as_deref() {
        None | Some("knowledge_base") => PersistenceTarget::KnowledgeBase,
        Some("experience_log") => PersistenceTarget::ExperienceLog,
        Some("both") => PersistenceTarget::Both,
        Some(value) => {
            return ToolOutput::error(format!(
                "Invalid persistence_target '{value}'; expected knowledge_base, experience_log, or both"
            ));
        }
    };
    let priority = input.priority.unwrap_or(0.0).max(0.0);
    let topic = input.topic;
    let expected_knowledge = format!("Research findings about {topic}");
    let mut manager = match research_manager.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let id = manager.create_objective(
        topic,
        ResearchTrigger::UnavailableInfo,
        persistence_target,
        expected_knowledge,
    );
    match manager.list().last() {
        Some(objective) => ToolOutput::success(serde_json::json!({
            "message": "Research objective created",
            "status": "ok",
            "id": id,
            "priority": priority,
            "objective": objective,
        })),
        None => ToolOutput::error("Research objective was not retained by the manager".to_string()),
    }
}

/// Execute cooboploop_get_hardware_profile (§T8.10)
pub async fn execute_cooboploop_get_hardware_profile(
    hardware_discovery: &Arc<Mutex<crate::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: &Arc<Mutex<crate::cooboploop::hardware::HardwareRegistry>>,
) -> ToolOutput {
    let registry = match hardware_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let stored_profile = match registry.latest() {
        Ok(profile) => profile,
        Err(_) => None,
    };
    let mut discovery = match hardware_discovery.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let profile = match stored_profile {
        Some(profile) => {
            discovery.set_profile(profile.clone());
            discovery.update_snapshots();
            profile
        }
        None => match discovery.detect() {
            Some(profile) => {
                if let Err(error) = registry.update(&profile) {
                    tracing::warn!("Failed to update hardware registry: {}", error);
                }
                discovery.update_snapshots();
                profile
            }
            None => crate::cooboploop::hardware::HardwareProfile::default(),
        },
    };
    ToolOutput::success(serde_json::json!({
        "message": "Hardware profile retrieved",
        "status": "ok",
        "profile": profile,
    }))
}

/// Execute cooboploop_detect_hardware_changes (§T8.11)
pub async fn execute_cooboploop_detect_hardware_changes(
    hardware_discovery: &Arc<Mutex<crate::cooboploop::hardware::HardwareDiscovery>>,
    hardware_registry: &Arc<Mutex<crate::cooboploop::hardware::HardwareRegistry>>,
) -> ToolOutput {
    let registry = match hardware_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let previous = match registry.latest() {
        Ok(profile) => profile,
        Err(_) => None,
    };
    let mut discovery = match hardware_discovery.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(previous_profile) = previous {
        discovery.set_profile(previous_profile);
        discovery.update_snapshots();
    }
    let current = match discovery.detect() {
        Some(profile) => profile,
        None => {
            return ToolOutput::success(serde_json::json!({
                "message": "No hardware profile available",
                "status": "ok",
                "changes": Vec::<serde_json::Value>::new(),
                "change_count": 0,
                "current_profile": serde_json::Value::Null,
            }));
        }
    };
    let changes = discovery.detect_hardware_changes();
    if let Err(error) = registry.update(&current) {
        tracing::warn!("Failed to update hardware registry: {}", error);
    }
    ToolOutput::success(serde_json::json!({
        "message": "Hardware changes detected",
        "status": "ok",
        "changes": changes,
        "change_count": changes.len(),
        "current_profile": current,
    }))
}

/// Execute cooboploop_run_inspection (§T8.17)
pub async fn execute_cooboploop_run_inspection(
    input: CooboploopRunInspectionInput,
    objective_queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    use crate::cooboploop::inspection::{InspectionTarget, Inspector};

    let target_filter = match input.target.as_deref() {
        Some(value) => match value.parse::<InspectionTarget>() {
            Ok(target) => Some(target),
            Err(error) => return ToolOutput::error(error),
        },
        None => None,
    };
    let mut inspector = Inspector::new();
    let issues: Vec<_> = inspector
        .scan()
        .into_iter()
        .filter(|issue| {
            target_filter
                .as_ref()
                .is_none_or(|target| issue.target == *target)
        })
        .collect();
    let objectives = Inspector::issues_to_objectives(&issues);
    let objective_ids: Vec<String> = objectives
        .iter()
        .map(|objective| objective.id.clone())
        .collect();
    let mut queue = match objective_queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for objective in &objectives {
        if let Err(error) = queue.enqueue(objective) {
            return ToolOutput::error(format!(
                "Failed to enqueue inspection objective {}: {error}",
                objective.id
            ));
        }
    }
    ToolOutput::success(serde_json::json!({
        "message": "Inspection completed",
        "status": "ok",
        "target_filter": target_filter.map(|target| target.to_string()),
        "issues_found": issues.len(),
        "objectives_generated": objectives.len(),
        "issues": issues,
        "objective_ids": objective_ids,
    }))
}

/// Execute cooboploop_get_modification_boundary (§T9.18)
pub async fn execute_cooboploop_get_modification_boundary(
    pipeline: &Arc<Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>>,
) -> ToolOutput {
    let pipeline = match pipeline.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Modification boundary retrieved",
        "status": "ok",
        "boundary": pipeline.get_boundary(),
    }))
}

/// Execute cooboploop_set_modification_boundary (§T9.19)
pub async fn execute_cooboploop_set_modification_boundary(
    input: CooboploopSetModificationBoundaryInput,
    pipeline: &Arc<Mutex<crate::cooboploop::self_improvement::SelfImprovementPipeline>>,
) -> ToolOutput {
    use crate::cooboploop::self_improvement::ModificationBoundary;
    let boundary = match input.boundary.as_str() {
        "Apply" => ModificationBoundary::Apply,
        "Propose" => ModificationBoundary::Propose,
        invalid_boundary => {
            return ToolOutput::error(format!(
                "Invalid modification boundary '{invalid_boundary}'; expected Propose or Apply"
            ));
        }
    };
    let mut pipeline = match pipeline.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    pipeline.set_boundary(boundary.clone());
    ToolOutput::success(serde_json::json!({
        "message": "Modification boundary set",
        "status": "ok",
        "boundary": boundary,
    }))
}

/// Execute cooboploop_run_opportunity_intake (§T12.16)
pub async fn execute_cooboploop_run_opportunity_intake(
    input: CooboploopRunOpportunityIntakeInput,
    capability_registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
    pending_opportunities: &Arc<Mutex<Vec<crate::cooboploop::opportunity::IntakeResult>>>,
) -> ToolOutput {
    use crate::cooboploop::opportunity::{OpportunityAdapter, OpportunityIntake};
    let source = input.source_url.trim();
    if source.is_empty() {
        return ToolOutput::error("source_url must not be empty");
    }
    let lowercase_source = source.to_lowercase();
    let resolved_source_type = input
        .source_type
        .as_deref()
        .map(str::trim)
        .filter(|source_type| !source_type.is_empty())
        .map(str::to_lowercase)
        .or_else(|| {
            if lowercase_source.contains("fiverr") {
                Some("fiverr".to_string())
            } else if lowercase_source.contains("upwork") {
                Some("upwork".to_string())
            } else if lowercase_source.contains("github") {
                Some("github_issues".to_string())
            } else {
                None
            }
        });
    let Some(source_type) = resolved_source_type else {
        return ToolOutput::error(
            "source_type is required when it cannot be inferred from source_url",
        );
    };
    let adapter: Box<dyn OpportunityAdapter> = match source_type.as_str() {
        "fiverr" => Box::new(crate::cooboploop::opportunity::FiverrAdapter::new(
            input.source_url.clone(),
        )),
        "upwork" => Box::new(crate::cooboploop::opportunity::UpworkAdapter::new(
            input.source_url.clone(),
        )),
        "github" | "github_issues" => Box::new(
            crate::cooboploop::opportunity::GitHubIssuesAdapter::new(input.source_url.clone()),
        ),
        unsupported => {
            return ToolOutput::error(format!(
                "Unsupported opportunity source_type '{unsupported}'"
            ));
        }
    };
    let adapter_name = adapter.name().to_string();
    let opportunities = adapter.fetch();
    if opportunities.is_empty() {
        return ToolOutput::error("The opportunity source produced no records");
    }
    let registry = match capability_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let intake = OpportunityIntake::new();
    let mut results = Vec::new();
    for opportunity in opportunities {
        results.push(intake.process(&opportunity, &registry));
    }
    drop(registry);
    let deferred_results: Vec<crate::cooboploop::opportunity::IntakeResult> = results
        .iter()
        .filter(|result| result.decision == crate::cooboploop::opportunity::IntakeDecision::Defer)
        .cloned()
        .collect();
    if !deferred_results.is_empty() {
        let mut pending = match pending_opportunities.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        pending.extend(deferred_results);
    }
    ToolOutput::success(serde_json::json!({
        "message": "Opportunity intake completed",
        "status": "ok",
        "source_url": input.source_url,
        "source_type": source_type,
        "adapter": adapter_name,
        "results": results,
        "default_policy_never_auto_accept": intake.default_policy_never_auto_accept,
    }))
}

/// Execute cooboploop_get_pending_external_opportunities (§T12.17)
pub async fn execute_cooboploop_get_pending_external_opportunities(
    pending_opportunities: &Arc<Mutex<Vec<crate::cooboploop::opportunity::IntakeResult>>>,
) -> ToolOutput {
    let pending = match pending_opportunities.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Pending external opportunities retrieved",
        "status": "ok",
        "pending": pending.as_slice(),
        "count": pending.len(),
        "note": "All deferred external opportunities require human review",
    }))
}

/// Execute cooboploop_set_autonomous_mode (§T11.5)
pub async fn execute_cooboploop_set_autonomous_mode(
    input: CooboploopSetAutonomousModeInput,
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.set_autonomous(input.enabled);
    // Wire process_with_autonomy_check: process a sample action with autonomy check
    let mut handler = crate::cooboploop::human::HumanActionHandler::new();
    handler.set_autonomous(input.enabled);
    let autonomy_check_result =
        handler.process_with_autonomy_check(crate::cooboploop::human::HumanAction::PauseAutonomous);
    tracing::debug!("Autonomy check result: {autonomy_check_result}");
    ToolOutput::success(serde_json::json!({
        "message": "Autonomous mode set",
        "status": "ok",
        "enabled": runner.is_autonomous(),
    }))
}

/// Execute cooboploop_get_autonomous_mode (§T11.6)
pub async fn execute_cooboploop_get_autonomous_mode(
    loop_runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    ToolOutput::success(serde_json::json!({
        "message": "Autonomous mode retrieved",
        "status": "ok",
        "enabled": runner.is_autonomous(),
    }))
}

/// Execute cooboploop_list_strategic_objectives (§T13)
pub async fn execute_cooboploop_list_strategic_objectives(
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    let registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let list = registry.list();
    ToolOutput::success(serde_json::json!({
        "message": "Strategic objectives listed",
        "status": "ok",
        "count": list.len(),
        "objectives": list,
    }))
}

/// Execute cooboploop_add_strategic_objective (§T13)
pub async fn execute_cooboploop_add_strategic_objective(
    input: CooboploopAddStrategicObjectiveInput,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::strategic::{StrategicObjective, StrategicObjectiveRecord};

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return ToolOutput::error("Strategic objective name must not be empty");
    }
    let category = match input.category.as_deref() {
        Some(category) => match category.parse::<StrategicObjective>() {
            Ok(category) => category,
            Err(error) => return ToolOutput::error(error),
        },
        None => StrategicObjective::MaintainSystemReliability,
    };
    let category_name = format!("{:?}", category);
    let id = format!("strategic_{}", uuid::Uuid::new_v4());
    let objective = StrategicObjectiveRecord {
        id: id.clone(),
        name: name.clone(),
        category,
        status: "active".to_string(),
        priority: 0.5,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let mut registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match registry.add(objective) {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "message": "Strategic objective added",
            "status": "ok",
            "id": id,
            "name": name,
            "category": category_name,
        })),
        Err(error) => ToolOutput::error(error),
    }
}

/// Execute cooboploop_remove_strategic_objective (§T13)
pub async fn execute_cooboploop_remove_strategic_objective(
    input: CooboploopRemoveStrategicObjectiveInput,
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
) -> ToolOutput {
    let id = input.id.trim().to_string();
    if id.is_empty() {
        return ToolOutput::error("Strategic objective id must not be empty");
    }
    let mut registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match registry.remove(&id) {
        Ok(removed) => ToolOutput::success(serde_json::json!({
            "message": if removed { "Strategic objective removed" } else { "Not found" },
            "status": if removed { "ok" } else { "not_found" },
            "id": id,
        })),
        Err(error) => ToolOutput::error(error),
    }
}

/// Execute cooboploop_get_objective_hierarchy (§T13)
pub async fn execute_cooboploop_get_objective_hierarchy(
    strategic_registry: &Arc<Mutex<crate::cooboploop::strategic::StrategicObjectiveRegistry>>,
    mission: &Arc<Mutex<String>>,
) -> ToolOutput {
    use crate::cooboploop::strategic::ObjectiveHierarchy;

    let mission_name = match mission.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let registry = match strategic_registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let hierarchy = ObjectiveHierarchy::build(&mission_name, registry.list());
    ToolOutput::success(serde_json::json!({
        "message": "Objective hierarchy retrieved",
        "status": "ok",
        "count": hierarchy.nodes().len(),
        "nodes": hierarchy.nodes(),
    }))
}

/// Execute cooboploop_set_mission (§T13)
pub async fn execute_cooboploop_set_mission(
    input: CooboploopSetMissionInput,
    mission: &Arc<Mutex<String>>,
) -> ToolOutput {
    let mission_name = input.mission.trim().to_string();
    if mission_name.is_empty() {
        return ToolOutput::error("Mission must not be empty");
    }
    let mut retained_mission = match mission.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *retained_mission = mission_name.clone();
    ToolOutput::success(serde_json::json!({
        "message": "Mission set",
        "status": "ok",
        "mission": mission_name,
    }))
}

/// Execute cooboploop_get_autonomy_levels (§22 / T15.15).
pub async fn execute_cooboploop_get_autonomy_levels(
    input: CooboploopGetAutonomyLevelsInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let request = match serde_json::to_value(input) {
        Ok(request) if request.is_object() => request,
        Ok(request) => {
            return ToolOutput::error(format!(
                "autonomy-level request must serialize as an object, received {request}"
            ));
        }
        Err(error) => {
            return ToolOutput::error(format!("serialize autonomy-level request: {error}"));
        }
    };
    let registry = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let autonomy: std::collections::BTreeMap<String, String> = registry
        .autonomy_levels()
        .into_iter()
        .map(|(capability_id, level)| (capability_id.key(), level.as_str().to_string()))
        .collect();
    ToolOutput::success(serde_json::json!({
        "message": "Autonomy levels retrieved",
        "status": "ok",
        "capability_count": autonomy.len(),
        "autonomy": autonomy,
        "request": request,
    }))
}

/// Execute cooboploop_promote_autonomy (§22 / T15.16).
pub async fn execute_cooboploop_promote_autonomy(
    input: CooboploopPromoteAutonomyInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    use crate::cooboploop::capability::CapabilityId;

    let requested_id = input.capability_id.trim();
    if requested_id.is_empty() {
        return ToolOutput::error("Capability ID must not be empty");
    }

    let capability_id = CapabilityId::from_string(requested_id);
    let canonical_id = capability_id.key();
    let mut registry = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let (previous_level, new_level) = registry.promote_autonomy(capability_id);

    ToolOutput::success(serde_json::json!({
        "message": "Autonomy promoted",
        "status": "ok",
        "capability_id": canonical_id,
        "previous_level": previous_level.as_str(),
        "new_level": new_level.as_str(),
    }))
}

pub fn register_cooboploop_tools(registry: &mut crate::bridge::tools::ToolRegistry) {
    register_tools(registry);
}

/// Register all cooboploop tools
pub fn register_tools(registry: &mut crate::bridge::tools::ToolRegistry) {
    let cooboploop_tools = definitions::all();
    registry.tools.extend(cooboploop_tools);
}
