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

- **v0.0.1** — `robot_architecture/v0.0.1/ARCHITECTURE.md`. What the codebase
  currently approximates. TIER 1 finishes conforming to this.
- **v0.0.2** — `robot_architecture/RoBoT Architecture v0.0.2.md`. Intermediate
  upgrade: elevate Context + Conversation to first-class, add Data Contracts.
  TIER 2 conforms existing systems to this.
- **v0.0.2.1** — `robot_architecture/v0.0.2.1/` (00.md-33.md + appendices). The
  FINAL architectural baseline. Adds Execution Engine, Tool Engine, Memory
  Hierarchy, Context Lifecycle, Retrieval Pipeline, Prompt Construction,
  Strategic Learning, Confidence System, Storage, Database Design, Background
  Workers, Security & Trust, Observability, Developer Interface/Control Plane,
  Configuration, Testing, Deployment. TIER 3 builds the missing subsystems.

## Current codebase state (verified 2026-08-11)

- Workspace: two independent programs — `robot_brain` (root, MCP server) and
  `test_suite/` (E2E tests via MCP protocol).
- Builds with **0 cargo warnings**, **128 MCP tools**, **333/333 tests pass**,
  0 code-quality issues. Coverage gap: 50 server tools untested (60.9%).
- `#![allow]` / `#[allow]` in `src/`: **0** (clean).
- `self_check.rs` files: **8 remain** (planner, learning, knowledge, experience,
  experience/reflection, experience/hypothesis, experience/hypothesis/support/graph,
  experience/hypothesis/services).
- Cognitive loop (P0/P1) DONE: `ExperienceRecorded → Reflection → Hypothesis →
  Knowledge → Reputation`; `run_agent_goal` agent loop works.
- P4 open: in-memory `JobQueue`; no loop-health metrics; generic MCP dispatch
  does not emit experiences.
- **No v0.0.2/v0.0.2.1 new subsystems exist**: no Context Engine, Conversation
  Engine, Execution Engine, Tool Engine, Retrieval Pipeline, Prompt
  Construction, AI Runtime, multimodal, GUI, security/trust, observability.

## Constraints

- Strict Rust coding standards (no panics/unwrap/expect, no placeholders, no
  `#[allow(...)]`, no ignored `_` vars). Enforced by the test suite.
- Incremental workflow: after EACH increment, run the gate (below) green, then
  commit + push, then STOP. Never batch.
- Large-file rule: split `.rs` files over ~1000 lines that mix
  responsibilities (see `.agents/LARGE_FILE_REFACTOR.md`).
- Local-first: the cognitive architecture must work against cloud/external
  models first. AI Runtime (Candle) is built last, as an enhancement layer.

## The verify gate (run after EVERY increment)

```bash
cargo build --release -p robot_brain          # 0 warnings
python3 .agents/live_test/live_test_all.py     # 54/54
cd test_suite && cargo build --release && ./target/release/test_suite  # 333/333, 0 code-quality
```
All three must pass. If any is red, the increment is NOT done. Fix it before
claiming done. Never commit a red gate.

---
# 3. APPROACH — three tiers of small increments

Work through three tiers in order. Each tier is a checklist of small
increments. Do them top-to-bottom, one at a time, with the verify gate green
between each.

- **TIER 1 — Finish v0.0.1** (clean baseline). Clear the remaining self_check
  debt, migrate the queue, add loop metrics, close the MCP→experience path.
  No new features; just finish what v0.0.1 requires. **End state = finished v0.0.1.**
- **TIER 2 — Reach v0.0.2** (upgrade existing systems). Introduce Data Contracts,
  then upgrade each existing subsystem (Memory, Knowledge, Experience, Learning,
  Planner, Skills/Workflows, World Model, Personality) to its v0.0.2 chapter.
  **End state = finished v0.0.2.**
- **TIER 3 — Reach v0.0.2.1** (add missing subsystems). Build the new engines
  from the v0.0.2.1 chapters: Execution, Tool, Memory Hierarchy, Context
  Lifecycle, Retrieval Pipeline, Prompt Construction, Strategic Learning,
  Confidence System, Storage/Database, Background Workers, Security & Trust,
  Observability, Developer Interface/Control Plane, Configuration, Testing,
  Deployment — then AI Runtime (Candle), Multimodal, GUI last.
  **End state = finished v0.0.2.1.**

**Why this order:** finish v0.0.1 first so no dead-code debt is carried into a
refactor. Upgrade foundation systems (Data Contracts, Memory, Knowledge) before
the Context/Conversation engines so those engines consume real contract-shaped
data, not stubs. Build the cognitive architecture against cloud/external models
first; AI Runtime (Candle) comes last as the local provider behind the
`InferenceProvider` trait, and is the prerequisite for Multimodal.

---

# 4. TIER 1 — Finish v0.0.1 (clean baseline)

> Goal: zero self_check.rs, SQLite-backed queue, loop-health metrics, generic
> MCP dispatch emits experiences. End state = a clean v0.0.1 baseline.
> Tick `[x]` when an increment is committed with a green gate.

## 1A. Remove remaining self_check.rs files (V2-09)

The proven pattern: wire a real MCP tool that calls the API, then delete the
self_check. One file per increment.

- [ ] **T1-01** Remove `src/planner/self_check.rs` — wire a planner API into an
      MCP tool (or a `test_suite/` integration test), then delete.
- [ ] **T1-02** Remove `src/learning/self_check.rs` — wire a learning API into an
      MCP tool, then delete.
- [ ] **T1-03** Remove `src/knowledge/self_check.rs` — wire a knowledge API into
      an MCP tool, then delete.
- [ ] **T1-04** Remove `src/experience/self_check.rs` — wire an experience API
      into an MCP tool, then delete.
- [ ] **T1-05** Remove `src/experience/reflection/self_check.rs` — wire a
      reflection API into an MCP tool, then delete.
- [ ] **T1-06** Remove `src/experience/hypothesis/self_check.rs` — wire a
      hypothesis API into an MCP tool, then delete.
- [ ] **T1-07** Remove `src/experience/hypothesis/support/graph/self_check.rs` —
      wire a graph API into an MCP tool, then delete.
- [ ] **T1-08** Remove `src/experience/hypothesis/services/self_check.rs` — wire
      a hypothesis-service API into an MCP tool, then delete.

**Done when:** `find src -name "self_check.rs"` returns empty; gate green.

## 1B. SQLite-backed JobQueue (V2-11)

- [ ] **T1-09** Add `job_queue` table + migration in `src/database/migrations/`.
- [ ] **T1-10** Wire enqueue/dequeue through `src/experience/queue.rs` to SQLite.
- [ ] **T1-11** Handle broadcast `Lagged` events explicitly (skip+log or drain)
      in the worker path.
- [ ] **T1-12** Update `src/bridge/app/initialization.rs` startup verification
      (remove the "pending full SQLite-backed queue integration" comment).

**Done when:** queue survives a process restart in a manual test; gate green.

## 1C. Loop-health metrics (V2-12)

- [ ] **T1-13** Add `loop_latency` metric capture around `AgentLoop::run`
      (`src/experience/metrics.rs`).
- [ ] **T1-14** Add `confidence_drift` metric capture in the event-spine
      handlers (`src/experience/integration/event_subscriber/handlers.rs`).
- [ ] **T1-15** Add promotion-throughput (reflection→hypothesis→knowledge)
      metric.
- [ ] **T1-16** Expose the three new metrics via the `get_system_status` MCP
      tool + the JSON report.

**Done when:** `get_system_status` live shows loop_latency / confidence_drift /
promotion_throughput; gate green.

## 1D. Close the generic MCP→experience path (V2-05)

- [ ] **T1-17** Hook `emit_experience_recorded` into the post-tool-execution
      dispatch wrapper in `src/bridge/mcp/handlers/`.
- [ ] **T1-18** Ensure idempotency (the agent loop already publishes once — no
      double-emit). Add a guard or source tag.

**Done when:** calling `store_memory` directly records an experience; no
double-emit from the agent loop; gate green.

**End of TIER 1 = finished v0.0.1. Tag: `v0.0.1-clean`.**

---

# 5. TIER 2 — Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.

## 2A. Data Contracts (Chapter 05)

Create `src/data_contracts/`. Types-only first; wire adapters incrementally.

- [ ] **T2-01** Create `src/data_contracts/` module skeleton (mod.rs, version
      field, shared traits).
- [ ] **T2-02** `Observation` struct + serde round-trip unit test.
- [ ] **T2-03** `ContextPacket` struct + serde round-trip test.
- [ ] **T2-04** `MemoryRecord` struct + serde round-trip test.
- [ ] **T2-05** `ExperienceRecord` — alias/migrate the existing type; serde
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

# 6. TIER 3 — Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Chapter refs are `robot_architecture/v0.0.2.1/<NN>.md`.

## 3A. Execution & Tool engines (Chapters 12 & 13)

- [ ] **T3-01** `src/execution/` skeleton — execution isolation, action
      authorization (Chapter 12).
- [ ] **T3-02** Execution: workflow graphs/DAGs + checkpoints.
- [ ] **T3-03** Execution: result normalization + recovery.
- [ ] **T3-04** `src/tools/` (Tool Engine) — capability registration contracts
      distinct from skills (Chapter 13).
- [ ] **T3-05** Tool: permissions + input/output contracts + isolation.

## 3B. Context subsystem (Chapters 07, 15, 16, 17)

- [ ] **T3-06** `src/context/` (Context Engine) skeleton — RetrievalPlanner,
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

## 3C. Conversation Engine (Chapter 06)

- [ ] **T3-12** `src/conversation/` skeleton — interaction/session ownership,
      lifecycle, interruption, traceability (Chapter 06).
- [ ] **T3-13** Conversation: Input → Understanding → Context Assembly →
      Reasoning → Planning → Tool Execution → Response → Learning pipeline.
- [ ] **T3-14** `converse` MCP tool that runs the full pipeline and returns a
      context-informed, memory-informed response.

## 3D. Strategic Learning & Confidence (Chapters 18 & 19)

- [ ] **T3-15** Strategic Learning: long-horizon evidence, policy/strategy
      changes, experiments, validation, rollback (Chapter 18).
- [ ] **T3-16** Confidence System: evidence, source quality, recency,
      relationship/skill/workflow confidence, decay (Chapter 19).

## 3E. Storage, Database, Workers (Chapters 21, 22, 23)

- [ ] **T3-17** Storage Architecture: durable persistence, transactions,
      backups, migrations, recovery, integrity (Chapter 21).
- [ ] **T3-18** Database Design: schema ownership, migration discipline,
      indexes, constraints, transactional integrity (Chapter 22).
- [ ] **T3-19** Background Workers hardening: ownership, queues, retries,
      idempotency, cancellation, backpressure, supervision, health (Chapter 23).

## 3F. Governance & Safety (Chapters 24, 25, 26)

- [ ] **T3-20** AI Contributor Operating Agreement: human/AI contribution
      boundaries, review gates, traceability (Chapter 24 — process + tests).
- [ ] **T3-21** Security & Trust: identity, authorization, capability
      security, trust boundaries, memory protection, audit (Chapter 25).
- [ ] **T3-22** Self-Improvement/Evolution: controlled hypotheses,
      experiments, promotion gates, rollback, human control (Chapter 26).
- [ ] **T3-23** Self-Improvement: a hypothesis lifecycle runs
      confirmed→confidence-increase / rejected→decrease end-to-end.

## 3G. Observability & Control Plane (Chapters 27, 28)

- [ ] **T3-24** Cognitive Monitoring/Observability: traces, correlation,
      metrics, events, decision evidence, health, retention, privacy
      (Chapter 27).
- [ ] **T3-25** Developer Interface/Control Plane: inspection + control,
      read/write separation, permissions, safe mutation, audit, recovery
      (Chapter 28).
- [ ] **T3-26** Control Plane: cognitive traces reconstruct the full request
      lifecycle; capability-denied actions blocked + audited.

## 3H. Config, Testing, Deployment (Chapters 29, 30, 31)

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

## 3I. AI Runtime / Model Manager (Chapter 14 + appendix)

- [ ] **T3-31** `InferenceProvider` trait + `src/ai_runtime/` skeleton; cloud
      provider implementation first.
- [ ] **T3-32** Model Manager: discovery, metadata, selection, lifecycle.
- [ ] **T3-33** Candle-based local LLM provider.
- [ ] **T3-34** Candle-based local embeddings provider.
- [ ] **T3-35** `inference` MCP tool (routes through the runtime; cloud and
      local interchangeable behind the trait).

## 3J. Multimodal (Appendix A)

- [ ] **T3-36** Audio Engine: STT (Whisper via Candle) + audio ingest
      (WAV/MP3/FLAC/OGG/M4A).
- [ ] **T3-37** Audio Engine: TTS (Piper/Kokoro via Candle).
- [ ] **T3-38** Vision Engine: OCR + image understanding + screenshot analysis.
- [ ] **T3-39** `transcribe` / `synthesize` / `ocr` MCP tools (real results).

## 3K. GUI / Dashboard (Chapter 28)

- [ ] **T3-40** `src/control_plane/` API + event stream (runtime stays
      headless-operable without GUI).
- [ ] **T3-41** Frontend (separate crate/dir) consuming the event stream;
      renders real (not fake) system events.

## 3L. Future expansion & roadmap (Chapters 32, 33)

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
- Queue is SQLite-backed and survives a process restart.
- `get_system_status` shows loop_latency / confidence_drift /
  promotion_throughput.
- Generic MCP tool execution emits an experience (no double-emit).
- Gate green: 0 build warnings, 54/54 live, 333/333 suite.

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

# 8. v0.0.1 CONFORMANCE WORK (legacy status — for reference)

> Moved here from AGENTS.md on 2026-08-11. Historical status of the v0.0.1
> conformance work (P0-P4). TIER 1 above supersedes this for forward planning,
> but it records what was already done so progress isn't re-attempted.

## P0 — event spine drives learning — DONE
- V2-01/02/03: `ExperienceRecorded → Reflection → Hypothesis → Knowledge →
  Reputation` wired in `src/experience/integration/event_subscriber/handlers.rs`.

## P1 — cognitive loop — PARTIAL
- V2-04: goal-driven `src/agent/` loop DONE; `run_agent_goal` MCP tool works
  (status=Achieved, confidence=0.507).
- V2-05: generic MCP dispatch does NOT auto-emit experience (→ T1-17/T1-18).

## P2 — stub chapters — DONE
- V2-06: World Model exists. V2-07: `src/agent/safety_gate/` (sandbox,
  rollback, hallucination, uncertainty). V2-08: Personality emotional_weight →
  confidence (`personality/decision_making.rs:49-51`).

## P3 — self-check probes — REMAINING (→ T1-01..T1-08)
- V2-09: 8 self_check.rs files remain. Pattern: wire MCP tool, delete self_check.

## P3.1 — `#![allow]` violations — RESOLVED
- 2026-08-11: `grep -rn '#!\[allow' src` returns 0; `grep -rln '#\[allow' src`
  returns 0. Both clean.

## P4 — performance maturity — REMAINING (→ T1-09..T1-16)
- V2-11: in-memory JobQueue (→ SQLite). V2-12: no loop-health metrics.

## Verified state (2026-08-11)
- 0 cargo warnings; 128 MCP tools; 333/333 tests; 0 code-quality issues.
- Coverage gap: 50 server tools untested (60.9%) — suite exits non-zero.
- Large-file refactors done: `personality/personality.rs` (352→101, split into
  presets/adaptation/decision_making); `memory/handlers.rs` (400→ directory).
