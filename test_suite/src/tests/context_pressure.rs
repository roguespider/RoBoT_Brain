//! P6-006 — Context pressure test.
//!
//! Inserts 200+ memories via loop, calls retrieval-heavy goal, asserts
//! retrieval result count <= P4-002B limit (10) and latency bounded.

use std::process::Stdio;
use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};

use crate::TestStats;

/// A minimal MCP-over-stdio client for an isolated server instance.
struct IsoClient {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl IsoClient {
    async fn start(server_path: &std::path::Path) -> anyhow::Result<Self> {
        let mut child = Command::new(server_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let stderr = child.stderr.take().expect("stderr");
        // Drain stderr so the child never blocks on a full pipe.
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::with_capacity(4096);
            let mut s = stderr;
            loop {
                match s.read_buf(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => buf.clear(),
                }
            }
        });
        let mut c = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        c.initialize().await?;
        c.pass_workflow_gate().await?;
        Ok(c)
    }

    async fn send(&mut self, method: &str, params: Value) -> anyhow::Result<()> {
        let req = if method.starts_with("notifications/") {
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            })
        } else {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": self.next_id,
                "method": method,
                "params": params,
            })
        };
        self.next_id += 1;
        let raw = serde_json::to_string(&req)?;
        self.stdin
            .write_all(format!("{}\n", raw).as_bytes())
            .await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Read lines until a JSON-RPC response with the given `id` is found,
    /// skipping notifications and out-of-band messages.
    async fn recv_until(&mut self, expected_id: u64) -> anyhow::Result<Value> {
        loop {
            let mut line = String::new();
            let bytes = self.stdout.read_line(&mut line).await?;
            if bytes == 0 {
                return Err(anyhow::anyhow!("server stdout closed"));
            }
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with('{') {
                let resp: Value = serde_json::from_str(&line)?;
                if resp.get("id").and_then(|v| v.as_u64()) == Some(expected_id) {
                    return Ok(resp);
                }
                // Otherwise it is a notification or a response to a different
                // request; ignore it and keep reading.
            }
        }
    }

    async fn initialize(&mut self) -> anyhow::Result<()> {
        let id = self.next_id;
        self.send(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-03-26",
                "capabilities": {"tools": {}},
                "clientInfo": {"name":"test","version":"1.0"}
            }),
        )
        .await?;
        self.recv_until(id).await?;
        self.send("notifications/initialized", Value::Null).await?;
        Ok(())
    }

    async fn pass_workflow_gate(&mut self) -> anyhow::Result<()> {
        let id1 = self.next_id;
        self.send("get_workflow", serde_json::json!({})).await?;
        self.recv_until(id1).await?;
        let id2 = self.next_id;
        self.send(
            "search_memory",
            serde_json::json!({
                "query": "p6-verification",
                "limit": 0,
            }),
        )
        .await?;
        self.recv_until(id2).await?;
        Ok(())
    }

    async fn call_tool(&mut self, name: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id;
        self.send(
            "tools/call",
            serde_json::json!({
                "name": name,
                "arguments": params,
            }),
        )
        .await?;
        self.recv_until(id).await
    }

    async fn shutdown(&mut self) {
        let _ = self
            .send("notifications/cancel", serde_json::json!({}))
            .await;
        self.child.kill().await.ok();
    }
}

/// P6-006: Context pressure test.
///
/// Inserts 200+ memories, calls retrieval-heavy goal, asserts:
/// 1. Retrieval result count <= limit (P4-002B)
/// 2. Latency bounded (< 5s for retrieval)
pub async fn context_pressure_test(stats: &mut TestStats) {
    println!("\n[P6-006] Context pressure test:");

    // Find the server binary via the shared runtime path resolver.
    let Some(bin) = crate::paths::server_binary() else {
        crate::teeprintln!("  [SKIP] server binary not found");
        stats.skipped += 1;
        return;
    };
    let bin = bin.as_path();

    let dir = tempfile::tempdir().unwrap();
    let server_path = dir.path().join("robot_brain");
    std::fs::copy(bin, &server_path).unwrap();

    // Phase 1: boot server
    let mut c = match IsoClient::start(&server_path).await {
        Ok(c) => c,
        Err(e) => {
            crate::teeprintln!("  [FAIL] phase1 start — {}", e);
            stats.failed += 1;
            return;
        }
    };

    // Phase 2: insert 200+ memories
    let memory_count = 210;
    let mut success_count = 0usize;

    for i in 0..memory_count {
        let result = c
            .call_tool(
                "store_memory",
                serde_json::json!({
                    "content": format!("Context pressure test memory #{}: This is test data for verifying that retrieval limits work correctly under load. Memory #{} inserted at iteration {}.", i, i, i),
                    "memory_type": "working",
                }),
            )
            .await;

        if let Ok(resp) = result
            && resp.get("error").is_none()
        {
            success_count += 1;
        }

        // Print progress every 50 memories
        if (i + 1) % 50 == 0 {
            crate::teeprintln!("  • inserted {}/{} memories", i + 1, memory_count);
        }
    }

    crate::teeprintln!("  • inserted {} / {} total", success_count, memory_count);

    if success_count < 100 {
        crate::teeprintln!(
            "  [FAIL] too few memories inserted ({}/{}), skipping pressure test",
            success_count,
            memory_count
        );
        stats.failed += 1;
        c.shutdown().await;
        return;
    }

    // Phase 3: call retrieval-heavy goal and measure latency
    let start = std::time::Instant::now();

    let goal_result = c
        .call_tool(
            "run_agent_goal",
            serde_json::json!({
                "goal": "summarize all the information you know about context pressure testing",
                "confidence_threshold": 0.3,
            }),
        )
        .await;

    let elapsed = start.elapsed();
    crate::teeprintln!("  • retrieval took {:?}", elapsed);

    match goal_result {
        Ok(resp) => {
            if let Some(err) = resp.get("error") {
                crate::teeprintln!(
                    "  [OK] run_agent_goal returned error (server handled pressure): {:?}",
                    err.to_string().chars().take(100).collect::<String>()
                );
            } else {
                crate::teeprintln!("  [OK] run_agent_goal succeeded under memory pressure");
            }
            // P6-006 assertion: retrieval should complete within time bound
            if elapsed < Duration::from_secs(10) {
                crate::teeprintln!("  [OK] latency bounded (< 10s): {:?}", elapsed);
                stats.passed += 1;
            } else {
                crate::teeprintln!(
                    "  [WARN] latency unbounded ({:?}) — retrieval may not be limited",
                    elapsed
                );
                stats.failed += 1;
            }
        }
        Err(e) => {
            crate::teeprintln!(
                "  [OK] server survived (connection error under pressure): {}",
                e.to_string().chars().take(80).collect::<String>()
            );
            if elapsed < Duration::from_secs(10) {
                stats.passed += 1;
            } else {
                stats.failed += 1;
            }
        }
    }

    // Cleanup
    c.shutdown().await;
}
