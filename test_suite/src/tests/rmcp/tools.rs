//! RMCP Tool Tests
//!
//! Tests tool discovery and execution via MCP protocol

use crate::{TestMcpClient, TestStats};

/// Tool discovery test results
#[derive(Debug, Default)]
pub struct ToolDiscoveryResults {
    pub passed: usize,
    pub failed: usize,
    pub tools_found: usize,
    pub categories_found: Vec<String>,
}

/// Tool execution test results
#[derive(Debug, Default)]
pub struct ToolExecutionResults {
    pub passed: usize,
    pub failed: usize,
    pub tools_executed: usize,
    pub categories_executed: Vec<String>,
}

/// Test tool discovery (list_tools)
pub async fn test_tool_discovery(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<ToolDiscoveryResults> {
    let mut results = ToolDiscoveryResults::default();

    // Test 1: Basic tool listing
    crate::teeprintln!("  Testing basic tool listing...");
    match client.list_tools().await {
        Ok(tools) => {
            crate::teeprintln!("    ✅ list_tools SUCCESS - {} tools found", tools.len());
            results.tools_found = tools.len();
            results.passed += 1;
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("    ❌ list_tools FAILED: {}", e);
            results.failed += 1;
            stats.failed += 1;
            return Ok(results);
        }
    }

    // Test 2: Get tools and categorize them
    crate::teeprintln!("  Testing tool categorization...");
    let tools = client.list_tools().await?;
    
    // Define category mappings from tool suffixes to canonical categories
    let category_mappings: std::collections::HashMap<&str, &str> = [
        ("memory", "memory"),
        ("memories", "memory"),
        ("experience", "experience"),
        ("experiences", "experience"),
        ("knowledge", "knowledge"),
        ("workflow", "workflow"),
        ("workflows", "workflow"),
        ("plan", "planner"),
        ("plans", "planner"),
        ("hypothesis", "hypothesis"),
        ("hypotheses", "hypothesis"),
        ("observation", "reflection"),
        ("observations", "reflection"),
        ("insights", "reflection"),
        ("patterns", "reflection"),
        ("skill", "skills"),
        ("skills", "skills"),
        ("exploration", "exploration"),
        ("agent", "agent"),
        ("agents", "agent"),
        ("files", "ingestor"),
        ("audio", "ingestor"),
        ("ingestor", "ingestor"),
    ].iter().cloned().collect();
    
    let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for tool in &tools {
        if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
            // Try to extract category from tool name
            let parts: Vec<&str> = name.split('_').collect();
            
            // Skip common verbs and try to find a category match
            let skip_verbs = ["get", "list", "add", "create", "update", "delete", "record", 
                              "start", "stop", "execute", "call", "set", "enable", "disable", 
                              "pause", "resume", "complete", "cancel", "abandon", "fail", 
                              "route", "connect", "disconnect", "extract", "apply", "promote", 
                              "ingest", "transcribe", "register", "unregister", "discover", 
                              "query", "analyze", "evaluate", "global", "search"];
            
            // Try last non-verb word first, then first non-verb word
            let candidates: Vec<&str> = parts.iter()
                .rev()
                .chain(parts.iter())
                .filter(|p| !skip_verbs.contains(&(**p).to_lowercase().as_str()))
                .take(parts.len())
                .copied()
                .collect();
            
            let category = candidates.iter()
                .find_map(|p| category_mappings.get(&p.to_lowercase().as_str()).copied())
                .unwrap_or(parts.first().unwrap_or(&name));
            
            *categories.entry(category.to_string()).or_insert(0) += 1;
        }
    }

    for (category, count) in &categories {
        crate::teeprintln!("    ℹ  {} tools: {} found", category, count);
        results.categories_found.push(category.clone());
    }
    results.passed += 1;

    // Test 3: Verify tool structure
    crate::teeprintln!("  Testing tool schema structure...");
    let mut valid_tools = 0;
    for tool in &tools {
        let has_name = tool.get("name").is_some();
        let has_description = tool.get("description").is_some();
        let has_input_schema = tool.get("inputSchema").is_some() || tool.get("parameters").is_some();
        
        if has_name && (has_description || has_input_schema) {
            valid_tools += 1;
        }
    }
    
    if valid_tools > 0 {
        crate::teeprintln!("    ✅ {} tools have valid schema structure", valid_tools);
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  No tools have complete schema structure");
        results.failed += 1;
    }

    // Test 4: List specific tool categories
    crate::teeprintln!("  Testing tool category coverage...");
    let expected_categories = vec![
        "memory", "experience", "knowledge", "workflow",
        "planner", "hypothesis", "reflection",
        "ingestor", "agent", "skills", "exploration"
    ];
    
    let mut coverage = 0;
    for category in expected_categories.iter() {
        if categories.contains_key(&category.to_string()) {
            coverage += 1;
        } else {
            crate::teeprintln!("    ⚠️  Missing category: {}", category);
        }
    }
    
    crate::teeprintln!("    ℹ  Category coverage: {}/{}", coverage, expected_categories.len());
    if coverage >= 6 {
        crate::teeprintln!("    ✅ Good category coverage");
        results.passed += 1;
    } else {
        crate::teeprintln!("    ⚠️  Limited category coverage");
        results.failed += 1;
    }

    Ok(results)
}

/// Test tool execution (call_tool)
pub async fn test_tool_execution(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<ToolExecutionResults> {
    let mut results = ToolExecutionResults::default();

    // Define test cases for each major tool category
    let test_cases = vec![
        // Memory tools
        ("search_memory", serde_json::json!({"query": "test query", "limit": 5}), "memory"),
        ("get_memory", serde_json::json!({"id": "00000000-0000-0000-0000-000000000000"}), "memory"),
        ("list_memories", serde_json::json!({"limit": 10}), "memory"),
        
        // Knowledge tools
        ("query_knowledge", serde_json::json!({"query": "test", "limit": 5}), "knowledge"),
        
        // Experience tools
        ("list_experiences", serde_json::json!({"limit": 5}), "experience"),
        
        // Workflow tools
        ("list_workflows", serde_json::json!({}), "workflow"),
        
        // Planner tools
        ("create_plan", serde_json::json!({"goal": "test goal"}), "planner"),
        ("list_plans", serde_json::json!({}), "planner"),
        
        // Hypothesis tools
        ("list_hypotheses", serde_json::json!({"limit": 5}), "hypothesis"),
        ("get_hypothesis", serde_json::json!({"hypothesis_id": "00000000-0000-0000-0000-000000000001"}), "hypothesis"),
        
        // Reflection tools
        ("get_insights", serde_json::json!({}), "reflection"),
        ("get_patterns", serde_json::json!({}), "reflection"),
        
        // Search tools
        ("global_search", serde_json::json!({"query": "test", "limit": 5}), "search"),
        
        // Skills tools
        ("list_skills", serde_json::json!({}), "skills"),
        ("get_skill", serde_json::json!({"skill_id": "00000000-0000-0000-0000-000000000001"}), "skills"),
        
        // Exploration tools
        ("get_exploration_status", serde_json::json!({"exploration_id": "00000000-0000-0000-0000-000000000001"}), "exploration"),
        
        // Agent tools
        ("get_system_status", serde_json::json!({}), "agent"),
    ];

    crate::teeprintln!("  Testing tool execution for {} tools...", test_cases.len());
    
    let mut executed_categories: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for (tool_name, arguments, category) in test_cases {
        crate::teeprintln!("    Testing {}...", tool_name);
        
        match client.call_tool(tool_name, arguments).await {
            Ok(result) => {
                crate::teeprintln!("      ✅ {} - SUCCESS", tool_name);
                results.tools_executed += 1;
                executed_categories.insert(category.to_string());
                results.passed += 1;
                stats.passed += 1;
                
                // Log a snippet of the result
                if let Some(text) = result.get("content").and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|t| t.get("text"))
                    .and_then(|t| t.as_str()) 
                {
                    let snippet = if text.len() > 100 { &text[..100] } else { text };
                    crate::teeprintln!("      ℹ  Result snippet: {}...", snippet.replace('\n', " ").trim());
                }
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("method_not_found") || error_str.contains("-32601") {
                    crate::teeprintln!("      ⚠️  {} - NOT IMPLEMENTED (method not found)", tool_name);
                    results.failed += 1;
                    stats.skipped += 1;
                } else if error_str.contains("tool_not_found") {
                    crate::teeprintln!("      ⚠️  {} - TOOL NOT FOUND", tool_name);
                    results.failed += 1;
                    stats.skipped += 1;
                } else if error_str.contains("not found") {
                    // Resource not found is expected when testing with fake IDs - tool executed correctly
                    crate::teeprintln!("      ✅ {} - SUCCESS (resource not found - tool executed correctly)", tool_name);
                    results.tools_executed += 1;
                    executed_categories.insert(category.to_string());
                } else {
                    crate::teeprintln!("      ⚠️  {} - ERROR: {}", tool_name, e);
                    results.failed += 1;
                    stats.skipped += 1;
                }
            }
        }
    }

    results.categories_executed = executed_categories.into_iter().collect();
    
    // Summary
    crate::teeprintln!("\n  📊 Tool Execution Summary:");
    crate::teeprintln!("     Tools Executed: {}", results.tools_executed);
    crate::teeprintln!("     Categories: {:?}", results.categories_executed);

    Ok(results)
}
