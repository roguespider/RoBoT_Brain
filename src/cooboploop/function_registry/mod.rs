// Function registry for CoObOpLoop tools (§T17 / §23)
// Maps each MCP tool to its implementation module for coverage verification.

pub const COOBOPLOOP_REGISTRY: &[(&str, &str, &str)] = &[
    ("cooboploop_enqueue_goal", "queue", "enqueue"),
    ("cooboploop_list_goals", "queue", "list"),
    ("cooboploop_get_goal", "queue", "get"),
    ("cooboploop_update_goal_status", "queue", "update_status"),
    ("cooboploop_run_source_discovery", "sources", "discover_all"),
    ("cooboploop_evaluate_goal", "evaluation", "evaluate"),
    (
        "cooboploop_reprioritize_queue",
        "evaluation",
        "reprioritize",
    ),
    ("cooboploop_set_priority_policy", "evaluation", "set_policy"),
    (
        "cooboploop_record_capability_outcome",
        "capability",
        "record",
    ),
    ("cooboploop_get_capability_assessment", "capability", "get"),
    ("cooboploop_list_capabilities", "capability", "list"),
    ("cooboploop_start_loop", "loop_runner", "start"),
    ("cooboploop_stop_loop", "loop_runner", "stop"),
    ("cooboploop_get_loop_status", "loop_runner", "status"),
    ("cooboploop_run_single_cycle", "loop_runner", "run_cycle"),
    ("cooboploop_step_loop", "loop_runner", "step"),
    (
        "cooboploop_run_post_task_evaluation",
        "post_task",
        "evaluate",
    ),
    ("cooboploop_get_idle_state", "idle", "get_state"),
    (
        "cooboploop_configure_idle_reevaluation_interval",
        "idle",
        "set_interval",
    ),
    ("cooboploop_create_research_objective", "research", "create"),
    ("cooboploop_get_hardware_profile", "hardware", "profile"),
    ("cooboploop_detect_hardware_changes", "hardware", "detect"),
    ("cooboploop_run_inspection", "inspection", "run"),
    (
        "cooboploop_get_modification_boundary",
        "self_improvement",
        "get_boundary",
    ),
    (
        "cooboploop_set_modification_boundary",
        "self_improvement",
        "set_boundary",
    ),
    ("cooboploop_set_autonomous_mode", "human", "set_autonomous"),
    ("cooboploop_get_autonomous_mode", "human", "get_autonomous"),
    ("cooboploop_run_opportunity_intake", "opportunity", "intake"),
    (
        "cooboploop_get_pending_external_opportunities",
        "opportunity",
        "pending",
    ),
    ("cooboploop_list_strategic_objectives", "strategic", "list"),
    ("cooboploop_add_strategic_objective", "strategic", "add"),
    (
        "cooboploop_remove_strategic_objective",
        "strategic",
        "remove",
    ),
    (
        "cooboploop_get_objective_hierarchy",
        "strategic",
        "hierarchy",
    ),
    ("cooboploop_set_mission", "strategic", "set_mission"),
    (
        "cooboploop_get_autonomy_levels",
        "capability",
        "autonomy_levels",
    ),
    ("cooboploop_promote_autonomy", "capability", "promote"),
];

pub fn all_registered() -> Vec<&'static str> {
    COOBOPLOOP_REGISTRY
        .iter()
        .map(|(name, _, _)| *name)
        .collect()
}
