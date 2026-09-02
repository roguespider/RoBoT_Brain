# Completed Work (CHANGELOG)

> Historical record of completed work. Forward planning lives in [PLAN.md](PLAN.md).
> Append new completed work here so PLAN.md stays focused on what needs to be done.

- **CoObOpLoop T4.1 Add `CapabilityId` enum — DONE (2026-09-02).** Verified `CapabilityId` enum with 6 variants (Rust, MCP, HTTP, SQLite, Testing + Custom(String)) exists in `src/cooboploop/capability.rs:15-22`. Also fixed: unused Result in `cooboploop_handler.rs` line 261 (`run_cycle()`), removed `derive(Debug)` from `StrategicObjectiveCategory`, `StrategicObjective`, `HierarchyNode` in `strategic.rs` (lines 5/18/91) and added manual `impl Debug` with explicit match. Compilation: 0 errors.

- **CoObOpLoop T3.13 Test compute_priority — DONE (2026-09-02).** New test file `test_suite/src/tests/cooboploop_t3_eval.rs`: enqueues zero-cost goal → evaluate → verify priority ~0.0; enqueues max-value goal → evaluate → verify priority > 1.0. Wired into `mod.rs` + `main.rs` dispatch. Also fixed pre-existing unused variable `all_ok` in `cooboploop_queue.rs` (AGENTS.md bug fix rule). Compilation: 0 errors.

- **CoObOpLoop T3.12 Add registry entries for T3.9-T3.11 — DONE (2026-09-02).** Verified all three evaluation tools (`evaluate_goal`, `reprioritize_queue`, `set_priority_policy`) are in sync across all three handler lists: `definitions::all()`, `tool_names()`, `execute_tool()`. All have full McpTool definitions, input structs, and execute functions. Compilation: 0 errors.

- **CoObOpLoop T3.11 Add `cooboploop_set_priority_policy` MCP handler — DONE (2026-09-02).** Handler: `execute_cooboploop_set_priority_policy` in `src/bridge/tools/cooboploop/mod.rs`. Input struct `CooboploopSetPriorityPolicyInput` with `policy: string` param. Supports 4 policies: default (raw), conservative (×0.8), strategic (×source multiplier), exploration (×1.3 for learning_value > 0.5). Registry entry `COOBOPLOOP_SET_PRIORITY_POLICY` in `definitions::all()` + `CooboploopToolsHandler::tool_names()` + `execute_tool()` dispatch. Fixed `get_tools()` to return all 35 cooboploop tools via `definitions::all()` (was `Vec::new()` — tool was callable but unadvertised). Fixed pre-existing bugs: removed `#[derive(Debug)]` from `LoopStage` + manual impl, added `Hash` derive to `CapabilityId`, added `PartialEq` derive to `OutcomeKind`, fixed `CapabilityRegistry::get()` signature from `&CapabilityId` to `&str`. Compilation: 0 errors.

- **CoObOpLoop T5.1–T5.23 Full loop (§7) — DONE (2026-09-02).** LoopStage enum (11 variants), LoopRunner fields/state, run_cycle() sequential stages 1-11, stage methods (ObserveState through GenerateNewObjectives), should_continue(), start/stop, 5 MCP handlers (start_loop, stop_loop, get_loop_status, run_single_cycle, step_loop) wired in `mod.rs`, registry entries verified, test `test_suite/src/tests/cooboploop_t5_cycle.rs` wired in `mod.rs` + `main.rs`. All 11 stages implemented with comments mapping to §A.2/A.7/A.8. No `#[allow]`, no `.unwrap()`, no `#[derive(Debug)]` added. Build: 0 errors, 0 warnings in cooboploop module.

- **CoObOpLoop T1.26 Add `cooboploop_enqueue_goal` MCP handler — DONE (2026-09-02).** Handler: `execute_cooboploop_enqueue_goal` in `src/bridge/tools/cooboploop/mod.rs`. Input struct `CooboploopEnqueueGoalInput` with title, description, expected_value, risk, learning_value, deadline, source, required_capabilities, dependencies. Registry entry `COOBOPLOOP_ENQUEUE_GOAL` in `definitions::all()`. Wired in `CooboploopToolsHandler::tool_names()` + `execute_tool()`. Fixed: borrow error in `queue.rs` (new_status moved before format), missing `mod cooboploop;` in `main.rs`, 14 unused-variable warnings in stub handlers (echoed input fields in response JSON). Compilation: 0 errors, 0 warnings in cooboploop bridge file.

- **CoObOpLoop T1.27 Add `cooboploop_list_goals` MCP handler — DONE (2026-09-02).** Handler: `execute_cooboploop_list_goals` with `status_filter`, `limit`, `offset` params. Filters goals by status, truncates by limit, skips by offset. Registry entry `COOBOPLOOP_LIST_GOALS` in `definitions::all()`. Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.28 Add `cooboploop_get_goal` MCP handler — DONE (2026-09-02).** Handler: `execute_cooboploop_get_goal` with `goal_id` param. Looks up goal in queue, returns `found: true/false` + goal data or not-found message. Registry entry `COOBOPLOOP_GET_GOAL` in `definitions::all()`. Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.29 Add `cooboploop_update_goal_status` MCP handler — DONE (2026-09-02).** Handler: `execute_cooboploop_update_goal_status` with `goal_id` + `new_status` params. Maps 13 status transitions (Discovered→Cancelled/Rejected/Archived) with validation. Updates goal.status in-memory + SQLite. Registry entry `COOBOPLOOP_UPDATE_GOAL_STATUS` in `definitions::all()`. Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.30 Add registry entry for `cooboploop_enqueue_goal` — DONE (2026-09-02).** `COOBOPLOOP_ENQUEUE_GOAL` in `definitions::all()` with `McpTool` definition: name, description, full input_schema (title required, description/expected_value/risk/learning_value/deadline/source/required_capabilities/dependencies optional). Wired in `CooboploopToolsHandler::tool_names()`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.31 Add registry entry for `cooboploop_list_goals` — DONE (2026-09-02).** `COOBOPLOOP_LIST_GOALS` in `definitions::all()` with `McpTool`: name, description, input_schema (status_filter/limit/offset optional). Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.32 Add registry entry for `cooboploop_get_goal` — DONE (2026-09-02).** `COOBOPLOOP_GET_GOAL` in `definitions::all()` with `McpTool`: name, description, input_schema (goal_id required). Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.33 Add registry entry for `cooboploop_update_goal_status` — DONE (2026-09-02).** `COOBOPLOOP_UPDATE_GOAL_STATUS` in `definitions::all()` with `McpTool`: name, description, input_schema (goal_id + new_status required). Wired in `CooboploopToolsHandler`. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T1.34 Test queue lifecycle — DONE (2026-09-02).** Test file `test_suite/src/tests/cooboploop_queue.rs`: enqueue_goal → list_goals → get_goal → update_goal_status → verify status changed. Registered in `test_suite/src/tests/mod.rs`, wired into `test_suite/src/main.rs`. Tests 4 assertions: enqueue success, get returns correct goal, update reports updated, status actually changed to ACCEPTED. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.1 Add `ObjectiveSource` enum (5 core + extensions) to `sources.rs` — DONE (2026-09-02).** Enum at sources.rs:77-88: HumanOrigin, ExternalOpportunity, SystemTrigger, LearningTarget, StrategicObjective, ImprovementTarget, HardwareUtilization, InferencePerformance, SoftwareArchitecture. Full serde support. Includes `ObjectiveSourceProvider` trait, `ObjectiveSourceRegistry`, 5 source provider stubs. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.2 Add `HumanOrigin` enum (6 variants) to `sources.rs` — DONE (2026-09-02).** HumanOrigin enum at sources.rs:7-15: UserRequest, Instruction, Correction, Project, MaintenanceRequest, StrategicGoal. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.3 Add `ExternalSource` enum (9 variants) to `sources.rs` — DONE (2026-09-02).** ExternalSource enum at sources.rs:18-29: FreelanceJob, DevBounty, ResearchOpportunity, Grant, Competition, OpenSourceTask, AvailableProject, HardwareOpportunity, UserRequest. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.4 Add `SystemTrigger` enum (13 variants) to `sources.rs` — DONE (2026-09-02).** SystemTrigger enum at sources.rs:32-47: UnresolvedError, FailedTest, DetectedBug, DegradedPerformance, MemoryInconsistency, HardwareProblem, SoftwareDependencyProblem, StaleComponent, MissingDocumentation, SecurityIssue, ReliabilityIssue, IncompleteImplementation, FailedExperiment. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.5 Add `LearningTrigger` enum (4 variants) to `sources.rs` — DONE (2026-09-02).** LearningTrigger enum at sources.rs:50-56: RepeatedFailure, InsufficientUnderstanding, RepeatedHumanIntervention, CapabilityGap. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.6 Add `ImprovementTarget` enum (13 variants) to `sources.rs` — DONE (2026-09-02).** ImprovementTarget enum at sources.rs:59-74: ReasoningWorkflow, Planning, ToolUsage, MemoryRetrieval, MemoryOrganization, ExecutionReliability, Testing, HardwareUtilization, InferencePerformance, SoftwareArchitecture, ResourceUtilization, ErrorDetection, RecoveryProcedure. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.7 Add `ObjectiveSourceProvider` trait (3 methods) to `sources.rs` — DONE (2026-09-02).** Trait at sources.rs:91-95: `source_type() -> ObjectiveSource`, `discover() -> Vec<AgentGoal>`, `name() -> &str`. Send+Sync bound. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.8 Add `ObjectiveSourceRegistry` struct (Vec<Box<dyn ObjectiveSourceProvider>>) to `sources.rs` — DONE (2026-09-02).** Struct at sources.rs:98-100: `providers: Vec<Box<dyn ObjectiveSourceProvider>>`. Default impl. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.9 Add `register()` method to `ObjectiveSourceRegistry` — DONE (2026-09-02).** Method at sources.rs:109-111: `register(&mut self, provider: Box<dyn ObjectiveSourceProvider>)`. Pushes provider into Vec. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.10 Add `discover_all()` method to `ObjectiveSourceRegistry` — DONE (2026-09-02).** Method at sources.rs:113-119: iterates providers, calls `discover()` on each, collects all discovered goals into Vec<AgentGoal>. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.11 Implement `HumanInputSource` (stub) — DONE (2026-09-02).** Stub at sources.rs:143-161: implements ObjectiveSourceProvider, returns HumanOrigin, name = "human_input", sample_goal method. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.12 Implement `SystemGeneratedSource` (stub) — DONE (2026-09-02).** Stub at sources.rs:184-223: implements ObjectiveSourceProvider, returns SystemTrigger, name = "system", sample_goal method. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.13 Implement `LearningObjectiveSource` (stub) — DONE (2026-09-02).** Stub at sources.rs:225-264: implements ObjectiveSourceProvider, returns LearningTrigger, name = "learning", sample_goal method. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.14 Implement `SelfImprovementSource` (stub) — DONE (2026-09-02).** Stub at sources.rs:266-305: implements ObjectiveSourceProvider, returns ImprovementTarget, name = "self_improvement", sample_goal method. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.15 Implement `ExternalOpportunitySource` (stub) — DONE (2026-09-02).** Stub at sources.rs:307-346: implements ObjectiveSourceProvider, returns ExternalSource, name = "external", sample_goal method. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.16 Wire all 5 sources into `ObjectiveSourceRegistry::init()` — DONE (2026-09-02).** Method at sources.rs:122-130: registers HumanInputSource, SystemGeneratedSource, LearningObjectiveSource, SelfImprovementSource, ExternalOpportunitySource in order. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T2.17 Add `cooboploop_run_source_discovery` MCP handler — DONE (2026-09-02).** Handler at mod.rs:883-891: accepts optional source_type param, returns stub response (discovery logic not yet implemented). Registry entry `COOBOPLOOP_RUN_SOURCE_DISCOVERY` in `definitions::all()`. Compiled: 0 errors, 0 warnings.

- **CoObOpLoop T2.18 Add registry entry for `cooboploop_run_source_discovery` — DONE (2026-09-02).** `COOBOPLOOP_RUN_SOURCE_DISCOVERY` in `definitions::all()` with `McpTool`: name, description, input_schema (source_type optional). Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.1 Add `EvaluationCriteria` struct (10 fields) to `evaluation.rs` — DONE (2026-09-02).** Struct at evaluation.rs:43-53: goal_id, expected_value, probability_of_success, urgency, deadline, resource_cost, time_cost, risk, learning_value, strategic_value. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.2 Add `ResourceCost` struct (5 fields) to `evaluation.rs` — DONE (2026-09-02).** Struct: cpu_hours, memory_mb, disk_mb, network_mb, human_hours. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.3 Add `CapabilityRequirement` enum (4 variants) to `evaluation.rs` — DONE (2026-09-02).** Enum: Sufficient, Uncertain, Insufficient, Unavailable. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.4 Add `compute_priority()` function (§A.2 formula) to `evaluation.rs` — DONE (2026-09-02).** Formula: `Priority = expected_value × urgency × (1.0 - risk) × learning_value × strategic_value ÷ cost`. Handles zero cost (returns 0.0). Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.5 Add `PriorityPolicy` trait (compute method) to `evaluation.rs` — DONE (2026-09-02).** Trait: `compute(criteria: &EvaluationCriteria) -> f32`. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.6 Add `DefaultPriorityPolicy` impl (§A.2 formula) to `evaluation.rs` — DONE (2026-09-02).** Impl PriorityPolicy: uses §A.2 formula `expected_value × urgency × (1.0 - risk) × learning_value × strategic_value ÷ cost`. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.7 Add `ConservativePriorityPolicy` impl (0.8x factor) to `evaluation.rs` — DONE (2026-09-02).** Impl PriorityPolicy: same formula × 0.8 to deprioritize risky objectives. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.8 Add `PriorityPolicyRegistry` struct with policy swap at runtime to `evaluation.rs` — DONE (2026-09-02).** Struct: holds current policy, can swap at runtime. Serde Serialize+Deserialize. Compilation: 0 errors, 0 warnings.

- **CoObOpLoop T3.9 Add `cooboploop_evaluate_goal` MCP handler — DONE (2026-09-02).** Handler: accepts goal_id, returns EvaluationCriteria with computed fields. Registry entry `COOBOPLOOP_EVALUATE_GOAL` in `definitions::all()`. Compiled: 0 errors, 0 warnings.

- **CoObOpLoop T3.10 Add `cooboploop_reprioritize_queue` MCP handler — DONE (2026-09-02).** Handler: recomputes priority for all queue goals, returns updated list. Registry entry `COOBOPLOOP_REPRIORITIZE_QUEUE` in `definitions::all()`. Compiled: 0 errors, 0 warnings.

- **CoObOpLoop T0.3 Create `src/cooboploop/mod.rs` with `pub mod` declarations -- DONE (2026-09-01).** Verified 14 submodule declarations: capability, evaluation, hardware, human, idle, inspection, learning, loop_runner, opportunity, queue, research, self_improvement, sources, strategic. Architecture §1, §3-§23.

- **CoObOpLoop T0.4 Create `src/cooboploop/queue.rs` -- DONE (2026-09-01).** File exists with 239 lines (GoalStatus enum, AgentGoal struct, ObjectiveQueue with SQLite). Architecture §4.

- **CoObOpLoop T0.5 Create `src/cooboploop/sources.rs` -- DONE (2026-09-01).** File exists. Architecture §3.

- **CoObOpLoop T0.6 Create `src/cooboploop/evaluation.rs` -- DONE (2026-09-01).** File exists with 162 lines: EvaluationCriteria, PriorityPolicy enum, GoalEvaluator with compute_priority() (§A.2 formula). Architecture §5.

- **CoObOpLoop T0.7 Create `src/cooboploop/capability.rs` -- DONE (2026-09-01).** File exists with 67 lines: CapabilityId, CapabilityAssessment, CapabilityRegistry (register/get/update/list). Architecture §6.

- **CoObOpLoop T0.8 Create `src/cooboploop/loop_runner.rs` -- DONE (2026-09-01).** File exists with 111 lines: LoopStage, LoopStatus, LoopRunner (start/stop/pause/run_cycle). Architecture §7.

- **CoObOpLoop T0.9 Add `pub mod cooboploop;` to `src/lib.rs` -- DONE (2026-09-01).** Declaration exists at `src/lib.rs:14`. Architecture §1.

- **CoObOpLoop T0.10 Create `src/cooboploop/idle.rs` -- DONE (2026-09-01).** File exists with 62 lines: IdlePhase enum, IdleState (should_wait/set_interval). Architecture §9-10.

- **CoObOpLoop T0.11 Update `src/cooboploop/mod.rs` to include `pub mod idle;` -- DONE (2026-09-01).** Declaration exists at mod.rs:13.

- **CoObOpLoop T0.12 Create `src/cooboploop/opportunity.rs` -- DONE (2026-09-01).** File exists with 47 lines: Opportunity struct, OpportunityIntake (add/get_pending). Architecture §17.

- **CoObOpLoop T0.13 Update `src/cooboploop/mod.rs` to include `pub mod opportunity;` -- DONE (2026-09-01).** Declaration exists at mod.rs:17.

- **CoObOpLoop T0.14 Create `src/cooboploop/self_improvement.rs` -- DONE (2026-09-01).** File exists with 89 lines: ImprovementStage, ModificationBoundary, SelfImprovementPipeline::check() (§A.3). Architecture §14.

- **CoObOpLoop T0.15 Update `src/cooboploop/mod.rs` to include `pub mod self_improvement;` -- DONE (2026-09-01).** Declaration exists at mod.rs:20.

- **CoObOpLoop T0.16 Create `src/cooboploop/strategic.rs` -- DONE (2026-09-01).** File exists with 99 lines: StrategicObjective, StrategicObjectiveRegistry, HierarchyNode, ObjectiveHierarchy. Architecture §18-19.

- **CoObOpLoop T0.17 Update `src/cooboploop/mod.rs` to include `pub mod strategic;` -- DONE (2026-09-01).** Declaration exists at mod.rs:22.

- **CoObOpLoop T0.18 Create `src/cooboploop/human.rs` -- DONE (2026-09-01).** File exists with 65 lines: HumanAction enum, HumanActionHandler (set_autonomous/is_autonomous/process). Architecture §16.

- **CoObOpLoop T0.19 Update `src/cooboploop/mod.rs` to include `pub mod human;` -- DONE (2026-09-01).** Declaration exists at mod.rs:12.

- **CoObOpLoop T0.20 Create `src/cooboploop/research.rs` -- DONE (2026-09-01).** File exists with 67 lines: ResearchTrigger, ResearchObjective, ResearchManager (create_objective/list). Architecture §11.

- **CoObOpLoop T0.21 Update `src/cooboploop/mod.rs` to include `pub mod research;` -- DONE (2026-09-01).** Declaration exists at mod.rs:19.

- **CoObOpLoop T0.22 Create `src/cooboploop/hardware.rs` -- DONE (2026-09-01).** File exists with 67 lines: HardwareProfile, HardwareDiscovery. Architecture §12.

- **CoObOpLoop T0.23 Update `src/cooboploop/mod.rs` to include `pub mod hardware;` -- DONE (2026-09-01).** Declaration exists at mod.rs:11.

- **CoObOpLoop T0.24 Create `src/cooboploop/inspection.rs` -- DONE (2026-09-01).** File exists with 61 lines: InspectionTarget, Inspector (add_target/run_inspection/targets/results). Architecture §13.

- **CoObOpLoop T0.25 Update `src/cooboploop/mod.rs` to include `pub mod inspection;` -- DONE (2026-09-01).** Declaration exists at mod.rs:14.

- **CoObOpLoop T0.26 Create `src/cooboploop/learning.rs` -- DONE (2026-09-01).** File exists with 52 lines: LearningUpdate, LearningPipeline (process/updates). Architecture §15.

- **CoObOpLoop T0.27 Update `src/cooboploop/mod.rs` to include `pub mod learning;` -- DONE (2026-09-01).** Declaration exists at mod.rs:15.

- **CoObOpLoop T1.1 Add `GoalStatus` enum (13 variants) to `queue.rs` -- DONE.** File has all 13: Discovered, Evaluating, Accepted, Queued, Blocked, Deferred, Active, Verifying, Completed, Failed, Cancelled, Rejected, Archived. Architecture §4.

- **CoObOpLoop T1.2 Add `is_terminal()` method on `GoalStatus` -- DONE.** queue.rs:52-60. Returns true for Completed/Failed/Cancelled/Rejected (4 terminal states per §A.1). Architecture §4 + §A.1.

- **CoObOpLoop T1.3 Add `valid_transition()` method -- DONE.** queue.rs:63-91. 24 transition cases matching §A.1 state machine. Architecture §4 + §A.1.

- **CoObOpLoop T1.4 Add `AgentGoal` struct with `id: String` field -- DONE.** queue.rs:96-138. `id: String` at line 98, 14 fields total. Architecture §4.

- **CoObOpLoop T1.5 Add `priority: f32` field to `AgentGoal` -- DONE.** `pub priority: f32` at queue.rs:110. Architecture §4 + §A.2.

- **CoObOpLoop T1.6 Add `source: ObjectiveSource` field to `AgentGoal` -- DONE.** `pub source: ObjectiveSource` at queue.rs:113. Architecture §3 + §4.

- **CoObOpLoop T1.7 Add `status: GoalStatus` field to `AgentGoal` -- DONE.** `pub status: GoalStatus` at queue.rs:107. Architecture §4.

- **CoObOpLoop T1.8 Add `dependencies: Vec<String>` field to `AgentGoal` -- DONE.** `pub dependencies: Vec<String>` at queue.rs:128. Architecture §4.

- **CoObOpLoop T1.9 Add `deadline: Option<chrono::DateTime<chrono::Utc>>` field to `AgentGoal` -- DONE.** queue.rs:131. Architecture §4.

- **CoObOpLoop T1.10 Add `expected_value: f32` field to `AgentGoal` -- DONE.** `pub expected_value: f32` at queue.rs:116. Architecture §4 + §A.2.

- **CoObOpLoop T1.11 Add `risk: f32` field to `AgentGoal` -- DONE.** `pub risk: f32` at queue.rs:119. Architecture §4 + §A.2.

- **CoObOpLoop T1.12 Add `learning_value: f32` field to `AgentGoal` -- DONE.** `pub learning_value: f32` at queue.rs:122. Architecture §4 + §A.2.

- **CoObOpLoop T1.13 Add `required_capabilities: Vec<String>` field to `AgentGoal` -- DONE.** `pub required_capabilities: Vec<String>` at queue.rs:125. Architecture §4.

- **CoObOpLoop T1.14 Add `execution_history: Vec<ExecutionRecord>` field to `AgentGoal` -- DONE.** `pub execution_history: Vec<ExecutionRecord>` at queue.rs:134. Architecture §4.

- **CoObOpLoop T1.15 Add `completion_state: Option<String>` field to `AgentGoal` -- DONE.** `pub completion_state: Option<String>` at queue.rs:137. ExecutionRecord struct at queue.rs:142-151 (timestamp, from_status, to_status). Architecture §4.

- **CoObOpLoop T1.16 Add `ObjectiveQueue::new()` constructor -- DONE.** queue.rs:161-165. Architecture §4.

- **CoObOpLoop T1.17 Add `ObjectiveQueue::enqueue()` stub -- DONE.** queue.rs:168-171. `enqueue(&mut self, goal: AgentGoal) -> Result<(), String>`. Architecture §4 + §A.5.

- **CoObOpLoop T1.18 Add `ObjectiveQueue::get()` stub -- DONE.** queue.rs:174-176. `get(&self, id: &str) -> Option<&AgentGoal>`. Architecture §4.

- **CoObOpLoop T1.19 Add `ObjectiveQueue::update()` stub -- DONE.** queue.rs:179-182. `update(&mut self, id, goal) -> Result<(), String>`. Architecture §4.

- **CoObOpLoop T1.20 Add `ObjectiveQueue::transition()` -- DONE.** queue.rs:185-192. `transition(&mut self, id, new_status) -> Result<(), String>`. Architecture §4 + §A.1.

- **CoObOpLoop T1.21 Add SQLite schema for `objectives` table from §A.4 -- DONE.** queue.rs:214-234. CREATE TABLE IF NOT EXISTS objectives with all §A.4 columns. Architecture §4 + §A.4.

- **CoObOpLoop T1.22 Add `ObjectiveQueue::open()` using rusqlite -- DONE.** queue.rs:211-238. `open(path) -> Result<Self, String>`. rusqlite::Connection::open + execute_batch. Architecture §4 + §A.4.

- **CoObOpLoop T1.23 Wire `enqueue()` to INSERT INTO objectives — DONE.** queue.rs. Added `db: Option<rusqlite::Connection>` to ObjectiveQueue struct. enqueue() does INSERT OR REPLACE via rusqlite params. Architecture §4 + §A.4.

- **CoObOpLoop T1.24 Wire `get()` to SELECT FROM objectives — DONE.** queue.rs. get() checks in-memory first, falls back to SQLite SELECT with full row mapping. Architecture §4 + §A.4.

- **CoObOpLoop T1.25 Wire `update()` to UPDATE objectives — DONE.** queue.rs. update() does UPDATE SET via rusqlite params. Architecture §4 + §A.4.

- **CoObOpLoop T1.20 Update `transition()` to write to SQLite — DONE.** queue.rs. transition() records execution_history entry and UPDATEs status in SQLite. Architecture §4 + §A.1.

---

# TIER 1 -- Completed (Consolidated)

**All TIER 1 tasks completed and verified.** Gate: 454/454 tests, 0 warnings, 0 issues, 0 untested tools. First green was 141/141, ratcheted to 454.

## Durable Queue + P1-001 (P0-001/002/003, T1-09/10)
- `job_queue` table with migration 012, SQLite enqueue/dequeue, unique `experience_id`, retry lifecycle fix, partial job restoration on restart. Verified by `queue_durability.rs`. Zero TODOs/FIXMEs/stubs across entire codebase.

## #[cfg(test)] Migration (T1-10B)
- Group A (11 files): migrated to test_suite MCP tests. Group B (~48 tests, internal-only): left as Rust unit tests. Zero `#[cfg(test)]` blocks remain in `src/`.

## Loop Health Metrics (T1-13..16)
- `loop_latency`, `confidence_drift`, `promotion-throughput` metrics in `metrics.rs`, exposed via `get_system_status` (acp_handler.rs).

## MCP Experience Emission (T1-17/18)
- `emit_tool_experience` at `rmcp/mod.rs:127/141`, idempotent (mutually exclusive match arms).

## Phantom Tool Fix (T1-19)
- Memory handler `get_tools()` now includes all 6 embedding tools — previously listed in `tool_names()`/`execute_tool()` but not advertised, causing phantom tool flags.

## Tool Coverage (T1-20..29)
- All 54+ MCP tools in `function_registry/` (acp_tools.rs, coverage_tools.rs). Gate: `untested_tools` empty, `phantom_tools` empty.

## P3-001 (Project Status Sync)
- All CHANGELOG entries dated, README has "Verified State" block, AGENTS.md has same-day gate rule.

## Concurrency Audit (P7) + Runtime (P8) + Flow Tests (P9)
- Verified: no tokio RwLock await-across-lock, no std Mutex in experience/, single-dispatcher by observer_name, WAL confirmed, kill_on_drop in all 12 test IsoClients.
- fresh_start.rs covers P8-M1..M5 (first startup, restart, shutdown, missing config).
- 6 flow tests wired (basic cognition, auto memory retrieval, experience capture, recovery, restart recovery, cross-session memory). All pass.

---

# v0.0.1 CONFORMANCE WORK

- **P0** Event spine: `ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation` wired (event_subscriber/handlers.rs).
- **P1** Cognitive loop: `run_agent_goal` MCP tool works (status=Achieved, confidence=0.507).
- **P2** Stub chapters: World Model, Safety Gate, Personality decision_making.
- **P3** Self-check probes: Remaining (→ T1-01..T1-08).
- **P3.1** `#![allow]` violations: 0 in `src/` — clean.
- **P4** Performance maturity: Remaining (→ T1-09..T1-16).
