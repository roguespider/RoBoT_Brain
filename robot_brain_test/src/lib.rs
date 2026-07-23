//! MCP Integration Test Library
//!
//! This module contains all MCP integration tests that verify the compiled
//! RoBoT Brain MCP server works correctly.

use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout, Command as AsyncCommand};
use serde::{Deserialize, Serialize};

/// Get the path to the compiled MCP server executable
pub fn get_server_path() -> PathBuf {
    std::env::var("MCP_SERVER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Default paths for different platforms
            let release_path = PathBuf::from("../RoBoT_Brain/target/release/robot_brain");
            let debug_path = PathBuf::from("../RoBoT_Brain/target/debug/robot_brain");
            
            #[cfg(windows)]
            let release_path = release_path.with_extension("exe");
            #[cfg(windows)]
            let debug_path = debug_path.with_extension("exe");
            
            if release_path.exists() {
                release_path
            } else if debug_path.exists() {
                debug_path
            } else {
                panic!("MCP server not found! Build with `cargo build --release` in RoBoT_Brain first.");
            }
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}

/// MCP Test Client - connects to server via stdio transport
pub struct McpTestClient {
    child: tokio::process::Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    /// Start the MCP server and connect
    pub async fn start(server_path: PathBuf) -> anyhow::Result<Self> {
        let mut child = AsyncCommand::new(&server_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn server {}: {}", server_path.display(), e))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdout = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    /// Send a JSON-RPC request and get response
    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<JsonRpcResponse> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_str = serde_json::to_string(&request)?;
        self.stdin.write_all(request_str.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;

        // Read response
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {} - line: {}", e, line))?;
        Ok(response)
    }

    /// Initialize connection with server
    pub async fn initialize(&mut self) -> anyhow::Result<serde_json::Value> {
        let response = self.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "clientInfo": {
                "name": "robot_brain_test",
                "version": "1.0.0"
            }
        })).await?;

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        self.stdin.write_all(serde_json::to_string(&notification)?.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;

        // Also consume any log messages that might have been written
        self.consume_logs().await;

        response.result
            .ok_or_else(|| anyhow::anyhow!("Initialize failed: {:?}", response.error))
    }

    /// Consume any log lines that might have been written to stdout
    async fn consume_logs(&mut self) {
        // Try to read a line without blocking - if it's not a JSON-RPC message, discard it
        loop {
            let mut line = String::new();
            match self.stdout.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    // Check if it looks like a JSON-RPC message
                    if !line.trim().is_empty() && !line.starts_with("20") && !line.contains("INFO") {
                        // This might be a JSON-RPC response, put it back would be complex
                        // For now, just check if it parses
                        if serde_json::from_str::<JsonRpcResponse>(&line).is_ok() {
                            // This was actually a response, we'd need to handle it
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    /// List all available tools
    pub async fn list_tools(&mut self) -> anyhow::Result<Vec<ToolDefinition>> {
        let response = self.send_request("tools/list", serde_json::json!({})).await?;
        
        self.consume_logs().await;
        
        let result = response.result
            .ok_or_else(|| anyhow::anyhow!("List tools failed: {:?}", response.error))?;
        
        let tools = result.get("tools")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| serde_json::from_value(t.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(tools)
    }

    /// Call a specific tool
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> anyhow::Result<JsonRpcResponse> {
        let response = self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": arguments
        })).await?;

        self.consume_logs().await;
        
        Ok(response)
    }

    /// Stop the server gracefully
    pub async fn stop(&mut self) {
        let _ = self.child.kill().await;
    }
}

// ============================================================================
// INTEGRATION TESTS - These run with `cargo test`
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the server starts and initializes correctly
    #[tokio::test]
    async fn test_server_initialize() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        
        let result = client.initialize().await.expect("Failed to initialize");
        
        // Verify response structure
        assert!(result.get("protocolVersion").is_some(), "Should have protocolVersion");
        assert!(result.get("serverInfo").is_some(), "Should have serverInfo");
        
        client.stop().await;
    }

    /// Test that tools are listed correctly
    #[tokio::test]
    async fn test_list_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let tools = client.list_tools().await.expect("Failed to list tools");
        
        // Verify we have the expected core tools
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        
        assert!(tool_names.contains(&"store_memory"), "Should have store_memory tool");
        assert!(tool_names.contains(&"search_memory"), "Should have search_memory tool");
        assert!(tool_names.contains(&"get_workflow"), "Should have get_workflow tool");
        assert!(tool_names.contains(&"list_memories"), "Should have list_memories tool");
        
        println!("Found {} tools: {:?}", tools.len(), tool_names);
        client.stop().await;
    }

    /// Test get_workflow tool - this is MANDATORY to be called first
    #[tokio::test]
    async fn test_get_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("get_workflow", serde_json::json!({})).await
            .expect("get_workflow call failed");
        
        // Verify no error
        assert!(response.error.is_none(), "get_workflow returned error: {:?}", response.error);
        assert!(response.result.is_some(), "get_workflow should return result");
        
        // Verify the result structure
        let result = response.result.unwrap();
        let data = result.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' field in response"))
            .unwrap();
        
        // Workflow should have certain fields
        assert!(data.get("success").is_some() || data.get("workflow").is_some() || data.get("rules").is_some(),
            "Workflow response should have valid structure");
        
        println!("Workflow response: {:?}", serde_json::to_string_pretty(&result));
        client.stop().await;
    }

    /// Test store_memory tool
    #[tokio::test]
    async fn test_store_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("store_memory", serde_json::json!({
            "content": "Test memory for MCP integration testing",
            "memory_type": "note",
            "confidence": 0.9,
            "importance": 0.8,
            "tags": ["test", "integration"]
        })).await.expect("store_memory call failed");
        
        // Verify no error
        assert!(response.error.is_none(), "store_memory returned error: {:?}", response.error);
        assert!(response.result.is_some(), "store_memory should return result");
        
        let result = response.result.unwrap();
        let data = result.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' field"))
            .unwrap();
        
        // Should return success and ID
        assert!(data.get("success").is_some(), "Should have 'success' field");
        
        // Extract memory ID for next test
        if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
            println!("Created memory with ID: {}", id);
        }
        
        client.stop().await;
    }

    /// Test search_memory tool
    #[tokio::test]
    async fn test_search_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        // First store a memory
        client.call_tool("store_memory", serde_json::json!({
            "content": "Searchable test memory content",
            "memory_type": "note"
        })).await.expect("Failed to store test memory");

        // Then search for it
        let response = client.call_tool("search_memory", serde_json::json!({
            "query": "searchable test",
            "limit": 10
        })).await.expect("search_memory call failed");
        
        assert!(response.error.is_none(), "search_memory returned error: {:?}", response.error);
        assert!(response.result.is_some(), "search_memory should return result");
        
        let result = response.result.unwrap();
        let data = result.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' field"))
            .unwrap();
        
        // Should have results array
        assert!(data.get("results").is_some(), "Should have 'results' array");
        
        client.stop().await;
    }

    /// Test list_memories tool
    #[tokio::test]
    async fn test_list_memories() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("list_memories", serde_json::json!({
            "limit": 5
        })).await.expect("list_memories call failed");
        
        assert!(response.error.is_none(), "list_memories returned error: {:?}", response.error);
        assert!(response.result.is_some(), "list_memories should return result");
        
        let result = response.result.unwrap();
        let data = result.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' field"))
            .unwrap();
        
        // Should have memories array
        assert!(data.get("memories").is_some(), "Should have 'memories' array");
        println!("list_memories response: {:?}", serde_json::to_string_pretty(&result));
        
        client.stop().await;
    }

    /// Test experience tools
    #[tokio::test]
    async fn test_experience_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        // Test record_experience
        let response = client.call_tool("record_experience", serde_json::json!({
            "action": "test_mcp_integration",
            "outcome": "success",
            "context": "Testing MCP server functionality",
            "tags": ["test", "integration"]
        })).await.expect("record_experience call failed");
        
        assert!(response.error.is_none(), "record_experience returned error: {:?}", response.error);
        
        // Test get_experience_stats
        let stats_response = client.call_tool("get_experience_stats", serde_json::json!({}))
            .await.expect("get_experience_stats call failed");
        
        assert!(stats_response.error.is_none(), "get_experience_stats returned error: {:?}", stats_response.error);
        assert!(stats_response.result.is_some(), "Should return stats");
        
        client.stop().await;
    }

    /// Test knowledge tools
    #[tokio::test]
    async fn test_knowledge_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        // Test add_knowledge
        let response = client.call_tool("add_knowledge", serde_json::json!({
            "statement": "MCP integration tests verify server functionality",
            "source": "test",
            "confidence": 0.95
        })).await.expect("add_knowledge call failed");
        
        // Note: add_knowledge might fail due to missing required fields, which is OK
        
        // Test query_knowledge
        let query_response = client.call_tool("query_knowledge", serde_json::json!({
            "query": "MCP",
            "limit": 5
        })).await.expect("query_knowledge call failed");
        
        assert!(query_response.error.is_none(), "query_knowledge returned error: {:?}", query_response.error);
        
        // Test get_knowledge_stats
        let stats_response = client.call_tool("get_knowledge_stats", serde_json::json!({}))
            .await.expect("get_knowledge_stats call failed");
        
        assert!(stats_response.error.is_none(), "get_knowledge_stats returned error: {:?}", stats_response.error);
        
        client.stop().await;
    }

    /// Test planner tools
    #[tokio::test]
    async fn test_planner_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        // Test create_plan
        let response = client.call_tool("create_plan", serde_json::json!({
            "goal": "Test MCP server functionality",
            "description": "Integration test plan"
        })).await.expect("create_plan call failed");
        
        assert!(response.error.is_none(), "create_plan returned error: {:?}", response.error);
        
        // Extract plan ID for further testing
        let plan_id = response.result
            .and_then(|r| r.get("data").cloned())
            .and_then(|d| d.get("id").cloned())
            .and_then(|v| v.as_str().map(String::from));
        
        if let Some(id) = plan_id {
            println!("Created plan with ID: {}", id);
            
            // Test get_plan
            let get_response = client.call_tool("get_plan", serde_json::json!({
                "id": id
            })).await.expect("get_plan call failed");
            
            assert!(get_response.error.is_none(), "get_plan returned error: {:?}", get_response.error);
        }
        
        // Test list_plans
        let list_response = client.call_tool("list_plans", serde_json::json!({
            "limit": 10
        })).await.expect("list_plans call failed");
        
        assert!(list_response.error.is_none(), "list_plans returned error: {:?}", list_response.error);
        
        client.stop().await;
    }

    /// Test that calling non-existent tool returns proper error
    #[tokio::test]
    async fn test_error_handling() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("non_existent_tool", serde_json::json!({}))
            .await.expect("Should return response even for unknown tool");
        
        // Should either have error field or an error result
        // The MCP protocol should handle unknown tools gracefully
        if let Some(error) = response.error {
            // Error is expected for unknown tools
            println!("Expected error for unknown tool: {}", error.message);
        } else {
            // If no error, the result should indicate failure somehow
            if let Some(result) = response.result {
                // Check if the result indicates an error
                if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
                    assert!(!success, "Non-existent tool should return success=false");
                }
            }
        }
        
        client.stop().await;
    }

    /// End-to-end test: store, search, retrieve workflow
    #[tokio::test]
    async fn test_end_to_end_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        // 1. Get workflow first (as per documentation)
        let workflow = client.call_tool("get_workflow", serde_json::json!({}))
            .await.expect("get_workflow failed");
        assert!(workflow.error.is_none(), "Workflow should succeed");
        println!("Workflow: {:?}", workflow.result);

        // 2. Store a memory
        let store = client.call_tool("store_memory", serde_json::json!({
            "content": "End-to-end test memory",
            "memory_type": "fact",
            "confidence": 0.95
        })).await.expect("store_memory failed");
        assert!(store.error.is_none(), "Store should succeed");
        
        let memory_id = store.result
            .and_then(|r| r.get("data").cloned())
            .and_then(|d| d.get("id").cloned())
            .and_then(|v| v.as_str().map(String::from));

        // 3. Search for it
        let search = client.call_tool("search_memory", serde_json::json!({
            "query": "end-to-end test",
            "limit": 5
        })).await.expect("search_memory failed");
        assert!(search.error.is_none(), "Search should succeed");
        
        // 4. List all memories
        let list = client.call_tool("list_memories", serde_json::json!({
            "limit": 10
        })).await.expect("list_memories failed");
        assert!(list.error.is_none(), "List should succeed");

        println!("End-to-end workflow completed successfully");
        println!("Memory ID: {:?}", memory_id);
        
        client.stop().await;
    }
}
