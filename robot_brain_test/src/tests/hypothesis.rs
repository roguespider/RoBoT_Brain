//! Hypothesis tool tests

use crate::client::McpTestClient;
use crate::common::get_server_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_observation() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("record_observation", serde_json::json!({
            "content": "Test observation from integration testing",
            "context": "Running integration tests",
            "observation_type": "success"
        })).await.expect("record_observation call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: record_observation ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: record_observation - {:?}", response.error);
        }
        assert!(passed, "record_observation should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_create_hypothesis() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("create_hypothesis", serde_json::json!({
            "statement": "Testing hypothesis creation in integration tests",
            "domain": "testing"
        })).await.expect("create_hypothesis call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: create_hypothesis ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: create_hypothesis - {:?}", response.error);
        }
        assert!(passed, "create_hypothesis should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_hypotheses() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_hypotheses", serde_json::json!({
            "limit": 10
        })).await.expect("list_hypotheses call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_hypotheses ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_hypotheses - {:?}", response.error);
        }
        assert!(passed, "list_hypotheses should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_hypothesis() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_hypothesis", serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000000"
        })).await.expect("get_hypothesis call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: get_hypothesis ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_hypothesis - {:?}", response.error);
        }
        assert!(passed, "get_hypothesis should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_add_evidence() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("add_evidence", serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000000",
            "content": "Test evidence",
            "direction": "support",
            "strength": 0.8
        })).await.expect("add_evidence call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: add_evidence ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: add_evidence - {:?}", response.error);
        }
        assert!(passed, "add_evidence should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_list_observations() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("list_observations", serde_json::json!({
            "limit": 10
        })).await.expect("list_observations call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: list_observations ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: list_observations - {:?}", response.error);
        }
        assert!(passed, "list_observations should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_evaluate_hypothesis() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("evaluate_hypothesis", serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000000"
        })).await.expect("evaluate_hypothesis call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: evaluate_hypothesis ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: evaluate_hypothesis - {:?}", response.error);
        }
        assert!(passed, "evaluate_hypothesis should return response");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_get_knowledge_hypothesis() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("get_knowledge", serde_json::json!({
            "limit": 10
        })).await.expect("get_knowledge call failed");

        let passed = response.error.is_none() && response.result.is_some();
        if passed {
            println!("✓ PASS: get_knowledge ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: get_knowledge - {:?}", response.error);
        }
        assert!(passed, "get_knowledge should succeed");
        
        client.stop().await;
    }

    #[tokio::test]
    async fn test_extract_knowledge() {
        let server_path = get_server_path();
        let mut client = McpTestClient::start(server_path).await.expect("Failed to start server");
        client.initialize().await.expect("Failed to initialize");

        let (response, elapsed) = client.call_tool_timed("extract_knowledge", serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000000",
            "knowledge_content": "Test extracted knowledge"
        })).await.expect("extract_knowledge call failed");

        let passed = response.result.is_some();
        if passed {
            println!("✓ PASS: extract_knowledge ({}ms)", elapsed);
        } else {
            println!("✗ FAIL: extract_knowledge - {:?}", response.error);
        }
        assert!(passed, "extract_knowledge should return response");
        
        client.stop().await;
    }
}
