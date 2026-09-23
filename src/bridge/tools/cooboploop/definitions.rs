// src/bridge/tools/cooboploop/definitions.rs
//! CoObOpLoop tool definitions and schema

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
