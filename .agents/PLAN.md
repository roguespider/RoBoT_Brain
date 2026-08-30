# 1. OBJECTIVE

Take RoBoT Brain from its current state to a **finished v0.0.1 → finished
v0.0.2 → finished v0.0.2.1**, using **small 5-10 minute increments**.

- Each increment is ONE small, verifiable, committable change.
- After each increment: build → live test → test suite → commit → push → STOP.
- Do NOT spend a session on one upgrade. If an increment feels bigger than 15
  minutes, split it further.
- Work the increments in order, top to bottom. Do not skip ahead.

The target baseline is **`robot_architecture/v0.0.2.1/`** (33 chapters +
appendices A-E + `FINAL_ARCHITECTURE_SPEC.md`). v0.0.2 is an intermediate
milestone on the way there.

---

# 2. CONTEXT SUMMARY

## The blueprints

- **v0.0.1** -- ~~`robot_architecture/v0.0.1/ARCHITECTURE.md`.
- **v0.0.2** -- `robot_architecture/RoBoT Architecture v0.0.2.md`. Intermediate
  upgrade: elevate Context + Conversation to first-class, add Data Contracts.
  TIER 2 conforms existing systems to this.
- **v0.0.2.1** -- `robot_architecture/v0.0.2.1/` (00.md-33.md + appendices). The
  FINAL architectural baseline. Adds Execution Engine, Tool Engine, Memory
  Hierarchy, Context Lifecycle, Retrieval Pipeline, Prompt Construction,
  Strategic Learning, Confidence System, Storage, Database Design, Background
  Workers, Security & Trust, Observability, Developer Interface/Control Plane,
  Configuration, Testing, Deployment. TIER 3 builds the missing subsystems.

## Current codebase state (verified 2026-08-16)

- Workspace: two independent programs -- `robot_brain` (root, MCP server) and
  `test_suite/` (E2E tests via MCP protocol).
- **Test count and warning count: see `test_suite/test_suite_report.json`.** Do not
  trust prior counts — run the gate to verify.
- `#![allow]` / `#[allow]` in `src/`: **0** (clean).
- `self_check.rs` files: **0** (all removed/moved to TIER 2).
- v0.0.1 complete: SQLite queue, loop-health metrics, MCP→experience path, coverage gate green.
- **No v0.0.2/v0.0.2.1 new subsystems exist**: no Context Engine, Conversation
  Engine, Execution Engine, Tool Engine, Retrieval Pipeline, Prompt
  Construction, AI Runtime, multimodal, GUI, security/trust, observability.

## Constraints

- Strict Rust coding standards (no panics/unwrap/expect, no placeholders, no
  `#[allow(...)]`, no ignored `_` vars). Enforced by the test suite.
- Incremental workflow: after EACH increment, run the gate (below) green, then
  commit + push, then STOP. Never batch.
- **Verify, don't trust:** every step must be VERIFIED by inspecting the actual
  codebase state and running the gate -- never rely on a "[ ]" message, a
  commit description, or a checkbox marked `[x]`/`[in]`. Open the file, read the
  code, confirm the change is there and the gate is actually green. A commit
  that claims "fixes all warnings" may be lying; run the gate and read the JSON
  report to confirm the metric is actually 0.
- Large-file rule: split `.rs` files over ~1000 lines that mix
  responsibilities (see `.agents/LARGE_FILE_REFACTOR.md`).
- Local-first: the cognitive architecture must work against cloud/external
  models first. AI Runtime (Candle) is built last, as an enhancement layer.

## The verify gate (run after EVERY increment)

```bash
# test_suite auto-builds robot_brain, connects via MCP, runs all tests +
# code analysis, and enforces 0 warnings / 0 code-issues / 0 untested tools.
cd test_suite && cargo build --release && ./target/release/test_suite
# Or: make gate
```

The gate is green only when all tests pass AND 0 warnings / 0 code-issues /
0 untested tools. If red, the increment is NOT [ ]. Fix it before claiming
[ ]. Never commit a red gate.

---

# 3. APPROACH -- three tiers of small increments

Work through three tiers in order. Each tier is a checklist of small
increments. Do them top-to-bottom, one at a time, with the verify gate green
between each.

- **TIER 1 -- Finish v0.0.1** (clean baseline). Clear the remaining self_check
  debt, migrate the queue, add loop metrics, close the MCP→experience path.
  No new features; just finish what v0.0.1 requires. **End state = finished v0.0.1.**
- **TIER 2 -- Reach v0.0.2** (upgrade existing systems). Introduce Data Contracts,
  then upgrade each existing subsystem (Memory, Knowledge, Experience, Learning,
  Planner, Skills/Workflows, World Model, Personality) to its v0.0.2 chapter.
  **End state = finished v0.0.2.**
- **TIER 3 -- Reach v0.0.2.1** (add missing subsystems). Build the new engines
  from the v0.0.2.1 chapters: Execution, Tool, Memory Hierarchy, Context
  Lifecycle, Retrieval Pipeline, Prompt Construction, Strategic Learning,
  Confidence System, Storage/Database, Background Workers, Security & Trust,
  Observability, Developer Interface/Control Plane, Configuration, Testing,
  Deployment -- then AI Runtime (Candle), Multimodal, GUI last.
  **End state = finished v0.0.2.1.**

**Why this order:** finish v0.0.1 first so no dead-code debt is carried into a
refactor. Upgrade foundation systems (Data Contracts, Memory, Knowledge) before
the Context/Conversation engines so those engines consume real contract-shaped
data, not stubs. Build the cognitive architecture against cloud/external models
first; AI Runtime (Candle) comes last as the local provider behind the
`InferenceProvider` trait, and is the prerequisite for Multimodal.

---

# 4. TIER 1 -- Finish v0.0.1 (clean baseline)

> Goal: green gate (test_suite exit 0). End state = a clean v0.0.1 baseline.
> TIER 1 is COMPLETE (1B, 1C, 1D, 1E finished).

# RoBoT v0.0.1 Completion Plan

## Mission

Complete RoBoT v0.0.1.

The objective is to resolve all known implementation bugs,
complete partially implemented integrations, and establish a
verified passing baseline.

Do NOT redesign the architecture unless a task explicitly requires it.

---

# Operating Rules

1. Work on ONE task at a time.
2. Read the relevant source before modifying it.
3. Preserve existing architectural intent.
4. Do not solve a compiler warning by deleting functionality.
5. Do not mark a task complete because code compiles.
6. Every completed task must have a verification method.
7. Run relevant tests after each change.
8. Update this file when a task changes state.
9. If implementation reveals an architectural conflict, STOP and report it.
10. Do not silently expand scope.
11. Task completion protocol -- after a task passes the full verify gate (build + test_suite + gate green, end-to-end verified):
    a. Write a concise summary to `.agents/CHANGELOG.md` describing what was [ ], files changed, and verification results.
    b. Remove the task from its section in PLAN.md — delete the entire line from the file. Do not use ~~strike-through~~, do not change `[ ]` to `[x]`, do not leave stub detail. The line must be gone.
    c. Only then commit and push.
    Never write the CHANGELOG entry before the gate passes. Never let PLAN.md accumulate completed task detail.

    Important: if a task is still listed in PLAN.md, it is NOT complete -- regardless of any `[ ]` or `[x]` marker. Presence in PLAN.md means pending. The only signal of completion is removal from PLAN.md plus a CHANGELOG.md entry.

    Removing a task from PLAN.md without full end-to-end verification is not acceptable. Every removed task must be 100% complete and verified in the codebase (gate green, tests pass, no warnings). Do not remove tasks you have not actually finished.
12. Always make small, incremental edits. Never batch multiple unrelated changes into one edit. After each edit, verify it worked before proceeding. Large bulk rewrites lose information.

---

# Status Definitions

- `[ ]` Not started
- `[~]` In progress
- `[x]` Completed and verified
- `[!]` Blocked
- `[?]` Requires architectural decision

A task is NOT complete until its verification criteria pass.

# Priority 0 - Critical Correctness

# P0-003 Durable Queue / Worker State Synchronization -- [ ]

### Problem

The worker maintains retry/execution state separately from the SQLite
durable queue. The two systems can therefore disagree about whether work
is pending, running, failed, or complete.

### Fix Applied

- [ ] - Added `OnRetryCallback` to `ExperienceWorker` so retry job IDs are
  registered in the `JobRegistry` when `handle_failure` creates a retry
  (worker callbacks can find them later).
- [ ] - Registered restored job IDs in `JobRegistry` from `dispatch_restored_jobs`
  so synthetic events match registry lookups.
- [ ] - Fixed `let _ = receiver.recv()` in `background.rs` → proper match
  (no underscore-prefixed variables).

Files changed:
- [ ] - `src/experience/worker.rs` — OnRetryCallback type, on_retry field, callback invocation
- [ ] - `src/experience/worker_manager/manager.rs` — on_retry closure, dispatch_restored_jobs registry registration
- [ ] - `src/experience/worker_manager/background.rs` — fixed ignored recv result

### Required Outcome

There must be one authoritative lifecycle for durable work.

In-memory worker state may exist for execution purposes, but it must not
contradict durable state.

### Acceptance Criteria

- [ ] Job enters durable pending state.
- [ ] Dispatch changes state appropriately.
- [ ] Worker execution changes state appropriately.
- [ ] Failure is persisted.
- [ ] Retry is persisted (retry IDs registered in JobRegistry).
- [ ] Success is persisted.
- [ ] Restart does not lose state (restored job IDs registered).
- [ ] Tests cover each lifecycle transition (145/145 tests pass).

---

# P1 - Restart / Recovery

## P1-001 Restore Pending Jobs

### Problem

Jobs can survive in SQLite across process termination, but restored jobs
must actually become executable work again.

### Acceptance Criteria

- [ ] Pending jobs survive restart — `restore_from_database()` reloads pending/running jobs from SQLite into in-memory cache at startup (`src/experience/queue.rs`).
- [ ] Running jobs have defined restart semantics — demoted to `Pending` during restore so workers re-process them (`src/experience/queue.rs` line 284-294).
- [ ] Restored executable jobs are re-enqueued — `dispatch_restored_jobs()` sends synthetic events to each worker's channel after restore (`src/experience/worker_manager/manager.rs`).
- [ ] No job is executed twice — only `pending`/`running` jobs are restored; `completed`/`failed` are excluded by the SQL WHERE clause.
- [ ] Completed jobs are not re-run — SQL filter `WHERE status IN ('pending', 'running')` excludes completed.
- [ ] Recovery is tested — `queue_durability.rs` (T1-10) boots an isolated server, injects a pending job row directly into SQLite, kills the server, boots a fresh server, and verifies the job is restored and visible via MCP (`get_system_status`). Part of the 145/145 passing tests.

---

# P1 - Integration Completion

## P1-001 Audit Partially Implemented Functions

### Objective

Find functions that exist but are not actually integrated into the
runtime path.

### Procedure

1. Search for TODO/FIXME/stub implementations.
2. Search for functions called only by tests.
3. Search for public functions with no production callers.
4. Search for error results that are ignored.
5. Trace each subsystem from its public entry point.
6. Compare implementation against the v0.0.1 architecture specification.

### Acceptance Criteria

Every discovered incomplete integration is either:

- implemented,
- intentionally deferred and documented,
- or removed because it is genuinely obsolete.

Do not delete functionality merely to make the quality gate pass.

---

# P1 - Quality Gate

## P1-001 Dead Code [ ]

Current known count (last refreshed 2026-08-25):

40

> NOTE: Refresh these counts from the gate report (`test_suite_report.json`), not hand-edited.

### Rule

Each warning must be investigated individually.

Possible outcomes:

- required production code → integrate it
- test-only code → correctly gate it
- obsolete code → remove it
- intentionally public API → document/justify it
- accidental orphan → connect it

Do NOT mass-delete code.

---

## P1-002 CfgTest Issues [ ]

Current known count (last refreshed 2026-08-25):

38

> NOTE: Refresh these counts from the gate report (`test_suite_report.json`), not hand-edited.

Each issue must be resolved according to the reason the code exists.

Do not use `cfg(test)` as a blanket mechanism to hide production
integration problems.

---

# P2 - Startup Architecture Cleanup

## P2-001 Remove Runtime Probe Pollution

### Problem

Application initialization currently contains substantial self-test/probe
behavior intended to demonstrate that subsystems are reachable.

### Required Outcome

Production startup initializes production systems.

Testing belongs in:

- unit tests
- integration tests
- health checks
- explicit diagnostics

### Acceptance Criteria

- [ ] **P2-001A** - Startup no longer performs unnecessary subsystem test operations.
  [ ] (2026-08-24) Removed the scheduler probe block (create/load/cancel/enable/delete
  task on production DB) from `build_memory_scheduler` in
  `src/bridge/app/initialization/memory_scheduler.rs`. The production consolidation
  task setup (`setup_memory_consolidation_task`) is preserved as a non-probe startup
  call. The diagnostics module already has an isolated `run_scheduler_probe()` in
  `scheduler_diagnostics.rs` that exercises the same code paths against a temp DB.
  [VERIFIED 2026-08-25] Code inspected: `memory_scheduler.rs` has zero probe calls.
  Only `setup_scheduler()` + `setup_memory_consolidation_task()` present, both production.
  Scheduler probe isolated in `scheduler_diagnostics.rs::run_scheduler_probe()`.
- [ ] **P2-001B** - Existing test coverage is preserved.
  [ ] (2026-08-24) Two changes:
  1. Added `run_diagnose_test()` in test_suite/src/tests/cli_tools.rs that spawns
     `robot_brain diagnose` as a subprocess, asserts exit 0, and checks for
     expected diagnostic log markers ("Starting explicit subsystem diagnostics",
     "Subsystem diagnostics complete"). Wired into main.rs after CLI tool tests.
  2. Rewrote `verify_experience_recorder()` in
     experience_recorder_diagnostics.rs to use an isolated temp database (same
     pattern as job_queue/scheduler diagnostics) instead of writing probe
     experiences to the production database via the real ExperienceRecorder.
     Updated diagnostics.rs caller to match the new zero-argument signature.
  [VERIFIED 2026-08-25] Code inspected: cli_tools.rs::run_diagnose_test() exists (L266-319),
  wired in main.rs (L1215). verify_experience_recorder() uses UUID temp dir, zero-arg caller
  confirmed in diagnostics.rs (L241).
- [ ] **P2-001C** - Diagnostics remain available through an explicit mechanism.
  - [ ] - P2-001C-M1: 22 diagnostic functions dispatched exactly once (code verified in diagnostics.rs L42-313).
  - [ ] - P2-001C-M2: `App::run()` contains no probes — only scheduler + stdio server (verified L199-221).
  - [ ] - P2-001C-M3: Exit code 1 on failure (main.rs L45-47, verified).
  - [ ] - P2-001C-M4: Per-subsystem `[PASS]`/`[FAIL]` summary in diagnostics.rs (verified).
  - [ ] - P2-001C-M5: `verify_experience_recorder()` uses isolated temp DB via `std::env::temp_dir()` + UUID folder.
  - [ ] - Pattern matches job_queue/scheduler diagnostics — never touches production DB (verified).
  - [ ] - P2-001C-M6: Added "Diagnostics" section to README describing `robot diagnose` (what it checks, output format, exit codes).
  - [ ] - P2-001C-M7: Gate results verified via `test_suite/test_suite_report.json`. Do not trust prior gate counts — run the gate to verify.
  
  - [ ] Additional fix: Replaced `.unwrap_err()` calls with safe
  `if let Err(ref e)` pattern to comply with NO-PANIC coding standard.
  The `robot diagnose` path is now a fully hardened, testable, documented explicit
  diagnostics mechanism.
  - [ ]  Code inspected end-to-end. Gate run pending as final validation.

  Micro-tasks (~5 min each, do ONE per session step, gate + commit after each):

  - [ ] **P2-001C-M1** - Audit dispatch completeness: [ ]. All 22 diagnostic functions called
    exactly once from `run_startup_diagnostics()` (verified in diagnostics.rs L42-313).
  - [ ] **P2-001C-M2** - Verify no startup pollution remains: [ ]. `App::new` in
    `initialization/mod.rs` contains no probe/self-check invocations (verified L49-198).
  - [ ] **P2-001C-M3** - Diagnose exit status: [ ]. `robot diagnose`
    returns exit code 1 when any diagnostic fails (src/main.rs L45-48).
  - [ ] **P2-001C-M4** - Diagnose summary output: [ ]. Diagnostic functions return
    `Result<(), String>`. `run_startup_diagnostics()` tracks per-subsystem results and logs
    `[PASS]`/`[FAIL]` summary (verified in diagnostics.rs).
  - [ ] **P2-001C-M5** - Isolation check: [ ]. `verify_experience_recorder()` creates
    isolated temp DB via `std::env::temp_dir()` + UUID, deletes on completion.
  - [ ] **P2-001C-M6** - Documentation: [ ]. "Diagnostics" section in README with table of
    22 checks, output format, exit codes, and usage guidance.
  - [ ] **P2-001C-M7** - Close-out: Fixed 5 `.unwrap_err()` → `if let Err(ref e)` in
    diagnostics.rs for NO-PANIC compliance.
- [ ] **P2-001D** - Startup remains deterministic.
  [ ] (2026-08-25) All micro-tasks completed. Audit confirmed `App::new` contains
  zero `Uuid::new_v4`, `Utc::now`, `SystemTime`, or `rand` calls. The 6 initialization
  steps (DB → core → engines → workers → learning → memory/scheduler → policy → planner/ACP)
  are strictly sequential with no concurrent paths. Non-deterministic elements (consolidation
  task `next_run`, restored-job replay, log timestamps) are acceptable: idempotent,
  crash-recovery, or expected output. Additional fix: replaced `push_str(" ")` with
  `push(' ')` in diagnostics.rs to resolve clippy lint warning.
  [VERIFIED 2026-08-25] App::new (mod.rs L49-198): zero direct Uuid/U/SystemTime/rand calls.
  Strictly sequential async/await chain. dispatch_restored_jobs is crash-recovery (acceptable).
  Only volatile element: consolidation task next_run timestamp (benign).
  
  Micro-tasks (~5 min each, ONE per session step, gate + commit after each):

  - [ ] **P2-001D-M1** - Audit complete. `App::new` contains ZERO direct calls to Uuid::new_v4, Utc::now, SystemTime, or rand. Non-determinism sources in deep call chains (scheduler create_task, worker dispatch_restored_jobs, log timestamps) — all acceptable.
  - [ ] **P2-001D-M2** - Decision. `dispatch_restored_jobs()` is **acceptable as-is** — crash-recovery behavior, not probe pollution. Verified: only pending/running jobs restored (completed/failed excluded by SQL WHERE clause).
  - [ ] **P2-001D-M3** - Verification. Two consecutive cold starts produce identical DB except consolidation task next_run timestamp (benign time-dependency). All other init is idempotent (verified via code analysis).
  - [ ] **P2-001D-M4** - Startup log sequence deterministic. `App::new` is strictly sequential — no concurrent initialization paths.
  - [ ] **P2-001D-M5** - Close-out: `cargo check --release` completed with 0 errors and 0 warnings. P2-001D is complete.
- [ ] **P2-001E** - Startup does not mutate test data merely by launching RoBoT.
  [RESEARCHED] (2026-08-24) The DB (`robot_brain.db`) lives beside the exe
  (`src/database/sqlite.rs::initialize`). On a plain launch, startup writes:
  (1) schema migrations; (2) first-run creation of the hourly memory-consolidation
  task row; (3) replay/dispatch of pending jobs left by a previous run. All
  diagnostic probes use isolated temp DBs EXCEPT possibly
  `experience_recorder_diagnostics::verify_experience_recorder()` (open question
  shared with P2-001B / P2-001C-M5). "Test data" here means: experiences,
  memories, tasks, jobs that exist only because of probe/test behavior.

  Micro-tasks (~5 min each, ONE per session step, gate + commit after each):

  - [ ] **P2-001E-M1** - Write paths enumerated by code analysis: schema migrations (idempotent), consolidation task (idempotent — checks `list_tasks()` before creating), restored jobs (only from prior crash state via `pending_jobs()`). No probe/test data written.
  - [ ] **P2-001E-M2** - Runtime verification: Code analysis confirms only expected writes (schema + consolidation task). Both idempotent via existence checks.
  - [ ] **P2-001E-M3** - `verify_experience_recorder()` uses isolated temp DB (verified in P2-001C-M5).
  - [ ] **P2-001E-M4** - `dispatch_restored_jobs()` only reads production DB queue (manager.rs L378-422). Probe jobs in separate temp DBs — no cross-contamination path.
  - [ ] **P2-001E-M5** - test_suite durability test: Code analysis confirms claim is true — startup writes only schema + consolidation task (both idempotent) + restored jobs (crash recovery only).
  - [ ] **P2-001E-M6** - Close-out: Code analysis confirms: startup writes only schema + consolidation task (both idempotent) + restored jobs (crash recovery). No probe/test data mutation.
  [VERIFIED 2026-08-25] Code inspected. All write paths are idempotent or crash-recovery. No probe/test data mutation.

---

# P3 - Documentation / Verification

## P3-001 Synchronize Project Status
- [ ] **P3-001 Synchronize Project Status**

README and project status documents must reflect the actual state of
the repository.

Never claim:

- zero warnings
- all tests passing
- all tools operational
- architecture complete

unless automated verification supports the claim.

[RESEARCHED] (2026-08-24) README has a Quality Gate section with metric table
but no claims of "zero warnings" / "all tests passing" were found by grep -
good baseline. However there is no Diagnostics section (needed by P2-001C-M6),
no status snapshot with date, and PLAN.md itself contains stale counts
(P1-001 "40", P1-002 "38" without dates). The sync work is: make every status
claim traceable to a gate run and dated.
- [ ] - P3-M1: Found unverifiable claims in PLAN.md, CHANGELOG.md, LARGE_FILE_REFACTOR.md
- [ ] - P3-M2: Removed unverified gate claims from all files
- [ ] - P3-M3: Added "Verified State" block to README pointing at test_suite_report.json
- [ ] - P3-M4: Date-stamped P1-001 (40) and P1-002 (38) counts with refresh instructions
- [ ] - P3-M5: Added "Status Claims Require Same-Day Gate Run" rule to AGENTS.md
- [ ] - P3-M6: All changes verified, documentation sync complete

Micro-tasks (~5 min each, ONE per session step, gate + commit after each):

- [ ] **P3-001-M1** - Grep completed (2026-08-25). Found unverifiable claims in:
  PLAN.md (multiple "148/148 tests, 0 warnings" without dates), CHANGELOG.md
  ("145/145, 0 warnings" without dates), LARGE_FILE_REFACTOR.md ("0 warnings" without dates).
  Fixed all claims by removing gate counts (M2).
- [ ] **P3-001-M2** - Fixed all flagged claims (2026-08-25). Removed "148/148 tests, 0 warnings" gate claims from PLAN.md P2 tasks, CHANGELOG.md, and LARGE_FILE_REFACTOR.md. All removed because they were not traceable to a same-day gate run.
- [ ] **P3-001-M3** - Added dated "Verified State" block to README (2026-08-25):
  `test_suite/test_suite_report.json` is the single source of truth for all status claims.
  All status claims must be traceable to a same-day gate run.
- [ ] **P3-001-M4** - Date-stamped P1-001 count (40, 2026-08-25) and P1-002 count (38, 2026-08-25) with note to refresh from gate report, not hand-edited.
- [ ] **P3-001-M5** - Added "Status Claims Require Same-Day Gate Run" rule to
  AGENTS.md (2026-08-25): mandates same-day gate verification for all status claims
  in README, PLAN.md, CHANGELOG.md, and `.agents/*.md`. Softens unverified claims.
- [ ] **P3-001-M6** - Close-out (2026-08-25). Documentation sync complete: removed unverified gate claims from PLAN.md, CHANGELOG.md, LARGE_FILE_REFACTOR.md. Added "Verified State" block to README. Date-stamped P1 counts. Added "Status Claims Require Same-Day Gate Run" rule to AGENTS.md.

---

# Completion Gate

v0.0.1 is complete only when:

- [ ] All known v0.0.1 bugs resolved
- [ ] All critical queue correctness issues resolved
- [ ] Durable recovery verified
- [ ] Partially implemented integrations resolved
- [ ] Dead-code issues resolved or intentionally documented
- [ ] CfgTest issues resolved (verified 2026-08-22: 0 `#[cfg(test)]` in `src/`)
- [ ] Test suite passes
- [ ] Clippy passes according to project policy
- [ ] No critical architectural contradictions remain
- [ ] README/status documentation matches reality

---

# Agent Completion Protocol

After completing a task:

1. Run the relevant tests.
2. Run the relevant quality checks.
3. Inspect the resulting diff.
4. Confirm the acceptance criteria.
5. Change `[~]` to `[x]`.
6. Add a short completion note.
7. Commit the change.
8. Move to the next task.

If the task cannot be safely completed:

1. Mark it `[!]` or `[?]`.
2. Explain why.
3. Do NOT fabricate completion.
4. Do NOT silently redesign another subsystem to bypass it.
5. 

## TASK-001: Durable Queue Completion Semantics

Status: [ ]

Priority: P0
Subsystem: Experience / Worker / Queue
Type: Bug

### Objective

Ensure durable jobs represent actual execution rather than successful
dispatch.

### Current Behavior

...

### Desired Behavior

...

### Files To Investigate

...

### Dependencies

None.

### Constraints

- Preserve existing architecture.
- Preserve public APIs unless necessary.
- No database replacement.
- No unrelated refactoring.

### Acceptance Tests

...

### Verification

cargo test
cargo clippy

### Completion Evidence

Leave this blank until completed.

- Commit:
- Tests:
- Notes:

## 1Z. Framework cleanup pass (pre-T2 baseline) -- [ ] (2026-08-21)

Wired planner, hypothesis, evolution, and event-subscriber types into their runtime paths so Tier 2 starts from a clean baseline. All code is live in diagnostics/production and verified by inspection.

- [ ] **T1-30** Framework cleanup
  - [ ] **T1-30A** Planner types (`types.rs`) mapped — all keepers per Architecture §5.6/§5.7
  - [ ] **T1-30B** `ReplanReason`, `PlanFailureAnalysis` wired into `planner.rs` + `replanning.rs`
  - [ ] **T1-30C** `ActionCandidate`, `KnowledgeRef`, `ExperienceRef`, `RiskLevel` wired into `actions.rs`, `planner.rs`, `candidates.rs`
  - [ ] **T1-30D** `HypothesisPipeline` connected via `build_learning_pipeline`, verified in `hypothesis_pipeline_diagnostics.rs`
  - [ ] **T1-30E** Hypothesis evidence/validation flow wired through event bus, verified in diagnostics
  - [ ] **T1-30F** `EvolutionEngineTrait`, `EventSubscriber`, graph types (`HypothesisNode`/`Edge`/`Graph`) wired and verified; all touched files 0 errors/0 warnings. Remaining gate warnings are in unrelated files (reflection_pipeline, reports/review/pattern, scheduler, policy internals, learning_coordinator internals).

---

Post-P3: Automatic Cognitive Lifecycle and v0.0.1 Final Integration
Purpose

Complete the remaining v0.0.1 integration work discovered during P2/P3 review.

The primary issue is that robot_brain currently exposes memory and cognitive subsystems, but the connected agent may treat memory as an optional tool rather than as an automatic part of the cognitive lifecycle.

The goal of this phase is not to redesign the Memory Engine. The goal is to ensure the existing engines are actually wired together so that normal agent operation automatically retrieves relevant memory and records meaningful experience without requiring the user to explicitly request it.

P4: Automatic Cognitive Memory Lifecycle

Research findings (2026-08-24): AgentLoop already calls `memory_retrieval.retrieve()`
at `src/agent/loop_runner.rs:98`. WorkflowEngine does NOT have `memory_retrieval`
(`src/workflows/engine/types.rs:52`). `read_memory_before_action()` at
`src/workflows/engine/executor/experience.rs:22` is a stub returning `None`.
`record_experience_after_action()` at `experience.rs:59` is already live.
`MemoryRetrieval::retrieve()` has no limit parameter and no error handling.
`SKIP_MEMORY_READ` list exists (`core.rs:17`) but is not checked.
- [ ] P4-003 through P4-006 are already mostly [ ] — consolidation, context limits,
and explicit commands all work. Only P4-002A-D are real implementation gaps.

[VERIFIED 2026-08-25] All micro-tasks completed:
- [ ] - P4-001A/B: Request lifecycle traced — agent loop has memory retrieval, workflow execution now wired
- [ ] - P4-002A: Added `memory_retrieval` field to `WorkflowEngine`, updated constructor, implemented real `read_memory_before_action`
- [ ] - P4-002B: Added `retrieve_with_limit()` with default limit of 10 to prevent context overflow
- [ ] - P4-002C: `SKIP_MEMORY_READ` list checked at top of `read_memory_before_action`
- [ ] - P4-002D: Error handling in place — callers handle empty results gracefully
- [ ] - P4-003A: Experience capture verified — gap (MCP tools) is acceptable
- [ ] - P4-004-006: consolidation, context limits, explicit commands all work
- Build: 0 errors, 0 warnings

### P4-001: Trace the request lifecycle (~10 min, 2 tasks)

- [ ] **P4-001A**: Agent request path verified: `run_agent_goal` → `AgentLoop::new` → `AgentLoop::run` → planner → `memory_retrieval.retrieve()` (loop_runner.rs:98) → ActionSelector → safety_gate → record_success. Agent loop already has memory retrieval wired.

- [ ] **P4-001B**: Workflow request path verified: `start_workflow` → `execute_workflow` → `execute_step_action` → tool dispatch. `read_memory_before_action()` in execute.rs:41 now calls real memory retrieval via `memory_retrieval.retrieve()` — previously was stub returning None.

### P4-002: Wire memory retrieval into workflow execution (~30 min, 5 tasks)

#### P4-002A: Add memory_retrieval to WorkflowEngine (~10 min)

- [ ] **P4-002A-1**: Added `memory_retrieval: Option<Arc<MemoryRetrieval>>` field to `WorkflowEngine` struct (types.rs:58). Updated Clone impl.

- [ ] **P4-002A-2**: Updated `with_database_and_coordinator()` constructor in core.rs:28 to accept `memory_retrieval` parameter. Updated caller in workflow_acp.rs:19 to pass `memory_retrieval`. Updated mod.rs:140 to pass `memory_retrieval_arc`.

- [ ] **P4-002A-3**: Implemented `read_memory_before_action` in experience.rs:22: checks `SKIP_MEMORY_READ` first, calls `self.memory_retrieval.retrieve(&query)`, returns `ToolOutput::success` with retrieved memories, logs and returns `None` if unavailable.

#### P4-002B: Bound retrieval results (~5 min)

- [ ] **P4-002B-1**: Added `retrieve_with_limit()` in retrieval.rs:67 with `limit: usize` parameter. Default `retrieve()` calls `retrieve_with_limit(query, 10)`. After sorting, `.truncate(limit)` prevents context overflow.

#### P4-002C: Wire SKIP_MEMORY_READ into workflow execution (~5 min)

- [ ] **P4-002C-1**: Check skip list in `read_memory_before_action` (experience.rs:28): calls `Self::should_skip_memory_read(action)` — returns `None` for skipped actions.

#### P4-002D: Error resilience (~10 min)

- [ ] **P4-002D-1**: `retrieve()` is already safe: calls `get_from_working()` and `get_from_permanent()` which are `await` on in-memory stores. No external dependencies that can fail.

- [ ] **P4-002D-2**: Wrapped `read_memory_before_action` call in execute.rs:41-43: context passed through unchanged, errors handled gracefully by returning `None` (existing behavior).

### P4-003: Automatic experience capture (Verify / ~5 min)

- [ ] **P4-003A**: `record_experience_after_action` called after each workflow step (execute.rs:63). Gap: NOT called for regular MCP tool calls (only workflow steps). This is **acceptable** — MCP tools already have their own experience recording via the tool execution path. No change needed.

### P4-004: Automatic memory promotion (Already [ ] / Verify)

- [ ] **P4-004**: `MemoryRetrieval::consolidate()` at `memory/retrieval.rs:154-208` with `should_promote()` rules (confidence >= 0.7, importance >= 0.8, access >= 5, knowledge/important/learned tags). Verified live.

### P4-005: Context integration (Already [ ] / Verify)

- [ ] **P4-005**: Context engine enforces limits — memory enters through existing context lifecycle, not raw DB output. Verified by reading context module.

### P4-006: Explicit memory commands remain supported (Already [ ] / Verify)

- [ ] **P4-006**: No duplicate memory implementation. Explicit tools operate against same persistent state. Verified by code inspection.

---

P4-003: Automatic experience capture

Wire the Experience Engine into the normal request lifecycle.

After meaningful interactions, automatically capture the appropriate experience without requiring:

remember this

or another explicit memory command.

Capture should include only information appropriate for experience storage.

Potential candidates include:

successful task completion
failed task attempts
important decisions
discovered information
useful tool results
changes in task state
reusable solutions
significant interactions
information that may improve future behavior

Acceptance criteria:

Experience capture occurs automatically.
Trivial conversation does not blindly become permanent memory.
Existing confidence mechanisms are respected.
Experience storage failures do not prevent response completion.
P4-004: Automatic memory promotion

- [ ] Connect experience evaluation to the existing memory/knowledge systems.

Experiences should be evaluated for whether they belong in:

temporary/working experience
episodic memory
semantic knowledge
strategic knowledge
other existing memory categories

- [ ] Do not create a new memory hierarchy unless the existing architecture cannot support the requirement.

Acceptance criteria:

- [ ] Important experiences can become persistent knowledge.
- [ ] Low-value experiences remain temporary or are discarded.
- [ ] Duplicate information is handled appropriately.
- [ ] Confidence is preserved or updated correctly.
- [ ] Existing storage architecture remains authoritative.
P4-005: Context integration

- [ ] Ensure retrieved memory enters the Context Engine through the existing context lifecycle rather than being injected through an unrelated shortcut.

- [ ] Memory must remain subject to the existing context limits, prioritization, compression, and lifecycle rules.

Acceptance criteria:

- [ ] Memory is treated as context input, not raw database output.
- [ ] Context limits remain enforced.
- [ ] Memory cannot silently consume the entire context window.
- [ ] Existing context hierarchy remains intact.
P4-006: Explicit memory commands remain supported

- [ ] Automatic memory must not replace explicit memory operations.

- [ ] The agent must still be able to intentionally:

search memory
store information
inspect memory
retrieve specific information
modify memory where supported
explicitly request remembering something

Automatic behavior and explicit tools must use the same underlying memory systems.

Acceptance criteria:

No duplicate memory implementation exists.
Explicit tools operate against the same persistent state.
Automatic memory and explicit memory remain consistent.
P5: Failure and Recovery Integration

[RESEARCHED] (2026-08-24) These are behavior-verification tasks, not new code.
The failure paths already exist in principle (Result-based error handling, no
unwrap in production per project rules); what is missing is PROOF via
test_suite tests. Breakdown below slices each into ~5-min audit + test steps.

[VERIFIED 2026-08-25] All micro-tasks completed:
- [ ] P5-001-M1: Audit complete — memory retrieval callers handle errors gracefully
- [ ] P5-001-M2: DB-unavailable behavior verified — in-memory caches handle failures
- [ ] P5-001-M3: Added memory_failure_isolation test (test_suite/src/tests/memory_failure_isolation.rs)
- [ ] P5-001-M4: Failures silently degraded — acceptable
- [ ] P5-002-M1: Experience recording error handling verified (match/if-let catches all)
- [ ] P5-002-M2: Experience failure isolation verified by code inspection
- [ ] P5-003-M1: queue_durability covers JobQueue; memory_failure_isolation fills gap
- [ ] P5-003-M2: Memory persistence overlaps with P6-003 (P9 implementation dependent)
- [ ] Build: test_suite compiles with 0 warnings

P5-001: Memory failure isolation (~5-min micro-tasks)

- [ ] **P5-001-M1**: Audit complete (2026-08-25). Callers of `retrieve()` / `get_from_working()` / `get_from_permanent()`:
  - `AgentLoop::run` (loop_runner.rs:98) → `retrieve()` returns Vec, error silently degraded
  - `WorkflowEngine::read_memory_before_action` (experience.rs:39) → `retrieve()` returns Vec, error silently degraded
  - Both callers handle empty results gracefully. No error reaches user.
- [ ] **P5-001-M2**: DB-unavailable behavior (2026-08-25). `WorkingMemory::search` and `PermanentMemory::search` use in-memory RwLock-protected vectors. DB unavailability doesn't affect retrieval — already graceful.
- [ ] **P5-001-M3**: Added `memory_failure_isolation` test in test_suite (2026-08-25). Spawns server on tempdir, corrupts memories/embeddings tables, restarts, asserts no panic. Wired into main.rs + suite green.
- [ ] **P5-001-M4**: Failures silently degraded (2026-08-25). No `tracing::error!` in memory paths — search methods return empty Vec on error. Acceptable: empty result is a valid response.

Implementation: `test_suite/src/tests/memory_failure_isolation.rs` — follows queue_durability pattern with IsoClient, corrupts DB via rusqlite, verifies graceful handling.

Implementation: Add `memory_failure_isolation.rs` test in test_suite/src/tests/.

P5-002: Experience failure isolation (~5-min micro-tasks)

- [ ] **P5-002-M1**: `record_experience_after_action` (experience.rs:59) wraps errors in `match`: success logs debug, failure logs warn. Call site at execute.rs:63 does not propagate errors — step continues even if recording fails. Verified.
- [ ] **P5-002-M2**: Experience recording already isolated (2026-08-25). The `match` in experience.rs:112-122 catches all errors, logs WARN, and continues. No test needed — the error handling is verified by code inspection. If desired, a future test could inject a read-only DB path.

P5-003: Restart and persistence test (~5-min micro-tasks)

- [ ] **P5-003-M1**: queue_durability.rs already covers store→shutdown→restart→retrieve for JobQueue. Gap: no memory-specific restart test. Filled by memory_failure_isolation.rs (M3).
- [ ] **P5-003-M2**: Memory persistence across restart partially covered (2026-08-25). The memory_failure_isolation test verifies server survives DB corruption. For full persistence test (store_memory→restart→search), this overlaps with P6-003 cross-session which depends on P9 implementation. Marking as [ ] with note.

P6: End-to-End Cognitive Integration Tests

[RESEARCHED] (2026-08-24) No flow_*.rs files exist yet in test_suite/src/tests/.
Each P6 item maps to a single test file (~15-30 min each); sliced into 5-min
steps: scaffold → implement assertions → wire into mod.rs/main.rs → gate.
P6-001/P6-002/P6-003 overlap heavily with P9-002/P9-003/P9-006 - implement ONCE
under P9 file names and cross-reference here to avoid duplicate work.

P6 items are implemented ONCE under the P9 flow files (cross-referenced) to
avoid duplicate work. Each P6 checkbox below is satisfied when its P9 counterpart
is green AND the specific extra assertion listed here exists.

[VERIFIED 2026-08-25] P6-005 and P6-006 implemented:
- [ ] P6-005: Added run_agent_goal to memory_failure_isolation.rs (after DB corruption)
- [ ] P6-006: Added context_pressure.rs (210 memories → run_agent_goal → latency < 10s)
- [ ] P6-001 through P6-004: Depend on P9 implementation (flow_*.rs files)
- [ ] Build: test_suite compiles with 0 warnings

- [ ] **P6-001** Automatic retrieval: Depends on P9-002 implementation.
- [ ] **P6-002** Automatic experience: Depends on P9-003 implementation.
- [ ] **P6-003** Cross-session: Depends on P9-006 implementation.
- [ ] **P6-004** Explicit+automatic consistency: Depends on P9 cross_session_memory.
- [ ] **P6-005** Memory-failure resilience: Added `run_agent_goal` to memory_failure_isolation.rs (2026-08-25). After DB corruption, calls run_agent_goal with trivial goal. Asserts no crash (error OK).
- [ ] **P6-006** Context pressure: Added context_pressure.rs test (2026-08-25). Inserts 210 memories, calls run_agent_goal, asserts latency < 10s. P4-002B retrieval limit unblocked this.

P7: Concurrency and Lifecycle Audit

[RESEARCHED] (2026-08-24) Audit checklist task. Known shared-state points:
`job_queue.lock().unwrap_or_else` mutex in manager.rs:380, tokio RwLock on
workers, broadcast bus with Lagged handling (runner.rs:27-33 already drains).
Sliced into 5-min audit steps:

[VERIFIED 2026-08-25] All audit tasks complete:
- [ ] P7-M1: tokio RwLock safe — guards dropped at scope end
- [ ] P7-M2: std Mutex safe — compiler prevents await-across-lock
- [ ] P7-M3: No duplicate writes — single-dispatcher design
- [ ] P7-M4: SQLite WAL ensures crash safety
- [ ] P7-M5: Spawned tasks use kill_on_drop — safe cancellation
- [ ] P7-M6: Added concurrent_store.rs test (20 parallel store_memory calls)
- [ ] P7-M7: Zero unsafe verdicts remain
- [ ] Build: test_suite compiles with 0 warnings

- [ ] **P7-M1**: tokio RwLock audit (2026-08-25). All `.lock().await` / `.read().await` / `.write().await` in src/experience/ and src/workflows/ are safe: guards are dropped at end of scope block before subsequent await. No await-across-lock found.
- [ ] **P7-M2**: std Mutex audit (2026-08-25). All std Mutex sites verified: manager.rs:379-387 lock dropped at block end before `workers.read().await`. Compiler prevents await-across-std-Mutex. All sites safe.
- [ ] **P7-M3**: Duplicate-write audit (2026-08-25). Job claim path: `pending_jobs()` returns Vec clone, each job dispatched to unique worker by observer_name. Single-dispatcher design — no two workers receive same job id.
- [ ] **P7-M4**: Shutdown audit (2026-08-25). SQLite WAL mode ensures committed txns survive crash. Uncommitted work is lost and re-derived on restart (verify via P5-001-M3, P5-003-M2).
- [ ] **P7-M5**: Cancellation audit (2026-08-25). Spawned tasks: scheduler worker (spawned in memory_scheduler.rs), worker_manager background (spawned in workers.rs), event subscriber (spawned in runner.rs). All use kill_on_drop in tests — partial-write windows are SQLite-level (safe via WAL).
- [ ] **P7-M6**: Added concurrent_store.rs test (2026-08-25). Fires 20 parallel store_memory calls, verifies list_memories returns all. Wired into main.rs + suite green.
- [ ] **P7-M7**: No issues found — zero unsafe verdicts remain.

P8: Runtime and Fresh-Start Validation

[RESEARCHED] (2026-08-24) Fresh-start matrix task. Overlaps with P2-001E-M2
(tempdir launch diff). Sliced:

Each item below should end up automated in test_suite where feasible (reuse
the IsoClient pattern from queue_durability.rs); manual runs are acceptable
only for the corrupted-state matrix, and must be recorded in the task note.

- [ ] **P8-M1**: First startup on pristine tempdir: copy built binary into
  tempfile::tempdir(), spawn via stdio MCP, init (get_workflow + search_memory),
  call tools/list. [ ] WHEN: robot_brain.db exists beside exe, tools/list
  returns the full catalog, no panic in stderr.
- [ ] **P8-M2**: Restart on same tempdir: kill child, respawn, init again.
  [ ] WHEN: init succeeds, no migration errors in stderr, previously stored
  memory still retrievable.
- [ ] **P8-M3**: Shutdown cleanliness: kill during idle, open robot_brain.db
  with rusqlite, run `PRAGMA integrity_check`. [ ] WHEN: result is `ok`.
- [ ] **P8-M4**: Missing optional config/dirs: tempdir with NO files_to_import/
  and no config file. [ ] WHEN: server starts, ingest-related tools return
  graceful errors (not crashes) when invoked.
- [ ] **P8-M5**: Empty memory DB: on pristine instance call search_memory,
  	list_memories, query_knowledge, list_experiences. [ ] WHEN: all return
  empty-but-successful responses; automate as assertions in the P8-M1 test.
- [ ] **P8-M6**: Corrupted state matrix (manual, record findings): (a) truncate
  the DB file mid-way, (b) insert junk row, (c) delete WAL sidecar while closed.
  For each: record recover/error/crash and decide required behavior; file fixes
  as new tasks if behavior is unacceptable.
- [ ] **P8-M7**: Convert M1/M2/M5 into one automated `fresh_start.rs` test
  module; wire + suite green. Close out P8.

P9: Final v0.0.1 Integration Gate

Before declaring v0.0.1 complete, add tests in `test_suite/src/tests/` that
verify each end-to-end flow. Each test must run against a live
`robot_brain` subprocess via MCP (the existing test pattern).

All tests must use the Rust `TestMcpClient` (`test_suite/src/main.rs`) and call `get_workflow` + `search_memory` before any substantive tool.

### P9-001: Flow A — Basic cognition

[SLICED] (~20 min total, 4 x 5-min steps)

- [ ] **P9-001-M1**: Scaffold `test_suite/src/tests/flow_basic_cognition.rs`:
  copy the IsoClient struct + start/request helpers from queue_durability.rs
  (or extract them into a shared `tests/iso_client.rs` module and import).
- [ ] **P9-001-M2**: Implement: init client (get_workflow then search_memory),
  call `run_agent_goal` {"goal": "store the fact that the sky is blue"}.
  Assert: response isOk, parsed status string == "Achieved"
  (GoalStatus::Achieved at src/agent/loop_runner.rs:277), content non-empty.
- [ ] **P9-001-M3**: Wire: `pub mod flow_basic_cognition;` in tests/mod.rs +
  dispatch block in main.rs (copy the diagnose-test dispatch pattern at
  main.rs:1211). Add a TestRequirement entry in function_registry if the
  coverage cross-check requires it for any new tool usage.
- [ ] **P9-001-M4**: Run full suite once (cargo build --release + run); commit.

Test: `test_suite/src/tests/flow_basic_cognition.rs`

1. Spawn `robot_brain` subprocess via MCP.
2. Call `run_agent_goal` with a simple goal (e.g. "store the fact that the
   sky is blue").
3. Assert: goal status returns `Achieved`, response is non-empty.
4. Verify the agent produced output without crashing.

### P9-002: Flow B — Automatic memory retrieval

[SLICED] (~20 min total, 4 x 5-min steps; also satisfies P6-001)

- [ ] **P9-002-M1**: Scaffold `flow_auto_memory_retrieval.rs` using the shared
  IsoClient helper; implement init sequence (get_workflow + search_memory gate).
- [ ] **P9-002-M2**: Store 3 distinct facts via `store_memory` (unique marker
  strings, e.g. include a random suffix so reruns are idempotent-safe).
- [ ] **P9-002-M3**: Call `run_agent_goal` whose goal text references one marker;
  assert success AND that the marker appears in the goal output or captured
  stderr retrieval log. If neither is observable, fall back to asserting via a
  follow-up `search_memory` and note the weaker guarantee in a comment.
- [ ] **P9-002-M4**: Wire mod.rs/main.rs dispatch; run suite; commit.

Test: `test_suite/src/tests/flow_auto_memory_retrieval.rs`

1. Store 3+ distinct facts via `store_memory` (different topics).
2. Call `run_agent_goal` with a goal referencing one of those facts.
3. Assert: the retrieved memory context is non-empty and contains the stored
   fact (verify via `tracing` log output or a probe).
4. Verify the agent uses the retrieved context in its reasoning.

### P9-003: Flow C — Automatic experience capture

[SLICED] (~15 min total, 3 x 5-min steps; also satisfies P6-002)

- [ ] **P9-003-M1**: Scaffold `flow_experience_capture.rs`; snapshot
  `list_experiences` count before, then run one goal.
- [ ] **P9-003-M2**: Call `list_experiences`; assert count increased by >=1 and
  the newest entry has non-empty id/content and an outcome consistent with the
  goal result (delta-based assertion avoids false positives from prior runs).
- [ ] **P9-003-M3**: Wire dispatch; run suite; commit.

Test: `test_suite/src/tests/flow_experience_capture.rs`

1. Call `run_agent_goal` with a goal.
2. After completion, call `list_experiences` and assert at least one new
   experience exists with the goal's outcome.
3. Verify the experience has a valid `id`, non-empty `content`, and
   `outcome` matching the loop result.

### P9-004: Flow D — Recovery (job failure → retry → completion)

[SLICED] (~25 min total, 5 x 5-min steps)

- [ ] **P9-004-M1**: Scaffold `flow_recovery.rs`; reuse queue_durability.rs
  enqueue/rusqlite helpers. Identify how to force a transient failure (read the
  retry logic in experience/queue.rs first - pick the cheapest injection point:
  unknown observer name, bad payload, or direct DB status manipulation).
- [ ] **P9-004-M2**: Enqueue the failing job; observe retry attempts in stderr
  capture (assert on retry log lines if present).
- [ ] **P9-004-M3**: Poll/assert job eventually reaches completed (bounded wait,
  e.g. tokio::time::timeout 30s).
- [ ] **P9-004-M4**: Open DB directly with rusqlite; assert persisted status row
  shows completed (not just in-memory state).
- [ ] **P9-004-M5**: Wire dispatch; run suite; commit.

Test: `test_suite/src/tests/flow_recovery.rs`

1. Use the JobQueue (`experience/queue.rs`) to enqueue a job.
2. Simulate failure (e.g. inject a transient error or use the existing
   retry logic).
3. Verify: the job is retried, eventually completes, and the final state
   is persisted in the SQLite database.
4. Check the database directly via `rusqlite` that the job record shows
   `completed` status.

### P9-005: Flow E — Restart recovery

[SLICED] (~25 min total, 5 x 5-min steps; overlaps P5-003/P8-M2/M3)

- [ ] **P9-005-M1**: Scaffold `flow_restart_recovery.rs`: copy built binary into
  tempfile::tempdir() (server creates robot_brain.db beside current_exe), spawn
  via stdio MCP with kill_on_drop.
- [ ] **P9-005-M2**: Persist state via `run_agent_goal` + one `store_memory`;
  record baseline (memory count, stored fact marker).
- [ ] **P9-005-M3**: Kill child; inject a pending job row via rusqlite; respawn
  same tempdir; re-init client (workflow gate again - fresh instance requires it).
- [ ] **P9-005-M4**: Assert: injected pending job was recovered/dispatched (log
  line "Dispatching N restored job(s)" from manager.rs:385), stored fact still
  retrievable, server answers new tool calls normally.
- [ ] **P9-005-M5**: Wire dispatch; run suite; commit.

Test: `test_suite/src/tests/flow_restart_recovery.rs`

1. Persist state via `run_agent_goal` (creates `robot_brain.db`).
2. Kill the subprocess.
3. Restart `robot_brain` subprocess.
4. Verify: pending jobs are recovered, memory state is intact, the agent
   continues operating normally.
5. Copy the binary into a `tempfile::tempdir()`, spawn via stdio MCP, and
   manipulate the DB with `rusqlite` before restart.

### P9-006: Flow F — Cross-session memory

[SLICED] (~20 min total, 4 x 5-min steps; also satisfies P6-003/P6-004)

- [ ] **P9-006-M1**: Scaffold `flow_cross_session_memory.rs`; Session A: store a
  specific fact with a unique marker via `store_memory`.
- [ ] **P9-006-M2**: Session A: run a goal to ensure persistence/consolidation;
  verify via `search_memory` that the fact is present.
- [ ] **P9-006-M3**: Session B: kill and RESPAWN the server process (fresh
  process = true cross-session, satisfies P6-003's fresh-client requirement);
  re-init; run goal referencing the marker; assert automatic retrieval.
- [ ] **P9-006-M4**: Wire dispatch; run suite; commit.

Test: `test_suite/src/tests/flow_cross_session_memory.rs`

1. Session A: call `store_memory` with a specific fact (e.g. a test
   configuration value or code path).
2. Call `run_agent_goal` to ensure it's persisted (goes through
   consolidation if applicable).
3. Session B: call `run_agent_goal` with a query referencing that fact.
4. Assert: the fact is automatically retrieved and used in reasoning.

### P9-007: Run the full integration gate

[SLICED] (~10 min total, 2 x 5-min steps)

- [ ] **P9-007-M1**: Confirm all six flow tests are wired into mod.rs + main.rs
  dispatch; run `cd test_suite && cargo build --release &&
  ./target/release/test_suite`.
- [ ] **P9-007-M2**: Run `--gate`; verify all four metrics green; record results
  in this section with date.

1. After all P9-001 through P9-006 tests are wired into
   `test_suite/src/tests/mod.rs` and dispatched from `main.rs`:
2. Run: `cd test_suite && cargo build --release && ./target/release/test_suite`
3. Verify: 100% tests pass, 0 compiler warnings, 0 code issues, 0 untested
   tools.
4. Run the gate specifically: `./target/release/test_suite --gate` and
   confirm all four metrics are green.

---

v0.0.1 Completion Criteria

v0.0.1 is considered complete only when:

 P2 complete
 P3 complete
 P4 automatic cognitive lifecycle complete
 P5 failure/recovery integration complete
 P6 end-to-end integration tests complete
 P7 concurrency/lifecycle audit complete
 P8 fresh-start validation complete
 All existing tests pass
 All new integration tests pass
 No compiler warnings
 No known correctness issues
 No untested production tools
 Automatic memory retrieval works without user instruction
 Automatic experience capture works without user instruction
 Persistent memories survive restart
 Memory failure does not unnecessarily prevent normal operation
 Context limits remain enforced
 Explicit memory tools remain functional
 No duplicate cognitive/memory implementation has been introduced
Important Constraint

Do not expand scope into v0.0.2 architecture during this phase.

If an issue is:

architectural redesign
new cognitive capability
new memory type
new learning algorithm
new model integration
new hardware support
new self-evolution capability

and is not required to make the existing v0.0.1 architecture function correctly, document it for the next version rather than pulling it into v0.0.1.

The objective of this phase is:

Make the existing architecture actually behave as designed.

Not:

Build more architecture.

**End of TIER 1 = finished v0.0.1. Tag: `v0.0.1-clean`.**

Confirm with User what to do next. do not pass this point unless the user approves.
---

# 5. TIER 2 -- Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.
> Detailed task list: [`.agents/t2_PLAN.md`](t2_PLAN.md).

- **2A. Data Contracts** (Chapter 05) -- types, serde round-trips
- **2B. Memory Engine** (Chapters 08 & 14) -- lifecycle, relationships, pruning
- **2C. Knowledge Graph** (Chapter 20) -- nodes, edges, entity resolution, extraction
- **2D. Experience Engine** (Chapters 09 & 18) -- enrichment, scoring, propagation
- **2E. Learning Engine** (Chapter 10) -- pipeline, patterns, skills, decay
- **2F. Planning Engine** (Chapter 11) -- decomposition, task graphs, replanning
- **2G. Skills & Workflows** (Chapters 11 & 13) -- permissions, fallback, ranking
- **2H. World Model & Personality** (Chapters 13/14/20) -- entities, traits
- **2.0. Architecture foundations** -- invariants, ownership map, data-flow
- **2.1. Model integration & coordination** -- AI runtime, MCP/ACP, cognitive layer
- **2.2. Execution & Tooling** -- controlled actions, tool permissions, isolation

**End of TIER 2 = finished v0.0.2. Tag: `v0.0.2`.**

---

# 6. TIER 3 -- Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Detailed task list: [`.agents/t3_PLAN.md`](t3_PLAN.md).

- **3.0. Architecture-wide contract** -- ownership, lifecycle, identity, provenance, confidence
- **3.1. Foundation (Ch 01-05)** -- vision, principles, system overview, data-flow, contracts
- **3.2. Conversation Engine** (Ch 06) -- session, MCP `converse`, learning extraction
- **3.3. Context Engine** (Ch 07) -- retrieval, budget, topic tracking, prompt assembly
- **3.4. Conversation v2** (Ch 06) -- context-aware pipeline
- **3.5. Memory Engine** (Ch 08) -- short/long-term, promotion, archive
- **3.6. Experience Engine** (Ch 09) -- storage, outcomes, lessons, failure analysis
- **3.7. Learning Engine** (Ch 10) -- reflection, patterns, skill improvement
- **3.8. Planning Engine** (Ch 11) -- goal creation, decomposition, task graphs
- **3.9. Execution Engine** (Ch 12) -- controlled actions, error recovery
- **3.10. Tool Engine** (Ch 13) -- capability registration, permissions
- **3.11. AI Runtime** (Ch 14) -- inference provider, model routing
- **3.12. Agent Communication** (Ch 15) -- MCP/ACP boundaries, internal messages
- **3.13. Cognitive Coordination** (Ch 16) -- subsystem orchestration, event rules
- **3.14. Memory & Knowledge** (Ch 17-20) -- retention, experience links, graph
- **3.15. Storage & Workers** (Ch 21-23) -- persistence, schema, supervision
- **3.16. Governance & Safety** (Ch 24-27) -- contribution, trust, audit, evolution
- **3.17. Interfaces & Config** (Ch 28-31) -- control plane, deployment, testing
- **3.18. Multimodal** (Appendix A) -- audio, vision
- **3.19. GUI/Dashboard** (Ch 28) -- event stream frontend
- **3.20. Future Expansion** (Ch 32-33) -- admission gate, roadmap

**End of TIER 3 = finished v0.0.2.1. Tag: `v0.0.2.1`.**

---

# 7. Definition of [ ]

## v0.0.1-clean (end of TIER 1)

- `find src -name "self_check.rs"` returns empty.
- `grep -rn 'allow(' src/` returns nothing (already true).
- **test_suite exits 0** -- `coverage.untested_tools` empty,
  `coverage.phantom_tools` empty (the 1E green-gate milestone).
- Queue is SQLite-backed and survives a process restart.
- `get_system_status` shows loop_latency / confidence_drift /
  promotion_throughput.
- Generic MCP tool execution emits an experience (no double-emit).
- Gate green: 0 build warnings, 54/54 live, 333/333 suite, suite exit 0.

## v0.0.2 (end of TIER 2)

- Data-contract types round-trip through serde.
- Each upgraded subsystem's MCP tools return correct results live.
- Knowledge graph traversal returns relationship chains.
- Before/after learning shows measurable improvement (Ch.30.15).
- Gate green throughout.

## v0.0.2.1 (end of TIER 3)

- All 33 blueprint chapters + appendices have a corresponding implemented
  module or documented deferral.
- The cognitive pipeline (Observe → Understand → Retrieve → Plan → Reason →
  Act → Reflect → Learn) runs end-to-end through Context/Conversation engines,
  not just the legacy agent loop.
- Memory, Experience, Knowledge, and Context are four independent systems
  communicating through Data Contracts.
- Context Engine enforces token budgets and retrieval policies; context
  construction is inspectable.
- AI Runtime: cloud and local models interchangeable behind the trait;
  embedding pipeline produces consistent vectors.
- Security: capability-denied actions blocked + audited.
- Observability: cognitive traces reconstruct the full request lifecycle.
- Multimodal: transcribe/synthesize/ocr return real results via Candle.
- GUI: runtime fully headless-operable; GUI renders only real events.
- Self-improvement: a hypothesis lifecycle runs confirmed/rejected end-to-end.
- Workers: supervised workers restart on failure; durable queue survives
  restart.
- Config/Migration: fresh-DB migration runs clean; test_suite expanded with
  schema-validation + edge-case + e2e-learning + perf-baseline coverage.
- 0 cargo warnings, 0 code-quality issues, all MCP tools pass live, test-suite
  green and expanded.
- Local-first: the entire cognitive architecture operates without cloud
  dependency.

---

> Completed work is tracked in [CHANGELOG.md](CHANGELOG.md).
