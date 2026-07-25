//! Search tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_search() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("global_search", serde_json::json!({
            "query": "test",
            "limit": 10
        })).await.expect("global_search call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: global_search ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: global_search - {:?}", response.error);
        }
        assert!(passed, "global_search should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_recommendations() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_recommendations", serde_json::json!({
            "limit": 5
        })).await.expect("get_recommendations call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_recommendations ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_recommendations - {:?}", response.error);
        }
        assert!(passed, "get_recommendations should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_reputation() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_reputation", serde_json::json!({
            "target": "test_target"
        })).await.expect("get_reputation call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_reputation ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_reputation - {:?}", response.error);
        }
        assert!(passed, "get_reputation should succeed");
        
        client.stop().await;
    }
}
