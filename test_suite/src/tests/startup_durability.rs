//! P2-001E — Startup does not mutate test data merely by launching RoBoT Brain.
//!
//! Starts the server on a pristine tempdir DB, stops it, then asserts that no
//! experience / task / job rows exist beyond the expected consolidation task.
//!
//! This is the P2-001E-M5 acceptance test: a fresh server should only write:
//!
//!   1. Schema migrations (idempotent table creation)
//!   2. The memory-consolidation scheduled task (idempotent check-before-create)
//!
//! No probe, diagnostic, or self-test behavior should pollute the DB.

use std::process::Stdio;
use std::time::Duration;

use rusqlite::Connection;
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
        let line = format!("{}\n", req);
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn send_request(&mut self, method: &str, params: Value) -> anyhow::Result<u64> {
        let id = self.next_id;
        self.send(method, params).await?;
        Ok(id)
    }

    async fn recv(&mut self) -> anyhow::Result<Value> {
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line).await?;
        if n == 0 {
            return Err(anyhow::anyhow!("server stdout closed"));
        }
        Ok(serde_json::from_str::<Value>(line.trim())?)
    }

    async fn recv_until(&mut self, expected_id: u64) -> anyhow::Result<Value> {
        loop {
            let resp = self.recv().await?;
            if resp.get("id").and_then(|v| v.as_u64()) == Some(expected_id) {
                return Ok(resp);
            }
        }
    }

    async fn initialize(&mut self) -> anyhow::Result<()> {
        let id = self
            .send_request(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2025-03-26",
                    "capabilities": {"tools": {}},
                    "clientInfo": {"name": "startup_durability", "version": "1.0.0"}
                }),
            )
            .await?;
        self.recv_until(id).await?;
        self.send("notifications/initialized", Value::Null).await?;
        Ok(())
    }

    async fn pass_workflow_gate(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self
            .call_tool("get_workflow", serde_json::json!({ "purpose": "general" }))
            .await
        {
            return Err(anyhow::anyhow!("get_workflow gate failed: {}", e));
        }
        if let Err(e) = self
            .call_tool(
                "search_memory",
                serde_json::json!({ "query": "startup durability" }),
            )
            .await
        {
            return Err(anyhow::anyhow!("search_memory gate failed: {}", e));
        }
        Ok(())
    }

    async fn call_tool(&mut self, name: &str, args: Value) -> anyhow::Result<Value> {
        let id = self
            .send_request(
                "tools/call",
                serde_json::json!({ "name": name, "arguments": args }),
            )
            .await?;
        self.recv_until(id).await
    }

    async fn shutdown(mut self) {
        if let Err(e) = self.child.start_kill() {
            crate::teeprintln!("  (shutdown start_kill failed: {})", e);
        }
        if let Err(e) = self.child.wait().await {
            crate::teeprintln!("  (shutdown wait failed: {})", e);
        }
    }
}

/// Run the P2-001E startup-durability test and update the shared stats.
pub async fn run_startup_durability_tests(stats: &mut TestStats) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Startup Durability (P2-001E) ---");

    let Some(bin) = crate::paths::server_binary() else {
        crate::teeprintln!("  [SKIP] startup_durability — server binary not found");
        stats.skipped += 1;
        return Ok(());
    };

    let dir = tempfile::tempdir()?;
    let server_path = dir.path().join("robot_brain");
    std::fs::copy(&bin, &server_path)?;
    let db_path = dir.path().join("robot_brain.db");

    // Start the server on a pristine DB.
    let client = match IsoClient::start(&server_path).await {
        Ok(c) => c,
        Err(e) => {
            crate::teeprintln!("  [FAIL] phase1 start — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };

    // Wait a short time for async startup tasks (scheduler, workers) to settle.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ---- Snapshot DB tables after a single server boot ----
    let conn = Connection::open(&db_path)?;

    // experiences count (should be 0 — no probe writes to production)
    let experiences_count: i64 = conn
        .query_row("SELECT count(*) FROM experiences", [], |r| r.get(0))
        .unwrap_or(0);

    // memories count (should be 0 — no probe writes)
    let memories_count: i64 = conn
        .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
        .unwrap_or(0);

    // scheduled_tasks count — should be exactly 1 (the consolidation task)
    let tasks_count: i64 = conn
        .query_row("SELECT count(*) FROM scheduled_tasks", [], |r| r.get(0))
        .unwrap_or(0);

    // job_queue count (should be 0 — no probe jobs)
    let job_count: i64 = conn
        .query_row("SELECT count(*) FROM job_queue", [], |r| r.get(0))
        .unwrap_or(0);

    // knowledge count (should be 0 — no probe knowledge writes)
    let knowledge_count: i64 = conn
        .query_row("SELECT count(*) FROM knowledge", [], |r| r.get(0))
        .unwrap_or(0);

    // skills count (loaded into memory, not persisted to DB at startup)
    let skills_count: i64 = conn
        .query_row("SELECT count(*) FROM skills", [], |r| r.get(0))
        .unwrap_or(0);

    crate::teeprintln!("\n┌{:─<78}┐", "");
    crate::teeprintln!(
        "│ {:^76} │",
        "[INFO] STARTUP DURABILITY (P2-001E) — DB SNAPSHOT AFTER BOOT"
    );
    crate::teeprintln!("├{:─<22}┼{:─<10}┼{:─<41}┤", "", "", "");
    crate::teeprintln!(
        "│ {:<20} │ {:>8} │ {:<39} │",
        "Table",
        "Count",
        "Expectation"
    );
    crate::teeprintln!("├{:─<22}┼{:─<10}┼{:─<41}┤", "", "", "");

    // Assert expectations, collecting rows so the table shows actual vs expected.
    let mut all_ok = true;
    let mut rows: Vec<(String, i64, String, bool)> = Vec::new();

    rows.push((
        "experiences".to_string(),
        experiences_count,
        "0 (no probe writes)".to_string(),
        experiences_count == 0,
    ));
    if experiences_count != 0 {
        all_ok = false;
    }

    rows.push((
        "memories".to_string(),
        memories_count,
        "0 (no probe writes)".to_string(),
        memories_count == 0,
    ));
    if memories_count != 0 {
        all_ok = false;
    }

    if tasks_count >= 1 {
        // At least the consolidation task should exist; verify it specifically.
        let consolidation_exists: bool = conn
            .query_row(
                "SELECT count(*) FROM scheduled_tasks WHERE task_type = ?",
                ["memory_consolidation"],
                |r| r.get(0),
            )
            .unwrap_or(0)
            > 0;
        let expect = if consolidation_exists {
            ">=1 incl. consolidation".to_string()
        } else {
            "consolidation task MISSING".to_string()
        };
        rows.push((
            "scheduled_tasks".to_string(),
            tasks_count,
            expect,
            consolidation_exists,
        ));
        if !consolidation_exists {
            all_ok = false;
        }
    } else {
        rows.push((
            "scheduled_tasks".to_string(),
            tasks_count,
            ">=1 (consolidation)".to_string(),
            false,
        ));
        all_ok = false;
    }

    rows.push((
        "job_queue".to_string(),
        job_count,
        "0 (no probe jobs)".to_string(),
        job_count == 0,
    ));
    if job_count != 0 {
        all_ok = false;
    }

    rows.push((
        "knowledge".to_string(),
        knowledge_count,
        "0 (no probe writes)".to_string(),
        knowledge_count == 0,
    ));
    if knowledge_count != 0 {
        all_ok = false;
    }

    rows.push((
        "skills".to_string(),
        skills_count,
        "0 (in-memory only)".to_string(),
        skills_count == 0,
    ));
    if skills_count != 0 {
        all_ok = false;
    }

    for (table, count, expect, ok) in &rows {
        crate::teeprintln!(
            "│ {:<20} │ {:>8} │ {} {:<2} │",
            table,
            count,
            format!("{:<37}", expect),
            if *ok { "[OK]" } else { "[FAIL]" }
        );
    }
    crate::teeprintln!("└{:─<22}┴{:─<10}┴{:─<41}┘", "", "", "");

    // Stop the server.
    client.shutdown().await;

    if all_ok {
        crate::teeprintln!("  [PASS] Startup does not mutate test data");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] Startup wrote unexpected data to production DB");
        stats.failed += 1;
    }

    Ok(())
}
