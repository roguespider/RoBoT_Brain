//! T1-10 — SQLite JobQueue process-restart durability.
//!
//! `test_suite` cannot import `robot_brain`'s `JobQueue` (the two programs are
//! independent; robot_brain is spawned as a subprocess). So this test verifies
//! the durability criterion the *way an operator would*: it boots the real
//! server binary against an isolated directory, injects a pending job row
//! directly into that server's `job_queue` SQLite table (simulating work that
//! was in flight when the previous process died), kills the server, then boots
//! a fresh server in the same directory and confirms the queued job is
//! restored into the live queue and visible via the `get_system_status` MCP
//! tool (`event_bus.pending_jobs`).
//!
//! This is the "queue survives a process restart" done-when criterion for
//! T1-10, exercised end-to-end through the public MCP interface plus a direct
//! SQLite inspection of the durable store.

use std::process::Stdio;
use std::time::Duration;

use rusqlite::Connection;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};

use crate::TestStats;

/// A minimal MCP-over-stdio client for an isolated server instance.
struct IsoClient {
    // Kept alive so the child runs until the client is dropped (kill_on_drop).
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
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params,
        });
        self.next_id += 1;
        let line = format!("{}\n", req);
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn recv(&mut self) -> anyhow::Result<Value> {
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line).await?;
        if n == 0 {
            return Err(anyhow::anyhow!("server stdout closed"));
        }
        Ok(serde_json::from_str::<Value>(line.trim())?)
    }

    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.send(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": {} },
                "clientInfo": { "name": "queue_durability", "version": "1.0.0" }
            }),
        )
        .await?;
        // Consume the initialize response (required to complete the handshake).
        self.recv().await?;
        self.send("notifications/initialized", Value::Null).await?;
        Ok(())
    }

    /// Pass the server's workflow gate (get_workflow -> search_memory) so
    /// substantive tools like get_system_status are unblocked.
    async fn pass_workflow_gate(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self
            .call_tool("get_workflow", serde_json::json!({ "purpose": "general" }))
            .await
        {
            return Err(anyhow::anyhow!("get_workflow gate step failed: {}", e));
        }
        if let Err(e) = self
            .call_tool("search_memory", serde_json::json!({ "query": "queue durability probe" }))
            .await
        {
            return Err(anyhow::anyhow!("search_memory gate step failed: {}", e));
        }
        Ok(())
    }

    async fn call_tool(&mut self, name: &str, args: Value) -> anyhow::Result<Value> {
        self.send(
            "tools/call",
            serde_json::json!({ "name": name, "arguments": args }),
        )
        .await?;
        loop {
            let resp = self.recv().await?;
            if resp.get("id").is_some() {
                return Ok(resp);
            }
        }
    }

    /// Stop the server child process explicitly so the `child` field is used.
    async fn shutdown(mut self) {
        if let Err(e) = self.child.start_kill() {
            crate::teeprintln!("  (shutdown start_kill failed: {})", e);
        }
        if let Err(e) = self.child.wait().await {
            crate::teeprintln!("  (shutdown wait failed: {})", e);
        }
    }
}

/// Run the T1-10 queue-durability test and update the shared stats.
pub async fn run_queue_durability_tests(stats: &mut TestStats) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- JobQueue Restart-Durability (T1-10) ---");

    let Some(bin) = crate::paths::server_binary() else {
        crate::teeprintln!("  [FAIL] queue_durability — server binary not found");
        stats.failed += 1;
        return Ok(());
    };

    let dir = tempfile::tempdir()?;
    let server_path = dir.path().join("robot_brain");
    std::fs::copy(&bin, &server_path)?;
    let db_path = dir.path().join("robot_brain.db");

    // ---- Phase 1: boot a server so the DB + job_queue migration exist. ----
    let mut c1 = match IsoClient::start(&server_path).await {
        Ok(c) => c,
        Err(e) => {
            crate::teeprintln!("  [FAIL] phase1 start — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    // Confirm the migration created the table.
    let table_exists = {
        let conn = Connection::open(&db_path)?;
        let n: i64 = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='job_queue'",
            [],
            |r| r.get(0),
        )?;
        n > 0
    };
    if !table_exists {
        crate::teeprintln!("  [FAIL] job_queue table missing after boot");
        stats.failed += 1;
        return Ok(());
    }
    // Baseline: a fresh server has no pending jobs restored.
    let baseline = pending_jobs(&mut c1).await.unwrap_or(0);
    crate::teeprintln!("  • phase1 baseline pending_jobs = {}", baseline);

    // Inject a pending job row that simulates work left in flight by a
    // crashed previous process. Use a deliberately-unique observer name that
    // the startup probe (which pops `experience_scorer` jobs) will not touch,
    // so the restored row is observable in the live queue rather than being
    // consumed by the probe's own lifecycle verification.
    let probe_id = "durability-probe-001";
    let probe_observer = "durability_probe_observer";
    {
        let conn = Connection::open(&db_path)?;
        let now = now_iso();
        conn.execute(
            "INSERT OR REPLACE INTO job_queue
                (id, observer_name, status, last_error, attempts, created_at, updated_at)
             VALUES (?1, ?2, 'pending', NULL, 0, ?3, ?3)",
            rusqlite::params![probe_id, probe_observer, now],
        )?;
    }
    crate::teeprintln!("  • injected pending row id={} observer={}", probe_id, probe_observer);

    // Kill server 1 (explicit shutdown simulates a process crash/exit).
    c1.shutdown().await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    // ---- Phase 2: boot a fresh server in the same dir; it must restore. ----
    let mut c2 = match IsoClient::start(&server_path).await {
        Ok(c) => c,
        Err(e) => {
            crate::teeprintln!("  [FAIL] phase2 start — {}", e);
            stats.failed += 1;
            return Ok(());
        }
    };
    let restored_pending = match pending_jobs(&mut c2).await {
        Ok(n) => n,
        Err(e) => {
            crate::teeprintln!("  [FAIL] phase2 get_system_status failed — {}", e);
            stats.failed += 1;
            c2.shutdown().await;
            return Ok(());
        }
    };
    crate::teeprintln!("  • phase2 restored pending_jobs = {}", restored_pending);

    // The restored live queue must contain the injected job (pending_count is
    // the in-memory count of pending jobs, which restore_from_database
    // rebuilds from the durable table).
    if restored_pending > baseline {
        crate::teeprintln!("  [OK] pending job survived process restart (restored > baseline)");
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] pending job did NOT survive restart (restored={}, baseline={})",
            restored_pending,
            baseline
        );
        stats.failed += 1;
    }

    // Confirm the row is still in SQLite with status=pending (not silently
    // completed/failed by the restore path).
    let row_status = {
        let conn = Connection::open(&db_path)?;
        conn.query_row(
            "SELECT status FROM job_queue WHERE id=?1",
            rusqlite::params![probe_id],
            |r| r.get::<_, String>(0),
        )
        .ok()
    };
    crate::teeprintln!("  • durable row status after restart = {:?}", row_status);
    if row_status.as_deref() == Some("pending") {
        crate::teeprintln!("  [OK] durable row intact (status=pending)");
        stats.passed += 1;
    } else {
        crate::teeprintln!("  [FAIL] durable row not intact");
        stats.failed += 1;
    }

    // Cleanup: stop the second server.
    c2.shutdown().await;
    Ok(())
}

/// Read `event_bus.pending_jobs` from `get_system_status`.
async fn pending_jobs(c: &mut IsoClient) -> anyhow::Result<i64> {
    let resp = c
        .call_tool("get_system_status", serde_json::json!({}))
        .await?;
    // If the server returned an MCP-level error, surface it.
    if let Some(err) = resp.get("error") {
        return Err(anyhow::anyhow!("MCP error: {}", err));
    }
    // result.content[0].text -> JSON with event_bus.pending_jobs
    let text = resp
        .pointer("/result/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!("no content text (resp={})", resp.to_string().chars().take(300).collect::<String>())
        })?;
    let parsed: Value = serde_json::from_str(text)?;
    let n = parsed
        .pointer("/event_bus/pending_jobs")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            anyhow::anyhow!("no pending_jobs field (text={})", text.chars().take(300).collect::<String>())
        })?;
    Ok(n)
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}
