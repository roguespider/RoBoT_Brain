# 1. OBJECTIVE
Read and apply AGENTS.md and lines 1-140 of PLAN.md 100% end-to-end.

Take RoBoT Brain from its current state to a **finished v0.0.1 → finished
v0.0.2 → finished v0.0.2.1**, using **small 5-10 minute increments**.

- Each increment is ONE small, verifiable, committable change.
- After each increment: build → live test → test suite → commit → push → go to next task.
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
  commit + push, then do next task. Never batch.
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
- After completing a task: build → test → commit → push → do "Task completion protocol" → go to very first task in PLAN.md (not STOP).
- If no tasks remain: you are done.
- Never batch tasks. Never skip. Never assume a task is done — verify it in codebase.

---

P6: End-to-End Cognitive Integration Tests

- [ ] No flow_*.rs files exist yet in test_suite/src/tests/.
Each P6 item maps to a single test file (~15-30 min each); sliced into 5-min
steps: scaffold → implement assertions → wire into mod.rs/main.rs → gate.
P6-001/P6-002/P6-003 overlap heavily with P9-002/P9-003/P9-006 - implement ONCE
under P9 file names and cross-reference here to avoid duplicate work.

P6 items are implemented ONCE under the P9 flow files (cross-referenced) to
avoid duplicate work. Each P6 checkbox below is satisfied when its P9 counterpart
is green AND the specific extra assertion listed here exists.

- [?] P6-001 through P6-004: Audit complete. All depend on P9 flow tests (flow_*.rs) not yet implemented. P6-001→P9-002, P6-002→P9-003, P6-003→P9-006, P6-004→P9 cross_session_memory. Cannot complete until P9 implemented. Marked [?] per PLAN.md rule 11.
- [ ] Build: test_suite compiles with 0 warnings





P7: Concurrency and Lifecycle Audit

- [ ]  Audit checklist task. Known shared-state points:
`job_queue.lock().unwrap_or_else` mutex in manager.rs:380, tokio RwLock on
workers, broadcast bus with Lagged handling (runner.rs:27-33 already drains).
Sliced into 5-min audit steps:

- [?] P7-M1 through P7-M7: Audit complete (2026-08-30). P7-M1: No tokio RwLock await-across-lock in experience/ or workflows/. P7-M2: No std Mutex in experience/. P7-M3: Single-dispatcher by observer_name in job_queue.rs. P7-M4: WAL confirmed in sqlite.rs. P7-M5: kill_on_drop in all test IsoClients. P7-M6: concurrent_store.rs wired in mod.rs:10, main.rs:1247. P7-M7: Gate passes with 0 code issues. Marked [?] per PLAN.md rule 11 — audit done but individual test assertions not added to test_suite. test_suite compiles with 0 warnings

- [?] **P7-M1** through **P7-M7**: Audit complete (2026-08-30). Verified: No tokio RwLock await-across-lock in experience/workflows; no std Mutex in experience; single-dispatcher by observer_name; WAL mode confirmed; kill_on_drop in all tests; concurrent_store.rs wired; gate passes clean. Marked [?] — audit done but individual test assertions not yet added to test_suite. Per PLAN.md rule 11.

P8: Runtime and Fresh-Start Validation

[RESEARCHED] (2026-08-24) Fresh-start matrix task. Overlaps with P2-001E-M2
(tempdir launch diff). Sliced:

Each item below should end up automated in test_suite where feasible (reuse
the IsoClient pattern from queue_durability.rs); manual runs are acceptable
only for the corrupted-state matrix, and must be recorded in the task note.

- [?] **P8-M6**: Audit complete. Corrupted state matrix (truncate DB, insert junk row, delete WAL sidecar) requires manual testing — not automated. Marked [?] per PLAN.md rule 11.

- **P8-M7 Convert M1/M2/M5 to fresh_start.rs -- DONE (2026-08-30).** Already implemented: fresh_start.rs exists at test_suite/src/tests/fresh_start.rs, module declared in mod.rs:18, dispatched in main.rs:1251. Covers M1 (pristine boot), M2 (restart survival), M5 (empty DB queries), M3 (shutdown integrity), M4 (missing config). Close out P8.

P9: Final v0.0.1 Integration Gate

Before declaring v0.0.1 complete, add tests in `test_suite/src/tests/` that
verify each end-to-end flow. Each test must run against a live
`robot_brain` subprocess via MCP (the existing test pattern).

All tests must use the Rust `TestMcpClient` (`test_suite/src/main.rs`) and call `get_workflow` + `search_memory` before any substantive tool.

### P9-001: Flow A — Basic cognition

[SLICED] (~20 min total, 4 x 5-min steps)

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

 - [ ] P2 complete
 - [ ] P3 complete
 - [ ] P4 automatic cognitive lifecycle complete
 - [ ] P5 failure/recovery integration complete
 - [ ] P6 end-to-end integration tests complete
 - [ ] P7 concurrency/lifecycle audit complete
 - [ ] P8 fresh-start validation complete
 - [ ] All existing tests pass
 - [ ] All new integration tests pass
 - [ ] No compiler warnings
 - [ ] No known correctness issues
 - [ ] No untested production tools
 - [ ] Automatic memory retrieval works without user instruction
 - [ ] Automatic experience capture works without user instruction
 - [ ] Persistent memories survive restart
 - [ ] Memory failure does not unnecessarily prevent normal operation
 - [ ] Context limits remain enforced
 - [ ] Explicit memory tools remain functional
 - [ ] No duplicate cognitive/memory implementation has been introduced
- [ ] Important Constraint

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

- [ ] Confirm with User what to do next. do not pass this point unless the user approves.

- [ ] do all tasks in .agents\CoObOpLoop_PLAN.md in order one task at a time.
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
