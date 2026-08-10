# 1. OBJECTIVE

Transform RoBoT Brain from its current v0.0.1-aligned state into a full realization of the **RoBoT Architecture v0.0.2** blueprint (`robot_architecture/RoBoT Architecture v0.0.2.md`), using a strict step-by-step roadmap:

1. **Finish v0.0.1** — clear all remaining cleanup/dead-code/self-check debt so the codebase is a clean baseline.
2. **Upgrade existing systems one at a time** — align each current subsystem (Memory, Experience, Knowledge, Learning, Planner, Skills, Workflows, World Model, Personality) to its v0.0.2 chapter, one system per phase.
3. **Add new systems in logical dependency order** — Context Engine, Conversation Engine, Observation System, Reasoning Engine, Execution Engine, Data Contracts, Security/Trust, AI Runtime, Multimodal, Observability/Control Plane, etc.

Each phase is independently buildable, testable, and committable, following the repo's mandatory incremental workflow ("one thing, verify it works, push, then next").

---

# 2. CONTEXT SUMMARY

## The blueprint (v0.0.2)

`robot_architecture/RoBoT Architecture v0.0.2.md` (~31,000 lines, 32 chapters + Appendices A-E). It is a full re-architecture whose headline idea is elevating **Context** and **Conversation** to first-class subsystems, plus a new **AI Runtime** layer for local model inference.

Defining shifts from v0.0.1:
- **Context Engine** as a first-class subsystem ("build the smallest possible prompt") — RetrievalPlanner, MemoryRetriever, ContextCompressor, PromptAssembler, TokenBudget, TopicTracker, SlidingWindow, RetrievalCache; 4-level memory hierarchy (L0 live, L1 working summary, L2 checkpoints, L3 raw DB); memory aging; context policies (not every question retrieves memory); per-item context scores.
- **Conversation Engine** as orchestration "central nervous system" (Input → Understanding → Context Assembly → Reasoning → Planning → Tool Execution → Response → Learning), event-driven.
- **Observation System** (entry point, normalization + classification), **Reasoning Engine** (evidence → decision, distinct from planning), **Execution Engine** (workflow graphs/DAGs/checkpoints/traces, isolated from reasoning), **Reflection System**.
- **Data Contracts** — canonical cognitive objects: Observation, ContextPacket, MemoryRecord, ExperienceRecord, Plan, Decision, ExecutionResult, Reflection, LearningUpdate.
- **AI Runtime / Model Manager** — Candle-based local inference (LLM, embeddings, Whisper STT, Piper/Kokoro TTS), Device Manager, Resource Manager, execution scheduler, tokenizer/streaming managers.
- **Multimodal** (Audio Engine STT/TTS, Vision Engine OCR/detection), **GUI/dashboard** (memory/experience/graph viewers, cognitive activity monitor, developer mode, task workspace), **Security & Trust** (identity, permissions, capability security, memory protection, audit), **Cognitive Observability/Control Plane** (cognitive traces, decision explanations, telemetry), **Background Workers**, **Self-Improvement/Evolution** (hypothesis, experimentation), config/runtime, deployment, testing, versioning/migration.

## Current codebase state (verified this session)

- Workspace: two independent programs — `robot_brain` (root, MCP server) and `test_suite/` (E2E tests via MCP protocol).
- Builds with **0 cargo warnings**, **113 MCP tools**, **333/333 tests pass**, 0 code-quality issues.
- Modules present: `database, experience, bridge, planner, skills, workflows, learning, knowledge, memory, cli, personality, agent, world_model`.
- Cognitive loop (P0/P1) DONE: `ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation`; `run_agent_goal` agent loop works (status=Achieved, confidence recorded); safety gate (`src/agent/safety_gate/`) and world model (`src/world_model/`) implemented.
- v0.0.1 cleanup NOT complete: **12 `self_check.rs` files remain** (planner, learning, knowledge, experience + subdirs, database, bridge/mcp/types, bridge/acp); **`#![allow(...)]` attributes still present in 8 production source files** (workflows/mod, workflows/engine/mod, memory/mod, knowledge/mod, experience/reputation/mod, experience/reflection/mod, experience/reflection/services/mod, experience/exploration/mod, database/models) — these violate the repo's coding standards enforced by the test suite.
- P4 (performance maturity) open: in-memory `JobQueue` not migrated to SQLite-backed queue; no loop-health metrics (loop_latency, confidence_drift, promotion throughput).
- **None of the v0.0.2 new subsystems exist**: no Context Engine, no Conversation Engine, no Observation/Reasoning/Execution engines, no AI Runtime, no Candle deps, no multimodal, no GUI, no security/trust, no cognitive observability.

## Constraints

- Strict Rust coding standards (no panics/unwrap/expect, no placeholders, no `#[allow(...)]`, no ignored `_` vars). Enforced by the test suite — any violation = test failure.
- Incremental workflow: after each phase, `cargo build --release -p robot_brain` (0 warnings) → `python3 .agents/live_test/live_test_all.py` (54/54) → `cd test_suite && cargo build --release && ./target/release/test_suite` (333/333, 0 code-quality) → commit → push → next.
- Large-file refactor rule: any `.rs` over ~320 lines mixing responsibilities should be split into a directory module.
- Local-first: cloud models optional; AI Runtime is an enhancement layer, not an architectural dependency. The cognitive-architecture refactor must work against cloud/external models first so it can be validated without a local model stack.

---

# 3. APPROACH OVERVIEW

A **phased, one-system-at-a-time roadmap** in three macro-stages, ordered so each phase's dependencies already exist:

- **STAGE 1 — Finish v0.0.1 (cleanup baseline).** Clear remaining self_check debt, `#[allow]` violations, and P4 performance items so the codebase is a clean, zero-debt starting point. No new features.
- **STAGE 2 — Upgrade existing systems to v0.0.2.** For each current subsystem, upgrade it to match its v0.0.2 chapter, one subsystem per phase, in dependency order (foundation first: Data Contracts → Memory → Knowledge Graph → Experience → Learning → Planner → Skills/Workflows → World Model → Personality). The Data Contract types are introduced early because every upgraded engine will communicate through them.
- **STAGE 3 — Add new systems in logical order.** Context Engine (the headline) → Conversation Engine (orchestration) → Observation System → Reasoning Engine → Execution Engine → Security & Trust → Cognitive Observability/Control Plane → AI Runtime (local Candle models) → Multimodal (Audio/Vision) → GUI/Dashboard → Self-Improvement/Evolution hardening → Background Workers hardening → Config/Runtime/Deployment/Testing/Versioning polish.

**Why this order:** v0.0.2's core philosophy is that intelligence emerges from cooperating subsystems communicating through stable contracts. Finishing v0.0.1 first avoids carrying dead-code debt into a refactor. Upgrading foundation systems (Data Contracts, Memory, Knowledge) before building the Context/Conversation engines means those engines can consume real, contract-shaped data instead of stubs. The Context Engine comes before the Conversation Engine because the Conversation Engine orchestrates context assembly. The AI Runtime is intentionally placed after the cognitive architecture so the entire pipeline can be built and validated against cloud/external models first via the `InferenceProvider` trait; the full Candle-based local provider (LLM, embeddings, Whisper, Piper/Kokoro) is then built as the local implementation of that trait — fully in scope — and is the prerequisite for the multimodal step that follows.

---

# 4. IMPLEMENTATION STEPS

> **Plan location:** This plan lives at `.agents/PLAN.md`, alongside the live-test
> and skills under `.agents/`. It is referenced from `AGENTS.md` ("Plan for Future
> Work (v0.0.1 → v0.0.2)") so the agent reads it at the start of every session.
> Keep that pointer in `AGENTS.md` in sync if this file is moved or renamed.

> Conventions: every step ends with the verify-commit-push cycle. `PB` = production build gate, `LT` = live-test gate, `TS` = test-suite gate. Reference chapters are from `RoBoT Architecture v0.0.2.md`.

## STAGE 1 — Finish v0.0.1 (clean baseline)

### Step 1.1 — Audit & convert remaining self_check.rs files (V2-09)
- **Goal:** No `self_check.rs` exists purely to silence dead-code; every public API is exercised by real runtime MCP traffic or a real `test_suite/` integration test.
- **Method:** For each of the 12 remaining files (`find src -name self_check.rs`): either (a) wire a real MCP tool that calls the API then delete the self_check (the proven pattern used for personality + world_model), or (b) convert it to a `test_suite/` integration test. Apply the Dead Code Resolution Protocol: implement if the architecture describes the feature, delete if deprecated.
- **Reference:** `src/{planner,learning,knowledge,experience,experience/reflection,experience/hypothesis,experience/hypothesis/support/graph,experience/hypothesis/services,experience/evolution,database,bridge/mcp/types,bridge/acp}/self_check.rs`
- **Verify:** PB 0 warnings; `find src -name self_check.rs` returns empty.

### Step 1.2 — Remove all `#[allow(...)]` / `#![allow(...)]` from production source (V2-10b follow-up)
- **Goal:** Zero `allow` attributes in `src/`; the test-suite code-quality check stays green.
- **Method:** For each of the 8 production files flagged (`workflows/mod.rs`, `workflows/engine/mod.rs`, `memory/mod.rs`, `knowledge/mod.rs`, `experience/reputation/mod.rs`, `experience/reflection/mod.rs`, `experience/reflection/services/mod.rs`, `experience/exploration/mod.rs`, `database/models.rs`): resolve the underlying dead-code/unused-var issue per the coding standards (use the value, restructure, or implement the feature) — never re-silence with `allow`. (test_suite's own `allow`s are out of scope for production but should be tracked separately.)
- **Reference:** `grep -rn '#\[allow\|#\[allow\|#!\[allow' src/`
- **Verify:** PB 0 warnings; `grep -rn 'allow(' src/` returns nothing.

### Step 1.3 — Migrate JobQueue to SQLite-backed queue (V2-11)
- **Goal:** Durable, restart-survivable task queue; explicit handling of broadcast `Lagged` events.
- **Method:** Replace the in-memory `JobQueue` with a SQLite-backed queue (new table via a migration in `src/database/migrations/`); wire dequeue/enqueue through `src/experience/queue.rs` / `src/experience/worker_manager/`; handle `Lagged` explicitly (skip + log, or drain). Update the startup verification in `src/bridge/app/initialization.rs` that currently notes "pending full SQLite-backed queue integration."
- **Reference:** `src/experience/queue.rs`, `src/experience/worker_manager/`, `src/bridge/app/initialization.rs` (lines ~155-172), `src/database/migrations/`
- **Verify:** PB; LT; TS queue-related tests; queue survives a process restart in a manual test.

### Step 1.4 — Add loop-health metrics (V2-12)
- **Goal:** The MetricsCollector tracks learning-loop health, not just counters.
- **Method:** Add `loop_latency`, `confidence_drift`, and promotion-throughput (reflection→hypothesis→knowledge) metrics to `src/experience/metrics.rs` and/or the appropriate collector; expose via `get_system_status` MCP tool and the JSON report. Wire timing capture around `AgentLoop::run` and the event-spine handlers.
- **Reference:** `src/experience/metrics.rs`, `src/agent/loop_runner.rs`, `src/experience/integration/event_subscriber/handlers.rs`
- **Verify:** PB; LT (system_status shows new metrics); TS 333/333.

### Step 1.5 — Close the generic MCP tool-execution → experience path (V2-05)
- **Goal:** Every MCP tool execution (not just `record_experience`) emits an `ExperienceRecorded`, fully closing §2.04.
- **Method:** Hook `emit_experience_recorded` into the post-tool-execution dispatch wrapper in `src/bridge/mcp/handlers/` so each `execute_*` records a lightweight experience. Ensure idempotency (the agent loop already publishes once — avoid double-emit).
- **Reference:** `src/bridge/mcp/handlers/`, `src/agent/loop_runner.rs` (lines ~237,251)
- **Verify:** PB; LT (call `store_memory` directly, confirm an experience is recorded); TS.

**End of Stage 1 = clean v0.0.1 baseline. Commit tag marker (optional): `v0.0.1-clean`.**

---

## STAGE 2 — Upgrade existing systems to v0.0.2 (one per phase)

### Step 2.0 — Introduce Data Contract types (Chapter 05)
- **Goal:** A `data_contracts` module defining the canonical cognitive objects every upgraded engine will exchange.
- **Method:** Create `src/data_contracts/` with versioned, self-describing, serializable structs: `Observation`, `ContextPacket`, `MemoryRecord`, `ExperienceRecord` (alias/migrate existing), `Plan`, `Decision`, `ExecutionResult`, `Reflection`, `LearningUpdate`. Each carries id, schema_version, timestamp, producer subsystem, optional parent/correlation id. Keep it types-only initially; wire adapters incrementally in later steps.
- **Reference:** Chapter 05; `src/experience/types/` (existing ExperienceRecord), `src/planner/engine/types.rs` (existing Plan)
- **Verify:** PB 0 warnings; TS; a unit test round-trips each contract through serde.

### Step 2.1 — Upgrade Memory Engine (Chapters 08 & 17)
- **Goal:** Memory as a continuously evolving knowledge network with lifecycle, relationships, confidence, consolidation, pruning.
- **Method:** Extend `src/memory/`: add explicit memory lifecycle states + promotion gate (Working → Candidate → Accepted → Permanent → Archived), memory relationship graph support, confidence field on memories, memory consolidation (merge duplicates, summarize aging low-importance memories into broader summaries, keep high-importance "anchor memories" standalone), pruning policy. Migrate `MemoryRecord` to the data-contract type. Remove the `#![allow(dead_code)]` from `memory/mod.rs` by exercising the new APIs via MCP tools.
- **Reference:** Chapters 08, 17; `src/memory/` (repository, retrieval, pipeline, permanent, working, types, events)
- **Verify:** PB; LT (store/search/list/relationship MCP tools); TS.

### Step 2.2 — Upgrade Knowledge Graph (Chapter 20)
- **Goal:** Concept relationships, graph storage, relationship confidence, entity resolution, graph reasoning, graph-based retrieval.
- **Method:** Extend `src/knowledge/` into a real graph: `knowledge_nodes` + `knowledge_edges` with relationship confidence, an entity-resolution pass (merge "Rust Compiler"/"rustc"), graph traversal queries, and a graph-extraction pipeline (entity detection → relationship extraction → confidence evaluation → graph update → knowledge integration). Migrate to `knowledge_nodes`/`knowledge_edges` tables (migration in `src/database/migrations/`). Expose via MCP tools; delete `knowledge/self_check.rs`.
- **Reference:** Chapter 20; `src/knowledge/` (store, query, types)
- **Verify:** PB; LT (query_knowledge/add_knowledge/global_search + new graph traversal tools); TS.

### Step 2.3 — Upgrade Experience Engine (Chapters 09 & 18)
- **Goal:** Experience records track full operational history, outcome analysis, learning signals, experience relationships; event-driven; workflow learning; success evaluation; confidence updates; reputation system; lessons learned.
- **Method:** Extend `src/experience/`: enrich ExperienceRecord with the full field set (goal, plan_id, result, success, execution_time, cost, confidence_change, tool_usage, lessons, related memories/knowledge/experiences), experience categories (conversation/planning/tool/code/debugging/...), workflow-level evaluation, multi-factor success scoring, confidence-update propagation to memory/relationships/tools. Migrate to the data-contract ExperienceRecord. Delete the experience-* `self_check.rs` files by exercising APIs via MCP tools.
- **Reference:** Chapters 09, 18; `src/experience/` (types, repository, scorer, reputation, reflection, hypothesis, evolution, exploration, integration)
- **Verify:** PB; LT (record_experience/list_experiences/get_insights); TS.

### Step 2.4 — Upgrade Learning Engine (Chapter 10)
- **Goal:** Continuous learning pipeline, pattern discovery, knowledge extraction, skill improvement, confidence updates; LearningItem/Skill/Hypothesis data models.
- **Method:** Extend `src/learning/`: formalize the learning pipeline (reflection → candidate → promotion → consolidation), pattern discovery, skill emergence from repeated successful experience, hypothesis lifecycle, confidence/decay management, generalization over memorization. Delete `learning/self_check.rs` via real MCP-tool traffic.
- **Reference:** Chapter 10; `src/learning/` (pipeline, promotion, lineage, candidates, hypothesis, memory_state, working_memory)
- **Verify:** PB; LT; TS; before/after learning shows performance improvement (Chapter 30.15).

### Step 2.5 — Upgrade Planning Engine (Chapter 11)
- **Goal:** Goal decomposition, task sequencing, dependency resolution, resource/risk/cost/time estimation, dynamic replanning, failure recovery, progress tracking, multi-step reasoning, plan optimization, long-term objective management.
- **Method:** Extend `src/planner/engine/`: richer `decompose_goal`, dependency-aware task graphs, candidate-plan generation + evaluation, dynamic replanning triggers, plan scoring. Migrate Plan to the data-contract type. Delete `planner/self_check.rs` via MCP tools.
- **Reference:** Chapter 11; `src/planner/engine/` (planner, types, actions, replanning)
- **Verify:** PB; LT (create_plan/get_plan/list_plans show real decomposed steps + dependencies); TS.

### Step 2.6 — Upgrade Skills & Workflows (Chapters 13 & 11 Workflow Learning)
- **Goal:** Tool/skill registration, permissions, execution, performance tracking, usage-as-experience, fallback, async/parallel/retry; workflow learning improves action sequences over time.
- **Method:** Extend `src/skills/registry/` (permissions, performance tracking, fallback, async/parallel/retry) and `src/workflows/engine/` (workflow-level learning, workflow confidence, workflow ranking). Remove `#![allow]` from `workflows/mod.rs` + `workflows/engine/mod.rs`. Delete `skills` dead code via MCP traffic.
- **Reference:** Chapter 13; `src/skills/registry/`, `src/workflows/engine/`
- **Verify:** PB; LT (register_skill/discover_skill/execute_skill + workflow tools); TS.

### Step 2.7 — Upgrade World Model (Chapter 14 concepts + 20 graph)
- **Goal:** World model entities/relationships feed the knowledge graph; blockers/dependencies/resources tracked with confidence.
- **Method:** Align `src/world_model/` with the knowledge graph (entities become knowledge_nodes; relationships become edges with confidence); ensure the world model is the "live environment" projection while the knowledge graph is durable. (self_check already removed.)
- **Reference:** `src/world_model/`; Chapter 20
- **Verify:** PB; LT (10 world_model MCP tools); TS.

### Step 2.8 — Upgrade Personality (Chapter 13 in v0.0.1 / fold into Conversation Ch.06 later)
- **Goal:** Personality drives emotional_weight and confidence adjustments, not just response style; presets + adaptation.
- **Method:** Finalize `src/personality/` (traits, emotional weight → confidence, presets, adaptation, decision_making, communication). Ensure no `allow` remains. Personality will later plug into the Conversation Engine's response generation.
- **Reference:** `src/personality/`; v0.0.2 Ch.06 response generation
- **Verify:** PB; LT (6 personality MCP tools); TS.

**End of Stage 2 = all existing systems upgraded to v0.0.2 contracts. Optional tag: `v0.0.2-systems-upgraded`.**

---

## STAGE 3 — Add new systems in logical dependency order

### Step 3.1 — Context Engine (Chapter 07) — THE HEADLINE
- **Goal:** A first-class Context Engine whose single responsibility is "build the smallest possible prompt that still allows the correct answer." Subsystems: ContextManager, WorkingContext, ActiveTaskContext, RetrievalPlanner, MemoryRetriever, ContextCompressor, PromptAssembler, TokenBudget, TopicTracker, RetrievalCache, SlidingWindow.
- **Method:** New `src/context/` module. Implement the 4-level memory hierarchy (L0 live, L1 ~200-token working summary always in prompt, L2 checkpoints ~300-500 tokens retrieved on demand, L3 raw DB unlimited). Implement memory aging (importance scoring; low-importance memories merge into summaries, high-importance "anchor memories" never collapse). Implement Context Policies (task detection → retrieval policy: e.g. "2+2" retrieves nothing; "rename worker.rs" retrieves current task only; "continue the Experience Engine" retrieves project summaries + architecture decisions + current task + related files). Implement per-item context scoring (similarity × recency × importance × confidence → final score; load until TokenBudget exhausted OR score < 0.60). Implement SlidingWindow (FIFO) + Continuous Compaction (collapse oldest message block into rolling summary injected as Memory Context). Implement layered context windows (System/Conversation/Planner/Memory/Experience/Knowledge/Tool/Execution layers). Enforce hard token budget (2048 default) with per-section allocations. Make context construction inspectable (retrieved/rejected memories, scores, token usage, retrieval timing). Wire it as the sole path through which the (future) Conversation Engine talks to Memory/Knowledge/Experience. Expose MCP tools for debugging (`inspect_context`, `get_context_score`).
- **Reference:** Chapter 07 (the entire chapter); the Context Engine design notes embedded in the blueprint (4-level memory, aging, policies, scores, sliding window, compaction).
- **Verify:** PB; LT (new context MCP tools; verify a simple question retrieves nothing, a complex one retrieves ranked context under budget); TS.

### Step 3.2 — Conversation Engine (Chapter 06)
- **Goal:** The orchestration "central nervous system": Input → Understanding → Context Assembly → Reasoning → Planning → Tool Execution → Response Generation → Learning, event-driven.
- **Method:** New `src/conversation/` module (manager, orchestrator, prompts, responses, intent, history, streaming). The Conversation Engine owns orchestration only — it calls the Context Engine, Memory, Experience, Planner, Skills, Tool/MCP, Safety, LLM interface, Response Generator. Publishes events (UserMessageReceived, IntentDetected, MemoryRetrieved, ToolRequested, ToolCompleted, ResponseGenerated, ConversationCompleted, KnowledgeLearned). Absorb the current `src/agent/loop_runner.rs` cognitive loop as one strategy the Conversation Engine can drive. Integrate Personality into response generation. (Works against cloud/external LLM first — no local model dependency yet.)
- **Reference:** Chapter 06; `src/agent/`
- **Verify:** PB; LT (a `converse` MCP tool that runs the full pipeline and returns a context-informed response); TS.

### Step 3.3 — Observation System (Chapter 03/04 entry point)
- **Goal:** Entry point for external information; normalize + classify incoming observations before they enter the cognitive pipeline.
- **Method:** New `src/observation/` module. Normalize heterogeneous inputs (user text, files, APIs, tool outputs, scheduled jobs, background agents) into the `Observation` data contract; classify (data type, priority, source, confidence, relevance, security level, expiration policy; categories: question/instruction/memory-candidate/event/observation/command/tool-output/planner-update/experience-update). Route to downstream systems.
- **Reference:** Chapter 04 §4.3-4.4; Chapter 05 Observation contract
- **Verify:** PB; LT; TS.

### Step 3.4 — Reasoning Engine (Chapters 03/04/05 Decision)
- **Goal:** Integrate available evidence (planner output, working context, retrieved knowledge, experiences, confidence, objectives) into a coherent, explainable Decision; distinct from planning.
- **Method:** New `src/reasoning/` module producing the `Decision` data contract (selected_plan, reason, confidence, alternatives, supporting_memory, supporting_experience, timestamp). Decisions remain explainable after execution.
- **Reference:** Chapter 04 §4.9; Chapter 05 §5.9 Decision
- **Verify:** PB; LT (a `reason`/`explain_decision` MCP tool); TS.

### Step 3.5 — Execution Engine (Chapter 12)
- **Goal:** Isolate execution from reasoning; workflow graphs/DAGs, deterministic pipelines, checkpoints (pause/resume/recover), execution traces, safety enforcement during execution, human-approval gates, experience + learning integration.
- **Method:** New `src/execution/` module (executor, workflow, dispatcher, monitoring, recovery, results). Convert approved Plans/DAGs into observable actions; normalize tool results into `ExecutionResult`; emit execution traces (Plan→Node→Retrieval→Tool→Result→Experience). Enforce permissions, budgets, recursion limits. Human-approval gate for destructive actions.
- **Reference:** Chapter 12; Chapter 05 ExecutionResult contract
- **Verify:** PB; LT; TS.

### Step 3.6 — Security & Trust Architecture (Chapter 25)
- **Goal:** Identity system, permission model, capability security, memory protection, audit system, trust evaluation.
- **Method:** New `src/security/` module. Identity for agents/users/tools; capability-based permissions (tools declare required capabilities; requester must hold them); memory protection (no direct cross-subsystem state mutation); audit log (SQLite `security_events`-style table); trust scoring of sources. Integrate with the Execution Engine's approval gate.
- **Reference:** Chapter 25; Appendix C Security Events
- **Verify:** PB; LT (permission-check MCP tools); TS.

### Step 3.7 — Cognitive Monitoring & Observability (Chapter 27) + Control Plane (Chapter 28)
- **Goal:** Cognitive traces, system telemetry, decision explanations, event monitoring; developer inspection tools.
- **Method:** New `src/observability/` module. Cognitive trace model (follow info through Conversation→Context→Memory→Knowledge→Planning→Execution→Response, recording input/output/decisions/confidence/timing/errors per stage). System metrics (CPU/mem/db/worker/queue/latency/inference time). Decision explanation layer. Expose via MCP tools (`get_cognitive_trace`, `explain_decision`, `get_system_health`) and the JSON report. (This is an expansion of the V2-12 metrics from Stage 1 into a full observability subsystem.)
- **Reference:** Chapters 27, 28 (non-GUI parts)
- **Verify:** PB; LT (trace + health MCP tools); TS.

### Step 3.8 — AI Runtime / Model Manager (Chapter 14 + appendix notes)
- **Goal:** Local, Candle-based model inference for LLM, embeddings, Whisper STT, Piper/Kokoro TTS; Device Manager, Resource Manager, execution scheduler, tokenizer manager, streaming manager. **Full scope, in-build** — this is greenfield work built per the blueprint (no Candle/local-model code exists in the codebase yet), not an upgrade of existing code.
- **Method:** Add `candle-core`/`candle-nn`/`candle-transformers`/`tokenizers` deps to `Cargo.toml`. New `src/ai_runtime/` module (model_manager, device_manager, resource_manager, scheduler, tokenizer_manager, streaming_manager) + `src/models/` directory layout for model metadata. Implement a stable `InferenceProvider` trait so cloud and local are interchangeable, then implement the local Candle-backed provider(s): LLM generation, embedding generation, Whisper speech-to-text, Piper/Kokoro text-to-speech. Centralize embedding generation (used by Memory, Knowledge Graph, Experience, Retrieval). Download/load/cache/validate models. Device selection (CPU/CUDA/Vulkan/Metal). Given the size, this step may itself be split into sub-steps (trait + embedding first, then LLM, then STT, then TTS) following the incremental one-thing-at-a-time rule — but all are in scope and must be completed before Step 3.9 (Multimodal) which depends on them.
- **Reference:** Chapter 14; Appendix A `ai_runtime/` + `models/` layout; blueprint "Odd Notes" (Whisper/Piper/Kokoro via Candle). Note: Candle/Whisper/Piper/Kokoro currently appear ONLY in the blueprint, not in `src/` or `Cargo.toml`.
- **Verify:** PB; LT (an `inference` MCP tool that routes through the runtime; transcribe/synthesize/embed sub-commands); embedding pipeline produces consistent vectors; cloud and local providers interchangeable behind the trait; TS.

### Step 3.9 — Multimodal: Audio Engine + Vision Engine (Chapters in Part VI / appendix)
- **Goal:** STT (Whisper via Candle), TTS (Piper/Kokoro via Candle), audio ingest (WAV/MP3/FLAC/OGG/M4A/MP4/WebM); OCR, image understanding, screenshot analysis, object detection, visual question answering.
- **Method:** New `src/audio/` (input, output, decoder, preprocessing, speech_to_text, text_to_speech, streaming, formats) and `src/vision/` (image_loader, preprocessing, ocr, object_detection, scene_analysis, screenshot, pipeline) modules, both executing models through the AI Runtime. Depends on Step 3.8.
- **Reference:** Appendix A `audio/`, `vision/` layouts; blueprint Odd Notes pipelines
- **Verify:** PB; LT (transcribe/synthesize/ocr MCP tools); TS.

### Step 3.10 — GUI / Dashboard (Chapter 28 GUI parts + appendix)
- **Goal:** Optional, decoupled developer UI: memory viewer, experience viewer, knowledge-graph visualization, cognitive activity monitor, developer mode, task workspace, architecture replay.
- **Method:** Keep GUI fully decoupled from the cognitive runtime (Rust core → API/event stream → frontend). New `src/control_plane/` exposing an API/event stream (the runtime must remain operable via API/CLI/MCP without the GUI). Frontend (desktop app / web dashboard) in a separate crate/dir consuming the event stream.
- **Reference:** Chapter 28 GUI sections; Appendix A `gui/`-equivalent
- **Verify:** Runtime works headless without GUI; GUI connects to the event stream and renders real (not fake) system events.

### Step 3.11 — Self-Improvement / Evolution hardening (Chapter 26)
- **Goal:** Learning-vs-evolution distinction; improvement pipeline; hypothesis system; experimentation; controlled system evolution.
- **Method:** Harden the existing hypothesis/exploration/evolution modules under `src/experience/` into the full Chapter 26 model (hypothesis generation → evidence gathering → confirmed/rejected → confidence change; experimentation framework; controlled evolution with rollback). Promote validated lessons into procedural memory/skills/policies. Knowledge abstraction ladder: Conversation → Experience → Pattern → Skill → Policy → Strategic Knowledge.
- **Reference:** Chapter 26; `src/experience/{hypothesis,exploration,evolution}/`
- **Verify:** PB; LT; TS; a hypothesis lifecycle runs end-to-end.

### Step 3.12 — Background Workers hardening (Chapter 23)
- **Goal:** Worker architecture, task queues (SQLite-backed from Step 1.3), worker supervision, memory/learning/maintenance workers.
- **Method:** Formalize `src/experience/worker_manager/` into a general supervised-worker framework with memory workers, learning workers, and maintenance workers (consolidation, pruning, confidence decay, archival). Supervisor restarts failed workers; queue is durable.
- **Reference:** Chapter 23; `src/experience/worker_manager/`
- **Verify:** PB; LT; TS; workers survive restart.

### Step 3.13 — Configuration / Runtime / Deployment / Testing / Versioning polish (Chapters 29-31, Appendix B)
- **Goal:** Layered config (system/user/runtime), migration strategy, deployment profiles, expanded testing strategy (cognitive evaluation, regression, benchmarks), schema versioning for all new tables.
- **Method:** New `src/config/` (layered config: env vars → config files → runtime overrides → DB-stored learned settings). Add migrations for all new tables (data_contracts, knowledge_nodes/edges, security_events, traces, models, tool_calls, conversations/messages, goals/plans/tasks, executions). Expand `test_suite/` with schema-validation matrix + edge cases + end-to-end learning-loop + performance baselines (the gaps noted in AGENTS.md). Deployment docs + profiles.
- **Reference:** Chapters 29-31; Appendix B schemas
- **Verify:** PB; LT; TS with new coverage; migrations run clean on a fresh DB.

**End of Stage 3 = full v0.0.2 realization. Tag: `v0.0.2`.**

---

# 5. TESTING AND VALIDATION

Every step must pass the three gates before commit/push (per the repo's incremental workflow):

```bash
# 1. Production build — must finish with 0 warnings
cargo build --release -p robot_brain

# 2. Live MCP test — 54/54 tools pass (authoritative live test, cleans DB first)
python3 .agents/live_test/live_test_all.py

# 3. End-to-end test suite — 333/333 (grows as new tools are added), 0 code-quality issues
cd test_suite && cargo build --release && ./target/release/test_suite
```

Additionally, per stage:

- **Stage 1 (finish v0.0.1):** `find src -name self_check.rs` returns empty; `grep -rn 'allow(' src/` returns nothing; queue survives a process restart; system_status shows loop_latency/confidence_drift/promotion_throughput.
- **Stage 2 (upgrade systems):** each upgraded subsystem's MCP tools return correct results live; data-contract types round-trip through serde; knowledge graph traversal returns relationship chains; before/after learning shows measurable improvement (Ch.30.15).
- **Stage 3 (new systems):**
  - Context Engine: a trivial question retrieves nothing; a complex question returns ranked context under the 2048-token budget with inspectable scores.
  - Conversation Engine: a `converse` call runs the full pipeline and returns a context-informed, memory-informed response.
  - Observation/Reasoning/Execution: observations normalize+classify; decisions are explainable; execution traces are complete Plan→Node→Tool→Result→Experience chains.
  - Security: capability-denied actions are blocked and audited.
  - Observability: cognitive traces reconstruct the full request lifecycle.
  - AI Runtime: embedding pipeline produces consistent vectors; an inference call routes through the runtime; cloud and local are interchangeable behind the trait.
  - Multimodal: transcribe/synthesize/ocr return real results via Candle.
  - GUI: runtime is fully headless-operable; GUI renders only real system events (never fake "thinking" animations — Ch.XX.14 rules).
  - Self-improvement: a hypothesis lifecycle runs confirmed→confidence-increase / rejected→confidence-decrease end-to-end.
  - Workers: supervised workers restart on failure; durable queue survives restart.
  - Config/Migration: fresh-DB migration runs clean; test_suite adds schema-validation + edge-case + e2e-learning + perf-baseline coverage.

**Definition of Done for v0.0.2:**
- All 32 blueprint chapters + appendices have a corresponding implemented module or documented deferral.
- The cognitive pipeline (Observe → Understand → Retrieve → Plan → Reason → Act → Reflect → Learn) runs end-to-end through Context/Conversation engines, not just the legacy agent loop.
- Memory, Experience, Knowledge, and Context are four independent systems communicating through Data Contracts.
- Context Engine enforces token budgets and retrieval policies; context construction is inspectable.
- 0 cargo warnings, 0 code-quality issues, all MCP tools pass live, test-suite green and expanded.
- Local-first: the entire cognitive architecture operates without cloud dependency; cloud/local models are interchangeable behind the AI Runtime trait.
