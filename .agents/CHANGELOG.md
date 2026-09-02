# Completed Work (CHANGELOG)

> Historical record of completed work. Forward planning lives in [PLAN.md](PLAN.md).
> Append new completed work here so PLAN.md stays focused on what needs to be done.

- **CoObOpLoop T0.2 Read `src/agent/types.rs` top 20 lines -- DONE (2026-09-01).** Verified agent goal/status types (Architecture §5.7), proper imports (serde), `AgentGoalId` type alias, `GoalStatus` enum (Pending/InProgress/Achieved/Failed/Declined). Documentation/read task, no code changes.

- **CoObOpLoop T0.1 Read `src/experience/queue.rs` top 20 lines -- DONE (2026-09-01).** Verified experience worker job queue (Architecture §23.5), proper imports, JobStatus enum at line 20. Documentation/read task, no code changes.

---

# TIER 1 -- Completed (Consolidated)

**All TIER 1 tasks completed and verified.** Last gate: 454/454 tests, 0 warnings, 0 issues, 0 untested tools.

## Durable Queue (P0-001/002/003, T1-09/10)
- `job_queue` table with migration 012, SQLite enqueue/dequeue, unique `experience_id` column, retry lifecycle fix, partial job restoration on restart. Verified by `queue_durability.rs`.

## #[cfg(test)] Migration (T1-10B)
- Group A (11 files): migrated to test_suite MCP tests. Group B (~48 tests, internal-only): left as Rust unit tests. Zero `#[cfg(test)]` blocks remain in `src/`.

## Loop Health Metrics (T1-13..16)
- `loop_latency`, `confidence_drift`, `promotion-throughput` metrics captured in `metrics.rs`, exposed via `get_system_status` (acp_handler.rs).

## MCP Experience Emission (T1-17/18)
- `emit_tool_experience` hooked at `rmcp/mod.rs:127/141`, idempotent (mutually exclusive match arms).

## Phantom Tool Fix (T1-19)
- Memory handler `get_tools()` now includes all 6 embedding tools — previously listed in `tool_names()`/`execute_tool()` but not advertised, causing phantom tool flags.

## Tool Coverage (T1-20..29)
- All 54+ MCP tools registered in `function_registry/` (acp_tools.rs, coverage_tools.rs). Gate: `untested_tools` empty, `phantom_tools` empty.

## P1-001/P1-002 (Durable Queue)
- Pending job restoration on restart verified (6 criteria). Zero TODOs/FIXMEs/stubs across entire codebase.

## P3-001 (Project Status Sync)
- All CHANGELOG entries dated, README has "Verified State" block, AGENTS.md has same-day gate rule.

## Concurrency Audit (P7)
- Verified: no tokio RwLock await-across-lock, no std Mutex in experience/, single-dispatcher by observer_name, WAL confirmed, kill_on_drop in all 12 test IsoClients.

## Runtime Validation (P8)
- fresh_start.rs covers all P8-M1..M5 tests: first startup, restart on same tempdir, shutdown cleanliness, missing optional config.

## Flow Tests (P9)
- 6 flow tests wired (basic cognition, auto memory retrieval, experience capture, recovery, restart recovery, cross-session memory). All pass.

## Gate Milestone
- 141/141 tests (first green), then ratcheted to 454/454. Zero warnings, zero issues, zero untested tools throughout.

---

# v0.0.1 CONFORMANCE WORK

- **P0** Event spine: `ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation` wired (event_subscriber/handlers.rs).
- **P1** Cognitive loop: `run_agent_goal` MCP tool works (status=Achieved, confidence=0.507).
- **P2** Stub chapters: World Model, Safety Gate, Personality decision_making.
- **P3** Self-check probes: Remaining (→ T1-01..T1-08).
- **P3.1** `#![allow]` violations: 0 in `src/` — clean.
- **P4** Performance maturity: Remaining (→ T1-09..T1-16).
