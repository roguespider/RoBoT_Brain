



//! Reflection tool tests
use crate::TestMcpClient;
use crate::TestStats;

/// Run reflection tool tests
pub async fn run_reflection_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    _filter: Option<&str>,
) -> anyhow::Result<()> {
    println!("\n--- Reflection Tools Tests ---");
    
    test_create_reflection(client, stats, "Learning Analysis", "analysis").await?;
    test_get_patterns(client, stats).await?;
    test_get_patterns(client, stats).await?;
    test_get_insights(client, stats).await?;
    test_get_insights(client, stats).await?;
    test_analyze_patterns(client, stats).await?;
    
    Ok(())
}

async fn test_create_reflection(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
    title: &str,
    reflection_type: &str,
) -> anyhow::Result<()> {
    match client.call_tool("create_reflection", serde_json::json!({
        "title": title,
        "reflection_type": reflection_type
    })).await {
        Ok(_) => {
            println!("  ✓ create_reflection('{}', {}) - SUCCESS", title, reflection_type);
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ create_reflection('{}', {}) - FAILED: {}", title, reflection_type, e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_patterns(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_patterns", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_patterns - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_patterns - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_get_insights(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("get_insights", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ get_insights - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ get_insights - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}

async fn test_analyze_patterns(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    match client.call_tool("analyze_patterns", serde_json::json!({})).await {
        Ok(_) => {
            println!("  ✓ analyze_patterns - SUCCESS");
            stats.passed += 1;
        }
        Err(e) => {
            println!("  ✗ analyze_patterns - FAILED: {}", e);
            stats.failed += 1;
        }
    }
    Ok(())
}
