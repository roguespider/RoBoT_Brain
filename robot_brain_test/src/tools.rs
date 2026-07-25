//! Complete list of all tools in the RoBoT Brain MCP server

/// Complete list of all tools in the RoBoT Brain MCP server (55 tools total)
pub fn get_all_tool_names() -> Vec<&'static str> {
    vec![
        // Memory Tools (4)
        "store_memory",
        "search_memory",
        "get_memory",
        "list_memories",
        // Experience Tools (4)
        "record_experience",
        "get_experience_stats",
        "list_experiences",
        "get_experience",
        // Knowledge Tools (5)
        "add_knowledge",
        "query_knowledge",
        "record_knowledge_application",
        "get_knowledge_stats",
        "get_mature_knowledge",
        // Planner Tools (9)
        "create_plan",
        "add_plan_step",
        "add_step_dependency",
        "get_plan",
        "list_plans",
        "start_plan",
        "complete_step",
        "fail_step",
        "cancel_plan",
        // Workflow Tools (9)
        "create_workflow",
        "add_workflow_step",
        "get_workflow_status",
        "list_workflows",
        "start_workflow",
        "pause_workflow",
        "resume_workflow",
        "cancel_workflow",
        "delete_workflow",
        // Agent Tools (5)
        "get_workflow",
        "list_tools",
        "get_tool",
        "connect_mcp_server",
        "call_tool",
        // Hypothesis Tools (9)
        "record_observation",
        "create_hypothesis",
        "add_evidence",
        "get_hypothesis",
        "list_hypotheses",
        "list_observations",
        "evaluate_hypothesis",
        "get_knowledge",
        "extract_knowledge",
        // Reflection Tools (4)
        "get_insights",
        "create_reflection",
        "analyze_patterns",
        "get_patterns",
        // Search Tools (3)
        "global_search",
        "get_recommendations",
        "get_reputation",
        // Ingestor Tools (5)
        "ingest_files",
        "list_importable",
        "transcribe_audio",
        "list_ingested_files",
        "delete_ingested_files",
    ]
}

/// Get tool names grouped by category
pub fn get_tools_by_category() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Memory", vec!["store_memory", "search_memory", "get_memory", "list_memories"]),
        ("Experience", vec!["record_experience", "get_experience_stats", "list_experiences", "get_experience"]),
        ("Knowledge", vec!["add_knowledge", "query_knowledge", "record_knowledge_application", "get_knowledge_stats", "get_mature_knowledge"]),
        ("Planner", vec!["create_plan", "add_plan_step", "add_step_dependency", "get_plan", "list_plans", "start_plan", "complete_step", "fail_step", "cancel_plan"]),
        ("Workflow", vec!["create_workflow", "add_workflow_step", "get_workflow_status", "list_workflows", "start_workflow", "pause_workflow", "resume_workflow", "cancel_workflow", "delete_workflow"]),
        ("Agent", vec!["get_workflow", "list_tools", "get_tool", "connect_mcp_server", "call_tool"]),
        ("Hypothesis", vec!["record_observation", "create_hypothesis", "add_evidence", "get_hypothesis", "list_hypotheses", "list_observations", "evaluate_hypothesis", "get_knowledge", "extract_knowledge"]),
        ("Reflection", vec!["get_insights", "create_reflection", "analyze_patterns", "get_patterns"]),
        ("Search", vec!["global_search", "get_recommendations", "get_reputation"]),
        ("Ingestor", vec!["ingest_files", "list_importable", "transcribe_audio", "list_ingested_files", "delete_ingested_files"]),
    ]
}
