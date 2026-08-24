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
  [DONE] (2026-08-24) App::new/App::run are probe-free; all diagnostics run only
  via `robot diagnose`. Diagnostics hardened in 5 commits (8efce54, 42e0493,
  c15acef, 462699f, 2563930): JobQueue/experience-repo/scheduler/worker probes
  now use isolated temp databases and buses; personality probes snapshot and
  restore live state. Gate green after each fix: 148/148 tests, 0 warnings,
  0 code issues, 0 untested tools.
- [x] **P2-001B** - Existing test coverage is preserved.
  [DONE] (2026-08-24) Verified via test_suite_report.json: passed=148 failed=0
  errors=0, 0 warnings, 0 code issues, 0 untested tools, overall_success=true.
  All subsystem APIs previously covered by startup probes remain exercised via
  `robot diagnose` diagnostics + the 148-test registry.
- [ ] **P2-001C** - Diagnostics remain available through an explicit mechanism.
- [ ] **P2-001D** - Startup remains deterministic.
- [ ] **P2-001E** - Startup does not mutate test data merely by launching RoBoT.

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

- [ ] **P4-001B**: Walk the workflow request path
  Files: `src/workflows/engine/executor/execute.rs:12-111`,
  `src/workflows/engine/executor/actions.rs:14-111`
  Trace: `start_workflow` → `execute_workflow` → `execute_step_action` →
  tool dispatch. Note that `read_memory_before_action` is stubbed.
  Compare against agent loop to identify gaps

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
P5-001: Memory failure isolation

Test behavior when:

database unavailable
memory retrieval fails
memory write fails
malformed memory record encountered
embedding/retrieval subsystem unavailable
experience processing fails

Required behavior:

A memory failure must not unnecessarily kill the user's request.

The system should degrade gracefully and record the failure through existing observability mechanisms.

P5-002: Experience failure isolation

Verify that an error during post-response experience processing cannot corrupt or invalidate the completed interaction.

The user's response should remain successful even if post-processing fails.

P5-003: Restart and persistence test

Verify:

store information
        ↓
shutdown
        ↓
restart robot_brain
        ↓
new request
        ↓
automatic retrieval
        ↓
previous information available

This must work without manually asking the agent to search memory.

P6: End-to-End Cognitive Integration Tests

Add integration tests covering the actual runtime rather than testing only individual functions.

P6-001: Automatic retrieval test

Store a known fact.

Start a new interaction.

Ask a question requiring that fact.

Verify that the agent receives the relevant memory automatically.

P6-002: Automatic experience test

Perform an interaction that produces a meaningful experience.

End the interaction.

Verify that the experience was automatically recorded.

P6-003: Cross-session memory test

Session A:

User provides important information.

Session B:

User asks something related.

Verify that the information from Session A is automatically retrieved.

P6-004: Explicit + automatic memory test

Verify that:

automatic memory
+
explicit memory tools

operate against the same persistent memory state.

P6-005: Memory failure test

Disable or break the memory subsystem.

Verify that the agent can still process a request and produce a response.

P6-006: Context pressure test

Create enough memories to make retrieval potentially large.

Verify that:

retrieval remains bounded
context remains within configured limits
relevant memories receive priority
irrelevant memories are excluded
context compression remains functional
P7: Concurrency and Lifecycle Audit

Review asynchronous cognitive operations for:

locks held across await
race conditions
duplicate writes
duplicate experience processing
shutdown during memory operations
cancellation during retrieval
cancellation during persistence
concurrent requests accessing shared memory state

Existing patterns established during P2/P3 should be reused.

Do not introduce unnecessary synchronization layers.

Acceptance criteria:

No known lock-across-await problems remain.
Concurrent requests do not corrupt memory state.
Shutdown leaves persistent state consistent.
Cancellation does not leave orphaned cognitive operations.
P8: Runtime and Fresh-Start Validation

Test robot_brain from a clean environment.

Verify:

database creation
required directories
configuration loading
default configuration
first startup
restart
shutdown
missing optional configuration
missing memory data
empty memory database
corrupted/invalid recoverable state

The system must not depend on artifacts left behind by previous development/test runs.

P9: Final v0.0.1 Integration Gate

Before declaring v0.0.1 complete, add tests in `test_suite/src/tests/` that
verify each end-to-end flow. Each test must run against a live
`robot_brain` subprocess via MCP (the existing test pattern).

All tests must use `RobotBrainClient` (`.agents/live_test/mcp_client.py` or
the Rust equivalent) and call `get_workflow` + `search_memory` before any
substantive tool.

### P9-001: Flow A — Basic cognition

Test: `test_suite/src/tests/flow_basic_cognition.rs`

1. Spawn `robot_brain` subprocess via MCP.
2. Call `run_agent_goal` with a simple goal (e.g. "store the fact that the
   sky is blue").
3. Assert: goal status returns `Achieved`, response is non-empty.
4. Verify the agent produced output without crashing.

### P9-002: Flow B — Automatic memory retrieval

Test: `test_suite/src/tests/flow_auto_memory_retrieval.rs`

1. Store 3+ distinct facts via `store_memory` (different topics).
2. Call `run_agent_goal` with a goal referencing one of those facts.
3. Assert: the retrieved memory context is non-empty and contains the stored
   fact (verify via `tracing` log output or a probe).
4. Verify the agent uses the retrieved context in its reasoning.

### P9-003: Flow C — Automatic experience capture

Test: `test_suite/src/tests/flow_experience_capture.rs`

1. Call `run_agent_goal` with a goal.
2. After completion, call `list_experiences` and assert at least one new
   experience exists with the goal's outcome.
3. Verify the experience has a valid `id`, non-empty `content`, and
   `outcome` matching the loop result.

### P9-004: Flow D — Recovery (job failure → retry → completion)

Test: `test_suite/src/tests/flow_recovery.rs`

1. Use the JobQueue (`experience/queue.rs`) to enqueue a job.
2. Simulate failure (e.g. inject a transient error or use the existing
   retry logic).
3. Verify: the job is retried, eventually completes, and the final state
   is persisted in the SQLite database.
4. Check the database directly via `rusqlite` that the job record shows
   `completed` status.

### P9-005: Flow E — Restart recovery

Test: `test_suite/src/tests/flow_restart_recovery.rs`

1. Persist state via `run_agent_goal` (creates `robot_brain.db`).
2. Kill the subprocess.
3. Restart `robot_brain` subprocess.
4. Verify: pending jobs are recovered, memory state is intact, the agent
   continues operating normally.
5. Copy the binary into a `tempfile::tempdir()`, spawn via stdio MCP, and
   manipulate the DB with `rusqlite` before restart.

### P9-006: Flow F — Cross-session memory

Test: `test_suite/src/tests/flow_cross_session_memory.rs`

1. Session A: call `store_memory` with a specific fact (e.g. a test
   configuration value or code path).
2. Call `run_agent_goal` to ensure it's persisted (goes through
   consolidation if applicable).
3. Session B: call `run_agent_goal` with a query referencing that fact.
4. Assert: the fact is automatically retrieved and used in reasoning.

### P9-007: Run the full integration gate

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
