// src/tools/agent/workflows.rs
// Workflow and list tools execution

use crate::api::bridge::tools::{ToolOutput, get_tools_async};

use super::inputs::{GetWorkflowInput, ListToolsInput};

/// Execute get_workflow tool - MUST be called before any other tool
pub async fn execute_get_workflow(input: GetWorkflowInput) -> Result<ToolOutput, anyhow::Error> {
    let purpose = input.purpose.unwrap_or_else(|| "general".to_string());

    let workflow = match purpose.to_lowercase().as_str() {
        "file_ingestion" | "ingest" | "import" => serde_json::json!({
            "workflow_name": "File Ingestion Workflow",
            "SCOPING_RULES": {
                "IMPORTANT": "ALWAYS look in the robot_brain directory, NOT the current project folder",
                "NEVER_explore_project": "DO NOT explore the project directory structure. Only look in files_to_import folder via list_importable tool.",
                "where_to_look": "Use import_folder path from list_importable response - this is the files_to_import location",
                "do_NOT_look_here": ["current project folder", "source code directories like src/", "anywhere except import_folder"],
                "reason": "robot_brain.exe, robot_brain.db, and files_to_import are ALL in the robot_brain directory",
                "look_at": "Check response.IMPORTANT_SCOPING.this_folder for the exact path to use"
            },
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start!"
                },
                {
                    "step": 2,
                    "tool": "list_importable",
                    "action": "Check available files in files_to_import folder",
                    "parameters": {"recursive": true},
                    "description": "Lists files ready for ingestion."
                },
                {
                    "step": 3,
                    "tool": "ingest_files",
                    "action": "Ingest ONE file",
                    "parameters": {"limit": 1},
                    "description": "Ingest files one at a time for best results."
                },
                {
                    "step": 4,
                    "tool": "get_insights",
                    "action": "Review actionable insights",
                    "description": "Insights with high confidence can guide decisions."
                }
            ],
            "critical_rules": [
                "Use global_search for comprehensive results",
                "Review patterns before making repetitive decisions",
                "Consider insight confidence levels when making decisions"
            ]
        }),

        "memory_search" | "memory" | "search" => serde_json::json!({
            "workflow_name": "Memory Search Workflow",
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start!"
                },
                {
                    "step": 2,
                    "tool": "global_search",
                    "action": "Search all data types",
                    "description": "Comprehensive search across all stored data."
                },
                {
                    "step": 3,
                    "tool": "get_patterns",
                    "action": "Review learned patterns",
                    "description": "Patterns may inform your approach."
                },
                {
                    "step": 4,
                    "tool": "get_insights",
                    "action": "Review actionable insights",
                    "description": "Insights with high confidence can guide decisions."
                }
            ],
            "critical_rules": [
                "Use global_search for comprehensive results",
                "Review patterns before making repetitive decisions",
                "Consider insight confidence levels when making decisions"
            ]
        }),

        _ => serde_json::json!({
            "workflow_name": "General MCP Workflow",
            "mandatory_steps": [
                {
                    "step": 1,
                    "tool": "get_workflow",
                    "action": "Review workflow rules",
                    "description": "You called get_workflow - good start! Always call this first."
                },
                {
                    "step": 2,
                    "tool": "list_tools",
                    "action": "See all available tools",
                    "description": "Get full list of MCP tools."
                },
                {
                    "step": 3,
                    "tool": "search_memory",
                    "action": "Check existing memory for relevant context",
                    "description": "Always check memory before taking any action of any kind."
                },
                {
                    "step": 4,
                    "tool": "get_patterns",
                    "action": "Review learned patterns",
                    "description": "Patterns may inform your approach."
                },
                {
                    "step": 5,
                    "tool": "PROCEED",
                    "action": "Take action based on gathered context",
                    "description": "Now you have context - proceed with your task."
                }
            ],
            "critical_rules": [
                "MUST call get_workflow first before ANY other tool",
                "MUST check memory (search_memory) before taking any action of any kind",
                "MUST review patterns (get_patterns) for repetitive decisions",
                "ALWAYS ask user before destructive operations (delete_ingested_files)"
            ],
            "destructive_operations": {
                "delete_ingested_files": {
                    "requires_confirmation": true,
                    "confirmation_value": "yes",
                    "warning": "This deletes files permanently!"
                }
            },
            "directory_structure": {
                "exe_location": "robot_brain.exe or robot_brain",
                "db_location": "robot_brain.db (in same directory as exe)",
                "import_folder": "files_to_import/ (in same directory as exe)",
                "note": "All paths are relative to executable location"
            },
            "quick_reference": {
                "list_importable": "Check files available for import",
                "ingest_files": "Ingest files (use limit=1 for single file)",
                "delete_ingested_files": "Delete files (MUST have confirmation='yes')",
                "search_memory": "Search stored memories",
                "global_search": "Search all data types",
                "analyze_patterns": "Detect patterns in experiences",
                "get_patterns": "Get stored patterns",
                "get_insights": "Get actionable insights",
                "workers": "Delegated worker execution via OpenRouter. Connect with connect_mcp_server: {\"name\": \"workers\", \"command\": \"python\", \"args\": [\"-u\", \".agents/.workers/workers.py\"]}. Then call delegate_to_workers, evaluate_worker_output, list_worker_models via call_tool. See get_usage_guide for full guide."
            }
        }),
    };

    Ok(ToolOutput::success(serde_json::json!({
        "status": "workflow_retrieved",
        "workflow": workflow,
        "reminder": "You MUST follow this workflow. Call get_workflow again anytime you need a reminder.",
        "version": "1.0"
    })))
}

/// Execute list_tools tool
pub async fn execute_list_tools(input: ListToolsInput) -> Result<ToolOutput, anyhow::Error> {
    let all_tools = get_tools_async().await;
    let total_count = all_tools.len();

    let mut filtered_tools: Vec<serde_json::Value> = all_tools
        .into_iter()
        .filter(|tool| {
            if let Some(ref filter) = input.filter {
                let filter_lower = filter.to_lowercase();
                tool.name.to_lowercase().contains(&filter_lower)
                    || tool.description.to_lowercase().contains(&filter_lower)
            } else {
                true
            }
        })
        .map(|tool| {
            // Truncate description to avoid exceeding stdio transport message limits
            let desc = if tool.description.len() > 200 {
                format!("{}...", &tool.description[..200])
            } else {
                tool.description.clone()
            };
            // Remove full input_schema from list response (use get_tool for details)
            serde_json::json!({
                "name": tool.name,
                "description": desc,
                "source": "robot_brain",
            })
        })
        .collect();

    // Include external MCP server tools (e.g., workers) so agents can discover them
    if let Some(client) = super::mcp_tools::get_mcp_client() {
        let external_tools = client.list_all_tools().await;
        for tool in external_tools {
            let tool_name = tool.name.to_string();
            let tool_desc = tool.description.clone().unwrap_or_default().to_string();
            if let Some(ref filter) = input.filter {
                let filter_lower = filter.to_lowercase();
                if !tool_name.to_lowercase().contains(&filter_lower)
                    && !tool_desc.to_lowercase().contains(&filter_lower)
                {
                    continue;
                }
            }
            let desc = if tool_desc.len() > 200 {
                format!("{}...", &tool_desc[..200])
            } else {
                tool_desc
            };
            let server = client.get_tool_server(&tool_name).await.unwrap_or_default();
            filtered_tools.push(serde_json::json!({
                "name": tool_name,
                "description": desc,
                "source": format!("external:{}", server),
            }));
        }
    }

    Ok(ToolOutput::success(serde_json::json!({
        "tools": filtered_tools,
        "count": filtered_tools.len(),
        "total_available": total_count,
        "external_tools_included": true,
    })))
}
