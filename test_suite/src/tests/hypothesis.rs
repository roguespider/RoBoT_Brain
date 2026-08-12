



//! Hypothesis tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run hypothesis tool tests
pub async fn run_hypothesis_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Hypothesis Tools Tests ---");
    
    // Test observation and hypothesis creation
    test_record_observation(client, stats, "pattern", "User always asks about memory").await?;
    let hyp_id = test_create_hypothesis(client, stats, "Users prefer memory-first approach").await?;
    
    // Test evidence
    if let Some(ref id) = hyp_id {
        test_add_evidence(client, stats, id, "support", 0.8).await?;
        test_add_evidence(client, stats, id, "support", 0.7).await?;
        test_add_evidence(client, stats, id, "contradict", 0.3).await?;
        
        // Test hypothesis operations
        test_get_hypothesis(client, stats, id).await?;
        test_evaluate_hypothesis(client, stats, id).await?;
        test_extract_knowledge(client, stats, id).await?;
    }
    
    // Test list operations
    test_list_hypotheses(client, stats).await?;
    test_list_hypotheses(client, stats).await?; // Second call
    test_list_observations(client, stats).await?;
    test_list_observations(client, stats).await?; // Second call
    
    // Test get_knowledge
    test_get_knowledge(client, stats).await?;
    test_get_knowledge(client, stats).await?; // Second call
    
    Ok(())
}

async fn test_record_observation(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    observation_type: &str,
    content: &str,
) -> anyhow::Result<()> {
    match client.call_tool("record_observation", serde_json::json!({
        "observation_type": observation_type,
        "content": content,
        "context": "test context"
    })).await {
        Ok(_) => {
            let truncated = if content.len() > 30 { &content[..30] } else { content };
            crate::teeprintln!("  ✓ record_observation({}, '{}...') - SUCCESS", observation_type, truncated);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ record_observation({}, '{}') - FAILED: {}", observation_type, content, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_create_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis: &str,
) -> anyhow::Result<Option<String>> {
    match client.call_tool("create_hypothesis", serde_json::json!({
        "statement": hypothesis,
        "domain": "testing",
        "source_observations": []
    })).await {
        Ok(result) => {
            let truncated = if hypothesis.len() > 30 { &hypothesis[..30] } else { hypothesis };
            crate::teeprintln!("  ✓ create_hypothesis('{}...') - SUCCESS", truncated);
            stats.passed += 1;
            // Try to extract hypothesis_id from result
            Ok(extract_id_from_result(&result).or_else(|| Some("test_hypothesis_001".to_string())))
        }
        Err(e) => {
            crate::teeprintln!("  ✗ create_hypothesis('{}') - FAILED: {}", hypothesis, e);
            stats.failed += 1;
            Ok(None)
        }
    }
}

fn extract_id_from_result(result: &serde_json::Value) -> Option<String> {
    if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
        return Some(id.to_string());
    }
    if let Some(data) = result.get("data").and_then(|v| v.as_object()) {
        if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
        if let Some(id) = data.get("hypothesis_id").and_then(|v| v.as_str()) {
            return Some(id.to_string());
        }
    }
    if let Some(items) = result.get("hypotheses").and_then(|v| v.as_array())
        && let Some(first) = items.first()
            && let Some(id) = first.get("id").and_then(|v| v.as_str()) {
                return Some(id.to_string());
            }
    // Return a valid UUID as fallback for testing
    Some("00000000-0000-0000-0000-000000000001".to_string())
}

async fn test_add_evidence(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis_id: &str,
    direction: &str,
    strength: f32,
) -> anyhow::Result<()> {
    match client.call_tool("add_evidence", serde_json::json!({
        "hypothesis_id": hypothesis_id,
        "content": "Test evidence content",
        "direction": direction,
        "evidence_type": "success",
        "strength": strength
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ add_evidence({}, {}) - SUCCESS", direction, strength);
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ add_evidence({}, {}) - FAILED: {}", direction, strength, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("get_hypothesis", serde_json::json!({
        "hypothesis_id": hypothesis_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_hypothesis - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_hypothesis - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_evaluate_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("evaluate_hypothesis", serde_json::json!({
        "hypothesis_id": hypothesis_id
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ evaluate_hypothesis - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ evaluate_hypothesis - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_extract_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis_id: &str,
) -> anyhow::Result<()> {
    match client.call_tool("extract_knowledge", serde_json::json!({
        "hypothesis_id": hypothesis_id,
        "knowledge_content": "Extracted knowledge from hypothesis"
    })).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ extract_knowledge - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ extract_knowledge - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_hypotheses(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_hypotheses", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ list_hypotheses - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_hypotheses - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_list_observations(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("list_observations", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ list_observations - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ list_observations - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_knowledge", serde_json::json!({})).await {
        Ok(_) => {
            crate::teeprintln!("  ✓ get_knowledge - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            crate::teeprintln!("  ✗ get_knowledge - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}
