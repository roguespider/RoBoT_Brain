//! Agent tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_workflow() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_workflow", serde_json::json!({})).await.expect("get_workflow call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_workflow ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_workflow - {:?}", response.error);
        }
        assert!(passed, "get_workflow should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_mcp_tools() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_tools", serde_json::json!({})).await.expect("list_tools call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_tools ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_tools - {:?}", response.error);
        }
        assert!(passed, "list_tools should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_tool() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_tool", serde_json::json!({
            "name": "store_memory"
        })).await.expect("get_tool call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_tool ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_tool - {:?}", response.error);
        }
        assert!(passed, "get_tool should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_tool_not_found() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_tool", serde_json::json!({
            "name": "non_existent_tool"
        })).await.expect("get_tool call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_tool (not found) ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_tool - {:?}", response.error);
        }
        assert!(passed, "get_tool should return response even for non-existent tool");
        
        client.stop().await;
    }
}
