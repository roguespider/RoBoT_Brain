//! T1-10B-20 — migrated from `src/database/queries/embeddings.rs`
//! `#[cfg(test)] mod tests` (test_get_and_delete_embedding_by_id).
//!
//! The src/ unit test exercised `get_embedding` and `delete_embedding` (the
//! by-embedding-id variants, which are #[cfg(test)] test-only functions).
//! test_suite cannot import robot_brain source, so the behavior is re-expressed
//! through the public MCP surface that invokes the by-memory-id variants:
//!   - `store_embedding` calls `queries::insert_embedding`
//!   - `get_embedding` (MCP) calls `queries::get_embedding_by_memory_id`
//!   - `delete_embedding` (MCP) calls `queries::delete_embedding_by_memory_id`
//!
//! The MCP tools use the by-memory-id variants (production code), not the
//! by-embedding-id variants (test-only). This migration tests the same
//! get/delete lifecycle via the production by-memory-id path.
//!
//! Flow: store_embedding -> get_embedding (found) -> delete_embedding ->
//! get_embedding (not found).

use crate::TestMcpClient;
use crate::TestStats;

fn payload_json(result: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let text = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no content text in tool result"))?;
    Ok(serde_json::from_str(text)?)
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

pub async fn run_embeddings_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Embeddings get+delete by memory_id (T1-10B-20) ---");

    // First store a memory to get a valid memory_id.
    let suffix = unique_suffix();
    let content = format!("EmbeddingTest-{}", suffix);
    let store_mem = client
        .call_tool(
            "store_memory",
            serde_json::json!({
                "content": content,
                "memory_type": "note",
                "confidence": 0.8,
                "importance": 0.7
            }),
        )
        .await;
    let memory_id = match store_mem {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
            .unwrap_or_default(),
        Err(e) => {
            crate::teeprintln!("  [FAIL] store_memory — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if memory_id.is_empty() {
        crate::teeprintln!("  [FAIL] store_memory: no id returned");
        stats.failed += 1;
        return Ok(());
    }

    // store_embedding: associate an embedding with the memory_id.
    let store_emb = client
        .call_tool(
            "store_embedding",
            serde_json::json!({
                "memory_id": memory_id,
                "embedding": [0.1, 0.2, 0.3, 0.4],
                "model": "test-model"
            }),
        )
        .await;
    if let Err(e) = store_emb {
        crate::teeprintln!("  [FAIL] store_embedding — {}", e);
        stats.failed += 1;
        return Ok(());
    }

    // get_embedding: verify the embedding is found.
    let get_emb = client
        .call_tool(
            "get_embedding",
            serde_json::json!({ "memory_id": memory_id }),
        )
        .await;
    let found = match get_emb {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("found").and_then(|f| f.as_bool()))
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_embedding (after store) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if !found {
        crate::teeprintln!("  [FAIL] get_embedding: embedding not found after store (memory_id={})", memory_id);
        stats.failed += 1;
        return Ok(());
    }

    // delete_embedding: delete the embedding by memory_id.
    let del_emb = client
        .call_tool(
            "delete_embedding",
            serde_json::json!({ "memory_id": memory_id }),
        )
        .await;
    let deleted = match del_emb {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("deleted").and_then(|d| d.as_bool()))
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] delete_embedding — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if !deleted {
        crate::teeprintln!("  [FAIL] delete_embedding: not deleted (deleted=false)");
        stats.failed += 1;
        return Ok(());
    }

    // get_embedding: verify the embedding is gone (found=false).
    let get_after = client
        .call_tool(
            "get_embedding",
            serde_json::json!({ "memory_id": memory_id }),
        )
        .await;
    let not_found = match get_after {
        Ok(r) => payload_json(&r)
            .ok()
            .and_then(|v| v.get("found").and_then(|f| f.as_bool()))
            .map(|f| !f)
            .unwrap_or(false),
        Err(e) => {
            crate::teeprintln!("  [FAIL] get_embedding (after delete) — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    if not_found {
        crate::teeprintln!("  [OK] get+delete embedding by memory_id: store->found->delete->not found");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] get_embedding: still found after delete (expected not found)");
        stats.failed += 1;
    }

    Ok(())
}
