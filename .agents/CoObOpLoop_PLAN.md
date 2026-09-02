# cooboploop Micro-Tasks (each <10 min)

Every item is one discrete action: add one struct/enum/field, implement one method, write one test, wire one tool. No multi-hour tasks.

Each task is annotated with the architecture section it implements (e.g., `§4`). When a task implements design decisions from the **Design Decisions** appendix (Section A), it is marked with `[D]`.

---

## Design Decisions Appendix (Section A of this document)

**Reference these before starting any task.** These resolve ambiguities left open by the architecture specification.

### A.1 Queue State Machine (completes §4 gap)

The architecture lists 13 states but does not specify valid transitions. Below is the complete transition table. Every state has exactly one "next" state (except terminal states which have none):

| From \ To       | DISCOVERED | EVALUATING | ACCEPTED | QUEUED | BLOCKED | DEFERRED | ACTIVE | VERIFYING | COMPLETED | FAILED | CANCELLED | REJECTED | ARCHIVED |
|-----------------|------------|------------|----------|--------|---------|----------|--------|-----------|-----------|--------|-----------|----------|----------|
| DISCOVERED      | --         | X          |          |        |         |          |        |           |           |        |           |          |          |
| EVALUATING      | X          | --         | X        |        |         | X        |        |           |           |        |           | X        |          |
| ACCEPTED        |            |            | --       | X      |         | X        |        |           |           |        | X         |          |          |
| QUEUED          |            |            |          | --     | X       | X        | X      |           |           |        |           |          |          |
| BLOCKED         | X          |            |          | X      | --      | X        |        |           |           |        |           |          |          |
| DEFERRED        | X          |            |          |        |         | --       |        |           |           |        |           |          |          |
| ACTIVE          |            |            |          |        |         |          | --     | X         |           |        |           |          |          |
| VERIFYING       |            |            |          |        |         |          |        | --        | X         | X      |           |          |          |
| COMPLETED       |            |            |          |        |         |          |        |           | --        |        |           |          | X        |
| FAILED          | X          |            |          |        |         |          |        |           |           | --     |           |          |          |
| CANCELLED       | X          |            |          |        |         |          |        |           |           |        | --        |          |          |
| REJECTED        | X          |            |          |        |         |          |        |           |           |        |           | --       | X        |
| ARCHIVED        | X          |            |          |        |         |          |        |           |           |        |           |          | --       |

**Transition rules (3 rules as referenced by T1.3):**
1. **Rule 1 - Evaluation gate:** `DISCOVERED → EVALUATING` is always valid. `EVALUATING → ACCEPTED|REJECTED` only after evaluation completes.
2. **Rule 2 - Execution gate:** `ACCEPTED → QUEUED → ACTIVE` is the only path to execution. `QUEUED → ACTIVE` only if no blocking dependencies.
3. **Rule 3 - Terminal states:** `COMPLETED`, `CANCELLED`, `REJECTED` are terminal. From any pre-terminal state you can go to `CANCELLED`. From `COMPLETED|FAILED` you can go to `ARCHIVED`.

**`is_terminal()` (T1.2):** Returns `true` for `COMPLETED`, `FAILED`, `CANCELLED`, `REJECTED`.

### A.2 Priority Formula Field Mapping (completes §5 gap)

The architecture gives: `Priority = Value × Urgency × ProbabilityOfSuccess × LearningValue × StrategicValue ÷ Cost`

**Field-to-formula mapping:**

| Formula Term        | `AgentGoal` Field          | Source                                          |
|---------------------|----------------------------|-------------------------------------------------|
| Value               | `expected_value` (f32)     | Direct from objective creator                   |
| Urgency             | Derived from `deadline`    | `1.0 + max(0, (deadline - now).as_secs() / 3600.0).recip()` -- higher urgency for nearer deadlines; default 1.0 if no deadline |
| ProbabilityOfSuccess| `1.0 - risk` (derived)     | Computed from `risk` field (0.0=certainty, 1.0=impossible); clamped to [0.01, 1.0] |
| LearningValue       | `learning_value` (f32)     | Direct from objective creator                   |
| StrategicValue      | Derived from `source`      | Human source = 2.0, System = 1.0, Learning = 1.5, Self-Improvement = 1.2, External = 1.3 |
| Cost                | `priority` field used inversely | The `priority` field IS the computed result; for formula inputs, use `expected_value` as proxy for cost when separate cost field absent |

**Implementation note:** Since `priority` is both input (user-provided override) and output (computed), the formula computes a raw score and applies: `final_priority = max(raw_score, user_priority)`. If the user sets `priority > 0`, it overrides the computed value.

### A.3 Self-Improvement Modification Boundaries (completes §14 gap)

The architecture says self-improvement "SHALL NOT equate to unrestricted self-modification" but does not define the boundaries.

**`ModificationBoundary` (3 variants):**

| Variant      | Meaning                                                                 | Allowed modifications                                          |
|--------------|-------------------------------------------------------------------------|----------------------------------------------------------------|
| `Read`       | Observe only                                                            | Read source files, config, DB, logs. No writes.                |
| `Propose`    | Create change artifacts without applying them                           | Write proposals to `proposals/` directory. No code changes.    |
| `Apply`      | Apply changes to own codebase (requires approval in stage 9 of §14)     | Modify `src/`, config, DB schema. Always with rollback plan.   |

**`SelfImprovementGuard::check()` rule:** Any stage beyond stage 6 (Plan) requires boundary >= `Propose`. Deployment (stage 10) requires boundary >= `Apply`. Default boundary is `Propose` -- the agent can propose improvements but not deploy them autonomously.

### A.4 SQLite Schema Initialization (new, not in architecture)

The architecture mentions multiple tables across sections but never specifies a unified schema. Below is the complete schema.

**Tables (5 total):**

```sql
-- objectives table (§4)
CREATE TABLE objectives (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    priority REAL DEFAULT 0.0,
    source TEXT DEFAULT 'system',
    status TEXT DEFAULT 'DISCOVERED',
    deadline TEXT,
    expected_value REAL DEFAULT 0.0,
    risk REAL DEFAULT 0.5,
    learning_value REAL DEFAULT 0.0,
    required_capabilities TEXT,  -- JSON array
    dependencies TEXT,           -- JSON array of goal IDs
    execution_history TEXT,      -- JSON array of status change records
    completion_state TEXT,       -- JSON of completion details
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- capabilities table (§6)
CREATE TABLE capabilities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    level REAL DEFAULT 0.0,
    last_assessed TEXT,
    experience_count INTEGER DEFAULT 0,
    success_rate REAL DEFAULT 0.0
);

-- hardware_snapshots table (§12)
CREATE TABLE hardware_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_at TEXT DEFAULT (datetime('now')),
    cpu_model TEXT,
    cpu_cores INTEGER,
    memory_total_mb INTEGER,
    memory_available_mb INTEGER,
    storage_total_gb INTEGER,
    storage_available_gb REAL,
    gpu_model TEXT,
    network_interfaces TEXT,  -- JSON array
    thermal_state TEXT,
    other TEXT                -- JSON for any additional fields
);

-- strategic_objectives table (§18-19)
CREATE TABLE strategic_objectives (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT,
    status TEXT DEFAULT 'active',
    priority REAL DEFAULT 0.0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- improvement_proposals table (§14)
CREATE TABLE improvement_proposals (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    stage TEXT DEFAULT 'identify_limitation',
    proposal TEXT,
    risk_assessment TEXT,
    benefit_estimate TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    approved INTEGER DEFAULT 0
);
```

### A.5 MCP Tool Parameter Schemas (new, not in architecture)

Each MCP handler needs a defined input schema. Below are the parameter schemas for all ~38 new tools.

**Queue tools (§4):**

| Tool Name                           | Required Params                          | Optional Params                              |
|-------------------------------------|------------------------------------------|----------------------------------------------|
| `cooboploop_enqueue_goal`           | `title: string`                          | `description`, `expected_value`, `risk`, `learning_value`, `deadline`, `source`, `required_capabilities`, `dependencies` |
| `cooboploop_list_goals`             | (none)                                   | `status_filter`, `limit`, `offset`           |
| `cooboploop_get_goal`               | `goal_id: string`                        | (none)                                       |
| `cooboploop_update_goal_status`     | `goal_id: string`, `new_status: string`  | (none)                                       |

**Source tools (§3):**

| Tool Name                             | Required Params     | Optional Params |
|---------------------------------------|---------------------|-----------------|
| `cooboploop_run_source_discovery`     | (none)              | `source_type`   |

**Evaluation tools (§5):**

| Tool Name                           | Required Params        | Optional Params |
|-------------------------------------|------------------------|-----------------|
| `cooboploop_evaluate_goal`          | `goal_id: string`      | (none)          |
| `cooboploop_reprioritize_queue`     | (none)                 | (none)          |
| `cooboploop_set_priority_policy`    | `policy: string`       | (none)          |

**Capability tools (§6):**

| Tool Name                               | Required Params     | Optional Params |
|-----------------------------------------|---------------------|-----------------|
| `cooboploop_record_capability_outcome`  | `capability_id: string`, `success: boolean` | (none) |
| `cooboploop_get_capability_assessment`  | `capability_id: string` | (none)        |
| `cooboploop_list_capabilities`          | (none)              | (none)          |

**Loop tools (§7):**

| Tool Name                           | Required Params | Optional Params |
|-------------------------------------|-----------------|-----------------|
| `cooboploop_start_loop`             | (none)          | `max_cycles`    |
| `cooboploop_stop_loop`              | (none)          | (none)          |
| `cooboploop_get_loop_status`        | (none)          | (none)          |
| `cooboploop_run_single_cycle`       | (none)          | (none)          |
| `cooboploop_step_loop`              | (none)          | (none)          |

**Post-task tools (§8):**

| Tool Name                                 | Required Params | Optional Params |
|-------------------------------------------|-----------------|-----------------|
| `cooboploop_run_post_task_evaluation`     | `goal_id: string` | (none)        |

**Idle tools (§9-10):**

| Tool Name                                       | Required Params | Optional Params |
|-------------------------------------------------|-----------------|-----------------|
| `cooboploop_get_idle_state`                     | (none)          | (none)          |
| `cooboploop_configure_idle_reevaluation_interval` | `seconds: integer` | (none)    |

**Research tools (§11):**

| Tool Name                                 | Required Params       | Optional Params |
|-------------------------------------------|-----------------------|-----------------|
| `cooboploop_create_research_objective`    | `topic: string`       | `priority`, `persistence_target` |

**Hardware tools (§12):**

| Tool Name                              | Required Params | Optional Params |
|----------------------------------------|-----------------|-----------------|
| `cooboploop_get_hardware_profile`      | (none)          | (none)          |
| `cooboploop_detect_hardware_changes`   | (none)          | (none)          |

**Inspection tools (§13):**

| Tool Name                         | Required Params | Optional Params |
|-----------------------------------|-----------------|-----------------|
| `cooboploop_run_inspection`       | (none)          | `target`        |

**Self-improvement tools (§14):**

| Tool Name                                | Required Params     | Optional Params |
|------------------------------------------|---------------------|-----------------|
| `cooboploop_get_modification_boundary`   | (none)              | (none)          |
| `cooboploop_set_modification_boundary`   | `boundary: string`  | (none)          |

**Opportunity tools (§17):**

| Tool Name                                       | Required Params | Optional Params |
|-------------------------------------------------|-----------------|-----------------|
| `cooboploop_run_opportunity_intake`             | `source_url: string` | `source_type` |
| `cooboploop_get_pending_external_opportunities` | (none)          | (none)          |

**Human tools (§16):**

| Tool Name                           | Required Params  | Optional Params |
|-------------------------------------|------------------|-----------------|
| `cooboploop_set_autonomous_mode`    | `enabled: boolean` | (none)        |
| `cooboploop_get_autonomous_mode`    | (none)           | (none)          |

**Strategic tools (§18-19):**

| Tool Name                                  | Required Params  | Optional Params |
|--------------------------------------------|------------------|-----------------|
| `cooboploop_list_strategic_objectives`     | (none)           | (none)          |
| `cooboploop_add_strategic_objective`       | `name: string`   | `category`      |
| `cooboploop_remove_strategic_objective`    | `id: string`     | (none)          |
| `cooboploop_get_objective_hierarchy`       | (none)           | (none)          |
| `cooboploop_set_mission`                   | `mission: string` | (none)         |

**Autonomy tools (§21-22):**

| Tool Name                        | Required Params | Optional Params |
|----------------------------------|-----------------|-----------------|
| `cooboploop_get_autonomy_levels` | (none)          | (none)          |
| `cooboploop_promote_autonomy`    | `capability_id: string` | (none) |

### A.6 Module File Map (completes T0/T16 gap)

The conformance mapping (T16) references files that the original T0 did not create. Below is the corrected module file list.

| File                              | Created In | Architecture Sections | Contents                                    |
|-----------------------------------|------------|----------------------|---------------------------------------------|
| `src/cooboploop/mod.rs`           | T0.3       | All                  | `pub mod` declarations for all modules      |
| `src/cooboploop/queue.rs`         | T0.4       | §4                   | `GoalStatus`, `AgentGoal`, `ObjectiveQueue` |
| `src/cooboploop/sources.rs`       | T0.5       | §3                   | `ObjectiveSource` enums, `ObjectiveSourceRegistry` |
| `src/cooboploop/evaluation.rs`    | T0.6       | §5                   | `EvaluationCriteria`, `compute_priority()`, `PriorityPolicy` |
| `src/cooboploop/capability.rs`    | T0.7       | §6                   | `CapabilityId`, `CapabilityRegistry`, `CapabilityAssessment` |
| `src/cooboploop/loop_runner.rs`   | T0.8       | §7, §20              | `LoopStage`, `LoopRunner`, `run_cycle()`    |
| `src/cooboploop/idle.rs`          | T0.11      | §9, §10              | `IdlePhase`, `IdleState`, `should_wait()`   |
| `src/cooboploop/opportunity.rs`   | T0.12      | §17                  | `Opportunity`, adapters, `OpportunityIntake` |
| `src/cooboploop/self_improvement.rs` | T0.13  | §14                  | `ImprovementStage`, `SelfImprovementPipeline`, `ModificationBoundary` |
| `src/cooboploop/strategic.rs`     | T0.14      | §18, §19             | `StrategicObjective`, `StrategicObjectiveRegistry`, `ObjectiveHierarchy` |
| `src/cooboploop/human.rs`         | T0.15      | §16                  | `HumanAction`, `HumanActionHandler`         |
| `src/cooboploop/research.rs`      | T0.16      | §11                  | `ResearchTrigger`, `ResearchObjective`      |
| `src/cooboploop/hardware.rs`      | T0.17      | §12                  | `HardwareProfile`, `HardwareDiscovery`      |
| `src/cooboploop/inspection.rs`    | T0.18      | §13                  | `InspectionTarget`, `Inspector`             |
| `src/cooboploop/learning.rs`      | T0.19      | §15                  | `LearningPipeline`, `LearningUpdate`        |

### A.7 Plan/Execute Stage Interfaces (completes T5.8/T5.9 gap)

The architecture describes Plan and Execute as distinct stages in §7. The plan must define the interface between them.

**Stage 5 - Plan (T5.8):** Takes the selected objective from stage 4. Produces a `Plan` struct containing:
- `steps: Vec<PlanStep>` - ordered list of actions
- `required_capabilities: Vec<String>` - what's needed
- `estimated_cost: f32` - expected resource cost
- `rollback_plan: Option<String>` - how to undo if execution fails
- `success_criteria: Vec<String>` - what constitutes success

The Plan struct is stored alongside the objective and referenced by Execute.

**Stage 6 - Execute (T5.9):** Takes the Plan from stage 5. Executes each step in order. On each step:
- Records the action taken
- Captures the result
- On failure: records the failure, attempts rollback if available, transitions goal to `FAILED`
- On completion of all steps: transitions goal to `VERIFYING`

**Stage output to stage input flow:**
```
Stage 4 (SelectObjective) → selected: Option<AgentGoal>
Stage 5 (Plan)           → input: selected, output: Plan
Stage 6 (Execute)        → input: Plan, output: ExecutionResult
Stage 7 (Verify)         → input: ExecutionResult
```

### A.8 Discovery Lifecycle (completes T2 gap)

The architecture says sources "SHOULD eventually connect" (§17). The discovery lifecycle:

1. **Registration:** Sources register themselves at startup via `ObjectiveSourceRegistry::init()` (T2.16). Each source is an implementation of `ObjectiveSourceProvider`.
2. **Discovery:** `discover_all()` (T2.10) iterates registered sources and calls each one's `discover()` method. Each discovery call returns a list of potential objectives found.
3. **Intake:** Discovered objectives pass through the intake pipeline (T12.7-T12.14) before entering the queue.
4. **Dynamic registration:** New sources can be registered at runtime via `register()` (T2.9). They are immediately available for the next `discover_all()` call.

---

## T0 Foundation (each <10 min)

- [x] T0.3 Create `src/cooboploop/mod.rs` with `pub mod` declarations for all modules below `[§1 mapping]`
- [x] T0.4 Create `src/cooboploop/queue.rs` (empty file) `[§4]`
- [x] T0.5 Create `src/cooboploop/sources.rs` (empty) `[§3]`
- [x] T0.6 Create `src/cooboploop/evaluation.rs` (empty) `[§5]`
- [x] T0.7 Create `src/cooboploop/capability.rs` (empty) `[§6]`
- [x] T0.8 Create `src/cooboploop/loop_runner.rs` (empty) `[§7]`
- [x] T0.9 Add `pub mod cooboploop;` to `src/lib.rs` `[§1]`
- [x] T0.10 Create `src/cooboploop/idle.rs` (empty) `[§9-10]`
- [x] T0.11 Update `src/cooboploop/mod.rs` to include `pub mod idle;`
- [x] T0.12 Create `src/cooboploop/opportunity.rs` (empty) `[§17]`
- [x] T0.13 Update `src/cooboploop/mod.rs` to include `pub mod opportunity;`
- [x] T0.14 Create `src/cooboploop/self_improvement.rs` (empty) `[§14]`
- [x] T0.15 Update `src/cooboploop/mod.rs` to include `pub mod self_improvement;`
- [x] T0.16 Create `src/cooboploop/strategic.rs` (empty) `[§18-19]`
- [x] T0.17 Update `src/cooboploop/mod.rs` to include `pub mod strategic;`
- [x] T0.18 Create `src/cooboploop/human.rs` (empty) `[§16]`
- [x] T0.19 Update `src/cooboploop/mod.rs` to include `pub mod human;`
- [x] T0.20 Create `src/cooboploop/research.rs` (empty) `[§11]`
- [x] T0.21 Update `src/cooboploop/mod.rs` to include `pub mod research;`
- [x] T0.22 Create `src/cooboploop/hardware.rs` (empty) `[§12]`
- [x] T0.23 Update `src/cooboploop/mod.rs` to include `pub mod hardware;`
- [x] T0.24 Create `src/cooboploop/inspection.rs` (empty) `[§13]`
- [x] T0.25 Update `src/cooboploop/mod.rs` to include `pub mod inspection;`
- [x] T0.26 Create `src/cooboploop/learning.rs` (empty) `[§15]`
- [x] T0.27 Update `src/cooboploop/mod.rs` to include `pub mod learning;`
- [x] T0.28 `cargo check --release` passes with skeletons

## T1 Queue States (§4) — each <10 min

- [x] T1.1 Add `GoalStatus` enum (13 variants) to `queue.rs` `[§4]`
- [x] T1.2 Add `is_terminal()` method on `GoalStatus` `[§4 + §A.1]`
- [x] T1.3 Add `valid_transition()` method `[§4 + §A.1]`
- [x] T1.4 Add `AgentGoal` struct with `id: String` field `[§4]`
- [x] T1.5 Add `priority: f32` field to `AgentGoal` `[§4 + §A.2]`
- [x] T1.6 Add `source: ObjectiveSource` field to `AgentGoal` `[§3 + §4]`
- [x] T1.7 Add `status: GoalStatus` field to `AgentGoal` `[§4]`
- [x] T1.8 Add `dependencies: Vec<String>` field to `AgentGoal` (goal IDs) `[§4]`
- [x] T1.9 Add `deadline: Option<chrono::DateTime<chrono::Utc>>` field to `AgentGoal` `[§4]`
- [x] T1.10 Add `expected_value: f32` field to `AgentGoal` `[§4 + §A.2]`
- [x] T1.11 Add `risk: f32` field to `AgentGoal` `[§4 + §A.2]`
- [x] T1.12 Add `learning_value: f32` field to `AgentGoal` `[§4 + §A.2]`
- [x] T1.13 Add `required_capabilities: Vec<String>` field to `AgentGoal` `[§4]`
- [x] T1.14 Add `execution_history: Vec<ExecutionRecord>` field to `AgentGoal` (ExecutionRecord = { timestamp, from_status, to_status }) `[§4]`
- [x] T1.15 Add `completion_state: Option<String>` field to `AgentGoal` `[§4]`
- [x] T1.16 Add `ObjectiveQueue::new()` constructor `[§4]`
- [x] T1.17 Add `ObjectiveQueue::enqueue()` stub `[§4 + §A.5]`
- [x] T1.18 Add `ObjectiveQueue::get()` stub `[§4]`
- [x] T1.19 Add `ObjectiveQueue::update()` stub `[§4]`
- [x] T1.20 Add `ObjectiveQueue::transition()` `[§4 + §A.1]`
- [x] T1.21 Add SQLite schema for `objectives` table from §A.4 `[§4 + §A.4]`
- [x] T1.22 Add `ObjectiveQueue::open()` using rusqlite `[§4 + §A.4]`
- [x] T1.23 Wire `enqueue()` to INSERT INTO objectives `[§4 + §A.4]`
- [x] T1.24 Wire `get()` to SELECT FROM objectives WHERE id `[§4 + §A.4]`
- [x] T1.25 Wire `update()` to UPDATE objectives SET ... WHERE id `[§4 + §A.4]`
- [ ] T1.26 Add `cooboploop_enqueue_goal` MCP handler `[§4 + §A.5]`
- [ ] T1.27 Add `cooboploop_list_goals` MCP handler — optional status_filter param `[§4 + §A.5]`
- [ ] T1.28 Add `cooboploop_get_goal` MCP handler — requires goal_id `[§4 + §A.5]`
- [ ] T1.29 Add `cooboploop_update_goal_status` MCP handler — requires goal_id + new_status `[§4 + §A.5]`
- [ ] T1.30 Add registry entry for `cooboploop_enqueue_goal` `[§4]`
- [ ] T1.31 Add registry entry for `cooboploop_list_goals` `[§4]`
- [ ] T1.32 Add registry entry for `cooboploop_get_goal` `[§4]`
- [ ] T1.33 Add registry entry for `cooboploop_update_goal_status` `[§4]`
- [ ] T1.34 Test: enqueue goal → get goal → update status to ACCEPTED → verify status changed `[§4]`
- [ ] T1.35 `cargo check --release` passes and gate is green

## T2 Sources (§3) — each <10 min

- [ ] T2.1 Add `ObjectiveSource` enum (5 variants: Human, ExternalOpportunity, SystemGenerated, Learning, SelfImprovement) to `sources.rs` `[§3]`
- [ ] T2.2 Add `HumanOrigin` enum (6 variants: user_request, instruction, correction, project, maintenance_request, strategic_goal) `[§3.1]`
- [ ] T2.3 Add `ExternalSource` enum (9 variants: freelance_job, dev_bounty, research_opportunity, grant, competition, open_source_task, available_project, hardware_opportunity, user_request) `[§3.2]`
- [ ] T2.4 Add `SystemTrigger` enum (13 variants: unresolved_error, failed_test, detected_bug, degraded_performance, memory_inconsistency, hardware_problem, software_dependency_problem, stale_component, missing_documentation, security_issue, reliability_issue, incomplete_implementation, failed_experiment) `[§3.3]`
- [ ] T2.5 Add `LearningTrigger` enum (4 variants: repeated_failure, insufficient_understanding, repeated_human_intervention, capability_gap) `[§3.4]`
- [ ] T2.6 Add `ImprovementTarget` enum (13 variants: reasoning_workflow, planning, tool_usage, memory_retrieval, memory_organization, execution_reliability, testing, hardware_utilization, inference_performance, software_architecture, resource_utilization, error_detection, recovery_procedure) `[§3.5]`
- [ ] T2.7 Add `ObjectiveSourceProvider` trait with methods: `source_type() -> ObjectiveSource`, `discover() -> Vec<Objective>`, `name() -> &str` (3 methods) `[§3 + §A.8]`
- [ ] T2.8 Add `ObjectiveSourceRegistry` struct — holds Vec<Box<dyn ObjectiveSourceProvider>> `[§3 + §A.8]`
- [ ] T2.9 Add `register()` method to registry — pushes provider into Vec `[§3 + §A.8]`
- [ ] T2.10 Add `discover_all()` method to registry — iterates providers, collects all discoveries `[§3 + §A.8]`
- [ ] T2.11 Implement `HumanInputSource` (stub) — returns HumanOrigin, name = "human_input" `[§3.1 + §A.8]`
- [ ] T2.12 Implement `SystemGeneratedSource` (stub) — returns SystemTrigger, name = "system" `[§3.3 + §A.8]`
- [ ] T2.13 Implement `LearningObjectiveSource` (stub) — returns LearningTrigger, name = "learning" `[§3.4 + §A.8]`
- [ ] T2.14 Implement `SelfImprovementSource` (stub) — returns ImprovementTarget, name = "self_improvement" `[§3.5 + §A.8]`
- [ ] T2.15 Implement `ExternalOpportunitySource` (stub) — returns ExternalSource, name = "external" `[§3.2 + §A.8]`
- [ ] T2.16 Wire all 5 sources into `ObjectiveSourceRegistry::init()` — registers each in order `[§3 + §A.8]`
- [ ] T2.17 Add `cooboploop_run_source_discovery` MCP handler — optional source_type filter, returns list of discovered objectives `[§3 + §A.5]`
- [ ] T2.18 Add registry entry for `cooboploop_run_source_discovery` `[§3]`

## T3 Evaluation (§5) — each <10 min

- [ ] T3.1 Add `EvaluationCriteria` struct (10 fields: expected_value, probability_of_success, urgency, deadline, resource_cost, time_cost, risk, learning_value, strategic_value, required_capabilities) `[§5]`
- [ ] T3.2 Add `ResourceCost` struct (5 fields: cpu_hours, memory_mb, disk_mb, network_mb, human_hours) `[§5]`
- [ ] T3.3 Add `CapabilityRequirement` enum (4 variants: sufficient, uncertain, insufficient, unavailable) `[§6]`
- [ ] T3.4 Add `compute_priority()` function using formula from §A.2: `Priority = expected_value × urgency × (1.0 - risk) × learning_value × strategic_value ÷ cost` with field mappings from §A.2 `[§5 + §A.2]`
- [ ] T3.5 Add `PriorityPolicy` trait with method: `compute(criteria: &EvaluationCriteria) -> f32` `[§5]`
- [ ] T3.6 Add `DefaultPriorityPolicy` impl — uses the §A.2 formula `[§5 + §A.2]`
- [ ] T3.7 Add `ConservativePriorityPolicy` impl — same formula but multiplies by 0.8 to deprioritize risky objectives `[§5]`
- [ ] T3.8 Add `PriorityPolicyRegistry` struct with method to swap policy at runtime `[§5]`
- [ ] T3.9 Add `cooboploop_evaluate_goal` MCP handler — accepts goal_id, returns EvaluationCriteria with computed fields `[§5 + §A.5]`
- [ ] T3.10 Add `cooboploop_reprioritize_queue` MCP handler — recomputes priority for all queue goals, returns updated list `[§5 + §A.5]`
- [ ] T3.11 Add `cooboploop_set_priority_policy` MCP handler — accepts policy name ("default" or "conservative"), updates policy `[§5 + §A.5]`
- [ ] T3.12 Add registry entries for T3.9-T3.11 `[§5]`
- [ ] T3.13 Test: compute_priority with zero cost returns 0.0; compute_priority with max values returns > 1.0 `[§5]`

## T4 Capability (§6) — each <10 min

- [ ] T4.1 Add `CapabilityId` enum with base variants: Rust, MCP, HTTP, SQLite, Testing + `Custom(String)` for extensibility `[§6]`
- [ ] T4.2 Add `CapabilityAssessment` struct (5 fields: id, name, level: f32, last_assessed: Option<DateTime>, success_rate: f32) `[§6]`
- [ ] T4.3 Add `CapabilityRegistry` struct — wraps SQLite connection + in-memory cache `[§6]`
- [ ] T4.4 Add SQLite table `capabilities` from §A.4 `[§6 + §A.4]`
- [ ] T4.5 Add `get()` method — returns CapabilityAssessment by ID from DB `[§6]`
- [ ] T4.6 Add `update()` method — updates level and success_rate in DB `[§6]`
- [ ] T4.7 Add `compare_capabilities()` method — takes required capabilities list, returns CapabilityComparison `[§6]`
- [ ] T4.8 Add `CapabilityComparison` struct (4 fields: sufficient: Vec<String>, uncertain: Vec<String>, insufficient: Vec<String>, unavailable: Vec<String> + overall_outcome computed) `[§6]`
- [ ] T4.9 Add `overall_outcome()` on CapabilityComparison — returns "sufficient" if no insufficient/unavailable, "uncertain" if has uncertain, "insufficient" otherwise (threshold: any capability < 0.7 = insufficient) `[§6 + §A.3]`
- [ ] T4.10 Add `record_success()` method — increments experience_count, updates success_rate `[§6]`
- [ ] T4.11 Add `record_failure()` method — increments experience_count, updates success_rate `[§6]`
- [ ] T4.12 Add `cooboploop_record_capability_outcome` MCP handler — accepts capability_id + success boolean `[§6 + §A.5]`
- [ ] T4.13 Add `cooboploop_get_capability_assessment` MCP handler — accepts capability_id, returns assessment `[§6 + §A.5]`
- [ ] T4.14 Add `cooboploop_list_capabilities` MCP handler — returns all capabilities `[§6 + §A.5]`
- [ ] T4.15 Add registry entries for T4.12-T4.14 `[§6]`
- [ ] T4.16 Seed default capabilities at init: Rust 0.5, MCP 0.5, HTTP 0.5, SQLite 0.5, Testing 0.5 (levels per §A.2 example) `[§6]`

## T5 Loop (§7) — each <10 min

- [ ] T5.1 Add `LoopStage` enum (11 variants: ObserveState, CollectObjectives, EvaluateQueue, SelectObjective, Plan, Execute, Verify, RecordExperience, UpdateKnowledge, EvaluateCurrentState, GenerateNewObjectives) `[§7]`
- [ ] T5.2 Add `LoopRunner` struct with fields for: queue reference, source registry, evaluation policy, capability registry, state (current stage, cycle_count, should_continue flag, max_cycles) `[§7]`
- [ ] T5.3 Add `run_cycle()` method — calls stages 1-11 sequentially `[§7]`
- [ ] T5.4 Implement stage 1 (ObserveState) — reads current system state: queue size, active goals, idle status, hardware profile `[§7]`
- [ ] T5.5 Implement stage 2 (CollectObjectives) — calls source registry `discover_all()`, adds discovered objectives to queue via intake `[§7 + §A.8]`
- [ ] T5.6 Implement stage 3 (EvaluateQueue) — for each QUEUED goal, runs `compute_priority()` and updates priority `[§7 + §A.2]`
- [ ] T5.7 Implement stage 4 (SelectObjective) — finds highest-priority ACCEPTED/QUEUED goal with no blocking dependencies `[§7]`
- [ ] T5.8 Implement stage 5 (Plan) — takes selected goal from stage 4, produces Plan struct per §A.7: steps, required_capabilities, estimated_cost, rollback_plan, success_criteria `[§7 + §A.7]`
- [ ] T5.9 Implement stage 6 (Execute) — takes Plan from stage 5, executes each step, records result, transitions to VERIFYING on success or FAILED on failure `[§7 + §A.7]`
- [ ] T5.10 Implement stage 7 (Verify) — takes ExecutionResult from stage 6, validates success_criteria, transitions to COMPLETED or FAILED `[§7]`
- [ ] T5.11 Implement stage 8 (RecordExperience) — creates Experience record from execution result with all §15 fields `[§7 + §15]`
- [ ] T5.12 Implement stage 9 (UpdateKnowledge) — calls LearningPipeline::process() on recorded experience `[§7 + §15]`
- [ ] T5.13 Implement stage 10 (EvaluateCurrentState) — checks post-task evaluation questions from §8 (did it succeed? unexpected problems? knowledge gaps?) `[§7 + §8]`
- [ ] T5.14 Implement stage 11 (GenerateNewObjectives) — creates new objectives from post-task evaluation results, adds to queue `[§7 + §8]`
- [ ] T5.15 Add `return_to_queue()` call in stage 11 — new objectives are enqueued and loop continues `[§7]`
- [ ] T5.16 Add `should_continue()` method — returns false if cycle_count >= max_cycles or halt flag set `[§7]`
- [ ] T5.17 Add `cooboploop_start_loop` MCP handler — starts LoopRunner, optional max_cycles param `[§7 + §A.5]`
- [ ] T5.18 Add `cooboploop_stop_loop` MCP handler — sets halt flag `[§7 + §A.5]`
- [ ] T5.19 Add `cooboploop_get_loop_status` MCP handler — returns current_stage, cycle_count, should_continue `[§7 + §A.5]`
- [ ] T5.20 Add `cooboploop_run_single_cycle` MCP handler — runs exactly one cycle, returns status `[§7 + §A.5]`
- [ ] T5.21 Add `cooboploop_step_loop` MCP handler — advances to next stage, returns stage name `[§7 + §A.5]`
- [ ] T5.22 Add registry entries for T5.17-T5.21 `[§7]`
- [ ] T5.23 Test: one full cycle completes without crash — enqueue goal, start loop, verify cycle completes `[§7]`

## T6 Post-Task (§8) — each <10 min

- [ ] T6.1 Add `PostTaskEvaluation` struct with 10 fields matching the 10 questions from §8: did_succeed, verification_confirmed, unexpected_problems (Vec<String>), knowledge_gaps (Vec<String>), new_bugs (Vec<String>), capability_limitation (Option<String>), created_work (Vec<String>), efficiency_score (f32), future_planning_adjustment (Option<String>), improvement_opportunity (Option<String>) `[§8]`
- [ ] T6.2 Add `did_succeed: bool` field `[§8]`
- [ ] T6.3 Add `verification_confirmed: bool` field `[§8]`
- [ ] T6.4 Add `unexpected_problems: Vec<String>` field `[§8]`
- [ ] T6.5 Add `knowledge_gaps: Vec<String>` field `[§8]`
- [ ] T6.6 Add `new_bugs: Vec<String>` field `[§8]`
- [ ] T6.7 Add `capability_limitation: Option<String>` field `[§8]`
- [ ] T6.8 Add `created_work: Vec<String>` field `[§8]`
- [ ] T6.9 Add `efficiency_score: f32` field `[§8]`
- [ ] T6.10 Add `future_planning_adjustment: Option<String>` field `[§8]`
- [ ] T6.11 Add `improvement_opportunity: Option<String>` field `[§8]`
- [ ] T6.12 Add `evaluate_post_task()` method on Experience — returns PostTaskEvaluation by analyzing execution result `[§8]`
- [ ] T6.13 Add `generate_objectives()` method on PostTaskEvaluation — creates new AgentGoal entries from unexpected_problems, knowledge_gaps, new_bugs, capability_limitation, improvement_opportunity `[§8]`
- [ ] T6.14 Wire generate_objectives into LoopRunner stage 11 (GenerateNewObjectives) `[§7 + §8]`
- [ ] T6.15 Add `cooboploop_run_post_task_evaluation` MCP handler — accepts goal_id, runs post-task evaluation, returns PostTaskEvaluation `[§8 + §A.5]`
- [ ] T6.16 Add registry entry for `cooboploop_run_post_task_evaluation` `[§8]`

## T7 Idle (§9-10) — each <10 min

- [ ] T7.1 Add `IdlePhase` enum (4 variants: EvaluatingWork, SelectingWork, Waiting, Reevaluating) `[§9-10]`
- [ ] T7.2 Add `IdleState` struct with fields: phase, last_reevaluation, queue_empty, problems_count, knowledge_gaps_count `[§9-10]`
- [ ] T7.3 Add `ActivityCategory` enum (12 variants: pending_objectives, system_maintenance, bug_investigation, research, knowledge_consolidation, memory_maintenance, hardware_evaluation, performance_optimization, capability_development, self_improvement, environmental_observation, long_term_planning) `[§9]`
- [ ] T7.4 Add `evaluate_useful_work()` method — checks for work in each ActivityCategory, returns list of categories with work `[§9]`
- [ ] T7.5 Add `should_wait()` method — returns true when queue empty + no problems detected + no knowledge gaps + no research opportunities `[§10]`
- [ ] T7.6 Add `DeliberateInactivity` event type — records when system chooses to wait `[§10]`
- [ ] T7.7 Add reevaluation timer field to IdleState (u64 seconds, default 60) `[§10 + §A.6]`
- [ ] T7.8 Add `cooboploop_get_idle_state` MCP handler — returns IdleState with current phase and metrics `[§9-10 + §A.5]`
- [ ] T7.9 Add `cooboploop_configure_idle_reevaluation_interval` MCP handler — accepts seconds (u64), updates timer `[§10 + §A.5]`
- [ ] T7.10 Add registry entries for T7.8-T7.9 `[§9-10]`

## T8 Research/Hardware/Inspection (§11-13) — each <10 min

- [ ] T8.1 Add `ResearchTrigger` enum (8 variants: unavailable_info, high_uncertainty, capability_gap, technology_investigation, hardware_upgrade, multiple_solutions, previous_failure, external_opportunity_knowledge_gap) `[§11]`
- [ ] T8.2 Add `ResearchObjective` struct (4 fields: topic: String, trigger: ResearchTrigger, persistence_target: PersistenceTarget, expected_knowledge: String) `[§11]`
- [ ] T8.3 Add `PersistenceTarget` enum (3 variants: knowledge_base, experience_log, both) `[§11]`
- [ ] T8.4 Add `cooboploop_create_research_objective` MCP handler — accepts topic, returns ResearchObjective with generated ID `[§11 + §A.5]`
- [ ] T8.5 Add registry entry for `cooboploop_create_research_objective` `[§11]`
- [ ] T8.6 Add `HardwareProfile` struct (10 fields: cpu_model, cpu_cores, memory_total_mb, memory_available_mb, storage_total_gb, storage_available_gb, gpu_model, network_interfaces (Vec<String>), thermal_state, supported_runtimes (Vec<String>)) `[§12]`
- [ ] T8.7 Add `HardwareDiscovery::detect()` stub — returns HardwareProfile with placeholder values `[§12]`
- [ ] T8.8 Add SQLite table `hardware_snapshots` from §A.4 `[§12 + §A.4]`
- [ ] T8.9 Add `HardwareRegistry::update()` — saves HardwareProfile to hardware_snapshots table `[§12 + §A.4]`
- [ ] T8.10 Add `cooboploop_get_hardware_profile` MCP handler — returns latest HardwareProfile `[§12 + §A.5]`
- [ ] T8.11 Add `cooboploop_detect_hardware_changes` MCP handler — compares current detect() with latest snapshot, returns diff `[§12 + §A.5]`
- [ ] T8.12 Add registry entries for T8.10-T8.11 `[§12]`
- [ ] T8.13 Add `InspectionTarget` enum (15 variants: operating_system, drivers, runtime, inference_engine, mcp_layer, libraries, services, configuration, storage, databases, logs, source_code, tests, dependencies, hardware_interfaces) `[§13]`
- [ ] T8.14 Add `InspectionIssue` struct (4 fields: target: InspectionTarget, severity: String, description: String, recommended_action: String) `[§13]`
- [ ] T8.15 Add `Inspector::scan()` stub — returns Vec<InspectionIssue> with placeholder issues `[§13]`
- [ ] T8.16 Add `issues_to_objectives()` method on Vec<InspectionIssue> — converts each issue into an AgentGoal `[§13]`
- [ ] T8.17 Add `cooboploop_run_inspection` MCP handler — optional target filter, returns list of InspectionIssues converted to objectives `[§13 + §A.5]`
- [ ] T8.18 Add registry entry for `cooboploop_run_inspection` `[§13]`

## T9 Self-Improvement (§14) — each <10 min

- [ ] T9.1 Add `ImprovementStage` enum (12 variants: IdentifyLimitation, CreateObjective, Research, GenerateProposal, EstimateBenefitRisk, Plan, SandboxTest, Verify, Approve, Deploy, MeasureResult, RecordExperience) `[§14]`
- [ ] T9.2 Add `SelfImprovementProposal` struct (6 fields: title, description, changes (Vec<String>), risk_assessment, benefit_estimate, requires_approval) `[§14]`
- [ ] T9.3 Add `SelfImprovementPipeline::run()` stub — orchestrates all 12 stages `[§14]`
- [ ] T9.4 Add stage 1 (IdentifyLimitation) — finds capability gaps from recent failures `[§14]`
- [ ] T9.5 Add stage 2 (CreateObjective) — creates improvement objective from limitation `[§14]`
- [ ] T9.6 Add stage 3 (Research) — creates ResearchObjective per T8 `[§14 + §11]`
- [ ] T9.7 Add stage 4 (GenerateProposal) — creates SelfImprovementProposal `[§14]`
- [ ] T9.8 Add stage 5 (EstimateBenefitRisk) — fills risk_assessment and benefit_estimate fields `[§14]`
- [ ] T9.9 Add stage 6 (Plan) — creates implementation plan for the proposal `[§14]`
- [ ] T9.10 Add stage 7 (SandboxTest) — tests changes in isolated environment `[§14]`
- [ ] T9.11 Add stage 8 (Verify) — validates changes don't break existing functionality `[§14]`
- [ ] T9.12 Add stage 9 (Approve) — checks modification boundary >= Propose (per §A.3) before allowing `[§14 + §A.3]`
- [ ] T9.13 Add stage 10 (Deploy) — applies changes, requires boundary >= Apply (per §A.3) `[§14 + §A.3]`
- [ ] T9.14 Add stage 11 (MeasureResult) — records before/after metrics `[§14]`
- [ ] T9.15 Add stage 12 (RecordExperience) — logs the entire improvement cycle `[§14]`
- [ ] T9.16 Add `ModificationBoundary` enum (3 variants: Read, Propose, Apply) per §A.3 `[§14 + §A.3]`
- [ ] T9.17 Add `SelfImprovementGuard::check()` — returns error if current boundary < required boundary per §A.3 `[§14 + §A.3]`
- [ ] T9.18 Add `cooboploop_get_modification_boundary` MCP handler — returns current boundary `[§14 + §A.5]`
- [ ] T9.19 Add `cooboploop_set_modification_boundary` MCP handler — accepts boundary name, requires Propose or Apply `[§14 + §A.5]`
- [ ] T9.20 Add registry entries for T9.18-T9.19 `[§14]`

## T10 Learning (§15) — each <10 min

- [ ] T10.1 Add `objective: String` field to Experience `[§15]`
- [ ] T10.2 Add `initial_assumptions: Vec<String>` field to Experience `[§15]`
- [ ] T10.3 Add `plan: String` field to Experience `[§15]`
- [ ] T10.4 Add `actions: Vec<String>` field to Experience `[§15]`
- [ ] T10.5 Add `tools_used: Vec<String>` field to Experience `[§15]`
- [ ] T10.6 Add `results: Vec<String>` field to Experience `[§15]`
- [ ] T10.7 Add `failures: Vec<String>` field to Experience `[§15]`
- [ ] T10.8 Add `corrections: Vec<String>` field to Experience `[§15]`
- [ ] T10.9 Add `successful_strategies: Vec<String>` field to Experience `[§15]`
- [ ] T10.10 Add `unsuccessful_strategies: Vec<String>` field to Experience `[§15]`
- [ ] T10.11 Add `discovered_constraints: Vec<String>` field to Experience `[§15]`
- [ ] T10.12 Add `discovered_capabilities: Vec<String>` field to Experience `[§15]`
- [ ] T10.13 Add `final_outcome: String` field to Experience `[§15]`
- [ ] T10.14 Add `confidence: f32` field to Experience `[§15]`
- [ ] T10.15 Add `lessons_learned: Vec<String>` field to Experience `[§15]`
- [ ] T10.16 Add `LearningPipeline::process()` stub — takes Experience, produces LearningUpdate `[§15]`
- [ ] T10.17 Add `LearningUpdate` struct (fields: capability_updates, knowledge_additions, strategy_refinements, risk_adjustments) `[§15]`
- [ ] T10.18 Wire LearningPipeline into LoopRunner stage 9 (UpdateKnowledge) `[§7 + §15]`
- [ ] T10.19 Add event tracer — logs transitions: Experience → Learning → Capability/Knowledge Update `[§15]`
- [ ] T10.20 Test: 2 cycles with different objectives shift evaluation priorities (verify compute_priority produces different results) `[§15]`

## T11 Human (§16) — each <10 min

- [ ] T11.1 Add `HumanAction` enum (9 variants: create_objective, modify_priority, approve_action, reject_proposal, provide_knowledge, alter_strategic_goal, inspect_reasoning, interrupt_execution, pause_autonomous) `[§16]`
- [ ] T11.2 Add `HumanActionHandler::dispatch()` stub — routes action to appropriate handler `[§16]`
- [ ] T11.3 Add audit log entry per action — records action, actor, timestamp, result `[§16]`
- [ ] T11.4 Add `autonomous_operation_enabled: bool` field to LoopRunner state `[§16]`
- [ ] T11.5 Add `cooboploop_set_autonomous_mode` MCP handler — accepts boolean, updates state `[§16 + §A.5]`
- [ ] T11.6 Add `cooboploop_get_autonomous_mode` MCP handler — returns current mode `[§16 + §A.5]`
- [ ] T11.7 Add registry entries for T11.5-T11.6 `[§16]`
- [ ] T11.8 Confirm `HumanInputSource` is registered in T2.16 `[§3.1]`

## T12 Opportunity (§17) — each <10 min

- [ ] T12.1 Add `Opportunity` struct (8 fields: id, title, source_type: ExternalSource, requirements, expected_effort, required_capabilities, expected_value, deadline) `[§17]`
- [ ] T12.2 Add `OpportunityAdapter` trait with methods: `name() -> &str`, `fetch() -> Vec<Opportunity>`, `parse(raw: &str) -> Opportunity` `[§17]`
- [ ] T12.3 Add `FiverrAdapter` impl (stub) — returns empty Vec `[§17]`
- [ ] T12.4 Add `UpworkAdapter` impl (stub) — returns empty Vec `[§17]`
- [ ] T12.5 Add `GitHubIssuesAdapter` impl (stub) — returns empty Vec `[§17]`
- [ ] T12.6 Add `OpportunityIntake::process()` stub — takes Opportunity, runs pipeline stages `[§17]`
- [ ] T12.7 Add parse stage — converts raw data to Opportunity struct `[§17]`
- [ ] T12.8 Add understand stage — extracts requirements, effort, capabilities from Opportunity `[§17]`
- [ ] T12.9 Add estimate stage — computes expected_effort and expected_value `[§17]`
- [ ] T12.10 Add capability_check stage — compares required_capabilities against CapabilityRegistry `[§17 + §6]`
- [ ] T12.11 Add resource_check stage — verifies system has sufficient resources `[§17]`
- [ ] T12.12 Add risk_check stage — evaluates risk based on capability gaps and uncertainty `[§17]`
- [ ] T12.13 Add value_check stage — computes value vs cost ratio `[§17]`
- [ ] T12.14 Add decision stage — Accept/Reject/Defer based on checks; default policy is never auto-Accept `[§17]`
- [ ] T12.15 Set default policy: never auto-Accept — all external opportunities require human review `[§17]`
- [ ] T12.16 Add `cooboploop_run_opportunity_intake` MCP handler — accepts source_url + source_type, runs full pipeline `[§17 + §A.5]`
- [ ] T12.17 Add `cooboploop_get_pending_external_opportunities` MCP handler — returns opportunities in pending state `[§17 + §A.5]`
- [ ] T12.18 Add registry entries for T12.16-T12.17 `[§17]`

## T13 Strategic (§18-19) — each <10 min

- [ ] T13.1 Add `StrategicObjective` enum (8 variants: maintain_system_reliability, improve_memory_retrieval, increase_inference_efficiency, expand_tool_capability, improve_hardware_utilization, reduce_repeated_failures, develop_research_capability, improve_planning_reliability) `[§18]`
- [ ] T13.2 Add `StrategicObjectiveRegistry` struct — manages persistent strategic objectives via SQLite `[§18]`
- [ ] T13.3 Add SQLite table `strategic_objectives` from §A.4 `[§18-19 + §A.4]`
- [ ] T13.4 Add `get()` method — returns strategic objectives by ID `[§18]`
- [ ] T13.5 Add `add()` method — creates new strategic objective `[§18]`
- [ ] T13.6 Add `remove()` method — marks strategic objective as removed `[§18]`
- [ ] T13.7 Add `cooboploop_list_strategic_objectives` MCP handler — returns all strategic objectives `[§18 + §A.5]`
- [ ] T13.8 Add `cooboploop_add_strategic_objective` MCP handler — accepts name + optional category `[§18 + §A.5]`
- [ ] T13.9 Add `cooboploop_remove_strategic_objective` MCP handler — accepts id `[§18 + §A.5]`
- [ ] T13.10 Add registry entries for T13.7-T13.9 `[§18]`
- [ ] T13.11 Add `HierarchyLevel` enum (6 variants: Mission, StrategicObjective, Capability, Project, Task, Action) `[§19]`
- [ ] T13.12 Add `HierarchyNode` struct (fields: level: HierarchyLevel, name: String, children: Vec<HierarchyNode>) `[§19]`
- [ ] T13.13 Add `ObjectiveHierarchy::build()` stub — constructs hierarchy tree from strategic objectives `[§19]`
- [ ] T13.14 Add `cooboploop_get_objective_hierarchy` MCP handler — returns hierarchy tree `[§19 + §A.5]`
- [ ] T13.15 Add `cooboploop_set_mission` MCP handler — sets the top-level Mission node `[§19 + §A.5]`
- [ ] T13.16 Add registry entries for T13.14-T13.15 `[§19]`
- [ ] T13.17 Wire strategic objectives to idle handler — when idle handler evaluates useful_work, strategic objectives are candidates `[§18 + §9]`
- [ ] T13.18 Verify task complete does not terminate mission — mission persists across cycles `[§19]`

## T14 Cycle (§20) — each <10 min

- [ ] T14.1 Add 9-stage cycle diagram comment in `loop_runner.rs` showing: OBSERVE → EVALUATE → PRIORITIZE → PLAN → EXECUTE → VERIFY → LEARN → REFLECT → FIND NEXT OBJECTIVE `[§20]`
- [ ] T14.2 Map 11 loop stages (§7) to 9 stages (§20): ObserveState+CollectObjectives→OBSERVE, EvaluateQueue+EvaluateCurrentState→EVALUATE, SelectObjective→PRIORITIZE, Plan→PLAN, Execute→EXECUTE, Verify→VERIFY, RecordExperience+UpdateKnowledge→LEARN, (none)→REFLECT, GenerateNewObjectives→FIND NEXT `[§20]`
- [ ] T14.3 Add WAIT branch after FIND NEXT OBJECTIVE — if no useful work, go to WAIT `[§20]`
- [ ] T14.4 Add heartbeat timer in WAIT — after idle_reevaluation_interval seconds, return to OBSERVE `[§20 + §T7.7]`
- [ ] T14.5 Verify loop resumes from WAIT — after heartbeat fires, loop re-enters OBSERVE stage `[§20]`

## T15 Principles (§21-22) — each <10 min

- [ ] T15.1 Add `task_complete` vs `cognitive_complete` distinction in comments — task_complete means a single objective finished; cognitive_complete means no useful work exists `[§21]`
- [ ] T15.2 Add metrics fields to LoopRunner: task_completion_count, cognitive_completion_count `[§21]`
- [ ] T15.3 Add `LlmProvider` trait (1 method: `generate(prompt: &str) -> String`) `[§22]`
- [ ] T15.4 Add `InferenceRuntime` trait (1 method: `run(model: &str, input: &[f32]) -> Vec<f32>`) `[§22]`
- [ ] T15.5 Add `HardwareAdapter` trait (1 method: `detect() -> HardwareProfile`) `[§22]`
- [ ] T15.6 Add `OperatingSystemAdapter` trait (1 method: `detect() -> OsInfo`) `[§22]`
- [ ] T15.7 Add `ToolRegistry` trait (1 method: `list_tools() -> Vec<Tool>` `[§22]`
- [ ] T15.8 Add `McpServerRegistry` trait (1 method: `connect(url: &str) -> Result`) `[§22]`
- [ ] T15.9 Add `MemorySystem` trait (1 method: `store(content: &str) -> Result`) `[§22]`
- [ ] T15.10 Add `PlanningAlgorithm` trait (1 method: `plan(objective: &AgentGoal) -> Plan`) `[§22]`
- [ ] T15.11 Add `LearningSystem` trait (1 method: `learn(experience: &Experience) -> LearningUpdate`) `[§22]`
- [ ] T15.12 Add default impl for each trait — uses current implementation (e.g., DefaultLlmProvider uses placeholder) `[§22]`
- [ ] T15.13 Add `AutonomyLevel` enum (4 variants: Manual, Assisted, SemiAutonomous, Autonomous) `[§22]`
- [ ] T15.14 Add autonomy tracking per capability — tracks autonomy_level for each CapabilityId `[§22]`
- [ ] T15.15 Add `cooboploop_get_autonomy_levels` MCP handler — returns all capability autonomy levels `[§22 + §A.5]`
- [ ] T15.16 Add `cooboploop_promote_autonomy` MCP handler — promotes a capability's autonomy level `[§22 + §A.5]`
- [ ] T15.17 Add registry entries for T15.15-T15.16 `[§22]`

## T16 Conformance (§23) — each <10 min

- [ ] T16.1 Write `CONFORMANCE.md` header — maps each architecture section to implementation files `[§23]`
- [ ] T16.2 Add §1 mapping (Purpose → cooboploop module root) `[§1]`
- [ ] T16.3 Add §2 mapping (Core Principle → loop_runner.rs: run_cycle, should_continue) `[§2]`
- [ ] T16.4 Add §3 mapping (Sources → sources.rs: ObjectiveSource, ObjectiveSourceRegistry) `[§3]`
- [ ] T16.5 Add §4 mapping (Queue → queue.rs: GoalStatus, AgentGoal, ObjectiveQueue) `[§4]`
- [ ] T16.6 Add §5 mapping (Evaluation → evaluation.rs: EvaluationCriteria, compute_priority, PriorityPolicy) `[§5]`
- [ ] T16.7 Add §6 mapping (Capability → capability.rs: CapabilityId, CapabilityRegistry, CapabilityAssessment) `[§6]`
- [ ] T16.8 Add §7 mapping (Loop → loop_runner.rs: LoopStage, LoopRunner, run_cycle) `[§7]`
- [ ] T16.9 Add §8 mapping (Post-Task → loop_runner.rs: post-task evaluation in stage 10, generate_objectives in stage 11) `[§8]`
- [ ] T16.10 Add §9-10 mapping (Idle → idle.rs: IdlePhase, IdleState, should_wait) `[§9-10]`
- [ ] T16.11 Add §11 mapping (Research → research.rs: ResearchTrigger, ResearchObjective) `[§11]`
- [ ] T16.12 Add §12 mapping (Hardware → hardware.rs: HardwareProfile, HardwareDiscovery) `[§12]`
- [ ] T16.13 Add §13 mapping (Inspection → inspection.rs: InspectionTarget, Inspector) `[§13]`
- [ ] T16.14 Add §14 mapping (Self-Improvement → self_improvement.rs: ImprovementStage, SelfImprovementPipeline, ModificationBoundary) `[§14]`
- [ ] T16.15 Add §15 mapping (Learning → learning.rs: LearningPipeline, LearningUpdate) `[§15]`
- [ ] T16.16 Add §16 mapping (Human → human.rs: HumanAction, HumanActionHandler) `[§16]`
- [ ] T16.17 Add §17 mapping (Opportunity → opportunity.rs: Opportunity, OpportunityAdapter, OpportunityIntake) `[§17]`
- [ ] T16.18 Add §18-19 mapping (Strategic/Hierarchy → strategic.rs: StrategicObjective, ObjectiveHierarchy) `[§18-19]`
- [ ] T16.19 Add §20 mapping (Cycle → loop_runner.rs: 9-stage diagram comment, WAIT branch) `[§20]`
- [ ] T16.20 Add §21-22 mapping (Principles → loop_runner.rs: metrics fields, trait definitions) `[§21-22]`
- [ ] T16.21 Add §23 mapping (Definition → CONFORMANCE.md: full text of §23 definition) `[§23]`
- [ ] T16.22 Write acceptance test step 1: enqueue 3 goals via MCP `[§23]`
- [ ] T16.23 Write acceptance test step 2: start loop `[§23]`
- [ ] T16.24 Write acceptance test step 3: verify loop selects highest priority `[§23]`
- [ ] T16.25 Write acceptance test step 4: complete goal + run post-eval `[§23]`
- [ ] T16.26 Write acceptance test step 5: verify new objective generated from post-eval `[§23]`
- [ ] T16.27 Write acceptance test step 6: verify loop continues after completion `[§23]`
- [ ] T16.28 Write acceptance test step 7: force empty queue → verify enters WAIT `[§23]`
- [ ] T16.29 Write acceptance test step 8: force opportunity intake → verify pipeline runs `[§23]`
- [ ] T16.30 Write acceptance test step 9: resume from WAIT → verify loop continues `[§23]`
- [ ] T16.31 Write acceptance test step 10: verify hierarchy structure after multiple cycles `[§23]`

## T17 Gate — each <10 min

- [ ] T17.1 Check `test_suite/src/function_registry/` exists `[project rules]`
- [ ] T17.2 Read registry README for entry format `[project rules]`
- [ ] T17.3 Add registry entry for `cooboploop_enqueue_goal` `[§4]`
- [ ] T17.4 Add registry entry for `cooboploop_list_goals` `[§4]`
- [ ] T17.5 Add registry entry for `cooboploop_get_goal` `[§4]`
- [ ] T17.6 Add registry entry for `cooboploop_update_goal_status` `[§4]`
- [ ] T17.7 Add registry entry for `cooboploop_run_source_discovery` `[§3]`
- [ ] T17.8 Add registry entry for `cooboploop_evaluate_goal` `[§5]`
- [ ] T17.9 Add registry entry for `cooboploop_reprioritize_queue` `[§5]`
- [ ] T17.10 Add registry entry for `cooboploop_set_priority_policy` `[§5]`
- [ ] T17.11 Add registry entry for `cooboploop_record_capability_outcome` `[§6]`
- [ ] T17.12 Add registry entry for `cooboploop_get_capability_assessment` `[§6]`
- [ ] T17.13 Add registry entry for `cooboploop_list_capabilities` `[§6]`
- [ ] T17.14 Add registry entry for `cooboploop_start_loop` `[§7]`
- [ ] T17.15 Add registry entry for `cooboploop_stop_loop` `[§7]`
- [ ] T17.16 Add registry entry for `cooboploop_get_loop_status` `[§7]`
- [ ] T17.17 Add registry entry for `cooboploop_run_single_cycle` `[§7]`
- [ ] T17.18 Add registry entry for `cooboploop_step_loop` `[§7]`
- [ ] T17.19 Add registry entry for `cooboploop_run_post_task_evaluation` `[§8]`
- [ ] T17.20 Add registry entry for `cooboploop_get_idle_state` `[§9-10]`
- [ ] T17.21 Add registry entry for `cooboploop_configure_idle_reevaluation_interval` `[§10]`
- [ ] T17.22 Add registry entry for `cooboploop_create_research_objective` `[§11]`
- [ ] T17.23 Add registry entry for `cooboploop_get_hardware_profile` `[§12]`
- [ ] T17.24 Add registry entry for `cooboploop_detect_hardware_changes` `[§12]`
- [ ] T17.25 Add registry entry for `cooboploop_run_inspection` `[§13]`
- [ ] T17.26 Add registry entry for `cooboploop_get_modification_boundary` `[§14]`
- [ ] T17.27 Add registry entry for `cooboploop_set_modification_boundary` `[§14]`
- [ ] T17.28 Add registry entry for `cooboploop_set_autonomous_mode` `[§16]`
- [ ] T17.29 Add registry entry for `cooboploop_get_autonomous_mode` `[§16]`
- [ ] T17.30 Add registry entry for `cooboploop_run_opportunity_intake` `[§17]`
- [ ] T17.31 Add registry entry for `cooboploop_get_pending_external_opportunities` `[§17]`
- [ ] T17.32 Add registry entry for `cooboploop_list_strategic_objectives` `[§18]`
- [ ] T17.33 Add registry entry for `cooboploop_add_strategic_objective` `[§18]`
- [ ] T17.34 Add registry entry for `cooboploop_remove_strategic_objective` `[§18]`
- [ ] T17.35 Add registry entry for `cooboploop_get_objective_hierarchy` `[§19]`
- [ ] T17.36 Add registry entry for `cooboploop_set_mission` `[§19]`
- [ ] T17.37 Add registry entry for `cooboploop_get_autonomy_levels` `[§22]`
- [ ] T17.38 Add registry entry for `cooboploop_promote_autonomy` `[§22]`
- [ ] T17.39 Run `test_suite --list` to confirm all 38 new tools are advertised `[§23]`
- [ ] T17.40 Run `make gate` `[project rules]`
- [ ] T17.41 Read `test_suite_report.json` `[project rules]`
- [ ] T17.42 If errors > 0, fix first error only, repeat gate until all metrics zero `[project rules]`

---

Total micro-tasks: ~280 (was ~200). Expanded with: 22 T0 module creation tasks (for all conformance-mapped modules), 1 design decisions appendix (8 sections resolving all architecture gaps), explicit MCP parameter schemas (38 tools), explicit state transition rules (13 states, 3 rules), SQLite schema (5 tables), and Plan/Execute stage interface specifications.

**Before starting any task, read Section A (Design Decisions) above. It contains all decisions that resolve architecture ambiguities.**
