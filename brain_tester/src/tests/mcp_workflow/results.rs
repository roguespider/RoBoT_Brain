//! Result structs for MCP Workflow tests

/// MCP Workflow Integration Test Suite
///
/// This test suite validates that the agent will correctly use MCP workflows
/// for various task types and scenarios.
pub struct McpWorkflowTestResults {
    pub workflow_discovery: WorkflowDiscoveryResults,
    pub workflow_execution: WorkflowExecutionResults,
    pub workflow_tools: WorkflowToolsResults,
    pub agent_workflow_integration: AgentWorkflowIntegrationResults,
    pub end_to_end_scenarios: EndToEndScenarioResults,
    pub mcp_protocol_valid: bool,
}

pub struct WorkflowDiscoveryResults {
    pub get_workflow_available: bool,
    pub default_workflow_retrieved: bool,
    pub purpose_based_workflows: Vec<String>,
    pub workflow_rules_understood: bool,
}

pub struct WorkflowExecutionResults {
    pub create_workflow_succeeds: bool,
    pub workflow_id_generated: Option<String>,
    pub add_step_succeeds: bool,
    pub start_workflow_succeeds: bool,
    pub workflow_completes: bool,
    pub pause_resume_works: bool,
}

pub struct WorkflowToolsResults {
    pub total_tools: usize,
    pub workflow_tools: Vec<String>,
    pub agent_tools: Vec<String>,
    pub workflow_tool_definitions_valid: bool,
}

pub struct AgentWorkflowIntegrationResults {
    pub agent_discovers_workflow_first: bool,
    pub agent_uses_correct_workflow_for_purpose: bool,
    pub agent_chains_workflow_steps: bool,
    pub agent_respects_workflow_dependencies: bool,
}

pub struct EndToEndScenarioResults {
    pub file_ingestion_workflow: bool,
    pub memory_search_workflow: bool,
    pub experience_recording_workflow: bool,
    pub multi_step_workflow: bool,
}
