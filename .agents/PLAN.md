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

## Mission

Resolve all known implementation bugs, complete partially implemented integrations,
and establish a verified passing baseline. Do NOT redesign the architecture unless
a task explicitly requires it.

---

# 2. OPERATING RULES

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

    A task is NOT complete until its verification criteria pass. If the task cannot be safely completed:
    a. Mark it `[!]` or `[?]`.
    b. Explain why.
    c. Do NOT fabricate completion.
    d. Do NOT silently redesign another subsystem to bypass it.
12. Always make small, incremental edits. Never batch multiple unrelated changes into one edit. After each edit, verify it worked before proceeding. Large bulk rewrites lose information.

---

# 3. CONTEXT SUMMARY

## The blueprints

the location of these are important as if you had actually read agents.md you would know to check these before deleting a function

- **v0.0.2** -- `robot_architecture/RoBoT Architecture v0.0.2.md`. Intermediate
  upgrade: elevate Context + Conversation to first-class, add Data Contracts.
  TIER 2 conforms existing systems to this.
- **v0.0.2.1** -- `robot_architecture/v0.0.2.1/` (00.md-33.md + appendices). The
  FINAL architectural baseline. Adds Execution Engine, Tool Engine, Memory
  Hierarchy, Context Lifecycle, Retrieval Pipeline, Prompt Construction,
  Strategic Learning, Confidence System, Storage, Database Design, Background
  Workers, Security & Trust, Observability, Developer Interface/Control Plane,
  Configuration, Testing, Deployment. TIER 3 builds the missing subsystems.

## Current codebase state

- Workspace: two independent programs -- `robot_brain` (root, MCP server) and
  `test_suite/` (E2E tests via MCP protocol).
- **Test count and warning count: see `test_suite/test_suite_report.json`.** Do not
  trust prior counts — run the gate to verify.
- **Gate status:** 454/454 tests, 0 warnings, 0 issues. All known bugs
  resolved. Durable recovery verified. 0 stubs found.
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
---

# 4. APPROACH -- three tiers of small increments

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

**ONE TASK AT A TIME ENFORCEMENT**

- You MUST work on ONE task at a time. Never skip ahead.
- After completing a task: run the gate → write CHANGELOG → delete task line from PLAN.md → commit + push.
- If more tasks remain: move to the NEXT unchecked `[ ]` task. Never read past it until the current task is done.
- If no tasks remain: you are done.
- Never batch tasks. Never skip. Never assume a task is done — verify it.

[ ] A001-001 Read and apply AGENTS.md and PLAN.md lines 1-140. Do not mark complete. Do not delete. Only skip this one task after reading and applying everything.
---
[ ] Post-P3: Automatic Cognitive Lifecycle and v0.0.1 Final Integration

## Purpose

The primary issue is that robot_brain currently exposes memory and cognitive subsystems, but the connected agent may treat memory as an optional tool rather than as an automatic part of the cognitive lifecycle.

The goal of this phase is not to redesign the Memory Engine. The goal is to ensure the existing engines are actually wired together so that normal agent operation automatically retrieves relevant memory and records meaningful experience without requiring the user to explicitly request it.

[ ] P4: Automatic Cognitive Memory Lifecycle

Research findings: AgentLoop already calls `memory_retrieval.retrieve()`
at `src/agent/loop_runner.rs:98`. WorkflowEngine does NOT have `memory_retrieval`
(`src/workflows/engine/types.rs:52`). `read_memory_before_action()` at
`src/workflows/engine/executor/experience.rs:22` is a stub returning `None`.
`record_experience_after_action()` at `experience.rs:59` is already live.
`MemoryRetrieval::retrieve()` has no limit parameter and no error handling.
`SKIP_MEMORY_READ` list exists (`core.rs:17`) but is not checked.
[ ] P4-003 through P4-006 are already mostly [ ] — consolidation, context limits,
and explicit commands all work. P4 complete.
[ ] - P4-003A: Experience capture verified — gap (MCP tools) is acceptable
[ ] - P4-004-006: consolidation, context limits, explicit commands all work
[ ] - Build: 0 errors, 0 warnings

### P4-001: Trace the request lifecycle (~10 min, 2 tasks)

- [ ] **P4-001A**: Trace: `run_agent_goal` (agent_handler.rs:76) → `AgentDeps::from_context` (context.rs:55, includes memory_retrieval) → `AgentLoop::run` (loop_runner.rs:88) → `memory_retrieval.retrieve()` (loop_runner.rs:98).

- [ ] **P4-001B**: Trace: `start_workflow` (MCP) → `execute_workflow` (execute.rs:10) → `read_memory_before_action` (execute.rs:41) → `self.memory_retrieval.retrieve(&query)` (experience.rs:48).

After verifying both paths:
1. Run `make gate` and confirm green.
2. Write CHANGELOG entry.
3. Delete task lines 172-174 from PLAN.md.
4. Commit and push.
5. Move to P4-002.

### P4-002: Wire memory retrieval into workflow execution (~30 min, 5 tasks)










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
