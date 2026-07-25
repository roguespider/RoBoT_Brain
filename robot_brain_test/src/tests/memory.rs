//! Memory tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("store_memory", serde_json::json!({
            "content": "Test memory for integration testing",
            "memory_type": "note",
            "confidence": 0.95,
            "tags": ["test", "integration"]
        })).await.expect("store_memory call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: store_memory ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: store_memory - {:?}", response.error);
        }
        assert!(passed, "store_memory should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_search_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("search_memory", serde_json::json!({
            "query": "test",
            "limit": 10
        })).await.expect("search_memory call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: search_memory ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: search_memory - {:?}", response.error);
        }
        assert!(passed, "search_memory should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_memory() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_memory", serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        })).await.expect("get_memory call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_memory ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_memory - {:?}", response.error);
        }
        assert!(passed, "get_memory should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_memories() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_memories", serde_json::json!({
            "limit": 10
        })).await.expect("list_memories call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_memories ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_memories - {:?}", response.error);
        }
        assert!(passed, "list_memories should succeed");
        
        client.stop().await;
    }
}
