



//! Hypothesis tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run hypothesis tool tests
pub async fn run_hypothesis_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Hypothesis Tools Tests ---");
    
    // Test observation and hypothesis creation
    test_record_observation(client, stats, "pattern", "User always asks about memory").await?;
    test_create_hypothesis(client, stats, "Users prefer memory-first approach").await?;
    
    // Test evidence
    test_add_evidence(client, stats, "support", 0.8).await?;
    test_add_evidence(client, stats, "support", 0.7).await?;
    test_add_evidence(client, stats, "contradict", 0.3).await?;
    
    // Test hypothesis operations
    test_get_hypothesis(client, stats).await?;
    test_evaluate_hypothesis(client, stats).await?;
    test_extract_knowledge(client, stats).await?;
    
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
        "content": content
    })).await {
        Ok(_) => {
            let truncated = if content.len() > 30 { &content[..30] } else { content };
            println!("  ✓ record_observation({}, '{}...') - SUCCESS", observation_type, truncated);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ record_observation({}, '{}') - FAILED: {}", observation_type, content, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_create_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    hypothesis: &str,
) -> anyhow::Result<()> {
    match client.call_tool("create_hypothesis", serde_json::json!({
        "hypothesis": hypothesis
    })).await {
        Ok(_) => {
            let truncated = if hypothesis.len() > 30 { &hypothesis[..30] } else { hypothesis };
            println!("  ✓ create_hypothesis('{}...') - SUCCESS", truncated);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ create_hypothesis('{}') - FAILED: {}", hypothesis, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_add_evidence(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    evidence_type: &str,
    strength: f32,
) -> anyhow::Result<()> {
    match client.call_tool("add_evidence", serde_json::json!({
        "evidence_type": evidence_type,
        "strength": strength
    })).await {
        Ok(_) => {
            println!("  ✓ add_evidence({}, {}) - SUCCESS", evidence_type, strength);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ add_evidence({}, {}) - FAILED: {}", evidence_type, strength, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_hypothesis", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_hypothesis - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_hypothesis - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_evaluate_hypothesis(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("evaluate_hypothesis", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ evaluate_hypothesis - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ evaluate_hypothesis - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_extract_knowledge(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("extract_knowledge", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ extract_knowledge - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ extract_knowledge - FAILED: {}", e);
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
            println!("  ✓ list_hypotheses - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_hypotheses - FAILED: {}", e);
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
            println!("  ✓ list_observations - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ list_observations - FAILED: {}", e);
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
            println!("  ✓ get_knowledge - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_knowledge - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}
