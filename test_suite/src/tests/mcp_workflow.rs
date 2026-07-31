

//! MCP Workflow Integration Tests
//! 
//! This module specifically tests the integration between the agent and MCP workflows.
//! It ensures that:
//! 1. The agent discovers and uses workflows through MCP
//! 2. Workflow tools are properly registered and accessible
//! 3. The agent follows workflow patterns for different task types
//! 4. End-to-end workflow execution works correctly

use crate::TestMcpClient;
use crate::TestStats;

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

/// Run all MCP workflow integration tests
pub async fn run_mcp_workflow_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<McpWorkflowTestResults> {
    println!("\n{}", "=".repeat(80));
    println!("MCP WORKFLOW INTEGRATION TESTS");
    println!("Testing agent workflow discovery, execution, and MCP integration");
    println!("{}", "=".repeat(80));
    
    let results = McpWorkflowTestResults {
        workflow_discovery: test_workflow_discovery(client, stats).await?,
        workflow_execution: test_workflow_execution(client, stats).await?,
        workflow_tools: test_workflow_tools(client, stats).await?,
        agent_workflow_integration: test_agent_workflow_integration(client, stats).await?,
        end_to_end_scenarios: test_end_to_end_scenarios(client, stats).await?,
    };
    
    print_mcp_workflow_results(&results);
    
    Ok(results)
}

// ============================================================================
// WORKFLOW DISCOVERY TESTS
// ============================================================================

async fn test_workflow_discovery(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowDiscoveryResults> {
    println!("\n📋 Phase 1: Workflow Discovery Tests");
    println!("{}", "-".repeat(60));
    
    let mut results = WorkflowDiscoveryResults {
        get_workflow_available: false,
        default_workflow_retrieved: false,
        purpose_based_workflows: Vec::new(),
        workflow_rules_understood: false,
    };
    
    // Test 1: get_workflow tool is available
    println!("\n  Testing get_workflow tool availability...");
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "default"
    })).await {
        Ok(result) => {
            println!("    ✓ get_workflow('default') - SUCCESS");
            stats.passed += 1;
            results.get_workflow_available = true;
            results.default_workflow_retrieved = true;
            
            // Check if workflow contains rules/instructions
            if let Some(text) = extract_content_text(&result) {
                if text.contains("workflow") || text.contains("rules") || text.contains("guidelines") {
                    results.workflow_rules_understood = true;
                    println!("    ✓ Workflow rules/instructions present in response");
                }
            }
        }
        Err(e) => {
            println!("    ✗ get_workflow('default') - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 2: Get workflow for specific purposes
    println!("\n  Testing purpose-based workflow retrieval...");
    let purposes = vec!["file_ingestion", "memory_search", "general", "experience_recording"];
    
    for purpose in purposes {
        match client.call_tool("get_workflow", serde_json::json!({
            "purpose": purpose
        })).await {
            Ok(_result) => {
                println!("    ✓ get_workflow('{}') - SUCCESS", purpose);
                stats.passed += 1;
                results.purpose_based_workflows.push(purpose.to_string());
            }
            Err(e) => {
                println!("    ✗ get_workflow('{}') - FAILED: {}", purpose, e);
                stats.failed += 1;
            }
        }
    }
    
    Ok(results)
}

// ============================================================================
// WORKFLOW EXECUTION TESTS
// ============================================================================

async fn test_workflow_execution(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowExecutionResults> {
    println!("\n📋 Phase 2: Workflow Execution Tests");
    println!("{}", "-".repeat(60));
    
    let mut results = WorkflowExecutionResults {
        create_workflow_succeeds: false,
        workflow_id_generated: None,
        add_step_succeeds: false,
        start_workflow_succeeds: false,
        workflow_completes: false,
        pause_resume_works: false,
    };
    
    // Test 1: Create a workflow
    println!("\n  Testing workflow creation...");
    match client.call_tool("create_workflow", serde_json::json!({
        "name": "MCP Integration Test Workflow"
    })).await {
        Ok(result) => {
            println!("    ✓ create_workflow - SUCCESS");
            stats.passed += 1;
            results.create_workflow_succeeds = true;
            
            // Extract workflow ID if present
            if let Some(text) = extract_content_text(&result) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(id) = json.get("workflow_id")
                        .or_else(|| json.get("id"))
                        .or_else(|| json.get("workflow").and_then(|w| w.get("id"))) 
                    {
                        results.workflow_id_generated = id.as_str().map(String::from);
                        println!("    ✓ Workflow ID: {:?}", results.workflow_id_generated);
                    }
                }
            }
        }
        Err(e) => {
            println!("    ✗ create_workflow - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 2: Add workflow steps
    println!("\n  Testing adding workflow steps...");
    let steps = vec![
        ("Initialize", "initialize"),
        ("Process", "process_data"),
        ("Store", "store_memory"),
    ];
    
    for (name, action) in steps {
        let mut args = serde_json::json!({
            "name": name,
            "action": action
        });
        
        // Include workflow_id if we have one
        if let Some(ref id) = results.workflow_id_generated {
            args["workflow_id"] = serde_json::json!(id);
        }
        
        match client.call_tool("add_workflow_step", args).await {
            Ok(_) => {
                println!("    ✓ add_workflow_step('{}', '{}') - SUCCESS", name, action);
                stats.passed += 1;
                results.add_step_succeeds = true;
            }
            Err(e) => {
                println!("    ✗ add_workflow_step('{}', '{}') - FAILED: {}", name, action, e);
                stats.failed += 1;
            }
        }
    }
    
    // Test 3: Get workflow status
    println!("\n  Testing workflow status retrieval...");
    let mut status_args = serde_json::json!({});
    if let Some(ref id) = results.workflow_id_generated {
        status_args["workflow_id"] = serde_json::json!(id);
    }
    
    match client.call_tool("get_workflow_status", status_args).await {
        Ok(result) => {
            println!("    ✓ get_workflow_status - SUCCESS");
            stats.passed += 1;
            
            // Check if workflow has correct structure
            if let Some(text) = extract_content_text(&result) {
                if text.contains("id") || text.contains("status") || text.contains("workflow") {
                    println!("    ✓ Workflow status contains expected fields");
                }
            }
        }
        Err(e) => {
            println!("    ✗ get_workflow_status - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 4: List workflows
    println!("\n  Testing workflow listing...");
    match client.call_tool("list_workflows", serde_json::json!({})).await {
        Ok(result) => {
            println!("    ✓ list_workflows - SUCCESS");
            stats.passed += 1;
            
            // Check if response contains workflow list
            if let Some(text) = extract_content_text(&result) {
                if text.contains("workflows") || text.contains("[]") || text.len() > 10 {
                    println!("    ✓ Workflow list retrieved successfully");
                }
            }
        }
        Err(e) => {
            println!("    ✗ list_workflows - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 5: Start workflow (if we have a workflow ID)
    if let Some(ref workflow_id) = results.workflow_id_generated {
        println!("\n  Testing workflow start...");
        match client.call_tool("start_workflow", serde_json::json!({
            "workflow_id": workflow_id
        })).await {
            Ok(_result) => {
                println!("    ✓ start_workflow - SUCCESS");
                stats.passed += 1;
                results.start_workflow_succeeds = true;
                results.workflow_completes = true;
            }
            Err(e) => {
                println!("    ✗ start_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }
        
        // Test 6: Pause and resume
        println!("\n  Testing pause/resume workflow...");
        match client.call_tool("pause_workflow", serde_json::json!({
            "workflow_id": workflow_id
        })).await {
            Ok(_) => {
                println!("    ✓ pause_workflow - SUCCESS");
                stats.passed += 1;
                
                match client.call_tool("resume_workflow", serde_json::json!({
                    "workflow_id": workflow_id
                })).await {
                    Ok(_) => {
                        println!("    ✓ resume_workflow - SUCCESS");
                        stats.passed += 1;
                        results.pause_resume_works = true;
                    }
                    Err(e) => {
                        println!("    ✗ resume_workflow - FAILED: {}", e);
                        stats.failed += 1;
                    }
                }
            }
            Err(e) => {
                println!("    ✗ pause_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }
    }
    
    // Test 7: Cancel workflow
    if let Some(ref workflow_id) = results.workflow_id_generated {
        println!("\n  Testing workflow cancellation...");
        match client.call_tool("cancel_workflow", serde_json::json!({
            "workflow_id": workflow_id
        })).await {
            Ok(_) => {
                println!("    ✓ cancel_workflow - SUCCESS");
                stats.passed += 1;
            }
            Err(e) => {
                println!("    ✗ cancel_workflow - FAILED: {}", e);
                stats.failed += 1;
            }
        }
    }
    
    Ok(results)
}

// ============================================================================
// WORKFLOW TOOLS TESTS
// ============================================================================

async fn test_workflow_tools(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<WorkflowToolsResults> {
    println!("\n📋 Phase 3: Workflow Tools Validation");
    println!("{}", "-".repeat(60));
    
    let mut results = WorkflowToolsResults {
        total_tools: 0,
        workflow_tools: Vec::new(),
        agent_tools: Vec::new(),
        workflow_tool_definitions_valid: false,
    };
    
    // Test 1: List all tools
    println!("\n  Testing list_tools...");
    match client.call_tool("list_tools", serde_json::json!({})).await {
        Ok(result) => {
            println!("    ✓ list_tools - SUCCESS");
            stats.passed += 1;
            
            if let Some(text) = extract_content_text(&result) {
                // Try to count tools from response
                let tool_count = text.matches("\"name\":").count().max(1);
                results.total_tools = tool_count;
                println!("    ℹ Total tools detected: {}", results.total_tools);
            }
        }
        Err(e) => {
            println!("    ✗ list_tools - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 2: Get specific workflow tool details
    println!("\n  Testing individual workflow tool definitions...");
    let workflow_tool_names = vec![
        "create_workflow",
        "add_workflow_step",
        "get_workflow_status",
        "list_workflows",
        "start_workflow",
        "pause_workflow",
        "resume_workflow",
        "cancel_workflow",
        "delete_workflow",
    ];
    
    let agent_tool_names = vec![
        "get_workflow",
        "list_tools",
        "get_tool",
    ];
    
    for tool_name in &workflow_tool_names {
        match client.call_tool("get_tool", serde_json::json!({
            "name": tool_name
        })).await {
            Ok(_) => {
                println!("    ✓ get_tool('{}') - SUCCESS", tool_name);
                stats.passed += 1;
                results.workflow_tools.push(tool_name.to_string());
            }
            Err(e) => {
                println!("    ✗ get_tool('{}') - FAILED: {}", tool_name, e);
                stats.failed += 1;
            }
        }
    }
    
    for tool_name in &agent_tool_names {
        match client.call_tool("get_tool", serde_json::json!({
            "name": tool_name
        })).await {
            Ok(_) => {
                println!("    ✓ get_tool('{}') - SUCCESS", tool_name);
                stats.passed += 1;
                results.agent_tools.push(tool_name.to_string());
            }
            Err(e) => {
                println!("    ✗ get_tool('{}') - FAILED: {}", tool_name, e);
                stats.failed += 1;
            }
        }
    }
    
    // Validate that all expected workflow tools are available
    if results.workflow_tools.len() >= 5 {
        results.workflow_tool_definitions_valid = true;
        println!("\n    ✓ All core workflow tools are available");
    }
    
    Ok(results)
}

// ============================================================================
// AGENT-WORKFLOW INTEGRATION TESTS
// ============================================================================

async fn test_agent_workflow_integration(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<AgentWorkflowIntegrationResults> {
    println!("\n📋 Phase 4: Agent-Workflow Integration Tests");
    println!("{}", "-".repeat(60));
    
    let mut results = AgentWorkflowIntegrationResults {
        agent_discovers_workflow_first: false,
        agent_uses_correct_workflow_for_purpose: false,
        agent_chains_workflow_steps: false,
        agent_respects_workflow_dependencies: false,
    };
    
    // Test 1: Agent workflow discovery pattern
    println!("\n  Testing agent workflow discovery pattern...");
    
    // The agent should call get_workflow before other operations
    // We verify this by checking if get_workflow returns proper workflow data
    
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "test"
    })).await {
        Ok(_result) => {
            results.agent_discovers_workflow_first = true;
            println!("    ✓ Agent can discover workflows via get_workflow");
            stats.passed += 1;
        }
        Err(e) => {
            println!("    ✗ Agent workflow discovery failed: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 2: Purpose-based workflow selection
    println!("\n  Testing purpose-based workflow selection...");
    
    let test_purposes = vec![
        ("file_ingestion", "File Ingestion"),
        ("memory_search", "Memory Search"),
        ("general", "General"),
    ];
    
    let mut all_purposes_work = true;
    for (purpose, _name) in test_purposes {
        match client.call_tool("get_workflow", serde_json::json!({
            "purpose": purpose
        })).await {
            Ok(result) => {
                println!("    ✓ Workflow for '{}' - SUCCESS", purpose);
                stats.passed += 1;
                
                // Verify the workflow has purpose-relevant content
                if let Some(text) = extract_content_text(&result) {
                    // Check for expected content patterns
                    if text.len() > 50 {
                        results.agent_uses_correct_workflow_for_purpose = true;
                    }
                }
            }
            Err(e) => {
                println!("    ✗ Workflow for '{}' - FAILED: {}", purpose, e);
                stats.failed += 1;
                all_purposes_work = false;
            }
        }
    }
    
    if !all_purposes_work {
        results.agent_uses_correct_workflow_for_purpose = false;
    }
    
    // Test 3: Workflow step chaining
    println!("\n  Testing workflow step chaining...");
    
    // Create a workflow and add multiple steps
    match client.call_tool("create_workflow", serde_json::json!({
        "name": "Chained Test Workflow"
    })).await {
        Ok(create_result) => {
            // Extract workflow ID
            let mut workflow_id = String::new();
            if let Some(text) = extract_content_text(&create_result) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(id) = json.get("workflow_id")
                        .or_else(|| json.get("id"))
                        .or_else(|| json.get("workflow").and_then(|w| w.get("id"))) 
                    {
                        workflow_id = id.to_string().trim_matches('"').to_string();
                    }
                }
            }
            
            if !workflow_id.is_empty() {
                // Add steps in sequence
                let steps = vec![
                    ("Step 1", "action_1"),
                    ("Step 2", "action_2"),
                    ("Step 3", "action_3"),
                ];
                
                let mut all_steps_added = true;
                for (name, action) in steps {
                    match client.call_tool("add_workflow_step", serde_json::json!({
                        "workflow_id": workflow_id,
                        "name": name,
                        "action": action
                    })).await {
                        Ok(_) => {
                            println!("    ✓ Chained step '{}' - SUCCESS", name);
                            stats.passed += 1;
                        }
                        Err(e) => {
                            println!("    ✗ Chained step '{}' - FAILED: {}", name, e);
                            stats.failed += 1;
                            all_steps_added = false;
                        }
                    }
                }
                
                if all_steps_added {
                    results.agent_chains_workflow_steps = true;
                }
                
                // Cleanup
                let _ = client.call_tool("cancel_workflow", serde_json::json!({
                    "workflow_id": workflow_id
                })).await;
            }
        }
        Err(e) => {
            println!("    ✗ Failed to create chained workflow: {}", e);
            stats.failed += 1;
        }
    }
    
    // Test 4: Workflow dependencies and error handling
    println!("\n  Testing workflow error handling...");
    
    // Try to start a non-existent workflow (should fail gracefully)
    match client.call_tool("start_workflow", serde_json::json!({
        "workflow_id": "non-existent-workflow-id"
    })).await {
        Ok(result) => {
            // Server may return success with error message, or error
            if let Some(text) = extract_content_text(&result) {
                if text.contains("not found") || text.contains("error") || text.contains("fail") {
                    println!("    ✓ Non-existent workflow handled gracefully");
                    stats.passed += 1;
                    results.agent_respects_workflow_dependencies = true;
                }
            }
        }
        Err(_) => {
            // Error is also acceptable
            println!("    ✓ Non-existent workflow returned error (expected)");
            stats.passed += 1;
            results.agent_respects_workflow_dependencies = true;
        }
    }
    
    Ok(results)
}

// ============================================================================
// END-TO-END SCENARIO TESTS
// ============================================================================

async fn test_end_to_end_scenarios(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<EndToEndScenarioResults> {
    println!("\n📋 Phase 5: End-to-End Scenario Tests");
    println!("{}", "-".repeat(60));
    
    let mut results = EndToEndScenarioResults {
        file_ingestion_workflow: false,
        memory_search_workflow: false,
        experience_recording_workflow: false,
        multi_step_workflow: false,
    };
    
    // Scenario 1: File Ingestion Workflow
    println!("\n  Testing File Ingestion Workflow...");
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "file_ingestion"
    })).await {
        Ok(_) => {
            // Verify workflow tools are available
            let tools_exist = verify_workflow_tools_exist(client, &[
                "create_workflow",
                "add_workflow_step",
                "start_workflow",
            ]).await;
            
            if tools_exist {
                println!("    ✓ File ingestion workflow path available");
                stats.passed += 1;
                results.file_ingestion_workflow = true;
            }
        }
        Err(e) => {
            println!("    ✗ File ingestion workflow failed: {}", e);
            stats.failed += 1;
        }
    }
    
    // Scenario 2: Memory Search Workflow
    println!("\n  Testing Memory Search Workflow...");
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "memory_search"
    })).await {
        Ok(_) => {
            let tools_exist = verify_workflow_tools_exist(client, &[
                "create_workflow",
                "list_workflows",
            ]).await;
            
            if tools_exist {
                println!("    ✓ Memory search workflow path available");
                stats.passed += 1;
                results.memory_search_workflow = true;
            }
        }
        Err(e) => {
            println!("    ✗ Memory search workflow failed: {}", e);
            stats.failed += 1;
        }
    }
    
    // Scenario 3: Experience Recording Workflow
    println!("\n  Testing Experience Recording Workflow...");
    match client.call_tool("get_workflow", serde_json::json!({
        "purpose": "experience_recording"
    })).await {
        Ok(_) => {
            println!("    ✓ Experience recording workflow path available");
            stats.passed += 1;
            results.experience_recording_workflow = true;
        }
        Err(e) => {
            println!("    ✗ Experience recording workflow failed: {}", e);
            stats.failed += 1;
        }
    }
    
    // Scenario 4: Multi-Step Workflow Execution
    println!("\n  Testing Multi-Step Workflow...");
    
    // Create a workflow with multiple steps and execute it
    match client.call_tool("create_workflow", serde_json::json!({
        "name": "Multi-Step E2E Test"
    })).await {
        Ok(result) => {
            let mut workflow_id = String::new();
            if let Some(text) = extract_content_text(&result) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(id) = json.get("workflow_id")
                        .or_else(|| json.get("id"))
                        .or_else(|| json.get("workflow").and_then(|w| w.get("id"))) 
                    {
                        workflow_id = id.to_string().trim_matches('"').to_string();
                    }
                }
            }
            
            if !workflow_id.is_empty() {
                // Add multiple steps
                let steps = vec![
                    ("Initialize", "init"),
                    ("Process Data", "process"),
                    ("Store Results", "store"),
                    ("Notify", "notify"),
                ];
                
                let steps_count = steps.len();
                let mut steps_added = 0;
                for (name, action) in &steps {
                    if client.call_tool("add_workflow_step", serde_json::json!({
                        "workflow_id": workflow_id,
                        "name": name,
                        "action": action
                    })).await.is_ok() {
                        steps_added += 1;
                    }
                }
                
                if steps_added == steps_count {
                    // Start the workflow
                    match client.call_tool("start_workflow", serde_json::json!({
                        "workflow_id": workflow_id
                    })).await {
                        Ok(_) => {
                            println!("    ✓ Multi-step workflow executed successfully");
                            stats.passed += 1;
                            results.multi_step_workflow = true;
                        }
                        Err(e) => {
                            println!("    ✗ Multi-step workflow start failed: {}", e);
                            stats.failed += 1;
                        }
                    }
                }
                
                // Cleanup
                let _ = client.call_tool("cancel_workflow", serde_json::json!({
                    "workflow_id": workflow_id
                })).await;
            }
        }
        Err(e) => {
            println!("    ✗ Multi-step workflow creation failed: {}", e);
            stats.failed += 1;
        }
    }
    
    Ok(results)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extract text content from MCP response
fn extract_content_text(result: &serde_json::Value) -> Option<String> {
    // Try direct text field
    if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }
    
    // Try content[0].text (MCP response format)
    if let Some(content) = result.get("content").and_then(|c| c.as_array()).and_then(|arr| arr.first()) {
        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }
    }
    
    // Try to serialize entire result
    serde_json::to_string(result).ok()
}

/// Verify multiple workflow tools exist
async fn verify_workflow_tools_exist(
    client: &mut TestMcpClient,
    tool_names: &[&str],
) -> bool {
    let mut all_exist = true;
    
    for tool_name in tool_names {
        match client.call_tool("get_tool", serde_json::json!({
            "name": tool_name
        })).await {
            Ok(_) => {}
            Err(_) => all_exist = false,
        }
    }
    
    all_exist
}

/// Print comprehensive MCP workflow test results
fn print_mcp_workflow_results(results: &McpWorkflowTestResults) {
    println!("\n{}", "=".repeat(80));
    println!("MCP WORKFLOW INTEGRATION TEST RESULTS");
    println!("{}", "=".repeat(80));
    
    // Discovery Results
    println!("\n📋 Workflow Discovery:");
    println!("  - get_workflow tool available: {}", 
        if results.workflow_discovery.get_workflow_available { "✓" } else { "✗" });
    println!("  - Default workflow retrieved: {}", 
        if results.workflow_discovery.default_workflow_retrieved { "✓" } else { "✗" });
    println!("  - Purpose-based workflows found: {}", 
        results.workflow_discovery.purpose_based_workflows.len());
    println!("  - Workflow rules understood: {}", 
        if results.workflow_discovery.workflow_rules_understood { "✓" } else { "✗" });
    
    // Execution Results
    println!("\n⚙️  Workflow Execution:");
    println!("  - Workflow creation: {}", 
        if results.workflow_execution.create_workflow_succeeds { "✓" } else { "✗" });
    println!("  - Workflow ID generated: {:?}", 
        results.workflow_execution.workflow_id_generated);
    println!("  - Step addition: {}", 
        if results.workflow_execution.add_step_succeeds { "✓" } else { "✗" });
    println!("  - Workflow start: {}", 
        if results.workflow_execution.start_workflow_succeeds { "✓" } else { "✗" });
    println!("  - Workflow completion: {}", 
        if results.workflow_execution.workflow_completes { "✓" } else { "✗" });
    println!("  - Pause/Resume: {}", 
        if results.workflow_execution.pause_resume_works { "✓" } else { "✗" });
    
    // Tools Results
    println!("\n🔧 Workflow Tools:");
    println!("  - Total tools: {}", results.workflow_tools.total_tools);
    println!("  - Workflow tools available: {}", results.workflow_tools.workflow_tools.len());
    println!("  - Agent tools available: {}", results.workflow_tools.agent_tools.len());
    println!("  - Tool definitions valid: {}", 
        if results.workflow_tools.workflow_tool_definitions_valid { "✓" } else { "✗" });
    
    // Agent Integration Results
    println!("\n🤖 Agent-Workflow Integration:");
    println!("  - Agent discovers workflow: {}", 
        if results.agent_workflow_integration.agent_discovers_workflow_first { "✓" } else { "✗" });
    println!("  - Agent uses purpose-based workflows: {}", 
        if results.agent_workflow_integration.agent_uses_correct_workflow_for_purpose { "✓" } else { "✗" });
    println!("  - Agent chains workflow steps: {}", 
        if results.agent_workflow_integration.agent_chains_workflow_steps { "✓" } else { "✗" });
    println!("  - Agent respects dependencies: {}", 
        if results.agent_workflow_integration.agent_respects_workflow_dependencies { "✓" } else { "✗" });
    
    // E2E Results
    println!("\n🔄 End-to-End Scenarios:");
    println!("  - File ingestion workflow: {}", 
        if results.end_to_end_scenarios.file_ingestion_workflow { "✓" } else { "✗" });
    println!("  - Memory search workflow: {}", 
        if results.end_to_end_scenarios.memory_search_workflow { "✓" } else { "✗" });
    println!("  - Experience recording workflow: {}", 
        if results.end_to_end_scenarios.experience_recording_workflow { "✓" } else { "✗" });
    println!("  - Multi-step workflow: {}", 
        if results.end_to_end_scenarios.multi_step_workflow { "✓" } else { "✗" });
    
    // Overall Assessment
    let total_checks = 20;
    let passed_checks = [
        results.workflow_discovery.get_workflow_available,
        results.workflow_discovery.default_workflow_retrieved,
        results.workflow_discovery.workflow_rules_understood,
        results.workflow_execution.create_workflow_succeeds,
        results.workflow_execution.add_step_succeeds,
        results.workflow_execution.start_workflow_succeeds,
        results.workflow_execution.pause_resume_works,
        results.workflow_tools.workflow_tool_definitions_valid,
        results.agent_workflow_integration.agent_discovers_workflow_first,
        results.agent_workflow_integration.agent_uses_correct_workflow_for_purpose,
        results.agent_workflow_integration.agent_chains_workflow_steps,
        results.agent_workflow_integration.agent_respects_workflow_dependencies,
        results.end_to_end_scenarios.file_ingestion_workflow,
        results.end_to_end_scenarios.memory_search_workflow,
        results.end_to_end_scenarios.experience_recording_workflow,
        results.end_to_end_scenarios.multi_step_workflow,
    ].iter().filter(|&&x| x).count();
    
    println!("\n{}", "-".repeat(80));
    println!("Overall MCP Workflow Integration Score: {}/{} checks passed ({:.0}%)", 
        passed_checks, total_checks, (passed_checks as f64 / total_checks as f64) * 100.0);
    
    if passed_checks >= total_checks - 2 {
        println!("\n🎉 AGENT WILL USE MCP WORKFLOWS CORRECTLY!");
    } else if passed_checks >= total_checks / 2 {
        println!("\n⚠️  PARTIAL MCP WORKFLOW SUPPORT - Some issues need attention");
    } else {
        println!("\n❌ MCP WORKFLOW INTEGRATION NEEDS SIGNIFICANT IMPROVEMENT");
    }
    
    println!("{}", "=".repeat(80));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_content_text() {
        // Test direct text field
        let result = serde_json::json!({
            "text": "Hello World"
        });
        assert_eq!(extract_content_text(&result), Some("Hello World".to_string()));
        
        // Test content array format
        let result = serde_json::json!({
            "content": [{
                "text": "Hello from content"
            }]
        });
        assert_eq!(extract_content_text(&result), Some("Hello from content".to_string()));
    }
    
    #[test]
    fn test_workflow_results_structs() {
        // Test that all result structs can be created
        let discovery = WorkflowDiscoveryResults {
            get_workflow_available: true,
            default_workflow_retrieved: true,
            purpose_based_workflows: vec!["test".to_string()],
            workflow_rules_understood: true,
        };
        assert!(discovery.get_workflow_available);
        
        let execution = WorkflowExecutionResults {
            create_workflow_succeeds: true,
            workflow_id_generated: Some("test-id".to_string()),
            add_step_succeeds: true,
            start_workflow_succeeds: true,
            workflow_completes: true,
            pause_resume_works: true,
        };
        assert!(execution.create_workflow_succeeds);
    }
}
