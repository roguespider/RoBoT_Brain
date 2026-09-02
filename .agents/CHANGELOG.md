# Completed Work (CHANGELOG)

> Historical record of completed work. Forward planning lives in [PLAN.md](PLAN.md).
> Append new completed work here so PLAN.md stays focused on what needs to be done.

- **CoObOpLoop T0 Foundation (read tasks) -- DONE (2026-09-01).** T0.1: verified `src/experience/queue.rs` (experience worker job queue, Architecture §23.5). T0.2: verified `src/agent/types.rs` (agent goal/status types, Architecture §5.7). Both read-only, no code changes.

---

# TIER 1 -- Completed (Consolidated)

**All TIER 1 tasks completed and verified.** Gate: 454/454 tests, 0 warnings, 0 issues, 0 untested tools. First green was 141/141, ratcheted to 454.

## Durable Queue + P1-001 (P0-001/002/003, T1-09/10)
- `job_queue` table with migration 012, SQLite enqueue/dequeue, unique `experience_id`, retry lifecycle fix, partial job restoration on restart. Verified by `queue_durability.rs`. Zero TODOs/FIXMEs/stubs across entire codebase.

## #[cfg(test)] Migration (T1-10B)
- Group A (11 files): migrated to test_suite MCP tests. Group B (~48 tests, internal-only): left as Rust unit tests. Zero `#[cfg(test)]` blocks remain in `src/`.

## Loop Health Metrics (T1-13..16)
- `loop_latency`, `confidence_drift`, `promotion-throughput` metrics in `metrics.rs`, exposed via `get_system_status` (acp_handler.rs).

## MCP Experience Emission (T1-17/18)
- `emit_tool_experience` at `rmcp/mod.rs:127/141`, idempotent (mutually exclusive match arms).

## Phantom Tool Fix (T1-19)
- Memory handler `get_tools()` now includes all 6 embedding tools — previously listed in `tool_names()`/`execute_tool()` but not advertised, causing phantom tool flags.

## Tool Coverage (T1-20..29)
- All 54+ MCP tools in `function_registry/` (acp_tools.rs, coverage_tools.rs). Gate: `untested_tools` empty, `phantom_tools` empty.

## P3-001 (Project Status Sync)
- All CHANGELOG entries dated, README has "Verified State" block, AGENTS.md has same-day gate rule.

## Concurrency Audit (P7) + Runtime (P8) + Flow Tests (P9)
- Verified: no tokio RwLock await-across-lock, no std Mutex in experience/, single-dispatcher by observer_name, WAL confirmed, kill_on_drop in all 12 test IsoClients.
- fresh_start.rs covers P8-M1..M5 (first startup, restart, shutdown, missing config).
- 6 flow tests wired (basic cognition, auto memory retrieval, experience capture, recovery, restart recovery, cross-session memory). All pass.

---

# v0.0.1 CONFORMANCE WORK

- **P0** Event spine: `ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation` wired (event_subscriber/handlers.rs).
- **P1** Cognitive loop: `run_agent_goal` MCP tool works (status=Achieved, confidence=0.507).
- **P2** Stub chapters: World Model, Safety Gate, Personality decision_making.
- **P3** Self-check probes: Remaining (→ T1-01..T1-08).
- **P3.1** `#![allow]` violations: 0 in `src/` — clean.
- **P4** Performance maturity: Remaining (→ T1-09..T1-16).
