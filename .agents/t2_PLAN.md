# TIER 2 PLAN — Reach v0.0.2

## Purpose
Upgrade the existing subsystems to the v0.0.2 architecture in a dependency-first order.

**Architecture reference:** `robot_architecture/RoBoT Architecture v0.0.2.md` (33 chapters across Part I-IV). Every task in this plan cites the exact chapter and section it implements. Tasks that are non-trivial (estimate > 5 minutes) are pre-broken into `▸` micro-tasks of ~5 minutes each: a struct, a field, a function, a test, a wiring call, or a single commit.

## Execution rule for small tasks
**Each `▸` is one 5-minute increment.** One tiny code change → one verification (`cargo check --release` for pure code, `make gate` for wired tools) → commit → push → next.

**NEVER batch two `▸` bullets into one commit.** A struct + its fields = one pass; its serde round-trip test = a separate pass. Tool registration is one pass PER TOOL.

## Convention
- `- [ ]` = pending (not started).
- `- [x]` = fully complete AND gate-green.
- `[DONE]` removed from PLAN.md → moved to `.agents/CHANGELOG.md` (per AGENTS.md task completion protocol).
- `[BLOCKED]` = cannot proceed without external decision; explain in `.agents/context_save.md`.
- `[?]` = needs clarification; do NOT skip silently.
- Each task line shows: `T2-NN — Title — Chapter X.Y` where X.Y is the v0.0.2 chapter.
- Each `▸` micro-task shows the exact file/function, the change, and the verification command.

## How to read this plan
1. Find the first `- [ ]` line at the top of the file.
2. Read the architecture chapter referenced (file path + chapter number).
3. If the task has `▸` children, do them in order. Otherwise, do the single step described.
4. Run the verification command. Fix until green. Commit. Push. Mark complete.
5. Re-load this file, find the next `- [ ]`, repeat.

---

## 0. Architecture foundations and invariants
Set the rules that every v0.0.2 subsystem must preserve. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Part I, Chapters 1-4.

---

## 6. Planning Engine
Use the data contracts to make planning more structured. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 11 (Planning Engine).

---

## 7. Execution and Tooling surfaces
Make execution and tool use explicit, authorized, and observable. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 12 (Execution Engine) + Chapter 13 (Tool Engine).

- [ ] **T2-95** — Add execution-step fields for actions — Chapter 12.1 "Action execution".
  - **▸** In `src/execution/mod.rs`, define `pub struct ExecutionStep { pub id: String, pub action: String, pub params: serde_json::Value, pub timeout_ms: u64 }` deriving Clone, Serialize, Deserialize. Verify `cargo check --release`. Commit.
- [ ] **T2-96** — Add execution-step fields for external interactions — Chapter 12.3 "External interactions".
  - **▸** Add `pub target_kind: TargetKind` (enum: Local, Network, Filesystem, Tool) and `pub target: String` to `ExecutionStep` with `#[serde(default)]`. Verify `cargo check --release`. Commit.
- [ ] **T2-97** — Add execution-step fields for result handling — Chapter 12.4 "Result handling".
  - **▸** Add `pub expected_output_kind: OutputKind` (enum: None, Text, Json, Binary) to `ExecutionStep` with default `Json`. Verify `cargo check --release`. Commit.
- [ ] **T2-98** — Add result-normalization rules — Chapter 12.4.
  - **▸** Add `pub fn normalize_result(raw: &serde_json::Value, kind: OutputKind) -> serde_json::Value` wrapping strings as `{"text": s}` for Text, parsing JSON for Json, base64-encoding for Binary. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/execution_normalize.rs`. Wire + verify. Commit.
- [ ] **T2-99** — Add execution error-recovery paths — Chapter 12.5 "Error recovery".
  - **▸** In `src/execution/mod.rs`, define `pub enum RecoveryStrategy { Retry, Fallback(String), Abort }`. Verify `cargo check --release`. Commit.
  - **▸** In `src/execution/retry.rs`, add `pub struct RetryPolicy { pub max_retries: u32, pub backoff_ms: u64 }` and `pub fn execute_with_retry<F: FnMut() -> Result<ExecutionResult, ExecutionError>>(step: &ExecutionStep, policy: &RetryPolicy, mut f: F) -> Result<ExecutionResult, ExecutionError>`. Verify `cargo check --release`. Commit.
  - **▸** In `src/execution/fallback.rs`, add `pub fn execute_with_fallback(step: &ExecutionStep, primary: &str, fallback: &str) -> Result<ExecutionResult, ExecutionError>` (placeholder: returns Ok with primary unless `primary == "fail"`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn execute_with_recovery(step: &ExecutionStep, strategy: &RecoveryStrategy) -> Result<ExecutionResult, ExecutionError>` orchestrating retry/fallback/abort. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn log_recovery_event(step_id: &str, strategy: &RecoveryStrategy, outcome: &str)` writing to an events log table. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/execution_recovery.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-100** — Add tool-registration contracts distinct from skills — Chapter 13.1 "Tool registration".
  - **▸** In `src/tools/registry.rs`, define `pub struct ToolContract { pub name: String, pub description: String, pub input_schema: serde_json::Value, pub output_schema: serde_json::Value, pub version: String }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn register_tool(contract: ToolContract) -> Result<ToolId, ToolError>`. Verify `cargo check --release`. Commit.
- [ ] **T2-101** — Add tool permissions — Chapter 13.2 "Tool permissions".
  - **▸** Define `pub struct ToolPermission { pub tool_name: String, pub allowed_callers: Vec<String>, pub max_invocations_per_minute: u32 }` in `src/tools/permissions.rs`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn grant(p: ToolPermission)` and `pub fn revoke(tool_name: &str, caller: &str)` to a `ToolPermissionStore`. Verify `cargo check --release`. Commit.
- [ ] **T2-102** — Add tool authorization checks — Chapter 13.2.
  - **▸** Add `pub fn is_authorized(tool: &str, caller: &str) -> bool` checking `allowed_callers`. Verify `cargo check --release`. Commit.
  - **▸** Wire the check into the tool-invocation path. Verify `make gate`. Commit.
- [ ] **T2-103** — Add tool execution isolation rules — Chapter 13.3 "Tool execution".
  - **▸** Define `pub struct IsolationContext { pub working_dir: Option<PathBuf>, pub env_overrides: HashMap<String, String>, pub timeout_ms: u64 }` in `src/execution/isolation.rs`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_isolated<F: FnOnce() -> Result<ExecutionResult, ExecutionError>>(ctx: &IsolationContext, f: F) -> Result<ExecutionResult, ExecutionError>`. Verify `cargo check --release`. Commit.
- [ ] **T2-104** — Add external capability integration rules — Chapter 13.4 "External capability integration".
  - **▸** In `src/tools/external.rs`, define `pub struct ExternalCapability { pub name: String, pub endpoint: String, pub auth_kind: AuthKind, pub required_scopes: Vec<String> }` where `AuthKind` is `None | ApiKey | OAuth | MTls`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn validate_capability(c: &ExternalCapability) -> Result<(), ToolError>` (auth_kind != None if required_scopes non-empty). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/tools_capability.rs`. Wire + verify. Commit.

---

## 8. Model integration, agent communication, and coordination
Finish the cross-cutting v0.0.2 concepts that keep systems replaceable and coordinated. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 14 (Model Integration) + Chapter 15 (Agent Communication) + Chapter 16 (Cognitive Coordination Layer).

- [ ] **T2-105** — Add local-model integration rules under one abstraction — Chapter 14.1 "Local models".
  - **▸** In `src/models/mod.rs`, define `pub trait InferenceProvider { fn name(&self) -> &str; fn complete(&self, prompt: &str, opts: &InferenceOptions) -> Result<InferenceResponse, InferenceError>; fn is_local(&self) -> bool; }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub struct LocalProvider` implementing `InferenceProvider` with `is_local() == true`. Verify `cargo check --release`. Commit.
- [ ] **T2-106** — Add cloud-model integration rules under one abstraction — Chapter 14.2 "Cloud models".
  - **▸** Add `pub struct CloudProvider` implementing `InferenceProvider` with `is_local() == false`. Verify `cargo check --release`. Commit.
- [ ] **T2-107** — Add model-routing rules based on capability instead of provider name — Chapter 14.3 "Model routing".
  - **▸** Add `pub enum Capability { Chat, Embedding, Tool, Vision, LongContext }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn select_provider(registry: &ProviderRegistry, cap: Capability) -> Option<Box<dyn InferenceProvider>>` (placeholder: round-robin). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/models_routing.rs`. Wire + verify. Commit.
- [ ] **T2-109** — Add inference-management rules for scheduling — Chapter 14.5 "Inference management".
  - **▸** Add `pub struct InferenceQueue { /* ... */ }` with `pub fn enqueue(&mut self, req: InferenceRequest) -> RequestId` and `pub fn dequeue(&mut self) -> Option<InferenceRequest>`. Verify `cargo check --release`. Commit.
- [ ] **T2-110** — Add inference-management rules for validation — Chapter 14.5.
  - **▸** Add `pub fn validate_response(resp: &InferenceResponse, schema: &serde_json::Value) -> Result<(), ValidationError>` (placeholder: empty schema → Ok). Verify `cargo check --release`. Commit.
- [ ] **T2-111** — Add inference-management rules for model selection — Chapter 14.3 (selection is its own rule).
  - **▸** Add `pub struct ModelSelector` with `pub fn select_for_task(&self, task: &TaskDescriptor) -> Option<ModelId>` (placeholder: prefer Chat for chat tasks, else LongContext for long inputs). Verify `cargo check --release`. Commit.
- [ ] **T2-112** — Add agent-communication boundaries for MCP concepts — Chapter 15.1 "MCP integration".
  - **▸** In `src/communication/mcp.rs`, add `pub trait McpHandler { fn list_tools(&self) -> Vec<ToolDescriptor>; fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, McpError>; }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn dispatch_to_mcp(server: &dyn McpHandler, req: McpRequest) -> McpResponse`. Verify `cargo check --release`. Commit.
- [ ] **T2-113** — Add agent-communication boundaries for ACP concepts — Chapter 15.2 "ACP concepts".
  - **▸** In `src/communication/acp.rs`, add `pub struct AcpMessage { pub sender: String, pub receiver: String, pub conversation_id: String, pub payload: serde_json::Value, pub kind: AcpMessageKind }` and `pub enum AcpMessageKind { Request, Query, Inform, Subscribe, Response, Ack, Error }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn route_acp(router: &AcpRouter, msg: AcpMessage) -> Result<AcpMessage, AcpError>`. Verify `cargo check --release`. Commit.
- [ ] **T2-114** — Add internal communication rules for subsystem-to-subsystem events — Chapter 15.4 "Internal communication" + Chapter 16.1.
  - **▸** In `src/communication/events.rs`, define `pub struct InternalEvent { pub kind: String, pub source: String, pub correlation_id: String, pub payload: serde_json::Value, pub created_at: i64 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn publish_event(bus: &EventBus, event: InternalEvent)` and `pub fn subscribe(bus: &mut EventBus, kind: &str, handler: Box<dyn Fn(&InternalEvent) + Send + Sync>)`. Verify `cargo check --release`. Commit.
- [ ] **T2-115** — Add cognitive coordination rules for subsystem orchestration — Chapter 16.3 "System orchestration".
  - **▸** In `src/coordination/mod.rs`, define `pub struct Orchestrator { pub event_bus: EventBus, pub providers: ProviderRegistry }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn dispatch(orch: &Orchestrator, event: InternalEvent) -> Result<Vec<InternalEvent>, CoordinationError>` returning follow-up events. Verify `cargo check --release`. Commit.
- [ ] **T2-116** — Add cognitive coordination rules for decision routing — Chapter 16.4 "Decision routing".
  - **▸** Add `pub fn route_decision(orch: &Orchestrator, decision: &Decision) -> Result<ExecutionStep, CoordinationError>` mapping a Decision to an ExecutionStep. Verify `cargo check --release`. Commit.
- [ ] **T2-117** — Add event communication rules that avoid exposing private implementation details — Chapter 16.1 + Chapter 5.2.
  - **▸** Add a doc-test in `src/communication/events.rs` showing the public event schema and an "anti-example" comment about leaking a private field. Verify `cargo check --release`. Commit.
  - **▸** Add `test_suite/src/tests/communication_event_schema.rs`: subscribe to a kind, publish, assert only documented fields present in the received payload. Wire + verify `make gate`. Commit.

---

## 9. Skills, workflows, world model, and personality
Finish the remaining v0.0.2 consumer systems last. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 13 (Tool Engine — skills cross-link), Chapter 11.4 (Workflow generation), Chapter 14.3 (Personality in routing), Chapter 19 (Confidence System).

- [ ] **T2-118** — Add skill permissions in `src/skills/registry/` — Chapter 13.2 (tools/skills share permission model).
  - **▸** Reuse the `ToolPermission` type from T2-101. Add `pub fn grant_skill_permission(skill_id: &str, caller: &str)`. Verify `cargo check --release`. Commit.
- [ ] **T2-119** — Add skill performance tracking in `src/skills/registry/` — Chapter 10.4 (skill improvement is feedback).
  - **▸** Add `pub struct SkillMetrics { pub skill_id: String, pub invocation_count: u64, pub success_count: u64, pub avg_duration_ms: f32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn record_invocation(skill_id: &str, success: bool, duration_ms: u64)`. Verify `cargo check --release`. Commit.
- [ ] **T2-120** — Add skill fallback behavior — Chapter 12.5 (error recovery reuses fallback).
  - **▸** Add `pub fn skill_with_fallback(primary: &str, fallback: &str) -> SkillDescriptor`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/skills_fallback.rs`. Wire + verify. Commit.
- [ ] **T2-121** — Add async, parallel, and retry behavior for skills — Chapter 12.5 + Chapter 11.3 (parallel strategy).
  - **▸** Add `pub enum SkillExecutionMode { Sync, Async, Parallel, Retry(u32) }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub async fn execute_skill(skill_id: &str, mode: SkillExecutionMode, args: serde_json::Value) -> Result<serde_json::Value, SkillError>`. Verify `cargo check --release`. Commit.
- [ ] **T2-122** — Add workflow-level learning in `src/workflows/engine/` — Chapter 10 (learning applies to workflows).
  - **▸** Add `pub fn record_workflow_outcome(workflow_id: &str, success: bool, duration_ms: u64) -> LearningUpdate` mapping to a `TargetKind::Workflow` update. Verify `cargo check --release`. Commit.
- [ ] **T2-123** — Add workflow confidence in `src/workflows/engine/` — Chapter 19.5 "Workflow confidence".
  - **▸** Add `pub struct WorkflowConfidence { pub workflow_id: String, pub confidence: f32, pub sample_size: u32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn update_workflow_confidence(workflow_id: &str, success: bool)` using a Beta-distribution update. Verify `cargo check --release`. Commit.
- [ ] **T2-124** — Add workflow ranking — Chapter 8.5 (memory ranking workflow cross-link).
  - **▸** Add `pub fn rank_workflows(workflows: &[WorkflowConfidence]) -> Vec<(String, f32)>` sorting by confidence desc. Verify `cargo check --release`. Commit.
- [ ] **T2-125** — Add world-model entities aligned with the knowledge graph — Chapter 20 (Knowledge Graph) + Chapter 14 (cognition).
  - **▸** In `src/world_model/mod.rs`, define `pub struct WorldEntity { pub id: String, pub kind: String, pub label: String, pub knowledge_node_id: Option<String> }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn upsert_entity(e: WorldEntity) -> Result<(), WorldError>` and `pub fn link_to_knowledge_node(entity_id: &str, node_id: &str)`. Verify `cargo check --release`. Commit.
- [ ] **T2-126** — Add world-model relationships aligned with confidence-bearing edges — Chapter 19.4 + Chapter 20.3.
  - **▸** Define `pub struct WorldRelationship { pub source_id: String, pub target_id: String, pub kind: String, pub confidence: f32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn add_relationship(r: WorldRelationship) -> Result<(), WorldError>`. Verify `cargo check --release`. Commit.
- [ ] **T2-127** — Add personality traits and emotional-weight handling — Chapter 2 (Core Design Principles cross-link to confidence) + Chapter 19.
  - **▸** In `src/personality/mod.rs`, define `pub struct PersonalityTraits { pub curiosity: f32, pub caution: f32, pub verbosity: f32, pub risk_tolerance: f32, pub thoroughness: f32 }` with defaults 0.5. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn weight_action(traits: &PersonalityTraits, base_score: f32) -> f32` adjusting by risk_tolerance and caution. Verify `cargo check --release`. Commit.
- [ ] **T2-128** — Add personality presets, adaptation, decision-making, and communication rules — Chapter 2 + Chapter 19.
  - **▸** Define `pub enum PersonalityPreset { Balanced, Analytical, Creative, Cautious, Bold }` and `pub fn apply_preset(p: PersonalityPreset) -> PersonalityTraits`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn adapt_traits(traits: &mut PersonalityTraits, feedback: f32)`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn should_act(traits: &PersonalityTraits, confidence: f32) -> bool` (cautious: confidence > 0.7; bold: > 0.3; else base). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/personality_presets.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-129** — Add the v0.0.2 confidence coverage for knowledge, skills, relationships, workflows, and conclusions — Chapter 19 (entire chapter).
  - **▸** Audit: every record type in `src/data_contracts/` carries a `confidence: f32` field. Patch any that don't. Verify `cargo check --release`. Commit.
  - **▸** Audit: every record type carries a `metadata: Metadata` field. Patch any that don't. Verify `cargo check --release`. Commit.
  - **▸** Add `test_suite/src/tests/confidence_coverage.rs` (a reflection test): iterate all `DataContract` types, assert presence of `confidence` and `metadata` via reflection on the schema docs. Wire + verify `make gate`. Commit.

---

## Completion target
End state: finished v0.0.2. All 129 tasks removed from this file. All entries migrated to `.agents/CHANGELOG.md`. Gate stays green throughout. No `#[allow(*)]` in `src/`. No `#[cfg(test)]` in `src/`. No `.unwrap()`/`.expect()` in non-test code. No `todo!()`/`unimplemented!()` anywhere.

---

## APPENDIX: Function Wiring Catalog (from `robot_architecture/RoBoT Architecture v0.0.2.md`)

### Explicit Named Functions (Chapter 24.7 — Function Creation Rules)

| Function | Signature / Status | Wiring / Integration Requirement | Source Chapter |
|---|---|---|---|
| `advanced_reasoning_engine` | `fn advanced_reasoning_engine();` — **INVALID** (no caller, no behavior, no purpose) | Must have: defined purpose, caller, expected inputs/outputs, error behavior, tests/validation path. A function without integration is incomplete. | 24.7 |
| `retrieve_memory_context` | `fn retrieve_memory_context(query: &str) -> Result<Vec<Memory>, Error>` — **VALID** | Wired: `Context Engine` → `Memory System` → `Planner`. Must connect to a real caller (e.g., planner or reasoning engine) and return structured memory records. | 24.7 |

### Pipeline Wiring (Chapter 3.3 — Cognitive Processing Pipeline; Chapter 3.4 — Request Lifecycle)

Every request follows the same lifecycle. Functions must be wired in this order:

```
Observation (Step 1) → Context Construction (Step 2) → Memory Retrieval (Step 3)
  ↓
Experience Retrieval (Step 4) → Planning (Step 5) → Reasoning (Step 6)
  ↓
Skill Selection (Step 7) → Execution (Step 8) → Reflection (Step 9) → Learning (Step 10)
```

| Pipeline Step | Function / Component | Input Source | Output Target | Data Contract Used |
|---|---|---|---|---|
| Step 1 — Observation | Observation System / `Observation` conversion | External sources (user, voice, sensors, docs, APIs, files, tool outputs) | Context Construction | `Observation` (source_kind, content, tags) |
| Step 2 — Context Construction | Context Manager / `ContextPacket` assembly | Observation + conversation history + current goals + planner state + retrieved memories + relevant experiences + environmental observations | Memory Retrieval, Experience Retrieval, Planning | `ContextPacket` (session_id, observations, summary) |
| Step 3 — Memory Retrieval | Memory Retrieval (semantic, graph traversal, keyword, indexed docs, structured facts, workflow retrieval) | Context Construction | Working Memory promotion | `MemoryRecord` (content, importance, confidence, kind) |
| Step 4 — Experience Retrieval | Experience Engine / `ExperienceRecord` search | Context Construction | Planning / Reasoning input | `ExperienceRecord` (goal, plan_id, outcome, success, lessons) |
| Step 5 — Planning | Planner / `Plan` generation (decomposition, dependency analysis, resource estimation, tool selection, risk evaluation, alternative strategies) | Memory Retrieval + Experience Retrieval + Context | Skill Selection | `Plan` (goal, steps), `PlanStep` (id, action, params) |
| Step 6 — Reasoning | Reasoning Engine / `Decision` evaluation | Planning output + retrieved knowledge + experiences + observations + active goals + confidence estimates | Skill Selection | `Decision` (chosen_action, alternatives, confidence, rationale) |
| Step 7 — Skill Selection | Skill System / reusable capability selection | Reasoning output | Execution | Skill descriptor (name, parameters, permissions) |
| Step 8 — Execution | Execution Layer / `ExecutionResult` collection | Skill Selection | Reflection | `ExecutionResult` (step_id, success, output, error, duration_ms) |
| Step 9 — Reflection | Reflection System / outcome evaluation | Execution result | Learning | `Reflection` (experience_ids, insights, confidence) |
| Step 10 — Learning | Learning Engine / retention decision | Reflection output | Memory Storage, Knowledge Graph, Skill Updates | `LearningUpdate` (target_kind, target_id, old_confidence, new_confidence, reason) |

### Subsystem Function Catalog (derived from architecture chapters)

| Subsystem (Chapter) | Key Functions / Components | Wiring Notes |
|---|---|---|
| Context Engine (7) | `conversation_analysis`, `planner_requirements`, `memory_retrieval`, `experience_retrieval`, `knowledge_retrieval`, `context_ranking`, `deduplication`, `compression`, `token_budget_allocation` | Assembled into `ContextPacket`; feeds Memory Retrieval and Planning. Must not load everything into model — selective assembly only. |
| Memory Engine (8) | `working_memory`, `short_term_memory`, `long_term_memory`, `semantic_memory`, `episodic_memory`, `procedural_memory`, `graph_memory`, `memory_objects`, `memory_confidence`, `importance`, `memory_sources`, `memory_retrieval`, `hybrid_retrieval`, `memory_consolidation`, `reinforcement`, `forgetting`, `memory_lifecycle`, `memory_integrity` | Working Memory holds active info; Long-Term Memory stores durable knowledge. Retrieval promotes highest-value info into Working Memory. |
| Experience Engine (9) | `experience_lifecycle`, `experience_objects`, `experience_categories`, `event_based_architecture`, `workflow_learning`, `success_evaluation`, `confidence_updates`, `reputation_system`, `lessons_learned`, `failure_analysis`, `reinforcement_learning`, `experience_graph`, `pattern_discovery`, `skill_development`, `experience_consolidation` | Records observations and outcomes; feeds Learning Engine. Experience is separate from Memory (Chapter 2.4). |
| Learning Engine (10) | `pattern_discovery`, `knowledge_extraction`, `skill_improvement`, `confidence_updates`, `generalization` | Produces `LearningUpdate`; applies to Memory, Knowledge, Skills, Relationships, Workflows. |
| Planning Engine (11) | `goal_creation`, `task_decomposition`, `planning_strategies`, `plan_evaluation`, `workflow_generation`, `dynamic_replanning`, `plan_scoring` | Produces `Plan` and `PlanStep`; feeds Skill Selection. Must validate no cycles (`validate_no_cycles`). |
| Execution Engine (12) | `action_execution`, `external_interactions`, `result_handling`, `error_recovery` | Produces `ExecutionResult`; feeds Reflection. Must handle failures with `RecoveryStrategy` (Retry, Fallback, Abort). |
| Tool Engine (13) | `capability_based_selection`, `strong_contracts`, `stateless_execution`, `isolation`, `observability`, `tool_registry`, `tool_metadata`, `tool_selection`, `parameter_validation`, `execution_pipeline`, `structured_results`, `streaming_support`, `parallel_execution`, `retry_policies`, `timeouts`, `permission_system`, `sandboxing`, `tool_health_monitoring`, `capability_scoring`, `experience_integration`, `memory_integration`, `learning_integration`, `security` | Tools are registered via `ToolContract`; execution is isolated via `IsolationContext`. Must check authorization (`is_authorized`) before invocation. |
| Model Integration (14) | `local_model_integration`, `cloud_model_integration`, `model_routing`, `context_handling`, `inference_management`, `model_selection` | `InferenceProvider` trait abstracts local vs cloud; `Capability` enum (Chat, Embedding, Tool, Vision, LongContext) drives routing. |
| Agent Communication (15) | `mcp_integration`, `acp_concepts`, `internal_communication` | `McpHandler` trait for external callers; `AcpMessage` for agent-to-agent; `InternalEvent` for subsystem events. Events must carry `correlation_id`. |
| Cognitive Coordination (16) | `system_orchestration`, `decision_routing`, `event_communication` | `Orchestrator` dispatches events; `route_decision` maps `Decision` to `ExecutionStep`. Subsystems must NOT call each other directly — emit events only. |
| Memory Architecture (17) | `memory_layers`, `working_memory`, `short_term_memory`, `long_term_memory`, `semantic_memory`, `episodic_memory`, `procedural_memory`, `graph_memory`, `memory_promotion`, `memory_consolidation`, `memory_graph`, `memory_index_cards`, `confidence_model`, `importance_score`, `memory_retrieval`, `retrieval_ranking`, `memory_forgetting`, `memory_lifecycle`, `memory_integrity` | Memory promotion pipeline: Working → Candidate → Accepted → Permanent; any → Archived. `PromotionGate` controls thresholds (min_age, min_confidence, min_access_count). |
| Experience Architecture (18) | `experience_records`, `experience_relationships`, `experience_categories`, `event_based_architecture`, `workflow_learning`, `success_evaluation`, `confidence_updates`, `reputation_system`, `lessons_learned`, `failure_analysis`, `reinforcement_learning`, `experience_graph`, `pattern_discovery`, `skill_development`, `experience_consolidation` | `ExperienceRecord` links to `MemoryRecord` via `plan_id`; `related_experience_ids` creates experience graph. |
| Confidence System (19) | `fact_confidence`, `relationship_confidence`, `experience_confidence`, `skill_confidence`, `workflow_confidence`, `tool_confidence`, `strategy_confidence`, `confidence_data_model`, `confidence_scale`, `confidence_sources`, `confidence_updating`, `confidence_decay`, `contradiction_handling`, `confidence_decision_making`, `confidence_thresholds`, `hypothesis_confidence`, `confidence_memory_promotion`, `confidence_learning_integration`, `confidence_retrieval_integration`, `confidence_planning_integration`, `confidence_tool_execution_injection`, `confidence_history`, `explainable_confidence`, `reputation_integration` | Every numeric score must have `confidence: f32` (0.0–1.0). `Confidence` updates propagate to Memory, Tools, Relationships, Skills, Workflows. `ContradictionHandling` detects conflicts. |
| Knowledge Graph (20) | `graph_structure`, `node_architecture`, `node_types`, `relationship_architecture`, `relationship_confidence`, `relationship_types`, `knowledge_graph_construction`, `graph_extraction_pipeline`, `entity_resolution`, `graph_learning`, `graph_reasoning`, `graph_based_retrieval`, `graph_memory_hierarchy_integration`, `graph_context_lifecycle_integration`, `graph_planning_integration`, `graph_confidence_system_integration`, `graph_contradiction_handling`, `temporal_knowledge`, `knowledge_graph_storage`, `graph_query_types`, `knowledge_graph_maintenance`, `graph_compression`, `explainable_reasoning`, `security_trust_integration` | Nodes (`KnowledgeNode`) and edges (`KnowledgeEdge`) stored in SQLite; `find_path`, `get_subgraph`, `find_linked_concepts`, `find_supporting_evidence` are core queries. `EntityResolution` maps aliases to canonical IDs. |
| Storage Architecture (21) | `working_storage`, `session_storage`, `experience_storage`, `semantic_memory_storage`, `skill_storage`, `knowledge_graph_storage`, `archive_storage`, `operational_storage`, `storage_flow`, `memory_promotion`, `storage_confidence`, `provenance_tracking`, `versioning`, `data_lifecycle`, `storage_cleanup`, `storage_security`, `backup_strategy`, `storage_learning_integration`, `storage_retrieval_integration`, `storage_experience_integration`, `storage_knowledge_graph_integration` | SQLite is primary embedded DB. Hybrid model: relational + vector + graph + object storage. `MemoryPromotion` writes from working storage to long-term storage. `ProvenanceTracking` records `source`, `source_kind`, `created_by`. |
| Database Design (22) | `core_tables`, `system_metadata`, `configuration`, `memory_schema`, `memory_types`, `memory_embeddings`, `memory_relationships`, `knowledge_graph_nodes_edges`, `experience_schema`, `workflow_schema`, `skill_schema`, `lesson_schema`, `learning_schema`, `confidence_history`, `conversation_schema`, `planning_schema`, `execution_schema`, `ai_model_schema`, `model_usage`, `tool_schema`, `architecture_trace_schema`, `diagnostics_schema`, `relationships`, `indexing_strategy`, `data_integrity`, `archiving`, `backup_strategy` | Every table must have `id` (UUID v4), `correlation_id`, `created_at`, `version`. `MemoryEmbeddings` table links to `MemoryRecord`. `KnowledgeGraph` nodes/edges link to `MemoryNode`/`MemoryEdge`. |
| Background Workers (23) | `worker_architecture`, `worker_supervisor`, `task_queue`, `memory_worker`, `experience_worker`, `learning_worker`, `knowledge_graph_worker`, `maintenance_worker`, `worker_scheduling`, `sqlite_worker_coordination`, `worker_failure_handling`, `worker_observability`, `resource_management`, `rust_implementation_direction`, `future_distributed_workers`, `security_trust_integration` | `TaskQueue` has `priority`, `payload`, `memory_id`. `WorkerSupervisor` manages scheduling (Immediate, Scheduled, Resource-Based). SQLite coordination uses `worker_tasks`, `worker_status`, `worker_history` tables. |
| AI Contributor (24) | `advanced_reasoning_engine`, `retrieve_memory_context`, `trace_before_changing`, `minimal_change_principle`, `architecture_alignment_check`, `ai_code_review_checklist`, `human_authority`, `local_ai_contributors`, `multiple_ai_collaboration`, `repository_rules`, `git_change_management`, `documentation_requirement`, `ai_learning_boundary`, `future_autonomous_development`, `robt_development_contract` | Every new function must pass architecture alignment check: Does it belong? Does it connect? Does it improve capability? Does it increase complexity? `TraceBeforeChanging` requires tracing: Function → Callers → Dependencies → Data Flow → Tests → Architecture Purpose. |
| Security and Trust (25) | `security_philosophy`, `trust_model`, `security_layers`, `identity_system`, `permission_architecture`, `capability_based_security`, `memory_protection`, `knowledge_promotion_rules`, `tool_security`, `execution_security`, `ai_contributor_security`, `background_worker_security`, `audit_system`, `trust_evaluation_pipeline`, `risk_classification`, `rollback_and_recovery`, `trust_decay`, `reputation_system`, `security_through_explainability`, `future_self_modification_rules`, `rust_implementation_direction` | `AuditSystem` records: `actor`, `action`, `target`, `confidence_change`, `reason`. `PermissionArchitecture` grants/revokes via `ToolPermission`. `MemoryProtection` prevents unauthorized promotion. `RollbackAndRecovery` must preserve data integrity. |
| Self-Improvement (26) | `evolution_philosophy`, `evolution_ladder`, `self_improvement_loop`, `experience_as_foundation`, `improvement_candidates`, `hypothesis_system`, `controlled_experimentation`, `skill_evolution`, `workflow_evolution`, `memory_evolution`, `knowledge_graph_evolution`, `architecture_evolution`, `evolution_boundaries`, `confidence_based_evolution`, `failure_learning`, `evolution_memory`, `ai_assisted_evolution`, `background_worker_integration`, `evolution_manager`, `rust_implementation_direction`, `future_autonomous_improvement`, `evolution_contract` | `SelfImprovementLoop`: Experience → Improvement Candidates → Hypothesis → Controlled Experimentation → Skill/Workflow/Memory/Graph Evolution → Consolidation. `EvolutionBoundaries` prevent uncontrolled self-modification. `ConfidenceBasedEvolution` only promotes changes with sufficient confidence. |
| Observability (27) | `cognitive_trace_model`, `observability_layers`, `system_metrics`, `cognitive_tracing`, `decision_explanation_layer`, `event_architecture`, `cognitive_event_types`, `trace_storage`, `cognitive_timeline`, `cognitive_visualization_interface`, `debugging_mode`, `production_mode`, `performance_monitoring`, `anomaly_detection`, `trust_integration`, `security_integration`, `ai_contributor_integration`, `background_worker_integration`, `rust_implementation_direction`, `observability_rules`, `future_cognitive_debugger` | `CognitiveTraceModel` tracks every decision with `correlation_id`. `EventArchitecture` defines `MemoryEvents`, `ExperienceEvents`, `PlanningEvents`, `ExecutionEvents`, `EvolutionEvents`. `TraceStorage` uses `CognitiveEvents` and `DecisionRecords` tables. `ObservabilityRules`: explainability must be preserved; no hidden state changes. |
| Developer Interface (28) | `control_plane_architecture`, `interface_types`, `system_overview_dashboard`, `cognitive_explorer`, `memory_management_interface`, `knowledge_graph_explorer`, `worker_management_interface`, `learning_evolution_interface`, `confidence_management`, `security_administration`, `ai_contributor_interface`, `debugging_tools`, `configuration_management`, `control_plane_security`, `remote_management`, `rust_implementation_direction`, `developer_workflow`, `future_cognitive_development_environment` | `ControlPlaneArchitecture` provides CLI (`CommandLineInterface`), Dashboard (`DeveloperDashboard`), and API (`APIInterface`). `CognitiveExplorer` allows inspecting memory, graph, and experience. `DebuggingTools` include `TraceReplay`, `StateInspection`, `EventSearch`. `ControlPlaneSecurity` requires authentication and authorization. |
| Configuration (29) | `configuration_philosophy`, `configuration_layers`, `system_configuration`, `user_configuration`, `runtime_configuration`, `configuration_sources`, `configuration_files`, `example_configuration`, `runtime_profiles`, `startup_sequence`, `environment_validation`, `hardware_awareness`, `model_runtime_management`, `database_runtime_management`, `worker_runtime_management`, `feature_flags`, `runtime_state`, `state_persistence`, `configuration_validation`, `hot_reloading`, `configuration_security`, `control_plane_integration`, `rust_implementation_direction`, `runtime_manager`, `shutdown_sequence`, `recovery_and_restart`, `future_runtime_evolution` | `SystemConfiguration`: `memory_enabled`, `learning_enabled`, `planning_enabled`, `workers_enabled`. `WorkerRuntimeManagement`: `memory_worker` (priority normal), `learning_worker` (priority low). `StartupSequence`: initialize database → load configuration → start workers → connect to model providers. `ShutdownSequence`: stop workers → flush storage → close database connections → save state. `RecoveryAndRestart`: restore from last persistent state; replay events if needed. |
| Testing (30) | `testing_philosophy`, `validation_layers`, `unit_testing`, `integration_testing`, `architecture_validation`, `ai_generated_code_validation`, `cognitive_testing`, `memory_validation`, `knowledge_graph_validation`, `experience_system_validation`, `planning_validation`, `execution_validation`, `confidence_system_validation`, `learning_validation`, `evolution_testing`, `regression_testing`, `replay_testing`, `benchmark_system`, `failure_testing`, `performance_testing`, `security_testing`, `test_data_management`, `continuous_validation`, `rust_implementation_direction`, `validation_database`, `developer_workflow`, `testing_the_architecture_itself` | `MemoryValidation`: storage, ranking, consolidation, promotion. `KnowledgeGraphValidation`: node/edge integrity, traversal correctness. `ExperienceValidation`: recording, scoring, reputation updates. `PlanningValidation`: goal validation, dependency analysis, cycle detection. `ExecutionValidation`: tool invocation, isolation, error recovery. `ConfidenceValidation`: score accuracy, decay behavior, contradiction detection. `LearningValidation`: pattern detection, knowledge extraction, skill improvement. `EvolutionTesting`: controlled experimentation, boundary enforcement. `BenchmarkSystem`: memory benchmark, reasoning benchmark, tool benchmark, learning benchmark. `ReplayTesting`: deterministic replay of cognitive events. `ArchitectureTraceValidation`: verify event flow matches architecture spec. |
| Deployment (31) | `deployment_purpose`, `deployment_goals`, `high_level_deployment_architecture`, `deployment_philosophy`, `supported_platforms`, `directory_structure`, `bootstrap_process`, `bootstrap_manager`, `initialization_order`, `configuration_management`, `environment_detection`, `ai_model_deployment`, `database_deployment`, `database_migration`, `plugin_deployment`, `mcp_integration`, `runtime_validation`, `logging_infrastructure`, `health_monitoring`, `backup_architecture`, `recovery`, `update_architecture`, `resource_management`, `offline_operation`, `graceful_shutdown`, `deployment_diagnostics`, `continuous_deployment`, `future_expansion`, `success_criteria` | `BootstrapProcess`: detect environment → load config → initialize DB → deploy models → start workers → validate health. `InitializationOrder`: database → storage → memory → experience → learning → planning → execution → tool → model → communication → coordination. `OfflineOperation`: all cognitive functions must work without network; only cloud model calls require network. `GracefulShutdown`: complete current execution step → flush events → save state → close connections. `HealthMonitoring`: check database connectivity, worker status, model availability, memory usage. `BackupArchitecture`: full brain snapshot (database + embeddings + model weights + configuration). |
| Future Expansion (32) | `future_expansion_purpose`, `vision`, `core_expansion_principles`, `layered_growth_model`, `stable_core_philosophy`, `modular_expansion`, `ai_model_evolution`, `advanced_memory_evolution`, `knowledge_graph_evolution`, `learning_evolution`, `experience_evolution`, `reasoning_evolution`, `vision_expansion`, `audio_expansion`, `robotics_expansion`, `sensor_expansion`, `tool_ecosystem_growth`, `distributed_architecture`, `collaboration_architecture`, `personalization_evolution`, `explainability_expansion`, `architecture_trace_evolution`, `security_evolution`, `performance_evolution`, `deployment_evolution`, `research_platform`, `architectural_governance`, `long_term_roadmap`, `success_criteria` | `LayeredGrowthModel`: stable core (memory, experience, knowledge) → cognitive architecture (context, planning, reasoning) → intelligence infrastructure (tools, models, communication) → governance (security, observability, evolution) → interfaces (developer, deployment, future). `ModularExpansion`: new capabilities must implement existing traits (`InferenceProvider`, `McpHandler`, `AcpMessage`, etc.) rather than creating new interfaces. `StableCorePhilosophy`: core pipeline (Observe → Understand → Retrieve → Plan → Reason → Act → Reflect → Learn) must never change; only implementations evolve. |

### Wiring Rules (from Chapter 24 — AI Contributor Operating Agreement)

Every function must pass the architecture alignment check before being added:

1. **Does it belong?** — Must map to a subsystem defined in the architecture (Context, Memory, Experience, Learning, Planning, Execution, Tool, Model, Communication, Coordination, Security, Observability, Developer Interface, Configuration, Deployment).
2. **Does it connect?** — Must have a caller and must emit/consume events through the event bus (`InternalEvent`) or through the contract layer (`DataContract` types). Direct cross-subsystem imports are forbidden.
3. **Does it improve capability?** — Must solve a real problem documented in the architecture (e.g., retrieval accuracy, confidence tracking, failure recovery, explainability).
4. **Does it increase complexity?** — If yes, must include a simplification plan (e.g., consolidate duplicate retrieval strategies, remove deprecated memory layers).

Every function must also follow the trace-before-changing protocol:

```
Function → Callers → Dependencies → Data Flow → Tests → Architecture Purpose
```

No function may be changed based only on its name. Every change must be verified against the architecture purpose.


## gaps to be fixed

---

## Gaps — Actionable tasks (continuing from T2-129)

Each `▸` is one 5-minute increment: one file/function/test → `cargo check --release` or `make gate` → commit.

---

## Part I — Vision and Foundation (Ch 01-05)

- [x] **T2-130** — Wire the 10-step cognitive lifecycle pipeline — Chapter 3.4.
  - **▸** In `src/pipeline/mod.rs`, define `pub enum LifecycleStep { Observation, ContextConstruction, MemoryRetrieval, ExperienceRetrieval, Planning, Reasoning, SkillSelection, Execution, Reflection, Learning }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub struct CognitivePipeline { pub steps: Vec<LifecycleStep>, pub correlation_id: String }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_pipeline(p: &CognitivePipeline) -> Result<PipelineTrace, PipelineError>` (placeholder: iterate steps, return a trace). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/pipeline_lifecycle.rs`: create pipeline with all 10 steps, assert trace length == 10. Wire + verify `make gate`. Commit.

- [x] **T2-131** — Wire the 9-stage context assembly pipeline — Chapter 7.6.
  - **▸** In `src/context_engine/mod.rs`, define `pub enum AssemblyStage { ConversationAnalysis, PlannerRequirements, MemoryRetrieval, ExperienceRetrieval, KnowledgeRetrieval, ContextRanking, Deduplication, Compression, TokenBudgetAllocation }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_assembly(stages: &[AssemblyStage], correlation_id: &str) -> ContextAssembly` calling each stage in order. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/context_assembly_pipeline.rs`: 9 stages → assert `ContextAssembly.layers.len() == 9`. Wire + verify. Commit.

- [x] **T2-132** — Wire data contracts through the pipeline — Chapter 5.1 + 3.3.
  - **▸** In `src/data_contracts/mod.rs`, add `pub fn contract_for_step(step: LifecycleStep) -> &'static str` mapping each step to its contract name (`Observation`, `ContextPacket`, etc.). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn validate_contract_chain(trace: &PipelineTrace) -> bool` checking that each step's output contract matches the next step's input contract. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/data_contract_chain.rs`. Wire + verify `make gate`. Commit.

---

## Part II — Cognitive Architecture (Ch 06-12)

- [x] **T2-133** — Complete Conversation Engine lifecycle — Chapter 6.
  - **▸** In `src/conversation/mod.rs`, add `pub fn process_full_lifecycle(session: &mut ConversationSession, input_id: &str) -> ConversationState` advancing through `Analyzing → AssemblingContext → Processing → Responding → Completed`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn track_topic(session: &ConversationSession, topic: &str) -> bool` (placeholder: return true if topic changed). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/conversation_lifecycle.rs`. Wire + verify. Commit.

- [ ] **T2-134** — Complete Context Engine assembly pipeline — Chapter 7.
  - **▸** In `src/context_engine/mod.rs`, add `pub fn conversation_analysis(input: &str) -> Vec<String>` (placeholder: split input into words). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn planner_requirements(goal: &str) -> Vec<String>` (placeholder: return `vec!["knowledge".to_string()]`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn context_ranking(items: &[String]) -> Vec<(String, f32)>` (placeholder: score by length). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn deduplicate(items: &[String]) -> Vec<String>` (placeholder: use `HashSet`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn compress_context(items: &[String], budget: usize) -> Vec<String>` (placeholder: truncate to budget). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/context_engine_full.rs`: 9 stages → assert `ContextAssembly` has references/goals/layers/constraints. Wire + verify `make gate`. Commit.

- [x] **T2-134** — Complete Memory Engine promotion/consolidation — Chapter 8 + 17.
  - **▸** In `src/memory/permanent.rs`, add `pub fn promote_to_permanent(item: MemoryItem) -> Result<String, MemoryError>` (reuse existing `promote_research` logic). Verify `cargo check --release`. Commit.
  - **▸** In `src/memory/retrieval.rs`, add `pub fn retrieve_for_context(query: &str, budget: usize) -> Vec<MemoryRecord>` (placeholder: return first `budget` items). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_promotion.rs`. Wire + verify. Commit.

- [x] **T2-136** — Complete Experience Engine workflow/reputation — Chapter 9.
  - **▸** In `src/experience/reputation.rs`, add `pub fn update_reputation(tool_name: &str, success: bool) -> f32` (placeholder: +0.03 for success, -0.05 for failure). Verify `cargo check --release`. Commit.
  - **▸** In `src/experience/coordinator.rs`, add `pub fn evaluate_workflow(outcome: &str) -> f32` (placeholder: 1.0 for "success", 0.0 for "failure"). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/experience_reputation.rs`. Wire + verify. Commit.

- [ ] **T2-137** — Complete Learning Engine pipeline — Chapter 10.
  - **▸** In `src/learning/mod.rs`, add `pub fn pattern_discovery(experiences: &[ExperienceRecord]) -> Vec<Pattern>` (placeholder: group by `experience_type`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn extract_knowledge(patterns: &[Pattern]) -> Vec<ExtractedKnowledge>` (placeholder: generate rule string). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn skill_improvement(skill_id: &str, delta: f32) -> SkillImprovement`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_pipeline.rs`. Wire + verify `make gate`. Commit.

- [x] **T2-138** — Complete Planning Engine dependency/strategy — Chapter 11.
  - **▸** In `src/planner/mod.rs`, add `pub fn validate_no_cycles(steps: &[PlanStep]) -> bool` (DFS). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn topological_sort(steps: &[PlanStep]) -> Option<Vec<String>>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub enum PlanningStrategy { Sequential, Parallel, Greedy }` and `pub fn select_strategy(goal: &Goal) -> PlanningStrategy`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_dag.rs`. Wire + verify. Commit.

- [x] **T2-139** — Complete Execution Engine recovery/isolation — Chapter 12.
  - **▸** In `src/execution/mod.rs`, add `pub enum RecoveryStrategy { Retry, Fallback(String), Abort }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub struct IsolationContext { pub working_dir: Option<PathBuf>, pub env_overrides: HashMap<String, String>, pub timeout_ms: u64 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn execute_with_recovery(step: &ExecutionStep, strategy: &RecoveryStrategy) -> Result<ExecutionResult, ExecutionError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/execution_recovery.rs`. Wire + verify `make gate`. Commit.

---

## Part III — Intelligence Infrastructure (Ch 13-16)

- [x] **T2-140** — Complete Tool Engine contracts/permissions/isolation — Chapter 13.
  - **▸** In `src/tools/registry.rs`, add `pub struct ToolContract { pub name: String, pub description: String, pub input_schema: serde_json::Value, pub output_schema: serde_json::Value, pub version: String }`. Verify `cargo check --release`. Commit.
  - **▸** In `src/tools/permissions.rs`, add `pub struct ToolPermission { pub tool_name: String, pub allowed_callers: Vec<String>, pub max_invocations_per_minute: u32 }`. Verify `cargo check --release`. Commit.
  - **▸** In `src/execution/isolation.rs`, add `pub fn run_isolated<F: FnOnce() -> Result<ExecutionResult, ExecutionError>>(ctx: &IsolationContext, f: F) -> Result<ExecutionResult, ExecutionError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/tool_contract.rs`. Wire + verify. Commit.

- [x] **T2-141** — Complete Model Integration routing/queue — Chapter 14.
  - **▸** In `src/models/mod.rs`, add `pub enum Capability { Chat, Embedding, Tool, Vision, LongContext }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn select_provider(registry: &ProviderRegistry, cap: Capability) -> Option<Box<dyn InferenceProvider>>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub struct InferenceQueue { /* ... */ }` with enqueue/dequeue. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/model_routing.rs`. Wire + verify. Commit.

- [ ] **T2-142** — Complete Agent Communication event bus — Chapter 15 + 16.
  - **▸** In `src/communication/events.rs`, add `pub struct InternalEvent { pub kind: String, pub source: String, pub correlation_id: String, pub payload: serde_json::Value, pub created_at: i64 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn publish_event(bus: &EventBus, event: InternalEvent)` and `pub fn subscribe(...)`. Verify `cargo check --release`. Commit.
  - **▸** In `src/coordination/mod.rs`, add `pub fn route_decision(orch: &Orchestrator, decision: &Decision) -> Result<ExecutionStep, CoordinationError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/event_bus.rs`. Wire + verify `make gate`. Commit.

---

## Part IV — Memory and Knowledge (Ch 17-23)

- [ ] **T2-143** — Complete Memory Architecture layers/promotion — Chapter 17.
  - **▸** In `src/memory_hierarchy/mod.rs`, add `pub enum MemoryLayer { Working, ShortTerm, LongTerm, Semantic, Episodic, Procedural, Graph, Archive }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub struct PromotionGate { pub min_age_hours: u64, pub min_confidence: f32, pub min_access_count: u32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn should_promote(memory: &MemoryRecord, gate: &PromotionGate) -> bool`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_promotion_gate.rs`. Wire + verify. Commit.

- [ ] **T2-144** — Complete Experience Architecture consolidation — Chapter 18.
  - **▸** In `src/experience/types/experience.rs`, add `pub fn consolidate_experience(exp: &ExperienceRecord) -> Option<MemoryRecord>` (placeholder: return `Some` if `success == true`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn build_experience_graph(experiences: &[ExperienceRecord]) -> Vec<(String, String, String)>` (placeholder: link by `plan_id`). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/experience_consolidation.rs`. Wire + verify. Commit.

- [ ] **T2-145** — Complete Confidence System domains/decay — Chapter 19.
  - **▸** In `src/data_contracts/confidence.rs` (or new file), add `pub enum ConfidenceDomain { Fact, Relationship, Experience, Skill, Workflow, Tool, Strategy }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn decay_confidence(current: f32, hours_since_update: f64, decay_rate: f32) -> f32` using `current * 0.5_f32.powf(...)`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn detect_contradiction(a: f32, b: f32, threshold: f32) -> bool` (placeholder: `abs(a - b) > threshold`). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/confidence_decay.rs`. Wire + verify `make gate`. Commit.

- [ ] **T2-146** — Complete Knowledge Graph extraction/reasoning — Chapter 20.
  - **▸** In `src/knowledge/graph.rs`, add `pub fn traverse_from(conn: &Connection, start_id: &str, max_depth: usize) -> Result<Vec<KnowledgeEdge>, rusqlite::Error>` (BFS). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn find_all_paths(conn: &Connection, start: &str, end: &str, max_paths: usize) -> Result<Vec<Vec<String>>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn get_subgraph(conn: &Connection, node_id: &str, radius: usize) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>), rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_traversal.rs`. Wire + verify `make gate`. Commit.

- [ ] **T2-147** — Complete Storage Architecture layers/backup — Chapter 21.
  - **▸** In `src/database/mod.rs`, add `pub enum StorageLayer { Working, Session, Experience, Semantic, Skill, Graph, Archive, Operational }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn backup_database(path: &str) -> Result<(), DatabaseError>` (placeholder: copy file). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn restore_database(source: &str, target: &str) -> Result<(), DatabaseError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/storage_backup.rs`. Wire + verify. Commit.

- [ ] **T2-148** — Complete Database Design full schema — Chapter 22.
  - **▸** In `src/database/migrations/`, add migration for `experience_schema` table (`id`, `goal`, `plan_id`, `result`, `success`, `execution_time`, `cost`, `confidence_change`, `tool_usage`, `lessons`, `timestamp`). Verify `cargo check --release`. Commit.
  - **▸** Add migration for `workflow_schema` (`id`, `name`, `steps`, `dependencies`, `required_skills`, `estimated_cost`, `estimated_confidence`, `alternative_branches`). Verify `cargo check --release`. Commit.
  - **▸** Add migration for `confidence_history` (`id`, `item_id`, `old_confidence`, `new_confidence`, `reason`, `timestamp`). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/db_schema_full.rs`: assert all 3 tables exist. Wire + verify. Commit.

- [ ] **T2-149** — Complete Background Workers scheduling/coordination — Chapter 23.
  - **▸** In `src/cooboploop/`, add `pub struct TaskQueue { pub priority: u8, pub payload: String, pub memory_id: Option<String> }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub enum WorkerType { Memory, Experience, Learning, KnowledgeGraph, Maintenance }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn schedule_task(queue: &mut TaskQueue, worker: WorkerType, payload: String)`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/worker_scheduling.rs`. Wire + verify `make gate`. Commit.

---

## Part V — Governance and Evolution (Ch 24-27)

- [ ] **T2-150** — Implement AI Contributor Agreement checks — Chapter 24.
  - **▸** In `src/developer_interface/`, add `pub fn architecture_alignment_check(name: &str) -> bool` (placeholder: check if name contains valid subsystem keywords). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn trace_before_changing(function: &str) -> String` returning the trace string `"Function → Callers → Dependencies → Data Flow → Tests → Architecture Purpose"`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/ai_contributor_check.rs`. Wire + verify. Commit.

- [ ] **T2-151** — Implement Security and Trust audit/permission — Chapter 25.
  - **▸** In `src/security/` (new dir or existing), add `pub struct AuditRecord { pub actor: String, pub action: String, pub target: String, pub confidence_change: f32, pub reason: String }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn log_audit(record: AuditRecord) -> Result<(), SecurityError>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn check_permission(actor: &str, action: &str, target: &str) -> bool`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/security_audit.rs`. Wire + verify `make gate`. Commit.

- [ ] **T2-152** — Implement Self-Improvement loop — Chapter 26.
  - **▸** In `src/evolution/`, add `pub enum EvolutionStage { Experience, Candidate, Hypothesis, Experiment, SkillEvolution, WorkflowEvolution, MemoryEvolution, GraphEvolution, Consolidation }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_evolution_loop(experiences: &[ExperienceRecord]) -> Vec<LearningUpdate>` (placeholder: return empty). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/evolution_loop.rs`. Wire + verify. Commit.

- [ ] **T2-153** — Implement Observability event architecture — Chapter 27.
  - **▸** In `src/observability/`, add `pub enum CognitiveEventType { MemoryEvent, ExperienceEvent, PlanningEvent, ExecutionEvent, EvolutionEvent }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn record_cognitive_event(event_type: CognitiveEventType, correlation_id: &str, payload: serde_json::Value) -> Result<(), ObservabilityError>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn replay_events(correlation_id: &str) -> Vec<CognitiveEvent>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/observability_events.rs`. Wire + verify `make gate`. Commit.

---

## Part VI — Interfaces and Operations (Ch 28-32)

- [ ] **T2-154** — Implement Developer Interface control plane — Chapter 28.
  - **▸** In `src/developer_interface/`, add `pub struct ControlPlaneArchitecture { pub cli: CommandLineInterface, pub dashboard: DeveloperDashboard, pub api: APIInterface }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn inspect_memory(entity_id: &str) -> Option<MemoryRecord>` (placeholder: return `None`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn replay_trace(correlation_id: &str) -> Vec<TraceEvent>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/developer_interface.rs`. Wire + verify. Commit.

- [ ] **T2-155** — Implement Configuration runtime profiles — Chapter 29.
  - **▸** In `src/config/` (or `Cargo.toml` config), add `pub enum RuntimeProfile { Development, Testing, Production }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn load_config(profile: RuntimeProfile) -> Config`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn validate_config(config: &Config) -> Result<(), ConfigError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/config_profiles.rs`. Wire + verify. Commit.

- [ ] **T2-156** — Implement Testing architecture validation — Chapter 30.
  - **▸** In `test_suite/src/`, add `pub fn architecture_validation() -> bool` (placeholder: return `true`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn cognitive_testing() -> bool`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn replay_testing(trace: &PipelineTrace) -> bool`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/architecture_validation.rs`. Wire + verify `make gate`. Commit.

- [ ] **T2-157** — Implement Deployment bootstrap/init order — Chapter 31.
  - **▸** In `src/deployment/` (new dir), add `pub enum BootstrapStep { DatabaseInit, StorageInit, MemoryInit, ExperienceInit, LearningInit, PlanningInit, ExecutionInit, ToolInit, ModelInit, CommunicationInit, CoordinationInit }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_bootstrap() -> Result<BootstrapResult, DeploymentError>` (placeholder: iterate steps). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn graceful_shutdown() -> Result<(), DeploymentError>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/deployment_bootstrap.rs`. Wire + verify. Commit.

- [ ] **T2-158** — Implement Future Expansion stable core — Chapter 32.
  - **▸** In `src/architecture/` (new dir or `lib.rs`), add `pub const STABLE_CORE_PIPELINE: &[&str] = &["Observe", "Understand", "Retrieve", "Plan", "Reason", "Act", "Reflect", "Learn"];`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn verify_stable_core() -> bool` (placeholder: return `true`). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/future_expansion.rs`. Wire + verify. Commit.

---

## Appendix — Wiring Catalog (Ch 24.7, 3.3, 3.4, Subsystem Catalog)

- [ ] **T2-159** — Enforce architecture alignment check in code — Chapter 24.
  - **▸** In `src/developer_interface/`, add `pub fn enforce_alignment_check(name: &str) -> Result<(), AlignmentError>` that validates the function name maps to a known subsystem. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn enforce_trace_protocol(function: &str) -> String` returning the required trace format. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/alignment_check.rs`. Wire + verify `make gate`. Commit.

- [ ] **T2-160** — Wire pipeline wiring rules — Chapter 3.3 + 3.4.
  - **▸** In `src/pipeline/mod.rs`, add `pub fn verify_pipeline_order(trace: &PipelineTrace) -> bool` checking that steps follow the 10-step lifecycle order. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn verify_context_assembly_order(assembly: &ContextAssembly) -> bool` checking 9-stage order. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/pipeline_wiring.rs`. Wire + verify `make gate`. Commit.
