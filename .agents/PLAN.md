# 1. OBJECTIVE

Take RoBoT Brain from its current state to a **finished v0.0.1 → finished
v0.0.2 → finished v0.0.2.1**, using **small 10-15 minute increments**.

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

- **v0.0.1** -- ~~`robot_architecture/v0.0.1/ARCHITECTURE.md`~~ (DELETED 2026-08-16 — v0.0.1 is complete, architecture absorbed into v0.0.2.1).
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

# P0-003 Durable Queue / Worker State Synchronization

### Problem

The worker maintains retry/execution state separately from the SQLite
durable queue.

The two systems can therefore disagree about whether work is pending,
running, failed, or complete.

### Required Outcome

There must be one authoritative lifecycle for durable work.

In-memory worker state may exist for execution purposes, but it must not
contradict durable state.

### Acceptance Criteria

- [ ] Job enters durable pending state.
- [ ] Dispatch changes state appropriately.
- [ ] Worker execution changes state appropriately.
- [ ] Failure is persisted.
- [ ] Retry is persisted.
- [ ] Success is persisted.
- [ ] Restart does not lose state.
- [ ] Tests cover each lifecycle transition.

---

# P1 - Restart / Recovery

## P1-001 Restore Pending Jobs

### Problem

Jobs can survive in SQLite across process termination, but restored jobs
must actually become executable work again.

### Acceptance Criteria

- [ ] Pending jobs survive restart.
- [ ] Running jobs have defined restart semantics.
- [ ] Restored executable jobs are re-enqueued.
- [ ] No job is executed twice accidentally.
- [ ] Completed jobs are not re-run.
- [ ] Recovery is tested using a fresh process/database lifecycle where
      practical.

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

- [ ] Startup no longer performs unnecessary subsystem test operations.
- [ ] Existing test coverage is preserved.
- [ ] Diagnostics remain available through an explicit mechanism.
- [ ] Startup remains deterministic.
- [ ] Startup does not mutate test data merely by launching RoBoT.

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
- [ ] CfgTest issues resolved
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

**End of TIER 1 = finished v0.0.1. Tag: `v0.0.1-clean`.**

---

# 5. TIER 2 -- Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.

## 2A. Data Contracts (Chapter 05)

Create `src/data_contracts/`. Types-only first; wire adapters incrementally.

- [ ] **T2-01** Create `src/data_contracts/` module skeleton (mod.rs, version
      field, shared traits).
- [ ] **T2-02** `Observation` struct + serde round-trip unit test.
- [ ] **T2-03** `ContextPacket` struct + serde round-trip test.
- [ ] **T2-04** `MemoryRecord` struct + serde round-trip test.
- [ ] **T2-05** `ExperienceRecord` -- alias/migrate the existing type; serde
      round-trip test.
- [ ] **T2-06** `Plan` struct + serde round-trip test.
- [ ] **T2-07** `Decision` struct + serde round-trip test.
- [ ] **T2-08** `ExecutionResult` struct + serde round-trip test.
- [ ] **T2-09** `Reflection` struct + serde round-trip test.
- [ ] **T2-10** `LearningUpdate` struct + serde round-trip test.

**Done when:** all contracts round-trip through serde; gate green.

## 2B. Memory Engine (Chapters 08 & 14)

- [ ] **T2-11** Add explicit memory lifecycle states + promotion gate
      (Working → Candidate → Accepted → Permanent → Archived) in `src/memory/`.
- [ ] **T2-12** Add a confidence field to memories.
- [ ] **T2-13** Add memory relationship-graph support.
- [ ] **T2-14** Memory consolidation: merge duplicates, summarize aging
      low-importance memories, keep anchor memories standalone.
- [ ] **T2-15** Pruning policy for low-value/aged memories.
- [ ] **T2-16** Migrate `MemoryRecord` to the data-contract type.

**Done when:** store/search/list/relationship MCP tools work live; gate green.

## 2C. Knowledge Graph (Chapter 20)

- [ ] **T2-17** Add `knowledge_nodes` table + migration.
- [ ] **T2-18** Add `knowledge_edges` table + migration.
- [ ] **T2-19** Add relationship confidence on edges.
- [ ] **T2-20** Entity-resolution pass (merge aliases like "rustc"/"Rust Compiler").
- [ ] **T2-21** Graph traversal queries (relationship chains).
- [ ] **T2-22** Graph-extraction pipeline (entity detect → relationship
      extract → confidence evaluate → graph update → integrate).

**Done when:** graph traversal MCP tool returns relationship chains; gate green.

## 2D. Experience Engine (Chapters 09 & 18)

- [ ] **T2-23** Enrich `ExperienceRecord` field set (goal, plan_id, result,
      success, execution_time, cost, confidence_change, tool_usage, lessons,
      related refs).
- [ ] **T2-24** Add experience categories (conversation/planning/tool/code/...).
- [ ] **T2-25** Multi-factor success scoring.
- [ ] **T2-26** Confidence-update propagation to memory/relationships/tools.
- [ ] **T2-27** Migrate `ExperienceRecord` to the data-contract type.

**Done when:** record/list/insights MCP tools return the enriched fields; gate green.

## 2E. Learning Engine (Chapter 10)

- [ ] **T2-28** Formalize the learning pipeline (reflection → candidate →
      promotion → consolidation) in `src/learning/`.
- [ ] **T2-29** Pattern discovery from repeated successful experiences.
- [ ] **T2-30** Skill emergence from patterns.
- [ ] **T2-31** Confidence/decay management + generalization over memorization.

**Done when:** before/after learning shows measurable improvement (Ch.30.15);
gate green.

## 2F. Planning Engine (Chapter 11)

- [ ] **T2-32** Richer `decompose_goal` (more action verbs, better step gen).
- [ ] **T2-33** Dependency-aware task graphs.
- [ ] **T2-34** Candidate-plan generation + evaluation.
- [ ] **T2-35** Dynamic replanning triggers + plan scoring.
- [ ] **T2-36** Migrate `Plan` to the data-contract type.

**Done when:** create_plan returns real decomposed steps + dependencies; gate green.

## 2G. Skills & Workflows (Chapters 11 & 13)

- [ ] **T2-37** Skills: permissions + performance tracking in
      `src/skills/registry/`.
- [ ] **T2-38** Skills: fallback + async/parallel/retry.
- [ ] **T2-39** Workflows: workflow-level learning + confidence in
      `src/workflows/engine/`.
- [ ] **T2-40** Workflows: workflow ranking.

**Done when:** register/discover/execute_skill + workflow tools work live; gate green.

## 2H. World Model & Personality (Chapters 13/14/20)

- [ ] **T2-41** Align `src/world_model/` with the knowledge graph (entities →
      knowledge_nodes; relationships → edges with confidence).
- [ ] **T2-42** Finalize `src/personality/` (traits, emotional weight →
      confidence, presets, adaptation, decision_making, communication).

**Done when:** world_model + 6 personality MCP tools work live; gate green.

**End of TIER 2 = finished v0.0.2. Tag: `v0.0.2`.**

---

# 6. TIER 3 -- Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Chapter refs are `robot_architecture/v0.0.2.1/<NN>.md`.

## 3A. Execution & Tool engines (Chapters 12 & 13)

- [ ] **T3-01** `src/execution/` skeleton -- execution isolation, action
      authorization (Chapter 12).
- [ ] **T3-02** Execution: workflow graphs/DAGs + checkpoints.
- [ ] **T3-03** Execution: result normalization + recovery.
- [ ] **T3-04** `src/tools/` (Tool Engine) -- capability registration contracts
      distinct from skills (Chapter 13).
- [ ] **T3-05** Tool: permissions + input/output contracts + isolation.

## 3B. Conversation Engine (Chapter 06) -- the context system's primary consumer

> Build this BEFORE the Context subsystem. The Conversation Engine is the
> user-facing tool that drives conversation; Context Memory exists to serve it.
> Each task is a single file or ~50-100 lines. Gates green independently.

- [ ] **TC-01** `ConversationSession` struct -- fields: `session_id`,
      `turn_number`, `user_messages[]`, `agent_responses[]`, `learnings[]`.
      In-memory session tracker.
- [ ] **TC-02** `ConversationEngine` -- in-memory struct with methods:
      `start_session()`, `add_turn(session_id, user_msg, agent_resp)`,
      `get_session(session_id)`.
- [ ] **TC-03** `converse` MCP tool -- takes `message` string, finds or creates
      session, calls `run_agent_goal(message)`, stores turn, returns response.
      (This is the entry point users actually use.)
- [ ] **TC-04** Learning extraction from conversation -- simple function that
      takes (user_message, agent_response) and returns a list of extracted
      facts/learnings. Starts with keyword-based pattern matching,
      upgrades to LLM extraction later.
- [ ] **TC-05** Wire `extract_learnings` into `converse` -- each extracted
      learning is stored via existing `store_memory` (memory_type =
      `preference` or `fact`).
- [ ] **TC-06** Conversation persistence -- store conversation turns to
      SQLite (`conversation_turns` table). Simple: `session_id, turn_number,
      role (user/agent), content, timestamp`.

**Done when:** `converse` tool works end-to-end (user message → agent loop
→ response → conversation stored). Gate green.

## 3C. Context subsystem (Chapters 07, 15, 16, 17)

- [ ] **T3-06** `src/context/` (Context Engine) skeleton -- RetrievalPlanner,
      TokenBudget, TopicTracker, SlidingWindow (Chapter 07).
- [ ] **T3-07** Context: 4-level memory hierarchy (L0 live, L1 working summary,
      L2 checkpoints, L3 raw DB) (Chapter 14).
- [ ] **T3-08** Context Lifecycle: creation/refresh/compaction/checkpoint/
      expiration/reconstruction (Chapter 15).
- [ ] **T3-09** Retrieval Pipeline: candidate generation → ranking →
      confidence/provenance → diversity → budget (Chapter 16).
- [ ] **T3-10** Prompt Construction: source provenance, instruction hierarchy,
      model independence, reproducibility (Chapter 17).
- [ ] **T3-11** Context policies (not every question retrieves memory);
      per-item context scores.

## 3D. Conversation Engine v2 (Chapter 06) -- context-aware pipeline

> Follows 3B (basic converse) and 3C (context subsystem). Adds context
> awareness on top of the working base.

- [ ] **T3-12** `src/conversation/context/` -- session-scoped context retrieval
      that pulls from Context Memory (3B) + Working Memory.
- [ ] **T3-13** Conversation context assembly -- before calling agent loop,
      assemble context from session history + retrieved memories.
- [ ] **T3-14** `converse` v2 -- context-informed response using Context
      Memory as session context source.

## 3E. Strategic Learning & Confidence (Chapters 18 & 19)

- [ ] **T3-15** Strategic Learning: long-horizon evidence, policy/strategy
      changes, experiments, validation, rollback (Chapter 18).
- [ ] **T3-16** Confidence System: evidence, source quality, recency,
      relationship/skill/workflow confidence, decay (Chapter 19).

## 3F. Storage, Database, Workers (Chapters 21, 22, 23)

- [ ] **T3-17** Storage Architecture: durable persistence, transactions,
      backups, migrations, recovery, integrity (Chapter 21).
- [ ] **T3-18** Database Design: schema ownership, migration discipline,
      indexes, constraints, transactional integrity (Chapter 22).
- [ ] **T3-19** Background Workers hardening: ownership, queues, retries,
      idempotency, cancellation, backpressure, supervision, health (Chapter 23).

## 3G. Governance & Safety (Chapters 24, 25, 26)

- [ ] **T3-20** AI Contributor Operating Agreement: human/AI contribution
      boundaries, review gates, traceability (Chapter 24 -- process + tests).
- [ ] **T3-21** Security & Trust: identity, authorization, capability
      security, trust boundaries, memory protection, audit (Chapter 25).
- [ ] **T3-22** Self-Improvement/Evolution: controlled hypotheses,
      experiments, promotion gates, rollback, human control (Chapter 26).
- [ ] **T3-23** Self-Improvement: a hypothesis lifecycle runs
      confirmed→confidence-increase / rejected→decrease end-to-end.

## 3H. Observability & Control Plane (Chapters 27, 28)

- [ ] **T3-24** Cognitive Monitoring/Observability: traces, correlation,
      metrics, events, decision evidence, health, retention, privacy
      (Chapter 27).
- [ ] **T3-25** Developer Interface/Control Plane: inspection + control,
      read/write separation, permissions, safe mutation, audit, recovery
      (Chapter 28).
- [ ] **T3-26** Control Plane: cognitive traces reconstruct the full request
      lifecycle; capability-denied actions blocked + audited.

## 3I. Config, Testing, Deployment (Chapters 29, 30, 31)

- [ ] **T3-27** Configuration: layered precedence (defaults → install → system
      → profile → user → runtime), validation, secrets, profiles, change
      control (Chapter 29).
- [ ] **T3-28** Testing: unit/contract/integration/persistence/event/security/
      failure-injection/recovery/migration/adapter/GUI/e2e-cognitive/regression/
      property layers (Chapter 30).
- [ ] **T3-29** Expand `test_suite/`: schema-validation matrix, edge cases,
      e2e learning loop, performance baselines (the gaps from
      `.agents/TEST_SUITE_NOTES.md`).
- [ ] **T3-30** Deployment: reproducible + versioned, validation + rollback,
      migrations, backup/recovery (Chapter 31).

## 3J. AI Runtime / Model Manager (Chapter 14 + appendix)

- [ ] **T3-31** `InferenceProvider` trait + `src/ai_runtime/` skeleton; cloud
      provider implementation first.
- [ ] **T3-32** Model Manager: discovery, metadata, selection, lifecycle.
- [ ] **T3-33** Candle-based local LLM provider.
- [ ] **T3-34** Candle-based local embeddings provider.
- [ ] **T3-35** `inference` MCP tool (routes through the runtime; cloud and
      local interchangeable behind the trait).

## 3K. Multimodal (Appendix A)

- [ ] **T3-36** Audio Engine: STT (Whisper via Candle) + audio ingest
      (WAV/MP3/FLAC/OGG/M4A).
- [ ] **T3-37** Audio Engine: TTS (Piper/Kokoro via Candle).
- [ ] **T3-38** Vision Engine: OCR + image understanding + screenshot analysis.
- [ ] **T3-39** `transcribe` / `synthesize` / `ocr` MCP tools (real results).

## 3L. GUI / Dashboard (Chapter 28)

- [ ] **T3-40** `src/control_plane/` API + event stream (runtime stays
      headless-operable without GUI).
- [ ] **T3-41** Frontend (separate crate/dir) consuming the event stream;
      renders real (not fake) system events.

## 3M. Future expansion & roadmap (Chapters 32, 33)

- [ ] **T3-42** Future Expansion gate: new capabilities integrate through
      stable contracts; document the "does it belong in an existing boundary?"
      check (Chapter 32).
- [ ] **T3-43** Capability Roadmap: architectural-gates process documented
      (Chapter 33).

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
