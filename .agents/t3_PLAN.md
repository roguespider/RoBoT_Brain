# TIER 3 PLAN — Reach v0.0.2.1 (merged with gap_analysis_v0.0.2.1.md)

## Purpose
Build the missing subsystems from the v0.0.2.1 architecture (`robot_architecture/v0.0.2.1/` 00.md-33.md + FINAL_ARCHITECTURE_SPEC.md + appendices A-E) in dependency-first order. Every chapter, appendix, and runtime surface must be covered.

Source of truth for gaps: `.agents/gap_analysis_v0.0.2.1.md` (created 2026-09-15; method: one architecture chapter at a time; spec functions -> current module -> gap -> wiring).

## Execution rule (MANDATORY — AGENTS.md Incremental Workflow)
Treat each bullet as a single 10-15 minute increment: ONE tiny code change, ONE verification run (`bash .agents/scripts/make.sh gate`), then stop. Never batch unrelated changes.

---

## 0. Architecture-wide contract (Chapters 01-05 + Appendices A-E base)
Set the invariants that every v0.0.2.1 chapter, appendix, and runtime surface must obey. Per gap analysis: no centralized `design_principles/` enforcement module exists; principles are scattered across `agent/safety_gate/`, `bridge/app/state.rs`, `experience/` provenance fields.

- [ ] **T3-01** Write ownership boundaries (interaction, control plane, cognition, state, action, platform). Gap: no `principles/enforcer.rs`. Wiring: `safety_gate/mod.rs` -> `agent/loop_runner.rs` -> `bridge/app/state.rs`.
- [ ] **T3-02** Write lifecycle boundaries (ephemeral, session, working, persistent operational, persistent knowledge, archived). Gap: partial in `agent/context.rs`, `memory/` flat. Wiring: `context_engine/` (ch 07) -> `memory_hierarchy/` (ch 14) -> `database/`.
- [ ] **T3-03** Write identity/correlation rules for installations and sessions. Gap: `bridge/acp/` has correlation but no centralized identity module.
- [ ] **T3-04** Write identity/correlation rules for events, plans, actions, executions, tools, learning changes. Gap: `appendix-c.md` defines events but no `event_router/` routes them.
- [ ] **T3-05** Write provenance/evidence rules for durable information. Gap: `experience/` has provenance fields but no `design_decisions/` enforcement.
- [ ] **T3-06** Write confidence rules (separate from source quality, recency, contradiction, uncertainty, applicability). Gap: `confidence_system/` (ch 19) MISSING entirely.
- [ ] **T3-07** Write model-independence rule (no fixed model/provider dependency). Gap: `models/` has `LocalProvider` but no abstraction enforcing independence.
- [ ] **T3-08** Write controlled-effects rule (execution and tools remain authorized and traceable). Gap: `agent/safety_gate/` partial; `security/` (ch 25) MISSING.
- [ ] **T3-09** Write observability, failure visibility, versioned evolution, human control, compatibility rules. Gap: `observability/` (ch 27) MISSING; `developer_interface/` (ch 28) MISSING.

---

## 1. Foundation chapters 01-05 (Vision, Principles, Overview, Data Flow, Data Contracts)
Per gap analysis: philosophy embedded in `AGENTS.md`; contracts in `data_contracts/` (13 files); no executable `system_overview/` or `contract_validator/`.

- [ ] **T3-10** Summarize vision/philosophy (persistent cognitive architecture, continuity, long-term improvement). Wiring: feeds `bridge/app/initialization/core.rs` and `data_contracts/`.
- [ ] **T3-11** Summarize core design principles (modularity, explainability, memory-first, event-driven, confidence, controlled evolution). Wiring: `principles/enforcer.rs` (new) validates actions before execution.
- [ ] **T3-12** Summarize high-level system overview (subsystem relationships, cognitive pipeline). Wiring: `engines.rs` -> `execution/` (new, ch 12) -> `tool_engine/` (new, ch 13) -> `memory/` + `experience/`.
- [ ] **T3-13** Summarize data-flow (input processing, internal pipelines, output generation, system boundaries). Wiring: `data_contracts/` -> `event_router/` (new) -> `experience/` + `memory/` + `learning/`.
- [ ] **T3-14** Summarize data-contracts (shared structures, event contracts, API boundaries, serialization, interoperability). Wiring: `data_contracts/` -> `database/` (DB schema must match) -> `bridge/app/initialization/db.rs`.
- [ ] **T3-15** Add shared contract metadata: `version`, `timestamp`. Gap: contracts exist but metadata fields not enforced at runtime.
- [ ] **T3-16** Add shared contract metadata: `source`, `correlation`, `provenance`, `confidence`. Gap: `experience/` provenance partial; `knowledge/` confidence partial; no centralized enforcement.

---

## 2. Conversation Engine (Chapter 06) — TC-01 to TC-06
Per gap analysis: no dedicated `conversation_engine/`; logic split across `bridge/acp/` and `agent/context.rs`. Must preserve turn order and correlation IDs per `appendix-c.md`.

- [ ] **TC-01** `ConversationSession` struct (`session_id`, `turn_number`, `user_messages[]`, `agent_responses[]`, `learnings[]`). In-memory session tracker.
- [ ] **TC-02** `ConversationEngine` (`start_session()`, `add_turn()`, `get_session()`).
- [ ] **TC-03** `converse` MCP tool (entry point: `message` -> `run_agent_goal()` -> turn stored -> response returned). This is the user-facing surface.
- [ ] **TC-04** Learning extraction (`extract_learnings` from `(user_message, agent_response)`). Keyword-based first; upgrade to LLM later.
- [ ] **TC-05** Wire `extract_learnings` into `converse` (store via `store_memory`, `memory_type` = `preference` or `fact`).
- [ ] **TC-06** Conversation persistence (`conversation_turns` SQLite table: `session_id, turn_number, role, content, timestamp`).

**Done when:** `converse` works end-to-end. Gate green.

---

## 3. Context Engine (Chapter 07) — T3-27 to T3-36
Per gap analysis: partial in `agent/context.rs`, `bridge/mcp/context.rs`. No full lifecycle (create -> retrieve -> update -> expire -> archive). Context expiration must trigger `memory/` archive.

- [ ] **T3-27** Active session-state model.
- [ ] **T3-28** Working-memory model.
- [ ] **T3-29** Current retrieval-set model.
- [ ] **T3-30** Context-compression logic (long/repetitive context).
- [ ] **T3-31** Topic-tracking logic.
- [ ] **T3-32** Relevant-information selection logic.
- [ ] **T3-33** Token-budget enforcement.
- [ ] **T3-34** Policy-budget enforcement.
- [ ] **T3-35** Context-construction step.
- [ ] **T3-36** Context-assembly step for prompts.

Wiring: `context_engine/` (new) -> `memory/` (store) -> `retrieval_pipeline/` (ch 16, new) -> `prompt_construction/` (ch 17, new).

---

## 4. Memory Engine (Chapter 08) — T3-37 to T3-49
Per gap analysis: `memory/` (16 files) partial; no explicit `semantic_memory/` vs `procedural_memory/` split; forgetting mechanism missing. Must use `confidence_system/` (ch 19) for retention decisions.

- [ ] **T3-37** Short-term memory boundary.
- [ ] **T3-38** Long-term memory boundary.
- [ ] **T3-39** Memory-create operations.
- [ ] **T3-40** Memory-promote operations.
- [ ] **T3-41** Memory-demote operations.
- [ ] **T3-42** Memory-archive operations.
- [ ] **T3-43** Memory-retrieve operations.
- [ ] **T3-44** Working-memory records.
- [ ] **T3-45** Episodic-memory records.
- [ ] **T3-46** Semantic-memory records.
- [ ] **T3-47** Procedural-memory records.
- [ ] **T3-48** Experience-linked-memory records.
- [ ] **T3-49** Promotion rules (working/episodic -> durable knowledge) using `confidence_system/`.

Wiring: `memory/` -> `memory_hierarchy/` (ch 14, new) -> `database/` (hierarchy tables). `forgetting_policy.rs` (new) uses `confidence_system/`.

---

## 5. Experience Engine (Chapter 09) — T3-50 to T3-57
Per gap analysis: `experience/` (100 files) very complete. Minimal gap: experience-to-learning promotion manual; must verify uses `confidence_system/` (missing).

- [ ] **T3-50** Experience storage (execution history).
- [ ] **T3-51** Experience storage (outcomes).
- [ ] **T3-52** Outcome tracking (success/failure).
- [ ] **T3-53** Lesson capture (successes).
- [ ] **T3-54** Lesson capture (failures).
- [ ] **T3-55** Failure-analysis storage.
- [ ] **T3-56** Experience-processing flow (actions -> learning).
- [ ] **T3-57** Related-experience links (pattern analysis).

Wiring: `experience/` -> `learning/` (23 files) -> `cooboploop/` (loop). Verify promotion uses `confidence_system/`.

---

## 6. Learning Engine (Chapter 10) — T3-58 to T3-66
Per gap analysis: `learning/` (23 files) exists but lacks `strategic_learning/` (ch 18) integration; no `learning_update/` persistence to DB.

- [ ] **T3-58** Reflection stage in pipeline.
- [ ] **T3-59** Candidate stage in pipeline.
- [ ] **T3-60** Evaluation stage in pipeline.
- [ ] **T3-61** Promotion stage in pipeline.
- [ ] **T3-62** Consolidation stage in pipeline.
- [ ] **T3-63** Pattern discovery (repeated successful structures).
- [ ] **T3-64** Knowledge extraction (durable knowledge).
- [ ] **T3-65** Skill-improvement outputs.
- [ ] **T3-66** Confidence updates (evidence + repeated behavior).

Wiring: `learning/` -> `database/` (new `learning_updates` schema) -> `strategic_learning/` (new, ch 18). Need `learning_update_persist.rs`.

---

## 7. Planning Engine (Chapter 11) — T3-67 to T3-73
Per gap analysis: `planner/` (7 files) + `workflows/` (12 files) + `cooboploop/` exist. Planning engine not integrated with `execution_engine/` (ch 12, missing). Plans stored but not executed.

- [ ] **T3-67** Goal-creation records.
- [ ] **T3-68** Goal-validation rules.
- [ ] **T3-69** Task decomposition (smaller work items).
- [ ] **T3-70** Planning-strategy selection.
- [ ] **T3-71** Workflow-generation from plans.
- [ ] **T3-72** Plan-evaluation scoring.
- [ ] **T3-73** Feedback-driven replanning.

Wiring: `planner/` -> `execution/` (new, ch 12) -> `tool_engine/` (new, ch 13). Plan steps become `execution_request/` objects.

---

## 8. Execution Engine (Chapter 12) — T3-74 to T3-78 + function-level wiring
**CRITICAL GAP — 1 file only (`execution/` stub). ENTIRE ENGINE MISSING (12.1-12.41 unimplemented).** Per gap analysis function-level wiring table:

- [ ] **T3-74** Controlled action-execution path (`ExecutionRequest` 12.5 -> `ExecutionLifecycle` 12.6 -> `ExecutionGraph` 12.7 -> `ActionNode` 12.8 -> `ActionTypes` 12.9).
- [ ] **T3-75** Tool usage as distinct execution concern (`ToolExecution` 12.17 -> `tool_engine/` ch 13).
- [ ] **T3-76** External interaction records (`MCPIntegration` 12.18 -> `bridge/mcp/handler.rs`).
- [ ] **T3-77** Result handling and normalization (`ResultNormalization` 12.19 -> `experience/`).
- [ ] **T3-78** Error-recovery behavior (`ErrorHandling/Recovery` 12.23 -> `agent/safety_gate/`).

**Function-level wiring (must be done one function at a time per AGENTS.md):**
1. `ExecutionRequest` (12.5) -> `planner/` -> `execution/` -> `database/`
2. `ExecutionLifecycle` (12.6) -> `execution/` -> `agent/loop_runner.rs` -> `database/`
3. `ExecutionGraph` (12.7) -> `execution/` -> `workflows/` -> `database/`
4. `ActionNode` (12.8) -> `execution/` -> `tools/` -> `execution/`
5. `ExecutorAbstraction` (12.10) -> `execution/` -> `bridge/mcp/`
6. `ExecutionContext` (12.11) -> `execution/` -> `agent/context.rs` -> `context_engine/`
7. `ExecutionState` (12.12) -> `execution/` -> `database/` (state table)
8. `ExecutionBudget` (12.13) -> `execution/` -> `cooboploop/` -> `database/`
9. `Scheduler` (12.14) -> `execution/` -> `bridge/app/scheduler.rs` -> `workers.rs`
10. `ParallelExecution` (12.15) -> `execution/` -> `workers.rs` -> `database/`
11. `ResourceManager` (12.16) -> `execution/` -> `database/` (resource table)
12. `ToolExecution` (12.17) -> `execution/` -> `tool_engine/` (build ch 13 after 12.17)
13. `ResultNormalization` (12.19) -> `execution/` -> `experience/`
14. `ProgressTracking` (12.20) -> `execution/` -> `bridge/app/state.rs`
15. `Cancellation` (12.21) -> `execution/` -> `agent/safety_gate/rollback.rs`
16. `RetryPolicy` (12.22) -> `execution/` (new) -> `database/`
17. `Checkpointing` (12.24) -> `execution/` -> `database/` (checkpoint table)
18. `LongRunningJobs` (12.25) -> `execution/` -> `bridge/app/initialization/job_queue.rs`
19. `Observability` (12.26) -> `execution/` -> `observability/` (new, ch 27)
20. `ExecutionTrace` (12.27) -> `execution/` -> `database/` (trace table) -> `observability/`
21. `SafetyEnforcement` (12.28) -> `execution/` -> `agent/safety_gate/` -> `security/` (new, ch 25)
22. `HumanApproval` (12.29) -> `execution/` -> `developer_interface/` (new, ch 28)
23. `Idempotency/SideEffects` (12.30) -> `execution/` (new) -> `database/`
24. `ResultVerification` (12.31) -> `execution/` (new) -> `database/`
25. `ExperienceIntegration` (12.32) -> `execution/` -> `experience/`
26. `LearningIntegration` (12.33) -> `execution/` -> `learning/`
27. `Execution/PlanningFeedback` (12.34) -> `execution/` -> `planner/`
28. `ExecutionDeterminism` (12.35) -> `execution/` (new) -> `design_decisions/` (new)
29. `ExecutionReproducibility` (12.36) -> `execution/` -> `database/` (repro log)
30. `ExecutionStateMachine` (12.37) -> `execution/` (new) -> `database/`

---

## 9. Tool Engine (Chapter 13) — T3-79 to T3-83 + function-level wiring
Per gap analysis: `tools/` (4 files) very minimal. No `tool_registry/`, `tool_capability/`, `tool_invocation/`, `tool_audit/`, `tool_security/`. Must build after `ExecutionEngine` 12.17 (`ToolExecution`).

- [ ] **T3-79** Tool-capability records (`ToolCapability` -> `data_contracts/` -> `execution/`).
- [ ] **T3-80** Tool-registration flow (`ToolRegistry` -> `bridge/acp/registry.rs` -> `data_contracts/`).
- [ ] **T3-81** Tool-permission checks (`ToolSecurity` -> `security/` ch 25 -> `agent/safety_gate/`).
- [ ] **T3-82** Tool-execution flow (`ToolInvocation` -> `execution/` -> `bridge/mcp/`).
- [ ] **T3-83** External-capability integration rules (`ExternalAPIIntegration` -> `execution/` -> `bridge/app/initialization/` config).

**Function-level wiring (ch 13):**
- `ToolRegistry` -> `bridge/acp/registry.rs` -> `data_contracts/`
- `ToolCapability` -> `data_contracts/` -> `execution/`
- `ToolInvocation` -> `execution/` -> `bridge/mcp/`
- `ToolResultHandling` -> `execution/` -> `experience/`
- `ToolSecurity` -> `security/` (new) -> `agent/safety_gate/`
- `ToolAudit` -> `observability/` (new) -> `database/`
- `MCPToolIntegration` -> `bridge/mcp/handler.rs`
- `LocalModuleIntegration` -> `skills/` + `workflows/` -> `execution/`
- `ExternalAPIIntegration` -> `bridge/app/initialization/` (config) -> `execution/`
- `ToolLifecycle` -> `execution/` (register -> invoke -> result -> audit -> retire) -> `database/` (lifecycle table)

---

## 10. AI Runtime and Model Integration (Chapter 14) — T3-84 to T3-90
Per gap analysis: `models/` (1 file) has `LocalProvider` but no abstraction enforcing independence. `AI Runtime` (Candle) is last per architecture.

- [ ] **T3-84** Local-model runtime abstraction (`InferenceProvider` trait already exists; need runtime manager).
- [ ] **T3-85** Cloud-model runtime abstraction.
- [ ] **T3-86** Model-routing by capability (`Capability` enum exists in `models/`).
- [ ] **T3-87** Inference context handling (`InferenceContext` exists).
- [ ] **T3-88** Inference scheduling.
- [ ] **T3-89** Inference validation (`validate_response` exists but stub-level).
- [ ] **T3-90** Model selection management (`ModelSelector` exists).

Wiring: `models/` -> `execution/` (inference calls) -> `database/` (model usage table per `appendix-b.md`). Must enforce model-independence (T3-07).

---

## 11. Agent Communication Architecture (Chapter 15) — T3-91 to T3-94
Per gap analysis: `bridge/acp/` (message, agent, registry, router, system_agent) exists; `agent/` has context/decision/loop_runner. No dedicated `agent_protocol/` module.

- [ ] **T3-91** Agent protocol model (`ACP` message types, routing rules).
- [ ] **T3-92** MCP integration boundaries (`bridge/mcp/handler.rs` -> `execution/` -> `agent/`).
- [ ] **T3-93** ACP integration boundaries (`bridge/acp/` -> `agent/loop_runner.rs`).
- [ ] **T3-94** Internal communication rules (subsystem messages via `event_router/` new).

Wiring: `agent/loop_runner.rs` -> `execution/` (ch 12) -> `bridge/acp/` + `bridge/mcp/`. Must preserve correlation IDs per `appendix-c.md`.

---

## 12. Cognitive Coordination Layer (Chapter 16) — T3-95 to T3-98
Per gap analysis: no `cognitive_coordination/` module. Must orchestrate subsystems without collapsing boundaries.

- [ ] **T3-95** Subsystem-coordination rules (who calls whom, in what order).
- [ ] **T3-96** Event communication rules (`event_router/` new -> `experience/` + `memory/` + `learning/` + `execution/` + `observability/`).
- [ ] **T3-97** Decision-routing rules (`agent/loop_runner.rs` -> `planner/` -> `execution/` -> `tool_engine/`).
- [ ] **T3-98** Top-level orchestration rules for cognitive pipeline (`agent/loop_runner.rs` -> `context_engine/` -> `retrieval_pipeline/` -> `prompt_construction/`).

Wiring: `agent/loop_runner.rs` is the coordinator; it must call `execution/` (new), `retrieval_pipeline/` (new), `prompt_construction/` (new), `context_engine/` (new), `memory_hierarchy/` (new), `confidence_system/` (new), `observability/` (new) in sequence.

---

## 13. Memory and Knowledge Systems (Chapters 17-20) — T3-99 to T3-112
Per gap analysis: `memory/` partial; `knowledge/` (7 files) basic; `world_model/` (3 files) basic. `memory_hierarchy/` (ch 14) MISSING; `retrieval_pipeline/` (ch 16) MISSING; `confidence_system/` (ch 19) MISSING.

- [ ] **T3-99** Working-memory promotion rules (`memory_hierarchy/` new -> `memory/` -> `database/`).
- [ ] **T3-100** Permanent-memory retention rules (`database/` persistence + `confidence_system/` retention thresholds).
- [ ] **T3-101** Archived-memory retention rules (`database/` archive table + `forgetting_policy/`).
- [ ] **T3-102** Experience-to-outcome links (`experience/` -> `database/` experience table).
- [ ] **T3-103** Experience-to-learning-signal links (`experience/` -> `learning/` -> `database/` learning_updates).
- [ ] **T3-104** Confidence scoring for knowledge (`confidence_system/` new -> `knowledge/` -> `database/` confidence table).
- [ ] **T3-105** Confidence scoring for skills (`confidence_system/` -> `skills/` -> `database/`).
- [ ] **T3-106** Confidence scoring for relationships (`confidence_system/` -> `knowledge/` graph edges).
- [ ] **T3-107** Confidence scoring for workflows (`confidence_system/` -> `workflows/` -> `database/`).
- [ ] **T3-108** Knowledge-graph concept relationships (`knowledge/` -> `world_model/` -> `database/` graph nodes).
- [ ] **T3-109** Knowledge-graph storage (`database/` graph tables per `appendix-b.md`).
- [ ] **T3-110** Knowledge-graph confidence-bearing edges (`confidence_system/` -> `knowledge/` relationships).
- [ ] **T3-111** Knowledge discovery queries (`retrieval_pipeline/` new -> `knowledge/` -> `database/`).
- [ ] **T3-112** Knowledge-promotion rules (evidence + provenance preservation -> `database/` promotion log).

---

## 14. Storage, Database, and Workers (Chapters 21-23) — T3-113 to T3-126
Per gap analysis: `database/` (19 files) partial; `appendix-b.md` defines schemas but tables for execution (12), tool audit (13), context lifecycle (15), retrieval (16), confidence (19), storage (21), security (25), observability (27), developer interface (28), configuration (29), testing (30) missing. `background_workers/` partial (`workers.rs`, `scheduler.rs`).

- [ ] **T3-113** Durable-persistence architecture (`storage_architecture/` new -> `database/` -> `memory/` + `experience/` + `knowledge/`).
- [ ] **T3-114** Data-organization rules (`database/` schema organization per `appendix-b.md`).
- [ ] **T3-115** Backup strategy rules (`database/` backup + `deployment/` ch 31 rollback).
- [ ] **T3-116** SQLite architecture rules (`database/sqlite.rs` -> `database/queries/`).
- [ ] **T3-117** Schema design rules (`appendix-b.md` -> `database/models/` -> `database/queries/`).
- [ ] **T3-118** Indexing rules (`database/` index definitions for new tables).
- [ ] **T3-119** Migration strategy rules (`database/migrations/` new -> `database/init/`).
- [ ] **T3-120** Data-integrity rules (`database/` constraints + `execution/` idempotency 12.30).
- [ ] **T3-121** Worker architecture (`background_workers/` new -> `bridge/app/initialization/workers.rs` -> `execution/` long-running jobs 12.25).
- [ ] **T3-122** Task queue handling (`background_workers/` -> `execution/` -> `database/` task queue table).
- [ ] **T3-123** Worker supervision (`background_workers/` -> `observability/` ch 27 -> `database/` worker status).
- [ ] **T3-124** Memory-worker behavior (`background_workers/` -> `memory/` consolidation + `memory_hierarchy/` promotion).
- [ ] **T3-125** Learning-worker behavior (`background_workers/` -> `learning/` pipeline + `database/` learning_updates).
- [ ] **T3-126** Maintenance-worker behavior (`background_workers/` -> `database/` cleanup + `memory/` archive + `knowledge/` graph maintenance).

---

## 15. Governance, Safety, and Evolution (Chapters 24-27) — T3-127 to T3-139
Per gap analysis: `security/` (ch 25) MISSING; `self_improvement/` (ch 26) MISSING; `observability/` (ch 27) MISSING; `developer_interface/` (ch 28) MISSING. `agent/safety_gate/` partial (rollback, sandbox, hallucination). `agreement/` (ch 24) MISSING.

- [ ] **T3-127** AI contributor roles and rules (`agreement/` new -> `agent/loop_runner.rs` -> `developer_interface/` ch 28).
- [ ] **T3-128** Contribution standards and human-approval boundaries (`agreement/` -> `developer_interface/` approval gate -> `execution/` 12.29).
- [ ] **T3-129** Identity and permission rules (`security/` new -> `agent/safety_gate/` -> `database/` identity table).
- [ ] **T3-130** Capability-security rules (`security/` -> `tool_engine/` ch 13 -> `execution/` 12.28).
- [ ] **T3-131** Memory-protection rules (`security/` -> `memory/` -> `database/` encrypted fields).
- [ ] **T3-132** Audit rules (`security/` -> `observability/` ch 27 -> `database/` audit table).
- [ ] **T3-133** Trust-evaluation rules (`security/` -> `confidence_system/` ch 19 -> `database/` trust table).
- [ ] **T3-134** Learning-vs-evolution distinction (`self_improvement/` new -> `learning/` ch 10 -> `execution/` feedback 12.34).
- [ ] **T3-135** Hypothesis system (`self_improvement/` -> `experience/` hypothesis -> `database/` hypothesis table).
- [ ] **T3-136** Experimentation and controlled-change rules (`self_improvement/` -> `agent/safety_gate/` rollback -> `execution/` 12.30 idempotency).
- [ ] **T3-137** Tracing and telemetry rules (`observability/` new -> `execution/` 12.26-12.27 -> `database/` metrics/log tables).
- [ ] **T3-138** Explanation and event-monitoring rules (`observability/` -> `developer_interface/` ch 28 dashboards -> `database/` event tables per `appendix-c.md`).
- [ ] **T3-139** Debugger visibility for architectural evidence (`observability/` -> `developer_interface/` ch 28 -> `execution/` trace 12.27).

---

## 16. Interfaces, Runtime Management, Testing, and Deployment (Chapters 28-31) — T3-140 to T3-154
Per gap analysis: `developer_interface/` (ch 28) MISSING; `configuration/` (ch 29) MISSING; `testing/` (ch 30) PARTIAL (test_suite2 external); `deployment/` (ch 31) MISSING.

- [ ] **T3-140** Developer Interface inspection rules (`developer_interface/` new -> `execution/` state -> `database/` state table).
- [ ] **T3-141** Developer Interface command rules (`developer_interface/` -> `agent/loop_runner.rs` -> `execution/` commands).
- [ ] **T3-142** Safe-mutation and recovery rules (`developer_interface/` -> `agent/safety_gate/` rollback -> `database/` recovery log).
- [ ] **T3-143** Memory-management tools through control plane (`developer_interface/` -> `memory/` -> `memory_hierarchy/` ch 14).
- [ ] **T3-144** Worker-control tools through control plane (`developer_interface/` -> `background_workers/` ch 23 -> `execution/` 12.25).
- [ ] **T3-145** Debugging interfaces for trace inspection (`developer_interface/` -> `observability/` ch 27 -> `execution/` 12.27).
- [ ] **T3-146** Layered configuration precedence (`configuration/` new -> `database/` config table -> `execution/` budget/settings).
- [ ] **T3-147** Secrets and profile management (`configuration/` -> `security/` ch 25 -> `database/` secrets table).
- [ ] **T3-148** Runtime override handling (`configuration/` -> `execution/` -> `cooboploop/` settings).
- [ ] **T3-149** Unit and contract testing layers (`testing/` new -> `execution/` verification 12.31 -> `tool_engine/` capability validation).
- [ ] **T3-150** Integration and persistence testing layers (`testing/` -> `database/` test fixtures -> `memory/` + `experience/` + `knowledge/`).
- [ ] **T3-151** Recovery, event, security, regression testing layers (`testing/` -> `agent/safety_gate/` -> `security/` ch 25 -> `database/` regression log).
- [ ] **T3-152** Installation and startup deployment rules (`deployment/` new -> `configuration/` ch 29 -> `database/` deployment log).
- [ ] **T3-153** Upgrade, rollback, recovery deployment rules (`deployment/` -> `configuration/` -> `agent/safety_gate/` rollback -> `database/` rollback log).
- [ ] **T3-154** Versioned release validation (`deployment/` -> `testing/` -> `database/` version table).

---

## 17. Future Expansion (Chapters 32-33) — T3-155 to T3-156
Per gap analysis: `future_expansion/` MISSING; `roadmap/` MISSING. `robot_architecture/v0.0.2.1/32.md` and `33.md` documented.

- [ ] **T3-155** Stable-contract admission rule for future capabilities (`future_expansion/` new -> `developer_interface/` ch 28 -> `configuration/` ch 29 feature flags).
- [ ] **T3-156** Architectural-gate review and roadmap process (`roadmap/` new -> `cooboploop/` objective tracking -> `developer_interface/` milestone UI).

---

## 18. Appendices and Quarantine Material — T3-157 to T3-170
Per gap analysis: `appendix-a.md` (directory structure) needs ~15 new top-level dirs; `appendix-b.md` (database schemas) needs tables for ch 12-23, 25-30; `appendix-c.md` (events) needs `event_router/`; `appendix-d.md` (design decisions) needs `design_decisions/`; `appendix-e.md` (development guidelines) needs `development_guidelines/`.

- [ ] **T3-157** Appendix A directory-ownership rules (`src/` -> new dirs: `execution/`, `tool_engine/`, `retrieval_pipeline/`, `prompt_construction/`, `context_lifecycle/`, `memory_hierarchy/`, `confidence_system/`, `storage_architecture/`, `security/`, `observability/`, `developer_interface/`, `configuration/`, `testing/`, `deployment/`, `future_expansion/`).
- [ ] **T3-158** Appendix A source-tree coverage (major folders mapped to architecture chapters).
- [ ] **T3-159** Appendix B schema-domain coverage (system metadata, memory, knowledge, experience, learning, conversation, planning, execution, models, tools, tracing, diagnostics, configuration, history).
- [ ] **T3-160** Appendix B schema-versioning and migration discipline (`database/migrations/` new -> `database/init/` -> `appendix-b.md`).
- [ ] **T3-161** Appendix C event identity and versioning rules (`event_router/` new -> `data_contracts/` event contracts -> `appendix-c.md`).
- [ ] **T3-162** Appendix C event payload, metadata, confidence rules (`event_router/` -> `experience/` + `memory/` + `learning/` + `execution/` + `observability/`).
- [ ] **T3-163** Appendix C event lifecycle rules (`event_router/` -> `database/` event table -> `appendix-c.md`).
- [ ] **T3-164** Appendix D decision-record rules (`design_decisions/` new -> `agent/loop_runner.rs` -> `execution/` determinism 12.35).
- [ ] **T3-165** Appendix D supersession and status tracking (`design_decisions/` -> `database/` decision table -> `agent/safety_gate/` rollback).
- [ ] **T3-166** Appendix E architecture-first development rules (`development_guidelines/` new -> `cli/` lint/check -> `testing/` ch 30).
- [ ] **T3-167** Appendix E modularity and interface-before-implementation rules (`development_guidelines/` -> `data_contracts/` -> `execution/` -> `tool_engine/`).
- [ ] **T3-168** Appendix E model-replaceability and AI runtime rules (`development_guidelines/` -> `models/` -> `execution/` inference -> `database/` model usage).
- [ ] **T3-169** Appendix E review-discipline rules (`development_guidelines/` -> `agent/loop_runner.rs` -> `developer_interface/` ch 28 -> `testing/` ch 30).
- [ ] **T3-170** Keep `odd-notes.md` explicitly non-normative (quarantine material; not part of architecture spec).

---

## Critical Path Wiring Summary (from gap_analysis_v0.0.2.1.md)
These six paths must be completed in dependency order. Each path is a chain of new modules feeding existing ones.

**Path 1 (Execution):** `planner/` -> `execution/` (new, ch 12) -> `tool_engine/` (new, ch 13) -> `database/` (new tables) -> `experience/` + `learning/` -> `cooboploop/`

**Path 2 (Memory/Context):** `context_engine/` (ch 07) -> `context_lifecycle/` (ch 15, new) -> `memory/` -> `memory_hierarchy/` (ch 14, new) -> `retrieval_pipeline/` (ch 16, new) -> `prompt_construction/` (ch 17, new) -> `agent/loop_runner.rs`

**Path 3 (Confidence/Knowledge):** `confidence_system/` (ch 19, new) -> `knowledge/` (ch 20) -> `retrieval_pipeline/` (ch 16, new) -> `execution/` (budget/approval thresholds)

**Path 4 (Governance):** `security/` (ch 25, new) -> `agent/safety_gate/` -> `execution/` (12.28) -> `developer_interface/` (ch 28, new, approval) -> `self_improvement/` (ch 26, new)

**Path 5 (Observability):** `execution/` (12.26-12.27) + `tool_engine/` (13.6) -> `observability/` (ch 27, new) -> `database/` (metrics/log tables) -> `developer_interface/` (ch 28, dashboards)

**Path 6 (Configuration/Deployment):** `configuration/` (ch 29, new) -> `execution/` (budget/settings) -> `deployment/` (ch 31, new) -> `developer_interface/` (ch 28)

---

## Function-Level Wiring — Execution Engine (ch 12) — Deep Analysis
Every sub-section 12.1-12.41 must be covered. Per gap analysis table:

| Spec Function (12.md) | Status in Plan | Wiring To / From | Verification Method |
|---|---|---|---|
| 12.5 Execution Request | T3-74 | `planner/` -> `execution/` -> `database/` | `execution_request_from_plan()` exists; must expand to full `ExecutionRequest` with validation |
| 12.6 Execution Lifecycle | T3-74 (part of) | `execution/` -> `agent/loop_runner.rs` -> `database/` | State machine must be implemented |
| 12.7 Execution Graph | T3-74 (part of) | `execution/` -> `workflows/` -> `database/` | Graph of actions must connect to workflow engine |
| 12.8 Action Node | T3-74 (part of) | `execution/` -> `tools/` -> `execution/` | Action = tool call or local module |
| 12.9 Action Types | T3-74 (part of) | `execution/` -> `skills/` + `workflows/` + `bridge/mcp/` | Must support all three action sources |
| 12.10 Executor Abstraction | T3-74 (part of) | `execution/` -> `bridge/mcp/` | Must abstract over MCP and local execution |
| 12.11 Execution Context | T3-74 (part of) | `execution/` -> `agent/context.rs` -> `context_engine/` | Context feeds execution; must use `context_engine/` (new) |
| 12.12 Execution State | T3-74 (part of) | `execution/` -> `database/` (state table) | Must persist to DB |
| 12.13 Execution Budget | T3-74 (part of) | `execution/` -> `cooboploop/` -> `database/` | Budget from loop config |
| 12.14 Scheduler | T3-74 (part of) | `execution/` -> `scheduler.rs` -> `workers.rs` | Existing scheduler must delegate |
| 12.15 Parallel Execution | T3-74 (part of) | `execution/` -> `workers.rs` -> `database/` | Must use worker pool |
| 12.16 Resource Manager | T3-74 (part of) | `execution/` -> `database/` (resource table) | Resource tracking table missing |
| 12.17 Tool Execution | T3-75 | `execution/` -> `tool_engine/` (new, ch 13) | Must build `tool_engine/` first |
| 12.18 MCP Integration | T3-76 | `execution/` -> `bridge/mcp/handler.rs` | Partial; must wire fully |
| 12.19 Result Normalization | T3-77 | `execution/` -> `experience/` | Normalized results become experience records |
| 12.20 Progress Tracking | T3-77 (part of) | `execution/` -> `state.rs` -> `database/` | Partial; must complete |
| 12.21 Cancellation | T3-78 (part of) | `execution/` -> `rollback.rs` -> `database/` | Partial rollback exists |
| 12.22 Retry Policy | T3-78 (part of) | `execution/` (new) -> `database/` | Must implement retry logic |
| 12.23 Error Handling/Recovery | T3-78 (part of) | `execution/` -> `safety_gate/` | Partial; must complete |
| 12.24 Checkpointing | T3-78 (part of) | `execution/` -> `database/` (checkpoint table) | Checkpoint table missing |
| 12.25 Long-Running Jobs | T3-78 (part of) | `execution/` -> `job_queue.rs` -> `database/` | Partial; must complete |
| 12.26 Observability | T3-78 (part of) | `execution/` -> `observability/` (new, ch 27) | Must build `observability/` first |
| 12.27 Execution Trace | T3-78 (part of) | `execution/` -> `database/` (trace table) -> `observability/` | Trace table missing |
| 12.28 Safety Enforcement | T3-78 (part of) | `execution/` -> `safety_gate/` -> `security/` (new, ch 25) | Must build `security/` first |
| 12.29 Human Approval | T3-78 (part of) | `execution/` -> `developer_interface/` (new, ch 28) | Must build `developer_interface/` first |
| 12.30 Idempotency/Side Effects | T3-78 (part of) | `execution/` (new) -> `database/` | Must implement idempotency |
| 12.31 Result Verification | T3-78 (part of) | `execution/` (new) -> `database/` | Must implement verification |
| 12.32 Experience Integration | T3-78 (part of) | `execution/` -> `experience/` | Partial; verify full integration |
| 12.33 Learning Integration | T3-78 (part of) | `execution/` -> `learning/` | Partial; verify full integration |
| 12.34 Execution/Planning Feedback | T3-78 (part of) | `execution/` -> `planner/` | Partial; must complete feedback loop |
| 12.35 Execution Determinism | T3-78 (part of) | `execution/` (new) -> `design_decisions/` (new) | Must implement determinism |
| 12.36 Execution Reproducibility | T3-78 (part of) | `execution/` -> `database/` (repro log) | Reproducibility log missing |
| 12.37 Execution State Machine | T3-78 (part of) | `execution/` (new) -> `database/` | State machine missing |

---

## Function-Level Wiring — Tool Engine (ch 13) — Deep Analysis
Every sub-section 13.1-13.10 must be covered. Per gap analysis table:

| Spec Function (13.md) | Status in Plan | Wiring To / From | Verification Method |
|---|---|---|---|
| Tool Registry | T3-79 | `tool_engine/` -> `bridge/acp/registry.rs` -> `data_contracts/` | Must unify ACP registry with tool registry |
| Tool Capability | T3-79 | `tool_engine/` -> `data_contracts/` -> `execution/` | Capability contracts must match execution requests |
| Tool Invocation | T3-82 | `tool_engine/` -> `execution/` -> `bridge/mcp/` | Must invoke via execution engine |
| Tool Result Handling | T3-82 (part of) | `tool_engine/` -> `execution/` -> `experience/` | Results must become experience records |
| Tool Security | T3-81 | `tool_engine/` -> `security/` (new) -> `agent/safety_gate/` | Must enforce security before invocation |
| Tool Audit | T3-83 (part of) | `tool_engine/` -> `observability/` (new) -> `database/` | Audit trail must be stored |
| MCP Tool Integration | T3-82 (part of) | `tool_engine/` -> `bridge/mcp/handler.rs` | Must wire MCP handler to tool engine |
| Local Module Integration | T3-82 (part of) | `tool_engine/` -> `skills/` + `workflows/` -> `execution/` | Must support local module execution |
| External API Integration | T3-83 | `tool_engine/` -> `bridge/app/initialization/` (config) -> `execution/` | Must support external APIs |
| Tool Lifecycle | T3-83 (part of) | `tool_engine/` (new) -> `database/` (lifecycle table) | Register -> invoke -> result -> audit -> retire |

---

## Next Increment Recommendation (per AGENTS.md — ONE change at a time)
Given the gap analysis, the first function to implement is `ExecutionRequest` (12.5) because it unlocks everything downstream (lifecycle, graph, action nodes, tool execution). Implementation order (function-by-function, one at a time):

1. `execution/` module scaffold (new dir, 1 file -> expand)
2. `ExecutionRequest` struct + validation (`data_contracts/`)
3. `ExecutionLifecycle` state machine (`execution/` -> `database/`)
4. `ExecutionGraph` (connect to `workflows/`)
5. `ActionNode` (connect to `tools/`)
6. `ExecutorAbstraction` (connect to `bridge/mcp/`)
7. `ExecutionState` persistence (`database/` new table)
8. `ExecutionBudget` (`cooboploop/` integration)
9. `Scheduler` delegation (`bridge/app/scheduler.rs` -> `execution/`)
10. `ParallelExecution` (`workers.rs` -> `execution/`)
11. `ResourceManager` (`database/`)
12. `ToolExecution` (connect to `tool_engine/` — build ch 13 after 12.17)

This is the wiring path for TIER 3 (v0.0.2.1) per `.agents/gap_analysis_v0.0.2.1.md`.

---

## Deep Analysis — Coverage Verification
Every aspect of v0.0.2.1 is covered in this merged plan:

- **Chapters 01-05 (Foundation):** T3-01 to T3-16 cover vision, principles, overview, data flow, contracts, metadata, schema, event routing.
- **Chapter 06 (Conversation):** TC-01 to TC-06 cover session, engine, `converse` tool, learning extraction, persistence.
- **Chapter 07 (Context):** T3-27 to T3-36 cover session state, working memory, retrieval set, compression, topic tracking, selection, budget, policy, construction, assembly.
- **Chapter 08 (Memory):** T3-37 to T3-49 cover short/long-term boundaries, create/promote/demote/archive/retrieve, working/episodic/semantic/procedural/experience-linked records, promotion rules.
- **Chapter 09 (Experience):** T3-50 to T3-57 cover execution history, outcomes, success/failure tracking, lesson capture, failure analysis, processing flow, related links.
- **Chapter 10 (Learning):** T3-58 to T3-66 cover reflection, candidate, evaluation, promotion, consolidation, pattern discovery, knowledge extraction, skill improvement, confidence updates.
- **Chapter 11 (Planning):** T3-67 to T3-73 cover goal creation/validation, decomposition, strategy selection, workflow generation, scoring, replanning.
- **Chapter 12 (Execution):** T3-74 to T3-78 + full function-level table cover all 30 sub-functions (12.5-12.37) with wiring paths.
- **Chapter 13 (Tool):** T3-79 to T3-83 + full function-level table cover all 10 sub-functions (registry, capability, invocation, result, security, audit, MCP, local, external, lifecycle).
- **Chapter 14 (Model):** T3-84 to T3-90 cover local/cloud runtime, routing, context, scheduling, validation, selection.
- **Chapter 15 (Agent Communication):** T3-91 to T3-94 cover protocol, MCP, ACP, internal rules.
- **Chapter 16 (Coordination):** T3-95 to T3-98 cover coordination, events, routing, orchestration.
- **Chapters 17-20 (Memory/Knowledge):** T3-99 to T3-112 cover promotion/retention/archive, experience links, confidence scoring (knowledge/skills/relationships/workflows), graph relationships/storage/confidence/queries/promotion.
- **Chapters 21-23 (Storage/DB/Workers):** T3-113 to T3-126 cover persistence architecture, organization, backup, SQLite, schema, indexing, migration, integrity, worker architecture, queue, supervision, memory/learning/maintenance workers.
- **Chapters 24-27 (Governance):** T3-127 to T3-139 cover contributor roles, contribution standards, identity/permissions, capability security, memory protection, audit, trust evaluation, learning/evolution distinction, hypothesis, experimentation, tracing, explanation/debugger.
- **Chapters 28-31 (Interfaces/Config/Testing/Deployment):** T3-140 to T3-154 cover inspection/commands, mutation/recovery, memory/worker control, debugging, configuration precedence, secrets/profiles, runtime override, unit/contract/integration/recovery/event/security/regression testing, install/upgrade/rollback/release validation.
- **Chapters 32-33 (Future):** T3-155 to T3-156 cover stable-contract admission, architectural-gate review/roadmap.
- **Appendices A-E:** T3-157 to T3-170 cover directory ownership, source-tree, schema domain/versioning/migration, event identity/versioning/payload/lifecycle, design decisions/supersession, development guidelines/modularity/model-replaceability/review, odd-notes quarantine.

**Critical gaps explicitly called out:**
- `execution/` (ch 12): 1 file only — ENTIRE ENGINE MISSING. Every sub-section 12.1-12.41 unimplemented. Function-level wiring table included.
- `tools/` (ch 13): 4 files — stub-level. No `tool_registry/`, `tool_capability/`, `tool_invocation/`, `tool_audit/`, `tool_security/`. Function-level wiring included.
- `memory_hierarchy/` (ch 14): MISSING entirely.
- `context_lifecycle/` (ch 15): MISSING entirely.
- `retrieval_pipeline/` (ch 16): MISSING entirely.
- `prompt_construction/` (ch 17): MISSING entirely.
- `strategic_learning/` (ch 18): MISSING entirely.
- `confidence_system/` (ch 19): MISSING entirely.
- `security/` (ch 25): MISSING entirely.
- `observability/` (ch 27): MISSING entirely.
- `developer_interface/` (ch 28): MISSING entirely.
- `configuration/` (ch 29): MISSING entirely.
- `deployment/` (ch 31): MISSING entirely.
- `future_expansion/` (ch 32): MISSING (documented only).
- `roadmap/` (ch 33): MISSING (documented only).

**Wiring paths (6 critical paths) explicitly mapped:** Execution, Memory/Context, Confidence/Knowledge, Governance, Observability, Configuration/Deployment.

**Next increment (per AGENTS.md):** `ExecutionRequest` (12.5) is the unlock function. Implementation order listed (1-12) with verification method for each.

---

## Completion target
End state: finished v0.0.2.1. Gate stays green throughout (`bash .agents/scripts/make.sh gate` passes with 0 compiler warnings, 0 code issues, 0 untested tools, 100% tests). Every chapter 01-33 and appendix A-E covered by at least one task in this plan.



# Gap Analysis: robot_architecture/v0.0.2.1 vs Current Codebase vs .agents/t3_PLAN.md

Created: 2026-09-15
Method: One architecture chapter at a time; per chapter: spec functions -> current module -> gap -> wiring.
Source of truth: `robot_architecture/v0.0.2.1/` (00.md-33.md + FINAL_ARCHITECTURE_SPEC.md + appendices A-E)
Current codebase modules (confirmed via `find src -name '*.rs'`):
- `bridge/` (139 files): acp/, app/initialization/, mcp/, logging, state
- `experience/` (100 files): recording, scoring, repository, pipeline
- `learning/` (23), `memory/` (16), `cooboploop/` (21), `database/` (19)
- `execution/` (1), `data_contracts/` (13), `planner/` (7), `knowledge/` (7)
- `skills/`, `workflows/`, `agent/` (context/decision/loop_runner), `world_model/`, `personality/`
- `tools/` (4), `communication/` (4), `models/` (1), `cli/` (11), `research/` (14)

Note: `execution/` has only 1 file — Execution Engine (ch 12) is essentially unimplemented.
`tools/` has 4 files — Tool Engine (ch 13) is stub-level.
No `retrieval_pipeline/`, `prompt_construction/`, `context_lifecycle/`, `memory_hierarchy/`, `confidence_system/`, `storage_architecture/`, `security/`, `observability/`, `developer_interface/`, `configuration/`, `testing/` modules exist.

---

## Chapter 01 — Vision & Philosophy (L19-28)
Spec: Vision statement, philosophy of cognitive architecture, independence from model provider.
Current: `robot_architecture/v0.0.2.md` exists; no dedicated `vision/` module. Philosophy embedded in `AGENTS.md` and `FINAL_ARCHITECTURE_SPEC.md`.
Gap: None structural — philosophy is documented. Wiring: feeds into `bridge/app/initialization/core.rs` (initialization policy) and `data_contracts/` (contract design).
Wiring needed: `core.rs` -> `data_contracts/` (contract shapes must reflect vision of model-independence).

## Chapter 02 — Core Design Principles (L11-19)
Spec: Ownership, lifecycle, identity/correlation, provenance, evidence/uncertainty, failure visibility, model independence, controlled effects, observability, versioned evolution, human control, compatibility.
Current: Partially enforced via `agent/safety_gate/` (rollback, sandbox, hallucination), `bridge/app/state.rs`, `experience/` provenance fields.
Gap: No centralized `design_principles/` enforcement module; principles are scattered.
Wiring: `safety_gate/mod.rs` -> `agent/loop_runner.rs` -> `bridge/app/state.rs`. Need a `principles/enforcer.rs` that validates actions against all 12 principles before execution.

## Chapter 03 — High-Level System Overview (L28-42)
Spec: System diagram, component boundaries, data flow overview.
Current: `bridge/app/initialization/engines.rs` lists engines; `FINAL_ARCHITECTURE_SPEC.md` has overview.
Gap: No executable `system_overview/` module that validates component wiring at startup.
Wiring: `engines.rs` -> `execution/` (missing) -> `tool_engine/` (missing) -> `memory/` + `experience/`.

## Chapter 04 — Data Flow (L42-59)
Spec: Data-flow diagram, input -> processing -> output paths, event definitions.
Current: `data_contracts/` (13 files) defines contracts; `bridge/app/initialization/` has flow logic; `appendix-c.md` defines events.
Gap: No `data_flow/` engine that executes the diagram; events are defined but not routed through a pipeline.
Wiring needed: `data_contracts/` -> `event_router/` (new) -> `experience/` + `memory/` + `learning/`. Each contract change must trigger `appendix-c.md` event emission.

## Chapter 05 — Data Contracts (L59-69)
Spec: Contract schema, versioning, validation, backward compatibility.
Current: `data_contracts/` exists (13 files). Contracts for Experience, Memory, Knowledge, Plan, Skill, etc.
Gap: Contract validation is manual; no `contract_validator/` that enforces schema at runtime.
Wiring: `data_contracts/` -> `database/` (DB schema must match contracts) -> `bridge/app/initialization/db.rs`. Need `contract_validator.rs` called by `db.rs` on init.

## Chapter 06 — Conversation Engine (L69-82)
Spec: Conversation state, turn management, message history, context window management.
Current: `bridge/acp/` (message, agent, registry, router, system_agent); `agent/context.rs`; `communication/` (4 files).
Gap: No dedicated `conversation_engine/` module; conversation logic split across ACP bridge and agent context.
Wiring: `bridge/acp/message.rs` -> `agent/context.rs` -> `conversation_engine/` (new). Must preserve turn order and correlation IDs per `appendix-c.md`.

## Chapter 07 — Context Engine (L82-95)
Spec: Context lifecycle, context object, context retrieval, context update, context expiration.
Current: `agent/context.rs`; `bridge/mcp/context.rs`; `bridge/app/initialization/mcp_context.rs`.
Gap: No `context_engine/` with full lifecycle (create -> retrieve -> update -> expire -> archive). Only partial context storage.
Wiring: `context_engine/` (new) -> `memory/` (store context) -> `retrieval_pipeline/` (ch 16, new) -> `prompt_construction/` (ch 17, new). Context expiration must trigger `memory/` archive.

## Chapter 08 — Memory Engine (L95-103)
Spec: Memory types (episodic, semantic, procedural), memory storage, retrieval, update, forgetting.
Current: `memory/` (16 files); `experience/` (100 files) overlaps with episodic.
Gap: Memory engine is partial — no explicit `semantic_memory/` vs `procedural_memory/` split; forgetting mechanism missing.
Wiring: `memory/` -> `memory_hierarchy/` (ch 14, new) -> `database/`. Need `forgetting_policy.rs` that uses `confidence_system/` (ch 19) to decide retention.

## Chapter 09 — Experience Engine (L103-117)
Spec: Experience recording, experience scoring, experience repository, experience pipeline.
Current: `experience/` (100 files) — very complete: recording, scoring, repository, pipeline, diagnostics.
Gap: Minimal — experience pipeline is implemented (`learning_pipeline.rs`, `experience_recorder_diagnostics.rs`). Missing: experience-to-learning promotion is manual.
Wiring: `experience/` -> `learning/` (23 files) -> `cooboploop/` (loop runner). Already wired; verify `experience/` -> `learning/` promotion uses `confidence_system/` (missing).

## Chapter 10 — Learning Engine (L117-129)
Spec: Learning updates, learning pipeline, learning from experience, learning from feedback.
Current: `learning/` (23 files); `cooboploop/` (loop runner, objective queue, capability recording).
Gap: Learning pipeline exists but lacks `strategic_learning/` (ch 18) integration; no `learning_update/` persistence to DB.
Wiring: `learning/` -> `database/` (new schema for learning_updates) -> `strategic_learning/` (new). Need `learning_update_persist.rs`.

## Chapter 11 — Planning Engine (L129-139)
Spec: Plan creation, plan execution, plan update, plan cancellation, plan verification.
Current: `planner/` (7 files); `workflows/` (12 files); `cooboploop/` (goal/objective management).
Gap: Planning engine exists but is not integrated with `execution_engine/` (ch 12, missing). Plans are stored but not executed by an executor.
Wiring: `planner/` -> `execution/` (new) -> `tool_engine/` (new). Plan steps must become `execution_request/` objects.

## Chapter 12 — Execution Engine (L139-151) [CRITICAL GAP — 1 file only]
Spec (from 12.md outline): Execution Request, Execution Lifecycle, Execution Graph, Action Node, Action Types, Executor Abstraction, Execution Context, Execution State, Execution Budget, Scheduler, Parallel Execution, Resource Manager, Tool Execution, MCP Integration, Result Normalization, Progress Tracking, Cancellation, Retry Policy, Error Handling/Recovery, Checkpointing, Long-Running Jobs, Observability, Execution Trace, Safety Enforcement, Human Approval, Idempotency/Side Effects, Result Verification, Experience Integration, Learning Integration, Execution/Planning Feedback, Execution Determinism, Execution Reproducibility, Execution State Machine, Architectural Boundaries.
Current: `execution/` (1 file — likely stub). No executor, no scheduler, no resource manager, no checkpointing.
Gap: ENTIRE ENGINE MISSING. Every sub-section 12.1-12.41 unimplemented.
Wiring needed (function-by-function):
- `ExecutionRequest` (12.5) -> `planner/` (plan steps become requests)
- `ExecutionLifecycle` (12.6) -> `execution/` (new module) -> `agent/loop_runner.rs` (loop drives lifecycle)
- `ExecutionGraph` (12.7) -> `execution/` -> `workflows/` (graph of actions)
- `ActionNode` (12.8) -> `execution/` -> `tools/` (action = tool call or local module)
- `ExecutorAbstraction` (12.10) -> `execution/` -> `bridge/mcp/` (MCP execution path)
- `ExecutionContext` (12.11) -> `execution/` -> `agent/context.rs` (context feeds execution)
- `ExecutionState` (12.12) -> `execution/` -> `database/` (persist state)
- `ExecutionBudget` (12.13) -> `execution/` -> `cooboploop/` (budget from loop config)
- `Scheduler` (12.14) -> `execution/` -> `bridge/app/scheduler.rs` (existing scheduler must delegate)
- `ParallelExecution` (12.15) -> `execution/` -> `bridge/app/initialization/workers.rs` (worker pool)
- `ResourceManager` (12.16) -> `execution/` -> `database/` (resource tracking)
- `ToolExecution` (12.17) -> `execution/` -> `tool_engine/` (new, ch 13)
- `MCPIntegration` (12.18) -> `execution/` -> `bridge/mcp/handler.rs`
- `ResultNormalization` (12.19) -> `execution/` -> `experience/` (normalized results become experience)
- `ProgressTracking` (12.20) -> `execution/` -> `bridge/app/state.rs`
- `Cancellation` (12.21) -> `execution/` -> `agent/safety_gate/rollback.rs`
- `RetryPolicy` (12.22) -> `execution/` -> `execution/` (new)
- `ErrorHandling/Recovery` (12.23) -> `execution/` -> `agent/safety_gate/`
- `Checkpointing` (12.24) -> `execution/` -> `database/` (checkpoint table)
- `LongRunningJobs` (12.25) -> `execution/` -> `bridge/app/initialization/job_queue.rs`
- `Observability` (12.26) -> `execution/` -> `observability/` (new, ch 27)
- `ExecutionTrace` (12.27) -> `execution/` -> `database/` (trace table)
- `SafetyEnforcement` (12.28) -> `execution/` -> `agent/safety_gate/`
- `HumanApproval` (12.29) -> `execution/` -> `developer_interface/` (new, ch 28)
- `Idempotency/SideEffects` (12.30) -> `execution/` -> `execution/` (new)
- `ResultVerification` (12.31) -> `execution/` -> `execution/` (new)
- `ExperienceIntegration` (12.32) -> `execution/` -> `experience/`
- `LearningIntegration` (12.33) -> `execution/` -> `learning/`
- `Execution/PlanningFeedback` (12.34) -> `execution/` -> `planner/`
- `ExecutionDeterminism` (12.35) -> `execution/` -> `execution/` (new)
- `ExecutionReproducibility` (12.36) -> `execution/` -> `database/` (reproducibility log)
- `ExecutionStateMachine` (12.37) -> `execution/` -> `execution/` (new)

## Chapter 13 — Tool Engine (L151-165) [CRITICAL GAP — 4 files]
Spec (from 13.md outline): Tool registry, tool capability definition, tool invocation, tool result handling, tool security, tool audit, MCP tool integration, local module integration, external API integration, tool lifecycle.
Current: `tools/` (4 files). Very minimal.
Gap: No `tool_registry/`, `tool_capability/`, `tool_invocation/`, `tool_audit/`, `tool_security/`.
Wiring needed:
- `ToolRegistry` -> `bridge/acp/registry.rs` (ACP registry is separate; need unified tool registry)
- `ToolCapability` -> `data_contracts/` (capability contracts)
- `ToolInvocation` -> `execution/` (ch 12, new) -> `bridge/mcp/`
- `ToolResultHandling` -> `execution/` -> `experience/`
- `ToolSecurity` -> `security/` (new, ch 25) -> `agent/safety_gate/`
- `ToolAudit` -> `observability/` (new, ch 27) -> `database/`
- `MCPToolIntegration` -> `bridge/mcp/handler.rs`
- `LocalModuleIntegration` -> `execution/` -> `skills/` + `workflows/`
- `ExternalAPIIntegration` -> `execution/` -> `bridge/app/initialization/` (external config)
- `ToolLifecycle` -> `execution/` (register -> invoke -> result -> audit -> retire)

## Chapter 14 — Memory Hierarchy (L165-176) [MISSING]
Spec: Hierarchy levels (working, short-term, long-term, archival), promotion/demotion rules, hierarchy navigation.
Current: None. `memory/` is flat.
Gap: Entire hierarchy missing.
Wiring: `memory_hierarchy/` (new) -> `memory/` (existing flat storage) -> `database/` (hierarchy tables). Promotion uses `confidence_system/` (ch 19). Demotion uses `forgetting_policy/` (ch 8).

## Chapter 15 — Context Lifecycle (L176-185) [MISSING]
Spec: Context creation, retrieval, update, expiration, archival, correlation.
Current: Partial in `agent/context.rs`, `bridge/mcp/context.rs`.
Gap: No lifecycle engine; no expiration/archival logic.
Wiring: `context_lifecycle/` (new) -> `context_engine/` (ch 7) -> `memory/` (archive) -> `database/` (context table with expiry timestamps).

## Chapter 16 — Retrieval Pipeline (L185-199) [MISSING]
Spec: Retrieval stages (query -> filter -> rank -> retrieve -> format), retrieval strategies, retrieval evaluation.
Current: None dedicated. Some retrieval in `memory/` and `knowledge/`.
Gap: Entire pipeline missing.
Wiring: `retrieval_pipeline/` (new) -> `memory/` + `knowledge/` + `experience/` -> `prompt_construction/` (ch 17). Query comes from `context_engine/` (ch 7). Ranking uses `confidence_system/` (ch 19).

## Chapter 17 — Prompt Construction (L199-207) [MISSING]
Spec: Prompt assembly, context injection, instruction formatting, output parsing, prompt versioning.
Current: None dedicated. Prompt logic scattered in `agent/loop_runner.rs`, `bridge/app/initialization/`.
Gap: No `prompt_construction/` module.
Wiring: `prompt_construction/` (new) -> `retrieval_pipeline/` (ch 16) -> `context_engine/` (ch 7) -> `agent/loop_runner.rs` (consumes constructed prompt). Must support `data_contracts/` for prompt schema.

## Chapter 18 — Strategic Learning (L207-227) [MISSING]
Spec: Strategic objectives, strategic updates, strategic evaluation, strategic feedback.
Current: `cooboploop/` has strategic objectives (`cooboploop_add_strategic_objective`, `list_strategic_objectives`), goal queue, capability assessment. Partial.
Gap: No `strategic_learning/` module; strategic updates not integrated with `learning/` or `execution/`.
Wiring: `strategic_learning/` (new) -> `cooboploop/` (existing loop) -> `learning/` (ch 10) -> `execution/` (ch 12, feedback loop 12.34). Need `strategic_update_persist.rs`.

## Chapter 19 — Confidence System (L227-248) [MISSING]
Spec: Confidence scoring, confidence propagation, confidence thresholds, confidence audit.
Current: None dedicated. Some confidence-like fields in `experience/` (score fields) and `knowledge/`.
Gap: Entire system missing.
Wiring: `confidence_system/` (new) -> `memory/` (retention decisions) -> `knowledge/` (promotion decisions) -> `retrieval_pipeline/` (ranking) -> `execution/` (budget/approval thresholds). Must integrate with `database/` (confidence table).

## Chapter 20 — Knowledge Graph (L248-274)
Spec: Knowledge nodes, relationships, graph traversal, graph update, graph verification.
Current: `knowledge/` (7 files); `world_model/` (3 files); `bridge/app/initialization/knowledge_pipeline/`.
Gap: Graph traversal and verification missing; relationships are basic.
Wiring: `knowledge/` -> `world_model/` -> `database/` (graph tables). Need `graph_traversal/` (new) and `graph_verification/` (new) using `confidence_system/` (ch 19).

## Chapter 21 — Storage Architecture (L274-295) [MISSING]
Spec: Storage layers, storage policies, storage audit, storage versioning.
Current: `database/` (19 files) has DB access; no storage-layer abstraction.
Gap: No `storage_architecture/` module.
Wiring: `storage_architecture/` (new) -> `database/` -> `memory/` + `experience/` + `knowledge/`. Must define storage policies (hot/warm/cold) that feed `memory_hierarchy/` (ch 14).

## Chapter 22 — Database Design (L295-318)
Spec: Schema design, table definitions, index design, migration strategy, DB initialization.
Current: `database/` (19 files); `appendix-b.md` has schemas.
Gap: Schema is defined but not fully enforced; missing tables for execution, tool audit, context lifecycle, retrieval pipeline, confidence, storage.
Wiring: `database/` -> `appendix-b.md` (source of schema) -> `bridge/app/initialization/db.rs` (init). Need new migrations for ch 12-23, 25-30 tables.

## Chapter 23 — Background Workers (L318-343) [PARTIAL]
Spec: Worker types, worker lifecycle, worker scheduling, worker monitoring, worker recovery.
Current: `bridge/app/initialization/workers.rs`; `cooboploop/` loop; `bridge/app/scheduler.rs`.
Gap: No dedicated `background_workers/` module; worker recovery and monitoring partial.
Wiring: `background_workers/` (new) -> `bridge/app/initialization/workers.rs` -> `execution/` (ch 12, long-running jobs) -> `observability/` (ch 27).

## Chapter 24 — AI Contributor Operating Agreement (L343-372)
Spec: Agreement terms, contributor rules, agreement enforcement.
Current: `AGENTS.md` (personal rules); `robot_architecture/` docs; no `agreement/` module.
Gap: No executable agreement enforcement.
Wiring: `agreement/` (new) -> `agent/loop_runner.rs` (enforce before action) -> `developer_interface/` (ch 28, human approval).

## Chapter 25 — Security and Trust Architecture (L372-398) [MISSING]
Spec: Security boundaries, trust levels, access control, audit trails, encryption.
Current: `agent/safety_gate/` (rollback, sandbox, hallucination); `bridge/app/initialization/` has some security.
Gap: No `security/` module; access control and encryption missing.
Wiring: `security/` (new) -> `agent/safety_gate/` -> `tool_engine/` (ch 13, tool security) -> `execution/` (ch 12, safety enforcement) -> `database/` (encrypted fields) -> `observability/` (audit).

## Chapter 26 — Self-Improvement and Evolution Architecture (L398-422)
Spec: Self-improvement mechanisms, evolution rules, improvement verification, improvement audit.
Current: `agent/safety_gate/` has rollback; `experience/` has corrections; `cooboploop/` has loop.
Gap: No `self_improvement/` module; improvement is manual.
Wiring: `self_improvement/` (new) -> `learning/` (ch 10) -> `execution/` (ch 12, feedback) -> `agent/safety_gate/` (rollback if improvement fails). Must use `developer_interface/` (ch 28) for human approval of self-changes.

## Chapter 27 — Cognitive Monitoring and Observability Architecture (L422-446) [MISSING]
Spec: Monitoring metrics, observability pipeline, alert rules, audit logs, trace collection.
Current: `bridge/logging.rs`; some diagnostics (`experience_recorder_diagnostics.rs`, etc.).
Gap: No `observability/` module; no metrics pipeline, no alert rules.
Wiring: `observability/` (new) -> `execution/` (ch 12, traces) -> `tool_engine/` (ch 13, audit) -> `database/` (metrics/log tables) -> `developer_interface/` (ch 28, dashboards). Must consume `execution_trace/` (12.27) and `tool_audit/` (13.6).

## Chapter 28 — Developer Interface and Control Plane (L446-473) [MISSING]
Spec: Developer interface, control commands, status reporting, configuration interface, approval interface.
Current: `cli/` (11 files); `bridge/app/initialization/` has some control.
Gap: No `developer_interface/` module; approval interface missing (needed by 12.29 human approval, 26 self-improvement).
Wiring: `developer_interface/` (new) -> `execution/` (approval gate 12.29) -> `self_improvement/` (approval 26) -> `configuration/` (ch 29) -> `observability/` (ch 27, dashboards).

## Chapter 29 — Configuration and Runtime Management (L473-505) [MISSING]
Spec: Configuration schema, runtime settings, feature flags, runtime updates, configuration audit.
Current: `bridge/app/initialization/` has some config; `cooboploop/` has settings.
Gap: No `configuration/` module; no feature flags, no runtime update mechanism.
Wiring: `configuration/` (new) -> `database/` (config table) -> `execution/` (budget/settings) -> `developer_interface/` (ch 28, config UI) -> `observability/` (ch 27, config audit).

## Chapter 30 — Testing and Validation Architecture (L505-539) [PARTIAL — test_suite2]
Spec: Test architecture, validation methods, test coverage, test automation, test reporting.
Current: `.agents/scripts/test_suite2/` (full E2E suite via MCP); `test_suite_report.json` tracks 454/454.
Gap: No `testing/` module inside `src/`; tests are external. Architecture requires internal validation hooks.
Wiring: `testing/` (new, internal hooks) -> `execution/` (ch 12, verification 12.31) -> `tool_engine/` (ch 13, capability validation) -> `database/` (test fixtures) -> `developer_interface/` (ch 28, test runner UI).

## Chapter 31 — Deployment Architecture (L539-560) [MISSING]
Spec: Deployment model, deployment pipeline, deployment verification, rollback.
Current: None dedicated.
Gap: Entire deployment architecture missing.
Wiring: `deployment/` (new) -> `configuration/` (ch 29) -> `database/` (deployment log) -> `developer_interface/` (ch 28, deploy UI).

## Chapter 32 — Future Expansion Architecture (L560-595) [DOCUMENTED]
Spec: Expansion gates, future capabilities, extension points.
Current: `robot_architecture/v0.0.2.1/32.md` exists.
Gap: No `future_expansion/` module; gates are documentation only.
Wiring: `future_expansion/` (new) -> `developer_interface/` (ch 28) -> `configuration/` (ch 29, feature flags).

## Chapter 33 — Future Architecture and Capability Roadmap (L595-628) [DOCUMENTED]
Spec: Roadmap, capability maturity, milestone tracking.
Current: `t3_PLAN.md` is the roadmap; `FINAL_ARCHITECTURE_SPEC.md` has milestones.
Gap: No `roadmap/` executable module.
Wiring: `roadmap/` (new) -> `cooboploop/` (objective tracking) -> `developer_interface/` (ch 28, milestone UI).

---

## Appendix A — Directory Structure (L628-670)
Spec: Directory layout, module organization.
Current: `src/` has ~23 top-level dirs; architecture expects more (execution/, tool_engine/, retrieval_pipeline/, prompt_construction/, context_lifecycle/, memory_hierarchy/, confidence_system/, storage_architecture/, security/, observability/, developer_interface/, configuration/, testing/, deployment/, future_expansion/).
Gap: ~15 new top-level dirs needed.
Wiring: All new dirs feed into `bridge/app/initialization/engines.rs` (engine registration).

## Appendix B — Database Schemas (L670-701)
Spec: Schema definitions for all chapters.
Current: `appendix-b.md` defines schemas; `database/` has partial implementation.
Gap: Tables for execution (12), tool audit (13), context lifecycle (15), retrieval (16), confidence (19), storage (21), security (25), observability (27), developer interface (28), configuration (29), testing (30) missing.
Wiring: `database/` -> `appendix-b.md` (source) -> `bridge/app/initialization/db.rs` (init). Need migration files.

## Appendix C — Event Definitions (L701-734)
Spec: Event types, event schema, event routing.
Current: `appendix-c.md` defines events; `bridge/app/initialization/` has some event handling.
Gap: No `event_router/` module; events not routed to all subscribers.
Wiring: `event_router/` (new) -> `experience/` + `memory/` + `learning/` + `execution/` + `observability/`. Must use `data_contracts/` event contracts.

## Appendix D — Design Decisions (L734-764)
Spec: Design decisions (A.1-A.3, etc.), decision rationale.
Current: `AGENTS.md` references design decisions; `FINAL_ARCHITECTURE_SPEC.md` has some.
Gap: No `design_decisions/` module; decisions not enforced at runtime.
Wiring: `design_decisions/` (new) -> `agent/loop_runner.rs` (enforce A.3 self-improvement boundary) -> `execution/` (enforce determinism 12.35).

## Appendix E — Development Guidelines (L764-771)
Spec: Development practices, code standards.
Current: `AGENTS.md` (hard rules); `FINAL_ARCHITECTURE_SPEC.md`.
Gap: No executable guideline enforcement.
Wiring: `development_guidelines/` (new) -> `cli/` (lint/check commands) -> `testing/` (ch 30).

---

## Wiring Summary — Critical Paths

Path 1 (Execution): `planner/` -> `execution/` (new, 12) -> `tool_engine/` (new, 13) -> `database/` (new tables) -> `experience/` + `learning/` -> `cooboploop/`
Path 2 (Memory/Context): `context_engine/` (7) -> `context_lifecycle/` (15) -> `memory/` -> `memory_hierarchy/` (14) -> `retrieval_pipeline/` (16) -> `prompt_construction/` (17) -> `agent/loop_runner.rs`
Path 3 (Confidence/Knowledge): `confidence_system/` (19) -> `knowledge/` (20) -> `retrieval_pipeline/` (16) -> `execution/` (budget/approval)
Path 4 (Governance): `security/` (25) -> `agent/safety_gate/` -> `execution/` (12.28) -> `developer_interface/` (28, approval) -> `self_improvement/` (26)
Path 5 (Observability): `execution/` (12.26-12.27) + `tool_engine/` (13.6) -> `observability/` (27) -> `database/` (metrics) -> `developer_interface/` (28, dashboards)
Path 6 (Configuration/Deployment): `configuration/` (29) -> `execution/` (budget/settings) -> `deployment/` (31) -> `developer_interface/` (28)

---

## Function-Level Wiring — Execution Engine (ch 12) — One Function at a Time

For each spec function in 12.md, current state and wiring:

| Spec Function (12.md) | Current | Wiring To / From |
|---|---|---|
| 12.5 Execution Request | MISSING | `planner/` -> `execution/` -> `database/` |
| 12.6 Execution Lifecycle | MISSING | `execution/` -> `agent/loop_runner.rs` -> `database/` |
| 12.7 Execution Graph | MISSING | `execution/` -> `workflows/` -> `database/` |
| 12.8 Action Node | MISSING | `execution/` -> `tools/` -> `execution/` |
| 12.9 Action Types | MISSING | `execution/` -> `skills/` + `workflows/` + `bridge/mcp/` |
| 12.10 Executor Abstraction | MISSING | `execution/` -> `bridge/mcp/handler.rs` |
| 12.11 Execution Context | PARTIAL (`agent/context.rs`) | `execution/` -> `agent/context.rs` -> `context_engine/` |
| 12.12 Execution State | MISSING | `execution/` -> `database/` (state table) |
| 12.13 Execution Budget | MISSING | `execution/` -> `cooboploop/` (config) -> `database/` |
| 12.14 Scheduler | PARTIAL (`bridge/app/scheduler.rs`) | `execution/` -> `scheduler.rs` -> `workers.rs` |
| 12.15 Parallel Execution | PARTIAL (`workers.rs`) | `execution/` -> `workers.rs` -> `database/` |
| 12.16 Resource Manager | MISSING | `execution/` -> `database/` (resource table) |
| 12.17 Tool Execution | MISSING | `execution/` -> `tool_engine/` (new) |
| 12.18 MCP Integration | PARTIAL (`bridge/mcp/`) | `execution/` -> `bridge/mcp/handler.rs` |
| 12.19 Result Normalization | MISSING | `execution/` -> `experience/` |
| 12.20 Progress Tracking | PARTIAL (`state.rs`) | `execution/` -> `state.rs` -> `database/` |
| 12.21 Cancellation | PARTIAL (`rollback.rs`) | `execution/` -> `rollback.rs` -> `database/` |
| 12.22 Retry Policy | MISSING | `execution/` (new) -> `database/` |
| 12.23 Error Handling/Recovery | PARTIAL (`safety_gate/`) | `execution/` -> `safety_gate/` |
| 12.24 Checkpointing | MISSING | `execution/` -> `database/` (checkpoint table) |
| 12.25 Long-Running Jobs | PARTIAL (`job_queue.rs`) | `execution/` -> `job_queue.rs` -> `database/` |
| 12.26 Observability | MISSING | `execution/` -> `observability/` (new) |
| 12.27 Execution Trace | MISSING | `execution/` -> `database/` (trace table) -> `observability/` |
| 12.28 Safety Enforcement | PARTIAL (`safety_gate/`) | `execution/` -> `safety_gate/` -> `security/` (new) |
| 12.29 Human Approval | MISSING | `execution/` -> `developer_interface/` (new) |
| 12.30 Idempotency/Side Effects | MISSING | `execution/` (new) -> `database/` |
| 12.31 Result Verification | MISSING | `execution/` (new) -> `database/` |
| 12.32 Experience Integration | PARTIAL (`experience/`) | `execution/` -> `experience/` |
| 12.33 Learning Integration | PARTIAL (`learning/`) | `execution/` -> `learning/` |
| 12.34 Execution/Planning Feedback | PARTIAL (`planner/`) | `execution/` -> `planner/` |
| 12.35 Execution Determinism | MISSING | `execution/` (new) -> `design_decisions/` (new) |
| 12.36 Execution Reproducibility | MISSING | `execution/` -> `database/` (repro log) |
| 12.37 Execution State Machine | MISSING | `execution/` (new) -> `database/` |

---

## Function-Level Wiring — Tool Engine (ch 13) — One Function at a Time

| Spec Function (13.md) | Current | Wiring To / From |
|---|---|---|
| Tool Registry | PARTIAL (`bridge/acp/registry.rs`) | `tool_engine/` -> `bridge/acp/registry.rs` -> `data_contracts/` |
| Tool Capability | MISSING | `tool_engine/` -> `data_contracts/` -> `execution/` |
| Tool Invocation | MISSING | `tool_engine/` -> `execution/` -> `bridge/mcp/` |
| Tool Result Handling | MISSING | `tool_engine/` -> `execution/` -> `experience/` |
| Tool Security | PARTIAL (`safety_gate/`) | `tool_engine/` -> `security/` (new) -> `agent/safety_gate/` |
| Tool Audit | MISSING | `tool_engine/` -> `observability/` (new) -> `database/` |
| MCP Tool Integration | PARTIAL (`bridge/mcp/`) | `tool_engine/` -> `bridge/mcp/handler.rs` |
| Local Module Integration | PARTIAL (`skills/`, `workflows/`) | `tool_engine/` -> `skills/` + `workflows/` -> `execution/` |
| External API Integration | MISSING | `tool_engine/` -> `bridge/app/initialization/` (config) -> `execution/` |
| Tool Lifecycle | MISSING | `tool_engine/` (new) -> `database/` (lifecycle table) |

---

## Next Increment Recommendation (per AGENTS.md — ONE change)

Given the gap analysis, the first function to implement is `ExecutionRequest` (12.5) because it unlocks everything downstream (lifecycle, graph, action nodes, tool execution). Implementation order:
1. `execution/` module scaffold (new dir, 1 file -> expand)
2. `ExecutionRequest` struct + validation (`data_contracts/`)
3. `ExecutionLifecycle` state machine (`execution/` -> `database/`)
4. `ExecutionGraph` (connect to `workflows/`)
5. `ActionNode` (connect to `tools/`)
6. `ExecutorAbstraction` (connect to `bridge/mcp/`)
7. `ExecutionState` persistence (`database/` new table)
8. `ExecutionBudget` (`cooboploop/` integration)
9. `Scheduler` delegation (`bridge/app/scheduler.rs` -> `execution/`)
10. `ParallelExecution` (`workers.rs` -> `execution/`)
11. `ResourceManager` (`database/`)
12. `ToolExecution` (connect to `tool_engine/` — build 13 after 12.17)

This is the wiring path for TIER 3 (v0.0.2.1) per `.agents/t3_PLAN.md`.
