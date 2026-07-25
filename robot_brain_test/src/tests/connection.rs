//! Connection tests - Server initialization and MCP protocol compliance

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_initialize() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        
        let result = client.initialize().await.expect("Failed to initialize");
        
        assert!(result.get("protocolVersion").is_some(), "Should have protocolVersion");
        assert!(result.get("serverInfo").is_some(), "Should have serverInfo");
        
        println!("✓ PASS: Server initialization");
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let tools = client.list_tools().await.expect("Failed to list tools");
        
        assert!(!tools.is_empty(), "Should have tools registered");
        println!("\n=== Found {} tools registered ===", tools.len());
        
        for tool in &tools {
            println!("  - {}: {}", tool.name, tool.description);
        }
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_mcp_protocol_compliance() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, _) = client.call_tool_timed("get_workflow", serde_json::json!({})).await.expect("get_workflow failed");
        
        assert!(response.error.is_none(), "Should not have error: {:?}", response.error);
        assert!(response.result.is_some(), "Should have result");
        
        println!("✓ PASS: MCP protocol compliance");
        client.stop().await;
    }
}
