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
- **145/145 tests pass**, 0 untested tools, 0 phantom tools. Coverage gate: GREEN.
- **144 compiler warnings** (too-many-arguments, async-fn simplification, unused vars),
  **12 code issues** (emoji, dead code) — gate RED on these only.
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
  codebase state and running the gate -- never rely on a "done" message, a
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
0 untested tools. If red, the increment is NOT done. Fix it before claiming
done. Never commit a red gate.

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
> Tick `[x]` when an increment is committed with a green gate.
>
> **Work order:** 1E (coverage gate) FIRST -- it's the actual gate problem and
> the user's priority. Then 1B (queue), 1C (metrics), 1D (MCP→experience).
>
> **NOTE on self_check removal (moved to TIER 2):** the 8 `self_check.rs` files
> exercise APIs that have NO other callers (informed plans, replanning, action
> selection, policy engine, etc.). This is a binary crate, so removing a
> self_check surfaces dead-code warnings on those pub APIs (24 warnings for
> planner alone). Per the Dead Code Resolution Protocol, these APIs ARE
> described in v0.0.2.1 Chapter 11 (Planning) / Chapter 19 (Confidence), so
> they're incomplete stubs that must be WIRED into real MCP tools, not deleted.
> That wiring is TIER 2 work (T2-32..T2-36 for planner, similar for others).
> So: self_check removal happens DURING each system's TIER 2 upgrade, not as
> standalone TIER 1 cleanup. TIER 1 focuses on the gate + queue + metrics.

## 1A. (Moved to TIER 2) Remove self_check.rs files

> Moved: each self_check is removed as part of its system's TIER 2 upgrade,
> after the APIs it exercises are wired into real MCP tools. See T2-32..T2-42.
> Attempting standalone removal in TIER 1 creates dead-code warnings (binary
> crate flags unreached pub APIs), violating the 0-warnings gate.

## 1B. SQLite-backed JobQueue (V2-11) -- DONE

- [DONE] **T1-09** `job_queue` table + migration (SQLite)
- [DONE] **T1-10** Wire enqueue/dequeue to SQLite
- [DONE] **T1-10B** Migrate `#[cfg(test)]` blocks to test_suite
  - [DONE] T1-10B-01..T1-10B-11 (Group A: MCP-reachable, moved to test_suite)
  - [DONE] T1-10B-12..T1-10B-20, T1-10B-P, T1-10B-Z (Group B: internal-only, left as Rust unit tests)
- [DONE] **T1-11** Handle broadcast `Lagged` events
- [DONE] **T1-12** Startup verification in `initialization.rs`

## 1C. Loop-health metrics (V2-12) -- DONE

- [DONE] **T1-13** Add `loop_latency` metric
- [DONE] **T1-14** Add `confidence_drift` metric
- [DONE] **T1-15** Add promotion-throughput metric
- [DONE] **T1-16** Expose metrics via `get_system_status`

## 1D. Close the generic MCP→experience path (V2-05) -- DONE

- [DONE] **T1-17** Hook `emit_tool_experience` into post-tool-execution dispatch
- [DONE] **T1-18** Idempotency -- no double-emit

## 1E. Close the coverage gate (make test_suite exit 0) -- DONE

### 1E.1 -- Fix the phantom embedding tools

- [DONE] **T1-19** Fix 6 phantom embedding tools (commit b9b43ff)

### 1E.2 -- Add FunctionRegistry tests for untested tool groups

- [DONE] **T1-20** ACP tools (9) (commit 6b7d036)
- [DONE] **T1-21** System/session tools (4)
- [DONE] **T1-22** Memory/search extras (3)
- [DONE] **T1-23** Knowledge lifecycle (6)
- [DONE] **T1-24** Evidence/observation (3)
- [DONE] **T1-25** Reflection extras (3)
- [DONE] **T1-26** Skills extras (5)
- [DONE] **T1-27** Personality (6)
- [DONE] **T1-28** World model (10)
- [DONE] **T1-29** Agent/workflow extras (2)

**T1-21..T1-29 done together (commit 7775ca1).** 40 entries in `function_registry/coverage_tools.rs`.

**Green-gate milestone:** 141/141 tests pass, exit 0. untested=0, phantom=0.

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
11. When a task completes: move its detail (verification notes, file paths, commits, decisions) to `.agents/CHANGELOG.md`, leave only the task number and `[DONE]` marker in PLAN.md. Never let PLAN.md accumulate completed detail.
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

## P0-001 Durable Queue Completion Semantics

### Problem

The durable queue currently marks work complete after successfully
broadcasting an event rather than after the worker successfully processes
the work.

`broadcast_event()` using `try_send()` can also drop work when a worker
channel is full.

This can produce:

    event created
        ↓
    durable job created
        ↓
    worker channel full
        ↓
    event dropped
        ↓
    broadcast reports success
        ↓
    durable job marked complete

RoBoT therefore believes work was completed when it was not.

### Required Outcome

A durable job must remain pending/running until the worker confirms
successful processing.

A dropped or failed dispatch must NOT produce a completed job.

### Likely Files

- `src/experience/...`
- `src/workers/...`
- `src/queue/...`
- `src/app/...`

Do not assume these paths are exhaustive. Search the repository first.

### Acceptance Criteria

- [x] Worker dispatch failure is detectable.
- [x] Full worker channels do not silently lose jobs.
- [x] SQLite job status is not marked complete during dispatch.
- [x] Successful worker execution marks the durable job complete.
- [x] Worker failure records failure state.
- [x] Retry behavior is represented consistently.
- [x] Tests cover channel-full behavior.
- [x] Tests cover worker failure.
- [x] Tests cover successful completion.
- [x] `cargo test` passes.
- [x] `cargo clippy` passes according to project policy.

### Do Not

- Replace SQLite with another database.
- Remove the durable queue.
- Remove workers to simplify the problem.
- Hide failures with `unwrap`, `expect`, or ignored errors.
- Change unrelated architecture.

---

# P0-002 Unique Durable Job Identity

### Problem

Multiple observer jobs derived from the same experience/event can
currently use the same durable identifier.

This creates the possibility of one observer job replacing another.

### Required Outcome

Every independently executable durable job must have a unique job ID.

The relationship between:

- experience/event
- observer
- durable job
- retry attempt

must remain explicit.

### Acceptance Criteria

- [x] Each observer job receives a unique durable job ID.
- [x] Event/experience ID remains available as a parent/reference ID.
- [x] Multiple observers cannot overwrite each other's jobs.
- [x] Retry attempts do not corrupt the original job.
- [x] Database constraints enforce intended uniqueness.
- [x] Tests cover multiple observers for one event.

**Fix:** Added `ObserverJob::with_id(event, job_id)` constructor in `src/experience/worker.rs`. Updated `enqueue()` and `broadcast_event()` in `src/experience/worker_manager/manager.rs` to pass the pre-generated unique job ID to `ObserverJob::with_id()`, ensuring the `ObserverJob.job_id` matches the ID registered in the `JobQueue` and `JobRegistry`. This fixes the mismatch that previously caused worker completion callbacks to fail looking up the job in the registry.

---

# P0-003 Durable Queue / Worker State Synchronization -- DONE

### Problem

The worker maintains retry/execution state separately from the SQLite
durable queue. The two systems can therefore disagree about whether work
is pending, running, failed, or complete.

### Fix Applied

- Added `OnRetryCallback` to `ExperienceWorker` so retry job IDs are
  registered in the `JobRegistry` when `handle_failure` creates a retry
  (worker callbacks can find them later).
- Registered restored job IDs in `JobRegistry` from `dispatch_restored_jobs`
  so synthetic events match registry lookups.
- Fixed `let _ = receiver.recv()` in `background.rs` → proper match
  (no underscore-prefixed variables).

Files changed:
- `src/experience/worker.rs` — OnRetryCallback type, on_retry field, callback invocation
- `src/experience/worker_manager/manager.rs` — on_retry closure, dispatch_restored_jobs registry registration
- `src/experience/worker_manager/background.rs` — fixed ignored recv result

### Required Outcome

There must be one authoritative lifecycle for durable work.

In-memory worker state may exist for execution purposes, but it must not
contradict durable state.

### Acceptance Criteria

- [x] Job enters durable pending state.
- [x] Dispatch changes state appropriately.
- [x] Worker execution changes state appropriately.
- [x] Failure is persisted.
- [x] Retry is persisted (retry IDs registered in JobRegistry).
- [x] Success is persisted.
- [x] Restart does not lose state (restored job IDs registered).
- [x] Tests cover each lifecycle transition (145/145 tests pass).

---

# P1 - Restart / Recovery

## P1-001 Restore Pending Jobs

### Problem

Jobs can survive in SQLite across process termination, but restored jobs
must actually become executable work again.

### Acceptance Criteria

- [DONE] Pending jobs survive restart — `restore_from_database()` reloads pending/running jobs from SQLite into in-memory cache at startup (`src/experience/queue.rs`).
- [DONE] Running jobs have defined restart semantics — demoted to `Pending` during restore so workers re-process them (`src/experience/queue.rs` line 284-294).
- [DONE] Restored executable jobs are re-enqueued — `dispatch_restored_jobs()` sends synthetic events to each worker's channel after restore (`src/experience/worker_manager/manager.rs`).
- [DONE] No job is executed twice — only `pending`/`running` jobs are restored; `completed`/`failed` are excluded by the SQL WHERE clause.
- [DONE] Completed jobs are not re-run — SQL filter `WHERE status IN ('pending', 'running')` excludes completed.
- [DONE] Recovery is tested — `queue_durability.rs` (T1-10) boots an isolated server, injects a pending job row directly into SQLite, kills the server, boots a fresh server, and verifies the job is restored and visible via MCP (`get_system_status`). Part of the 145/145 passing tests.

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

## P1-001 Dead Code

Current known count:

40

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

## P1-002 CfgTest Issues

Current known count:

38

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

- [x] **P2-001A** - Startup no longer performs unnecessary subsystem test operations.
  [DONE] (2026-08-24) Removed the scheduler probe block (create/load/cancel/enable/delete
  task on production DB) from `build_memory_scheduler` in
  `src/bridge/app/initialization/memory_scheduler.rs`. The production consolidation
  task setup (`setup_memory_consolidation_task`) is preserved as a non-probe startup
  call. The diagnostics module already has an isolated `run_scheduler_probe()` in
  `scheduler_diagnostics.rs` that exercises the same code paths against a temp DB.
  Gate: 148/148 tests, 0 warnings, 0 code issues, 0 untested tools.
- [x] **P2-001B** - Existing test coverage is preserved.
  [DONE] (2026-08-24) Two changes:
  1. Added `run_diagnose_test()` in test_suite/src/tests/cli_tools.rs that spawns
     `robot_brain diagnose` as a subprocess, asserts exit 0, and checks for
     expected diagnostic log markers ("Starting explicit subsystem diagnostics",
     "Subsystem diagnostics complete"). Wired into main.rs after CLI tool tests.
  2. Rewrote `verify_experience_recorder()` in
     experience_recorder_diagnostics.rs to use an isolated temp database (same
     pattern as job_queue/scheduler diagnostics) instead of writing probe
     experiences to the production database via the real ExperienceRecorder.
     Updated diagnostics.rs caller to match the new zero-argument signature.
  Gate: 148/148 coverage tests, 0 warnings, 0 code issues.
- [ ] **P2-001C** - Diagnostics remain available through an explicit mechanism.
  [RESEARCHED] (2026-08-24) The explicit mechanism already exists: `robot diagnose`
  CLI (src/main.rs L39-44) calls `run_startup_diagnostics()` in
  `src/bridge/app/initialization/diagnostics.rs`, which dispatches 18 diagnostic
  functions across all subsystems. `App::run()` contains no probes (verified:
  only scheduler spawn + stdio server). `test_suite/src/tests/cli_tools.rs::
  run_diagnose_test()` covers the CLI path (exit 0 + start/complete markers).
  What remains to call P2-001C DONE: harden the diagnose path itself so it is a
  reliable, verifiable explicit mechanism.

  Micro-tasks (~5 min each, do ONE per session step, gate + commit after each):

  - [x] **P2-001C-M1** - Audit dispatch completeness: DONE (2026-08-24). All 18 diagnostic functions
    are called exactly once from `run_startup_diagnostics()`. Verified by grepping all *_diagnostics.rs
    files and cross-referencing with diagnostics.rs callers.
  - [x] **P2-001C-M2** - Verify no startup pollution remains: DONE (2026-08-24). `App::new` in
    `initialization/mod.rs` contains no probe/self-check invocations. All build_* and setup_*
    functions are clean (verified via grep).
  - [x] **P2-001C-M3** - Diagnose exit status: DONE (already implemented). `robot diagnose`
    returns exit code 1 when any diagnostic fails (src/main.rs L45-48). No changes needed.
  - [x] **P2-001C-M4** - Diagnose summary output: DONE (2026-08-24). Converted 14 void diagnostic
    functions to return `Result<(), String>`. Updated `run_startup_diagnostics()` to track per-subsystem
    results and log a summary with `[PASS]`/`[FAIL]` markers per subsystem. Extended expected markers
    in `run_diagnose_test` already covers the subsystems summary line.
  - [ ] **P2-001C-M5** - Isolation check: confirm `experience_recorder_diagnostics::
    verify_experience_recorder()` uses a temp DB (see P2-001B open question);
    if it writes to production DB, switch it to an isolated temp DB like the
    other probes.
  - [ ] **P2-001C-M6** - Documentation: add a short "Diagnostics" section to README
    describing `robot diagnose` as THE explicit diagnostics mechanism (what it
    checks, expected output, exit codes).
  - [ ] **P2-001C-M7** - Gate + close-out: run full gate, confirm diagnose test
    passes, mark P2-001C `[x]` with completion note.
- [ ] **P2-001D** - Startup remains deterministic.
  [RESEARCHED] (2026-08-24) Production startup (`App::new` in
  `src/bridge/app/initialization/mod.rs`) contains no probe/self-test logic and
  no random data generation. Deterministic steps: DB init + migrations,
  core infra build, engines build, observer registration, restored-job dispatch,
  idempotent consolidation-task registration, policy `load_defaults`, MCP tool
  registration. Known non-determinism sources found: (1) consolidation task
  `next_run = now + 3600s` on first creation (time-dependent, benign); (2)
  `dispatch_restored_jobs()` replays whatever pending jobs the previous run left
  in the DB - behavior depends on prior-run state, not launch inputs; (3) log
  lines embed timestamps/UUIDs. None of these are probes; the question is which
  are acceptable and which need documenting or fixing.

  Micro-tasks (~5 min each, ONE per session step, gate + commit after each):

  - [ ] **P2-001D-M1** - Audit each `App::new` step for randomness/time-dependence:
    grep the init modules for `Uuid::new_v4`, `Utc::now`, `rand`, `SystemTime`.
    Record findings in this task's note; classify each as acceptable-benign or
    fix-needed.
  - [ ] **P2-001D-M2** - Decide policy for `dispatch_restored_jobs()`: is replaying
    leftover jobs at startup "deterministic"? Document the decision here; if
    deemed non-deterministic pollution, move dispatch behind an explicit command
    or gate it.
  - [ ] **P2-001D-M3** - Verify two consecutive cold starts produce identical DB
    schema/state except for expected volatile rows (consolidation task next_run).
    Script it as a test_suite restart test if feasible.
  - [ ] **P2-001D-M4** - Confirm startup log sequence is stable across runs
    (same info lines in same order). Fix any order-dependent initialization.
  - [ ] **P2-001D-M5** - Gate + close-out: full gate green, mark P2-001D `[x]`
    with completion note.
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

  - [ ] **P2-001E-M1** - Enumerate every write path reachable from a plain
    `robot_brain server` launch (migrations, scheduler setup, job replay, event
    subscriber side effects). List them in this task's note.
  - [ ] **P2-001E-M2** - Launch the server against a fresh temp-dir copy, snapshot
    the DB before/after (sqlite3 dump diff), and confirm only expected rows
    (schema + consolidation task) change. Record result.
  - [ ] **P2-001E-M3** - Resolve the `verify_experience_recorder()` isolation
    question (shared with P2-001C-M5): confirm it writes to its own temp DB, not
    production. Fix if not.
  - [ ] **P2-001E-M4** - Verify `dispatch_restored_jobs()` cannot re-execute
    diagnostic/probe jobs left over from a crashed diagnose run (probe jobs use
    synthetic IDs in separate temp DBs - confirm no cross-contamination path).
  - [ ] **P2-001E-M5** - Add a test_suite durability-style test: start server on a
    pristine tempdir DB, stop, assert no experience/task/job rows beyond the
    expected consolidation task exist.
  - [ ] **P2-001E-M6** - Gate + close-out: full gate green, mark P2-001E `[x]`
    with completion note.

---

# P3 - Documentation / Verification

## P3-001 Synchronize Project Status

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

Micro-tasks (~5 min each, ONE per session step, gate + commit after each):

- [ ] **P3-001-M1** - Grep README + all `.agents/*.md` for unverifiable claims
  ("zero warnings", "all tests pass", "fully operational", "complete"). List each
  hit with file:line in this task's note.
- [ ] **P3-001-M2** - Fix each flagged claim: either cite the gate report that
  proves it (with date) or soften to current verified state.
- [ ] **P3-001-M3** - Add a dated "Verified state" block to README pointing at
  `test_suite/test_suite_report.json` as the single source of truth.
- [ ] **P3-001-M4** - Date-stamp the P1-001/P1-002 known-counts in this file and
  note they must be refreshed from the gate report, not hand-edited.
- [ ] **P3-001-M5** - Add the rule "status claims require a same-day gate run"
  to AGENTS.md or README if not already present.
- [ ] **P3-001-M6** - Gate + close-out: full gate green, mark P3-001 `[x]`.

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

## 1Z. Framework cleanup pass (pre-T2 baseline) -- DONE (2026-08-21)

Wired planner, hypothesis, evolution, and event-subscriber types into their runtime paths so Tier 2 starts from a clean baseline. All code is live in diagnostics/production and verified by inspection.

- [x] **T1-30** Framework cleanup
  - [x] **T1-30A** Planner types (`types.rs`) mapped — all keepers per Architecture §5.6/§5.7
  - [x] **T1-30B** `ReplanReason`, `PlanFailureAnalysis` wired into `planner.rs` + `replanning.rs`
  - [x] **T1-30C** `ActionCandidate`, `KnowledgeRef`, `ExperienceRef`, `RiskLevel` wired into `actions.rs`, `planner.rs`, `candidates.rs`
  - [x] **T1-30D** `HypothesisPipeline` connected via `build_learning_pipeline`, verified in `hypothesis_pipeline_diagnostics.rs`
  - [x] **T1-30E** Hypothesis evidence/validation flow wired through event bus, verified in diagnostics
  - [x] **T1-30F** `EvolutionEngineTrait`, `EventSubscriber`, graph types (`HypothesisNode`/`Edge`/`Graph`) wired and verified; all touched files 0 errors/0 warnings. Remaining gate warnings are in unrelated files (reflection_pipeline, reports/review/pattern, scheduler, policy internals, learning_coordinator internals).

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
P4-003 through P4-006 are already mostly done — consolidation, context limits,
and explicit commands all work. Only P4-002A-D are real implementation gaps.

### P4-001: Trace the request lifecycle (~10 min, 2 tasks)

- [ ] **P4-001A**: Walk the agent request path
  File: `src/agent/loop_runner.rs:60-305`
  Trace: `run_agent_goal` → `AgentLoop::new` → `AgentLoop::run` → planner →
  `memory_retrieval.retrieve()` → ActionSelector → safety_gate → record_success
  Document the full path in this section
  (~5 min: read loop_runner.rs, write the trace into this section)

- [ ] **P4-001B**: Walk the workflow request path
  Files: `src/workflows/engine/executor/execute.rs:12-111`,
  `src/workflows/engine/executor/actions.rs:14-111`
  Trace: `start_workflow` → `execute_workflow` → `execute_step_action` →
  tool dispatch. Note that `read_memory_before_action` is stubbed.
  Compare against agent loop to identify gaps
  (~5 min: read both files, write comparison into this section)

### P4-002: Wire memory retrieval into workflow execution (~30 min, 5 tasks)

#### P4-002A: Add memory_retrieval to WorkflowEngine (~10 min)

- [ ] **P4-002A-1**: Add field to struct
  File: `src/workflows/engine/types.rs:52`
  Add `pub(crate) memory_retrieval: Option<Arc<MemoryRetrieval>>` to
  `WorkflowEngine` struct. Update `Clone` impl to include it.

- [ ] **P4-002A-2**: Update constructor
  File: `src/workflows/engine/types.rs` (impl block) +
  `src/bridge/app/initialization/workflow_acp.rs:48`
  Add `memory_retrieval` parameter to `with_database_and_coordinator()`.
  Pass it through when constructing `WorkflowEngine`.

- [ ] **P4-002A-3**: Implement `read_memory_before_action` real logic
  File: `src/workflows/engine/executor/experience.rs:22-56`
  Replace stub with: check `SKIP_MEMORY_READ` first, then call
  `self.memory_retrieval.retrieve(action)` if available, return
  `ToolOutput` with retrieved memories, log+return `None` if unavailable.

#### P4-002B: Bound retrieval results (~5 min)

- [ ] **P4-002B-1**: Add limit parameter to `retrieve()`
  File: `src/memory/retrieval.rs:63`
  Add `limit: usize` parameter (default 10) to `retrieve()`. After sorting,
  call `.truncate(limit)`. Change return to `Result<Vec<RetrievalResult>, anyhow::Error>`.

#### P4-002C: Wire SKIP_MEMORY_READ into workflow execution (~5 min)

- [ ] **P4-002C-1**: Check skip list in `read_memory_before_action`
  File: `src/workflows/engine/executor/experience.rs:22-56`
  At top of `read_memory_before_action`, check
  `Self::should_skip_memory_read(action)` — return `None` for skipped actions.

#### P4-002D: Error resilience (~10 min)

- [ ] **P4-002D-1**: Wrap retrieve() error handling
  File: `src/memory/retrieval.rs:63-82`
  Wrap working/permanent memory search in try logic. Catch errors, log WARN,
  return empty Vec. All callers handle empty results gracefully.

- [ ] **P4-002D-2**: Wrap workflow executor call
  File: `src/workflows/engine/executor/execute.rs:41`
  In `execute_workflow`, wrap `read_memory_before_action` call in match.
  On error, log WARN and continue with empty memory context.

### P4-003: Automatic experience capture (Verify / ~5 min)

- [ ] **P4-003A**: Verify experience recording is wired
  File: `src/workflows/engine/executor/execute.rs:63`
  `record_experience_after_action` is called after each workflow step.
  Note: NOT called for regular MCP tool calls (only workflow steps).
  Decide if this gap matters or is acceptable.
  (~5 min: read execute.rs call site, write decision + rationale here)

### P4-004: Automatic memory promotion (Already done / Verify)

`MemoryRetrieval::consolidate()` at `memory/retrieval.rs:154-208` with
`should_promote()` rules (confidence >= 0.7, importance >= 0.8, access >= 5,
knowledge/important/learned tags). Verified live.

### P4-005: Context integration (Already done / Verify)

Context engine enforces limits — memory enters through existing context
lifecycle, not raw DB output. Verified by reading context module.

### P4-006: Explicit memory commands remain supported (Already done / Verify)

No duplicate memory implementation. Explicit tools operate against same
persistent state. Verified by code inspection.

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

Connect experience evaluation to the existing memory/knowledge systems.

Experiences should be evaluated for whether they belong in:

temporary/working experience
episodic memory
semantic knowledge
strategic knowledge
other existing memory categories

Do not create a new memory hierarchy unless the existing architecture cannot support the requirement.

Acceptance criteria:

Important experiences can become persistent knowledge.
Low-value experiences remain temporary or are discarded.
Duplicate information is handled appropriately.
Confidence is preserved or updated correctly.
Existing storage architecture remains authoritative.
P4-005: Context integration

Ensure retrieved memory enters the Context Engine through the existing context lifecycle rather than being injected through an unrelated shortcut.

Memory must remain subject to the existing context limits, prioritization, compression, and lifecycle rules.

Acceptance criteria:

Memory is treated as context input, not raw database output.
Context limits remain enforced.
Memory cannot silently consume the entire context window.
Existing context hierarchy remains intact.
P4-006: Explicit memory commands remain supported

Automatic memory must not replace explicit memory operations.

The agent must still be able to intentionally:

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

P5-001: Memory failure isolation (~5-min micro-tasks)

- [ ] **P5-001-M1**: Audit `MemoryRetrieval::retrieve()` and working/permanent
  search call sites for error propagation (src/memory/retrieval.rs:63).
  DONE WHEN: a written list in this task's note of every caller of retrieve()
  / get_from_working() / get_from_permanent(), each marked "error reaches user"
  or "silently degraded".
- [ ] **P5-001-M2**: Audit DB-unavailable behavior: rename robot_brain.db while
  the server runs (Windows: copy first - file may be locked), then call
  `search_memory` via RobotBrainClient. DONE WHEN: observed behavior (graceful
  error vs crash) is recorded in this task's note with the actual tool response.
- [ ] **P5-001-M3**: Add test_suite test `memory_failure_isolation`: spawn server
  on tempdir (reuse the IsoClient pattern from queue_durability.rs), stop it,
  insert a junk row into the memories table via rusqlite, restart, call
  `search_memory` + `list_memories`. DONE WHEN: test asserts no panic and either
  empty result or MCP error response; wired into mod.rs/main.rs; suite green.
- [ ] **P5-001-M4**: Verify failures are recorded via existing observability:
  grep the memory paths for `tracing::error!`/`metrics` on failure branches
  found in M1. DONE WHEN: each identified failure branch has a log/metric, or a
  fix commit adds one.

P5-002: Experience failure isolation (~5-min micro-tasks)

- [ ] **P5-002-M1**: Read `record_experience_after_action`
  (src/workflows/engine/executor/experience.rs) and its caller at
  execute.rs:63. DONE WHEN: a note states whether recording errors can propagate
  to the step result, citing the exact match/if-let that contains them.
- [ ] **P5-002-M2**: Add test_suite test `experience_failure_isolation`: run a
  workflow step whose experience recording fails (e.g. point recorder at a
  read-only DB path or inject an oversized payload). DONE WHEN: test asserts the
  tool response is still success while a WARN/ERROR appears in stderr capture;
  wired + suite green.

P5-003: Restart and persistence test (~5-min micro-tasks)

- [ ] **P5-003-M1**: Read test_suite/src/tests/queue_durability.rs and list which
  of store -> shutdown -> restart -> retrieve it already covers. DONE WHEN: gaps
  are written here as explicit bullet points (covered/not-covered per stage).
- [ ] **P5-003-M2**: Extend queue_durability.rs (or new flow_restart_memory.rs):
  store fact via `store_memory`, kill child, respawn same tempdir, call
  `run_agent_goal` referencing the fact WITHOUT explicit search. DONE WHEN:
  assertion on goal output containing the fact passes; wired + suite green.
  (If run_agent_goal output is not deterministic enough to assert on, assert on
  the retrieval log line instead and note the limitation.)

P6: End-to-End Cognitive Integration Tests

[RESEARCHED] (2026-08-24) No flow_*.rs files exist yet in test_suite/src/tests/.
Each P6 item maps to a single test file (~15-30 min each); sliced into 5-min
steps: scaffold → implement assertions → wire into mod.rs/main.rs → gate.
P6-001/P6-002/P6-003 overlap heavily with P9-002/P9-003/P9-006 - implement ONCE
under P9 file names and cross-reference here to avoid duplicate work.

P6 items are implemented ONCE under the P9 flow files (cross-referenced) to
avoid duplicate work. Each P6 checkbox below is satisfied when its P9 counterpart
is green AND the specific extra assertion listed here exists.

- [ ] **P6-001** Automatic retrieval = P9-002 complete PLUS: the test proves the
  agent received the memory without any explicit search tool call in the trace.
- [ ] **P6-002** Automatic experience = P9-003 complete PLUS: the asserted
  experience was created by the goal run itself (timestamp/count delta), not
  pre-existing.
- [ ] **P6-003** Cross-session = P9-006 complete PLUS: Session B uses a fresh
  client process (not just a second connection), proving persistence not cache.
- [ ] **P6-004** Explicit+automatic consistency: inside flow_cross_session_memory.rs,
  add assertion that the fact stored explicitly via `store_memory` is returned by
  `search_memory` AND surfaced by automatic retrieval - same persistent state.
- [ ] **P6-005** Memory-failure resilience: inside the P5-001-M3 test, after
  corrupting the DB, also call `run_agent_goal` with a trivial goal. DONE WHEN:
  goal completes (any status except crash/disconnect).
- [ ] **P6-006** Context pressure: DEPENDS ON P4-002B (retrieval limit). Test:
  insert 200+ memories via loop, call retrieval-heavy goal, assert retrieval
  result count <= limit and latency bounded. Scaffold only after P4-002B lands;
  until then this stays blocked.

P7: Concurrency and Lifecycle Audit

[RESEARCHED] (2026-08-24) Audit checklist task. Known shared-state points:
`job_queue.lock().unwrap_or_else` mutex in manager.rs:380, tokio RwLock on
workers, broadcast bus with Lagged handling (runner.rs:27-33 already drains).
Sliced into 5-min audit steps:

- [ ] **P7-M1**: Grep for `.lock().await` / `.read().await` / `.write().await`
  in src/experience/ and src/workflows/. For each hit, check whether the guard
  is alive across a subsequent `.await` in the same scope. DONE WHEN: every hit
  is listed here with file:line and verdict safe/unsafe.
- [ ] **P7-M2**: Grep for std `Mutex` (`lock().unwrap_or_else`) usage. The
  compiler prevents await-across-std-Mutex, so instead verify guards are dropped
  before awaits by scoping (e.g. manager.rs:379-387 block is correct). DONE WHEN:
  all std-Mutex sites listed with verdict; any guard held too long gets a fix
  commit.
- [ ] **P7-M3**: Duplicate-write audit: read the job claim path
  (queue pending_jobs / dequeue + worker_manager dispatch). DONE WHEN: a written
  answer exists to "can two workers receive the same job id?" with the code
  lines that prove it (claim flag, status transition, or single-dispatcher
  design).
- [ ] **P7-M4**: Shutdown audit: trace what happens to in-flight writes when the
  process is killed (kill_on_drop in tests mimics this). DONE WHEN: documented
  guarantee statement here, e.g. "SQLite WAL ensures committed txns survive;
  uncommitted work is lost and re-derived" - or a gap filed as a new task.
- [ ] **P7-M5**: Cancellation audit: check tokio tasks spawned at startup
  (scheduler worker, worker_manager background, event subscriber runner.rs)
  for partial-write windows on cancellation. DONE WHEN: each spawned task is
  listed with its cancellation behavior.
- [ ] **P7-M6**: Concurrent-request test `concurrent_store.rs`: spawn one server
  on tempdir, fire 20 parallel `store_memory` calls (tokio JoinSet), then
  `list_memories` and assert all 20 present with distinct ids. DONE WHEN: test
  green; wired + suite pass.
- [ ] **P7-M7**: Fix issues found in M1-M6, ONE fix per session step (gate +
  commit each). DONE WHEN: zero unsafe verdicts remain unresolved and P7 is
  marked `[x]` with the audit summary.

P8: Runtime and Fresh-Start Validation

[RESEARCHED] (2026-08-24) Fresh-start matrix task. Overlaps with P2-001E-M2
(tempdir launch diff). Sliced:

Each item below should end up automated in test_suite where feasible (reuse
the IsoClient pattern from queue_durability.rs); manual runs are acceptable
only for the corrupted-state matrix, and must be recorded in the task note.

- [ ] **P8-M1**: First startup on pristine tempdir: copy built binary into
  tempfile::tempdir(), spawn via stdio MCP, init (get_workflow + search_memory),
  call tools/list. DONE WHEN: robot_brain.db exists beside exe, tools/list
  returns the full catalog, no panic in stderr.
- [ ] **P8-M2**: Restart on same tempdir: kill child, respawn, init again.
  DONE WHEN: init succeeds, no migration errors in stderr, previously stored
  memory still retrievable.
- [ ] **P8-M3**: Shutdown cleanliness: kill during idle, open robot_brain.db
  with rusqlite, run `PRAGMA integrity_check`. DONE WHEN: result is `ok`.
- [ ] **P8-M4**: Missing optional config/dirs: tempdir with NO files_to_import/
  and no config file. DONE WHEN: server starts, ingest-related tools return
  graceful errors (not crashes) when invoked.
- [ ] **P8-M5**: Empty memory DB: on pristine instance call search_memory,
  list_memories, query_knowledge, list_experiences. DONE WHEN: all return
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

All tests must use `RobotBrainClient` (`.agents/live_test/mcp_client.py` or
the Rust equivalent) and call `get_workflow` + `search_memory` before any
substantive tool.

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

# 7. Definition of Done

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
