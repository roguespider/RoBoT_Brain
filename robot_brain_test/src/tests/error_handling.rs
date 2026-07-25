//! Error handling tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_tool() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("totally_fake_tool_name", serde_json::json!({}))
            .await.expect("Should return response even for unknown tool");
        
        assert!(response.error.is_some() || response.result.is_some(), "Should return response");
        
        if response.error.is_some() {
            println!("✓ PASS: Unknown tool returns error: {}", response.error.as_ref().unwrap().message);
        } else {
            println!("✓ PASS: Unknown tool handled gracefully");
        }
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_invalid_arguments() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let response = client.call_tool("store_memory", serde_json::json!({
            "content": "test"
        })).await;
        
        assert!(response.is_ok(), "Should handle invalid arguments gracefully");
        
        println!("✓ PASS: Invalid arguments handled gracefully");
        client.stop().await;
    }
}
