# CoObOpLoop Gaps (from robot_architecture/v0.0.2.1/CoObOpLoop.md vs codebase)

> Source: gap analysis comparing spec §1-23 against src/cooboploop/, bridge/mcp/handlers/cooboploop_handler.rs, bridge/tools/cooboploop/, database/migrations/cooboploop.rs. 38 MCP tools exist; wiring and loop-stage stubs are the main gaps.

## Critical: Dead-end stubs and missing enqueue paths
- [ ] T-COO-32: Fix `execute_cooboploop_run_single_cycle()` error swallow — calls `runner.run_cycle().ok()` which silently discards errors. Caller sees `status: ok` even when the cycle failed. Should propagate the error to ToolOutput.

## Critical: Loop state machine broken after WAIT
- [ ] T-COO-33: Add auto-restart after heartbeat — `tick_heartbeat()` transitions to `CyclePhase::Observe` but nothing calls `run_cycle()` again. After the loop enters WAIT and the heartbeat fires, the loop is stuck in Observe with no action. `step_loop()` should call `run_cycle()` when phase is Observe (it does, but only when phase != Wait).
- [ ] T-COO-34: Fix `step_loop()` logic — currently calls `tick_heartbeat()` when phase==Wait and returns early, then only calls `run_cycle()` when phase!=Wait. This means WAIT→Observe transition via heartbeat requires a second `step_loop()` call to actually advance. Should restart the cycle after heartbeat fires in the same call.

## Critical: Plan → Execute → Verify chain is disconnected
- [ ] T-COO-35: Wire `plan()` to transition steps — `plan()` creates a draft plan but leaves all steps in `StepStatus::Pending`. `execute()` reads step status (all Pending) and reports them. `verify()` counts Completed steps (zero). Steps are never transitioned to Ready/InProgress/Completed.
- [ ] T-COO-36: Wire `execute()` to run actual plan steps — currently iterates over plan.steps and just formats their current status strings. Does not execute the step.action, does not transition step status, does not produce real results.
- [ ] T-COO-37: Wire `verify()` to update goal status — currently only reads plan.steps status and computes efficiency. Should transition the goal's status to `Active → Verifying → Completed/Failed` based on actual step outcomes.

## Critical: Goal status flow never reaches Active
- [ ] T-COO-38: `select_objective()` only considers `QUEUED`/`ACCEPTED` — the spec (§4) says all accepted objectives enter the queue. DISCOVERED goals (from `enqueue_goal`) are never considered by select because it filters for QUEUED/ACCEPTED only. Should include DISCOVERED or ensure enqueue transitions to ACCEPTED.
- [ ] T-COO-39: `run_cycle()` transitions selected goal to ACTIVE but `plan()` never runs for ACTIVE goals — after `transition(goal, Active)`, the next `plan()` call should execute for that ACTIVE goal. Currently plan() uses `selected_goal` which is set, but plan() doesn't check if the goal is ACTIVE and take action.

## Wiring / Loop Integration (highest priority — loop stages are stubs)
- [ ] T-COO-01: Wire `loop_runner.run_cycle()` stubs — `plan()` (connect to planner engine), `execute()` (run selected goal actions), `verify()` (confirm outcome, not just set fields), `record_experience()` (call `experience_coordinator` methods, not just log `coordinator_active`)
- [ ] T-COO-02: Wire `post_task_evaluation.generate_objectives()` into queue — enqueue generated `AgentGoal` entries after `run_post_task_evaluation()`; loop `generate_new_objectives()` must enqueue results
- [ ] T-COO-03: Wire `inspection.issues_to_objectives()` into queue — `run_inspection()` should enqueue inspection-derived goals; add missing DB table `inspection_issues`
- [ ] T-COO-04: Wire `opportunity.IntakeResult` deferred/accepted into queue — `pending_opportunities` should feed `ObjectiveQueue`; add `opportunity_intake_results` DB table
- [ ] T-COO-05: Wire `research.ResearchManager` into loop — `create_research_objective()` should enqueue; `run_cycle()` should select research goals when idle
- [ ] T-COO-06: Wire `self_improvement.SelfImprovementPipeline` into loop — trigger from `post_task_evaluation` findings; `SelfImprovementGuard` should gate modifications
- [ ] T-COO-07: Wire `learning.LearningPipeline` feedback — `record_success()`/`record_failure()` should feed `LearningPipeline`; updates should adjust `GoalEvaluator` policy and `CapabilityRegistry`

## Idle-State / Deliberate Inactivity (§9-10)
- [ ] T-COO-08: Implement idle evaluation — when `new_goals.is_empty()`, evaluate pending objectives, system maintenance, bug investigation, research, capability development, self-improvement, environmental observation, long-term planning (not just `enter_wait()`)
- [ ] T-COO-09: Implement deliberate inactivity — `IdleState` should check `ObjectiveQueue` empty + no detected problems + maintenance current + no high-value research + no capability gaps + no external opportunities before `WAIT`

## Strategic / Hierarchy / Mission (§18-19)
- [ ] T-COO-10: Wire strategic objectives into evaluation — `StrategicObjectiveRegistry` should influence `GoalEvaluator.compute_priority()` (currently only `StrategicPolicy` boosts by source, not strategic alignment)
- [ ] T-COO-11: Wire objective hierarchy into loop — `ObjectiveHierarchy` (Mission → Strategic → Capability → Project → Task → Action) should constrain `select_objective()`; `get_objective_hierarchy()` should be used by loop
- [ ] T-COO-12: Wire mission into loop — `mission` should constrain objective selection and evaluation; `set_mission()` should affect `GoalEvaluator`

## Hardware / Runtime Adaptation (§12, §22)
- [ ] T-COO-13: Implement hardware-aware runtime adaptation — `detect_hardware_changes()` should trigger `runtime_adapters` selection; `HardwareProfile` should influence `CapabilityRegistry` and loop behavior
- [ ] T-COO-14: Implement runtime adapter replacement — `DefaultRuntimeAdapters` should support LLM/inference runtime switching per architecture §22

## Database / Persistence
- [ ] T-COO-15: Complete database migrations — add `objective_queue` (full schema with execution_history, completion_state), `experience_records`, `learning_updates`, `post_task_evaluations`, `inspection_issues`, `opportunity_intake_results`
- [ ] T-COO-16: Fix `loop_runner` persistence — `LoopRunner` creates `ObjectiveQueue::new()` instead of opening persistent DB; wire `open()` in handler/init

## Handler / Tool Wiring
- [ ] T-COO-17: Fix `execute_cooboploop_evaluate_goal()` — use `self.policy_registry` instead of `GoalEvaluator::default()`
- [ ] T-COO-18: Fix `execute_cooboploop_reprioritize_queue()` — use handler's policy registry, not new evaluator
- [ ] T-COO-19: Fix `execute_cooboploop_run_post_task_evaluation()` — pass `self.queue` and `self.loop_runner`; enqueue results
- [ ] T-COO-20: Fix `execute_cooboploop_create_research_objective()` — enqueue into `ObjectiveQueue`
- [ ] T-COO-21: Add missing `Reflect` stage wiring — `CognitiveCycleStage::Reflect` has no `LoopStage` equivalent; add reflection mechanism to `run_cycle()`

# T2-- Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.
> Detailed task list: [`.agents/t2_PLAN.md`](t2_PLAN.md).

---

# Research Engine Gaps (from robot_architecture/v0.0.2.1/research_engine.md vs src/research/ + bridge + agent wiring)

> Source: `robot_architecture/v0.0.2.1/research_engine.md` (§Architecture, §Tool Catalog, §Data Structures, §Integration Points, §Implementation Order R1-R16). 15 adapter/tool files missing; pipeline uses empty provider list; 9-tier cascade only has 3 tiers; evidence packet stub; no end-to-end verification.

## Adapter / Tool Catalog (missing implementations — R3-R4, R11, R14, per-source adapters)
- [ ] R-ADAPT-01: Implement `wikipedia` adapter (`get_wikipedia`) — `SearchProvider` for `SearchSource::Wikipedia`
- [ ] R-ADAPT-02: Implement `news` adapter (`search_news`) — `SearchSource::News`
- [ ] R-ADAPT-03: Implement `reddit` adapter (`search_reddit`, optional `subreddit`) — `SearchSource::Reddit`
- [ ] R-ADAPT-04: Implement `hackernews` adapter (`search_hackernews`) — `SearchSource::HN`
- [ ] R-ADAPT-05: Implement `youtube` adapter (`search_youtube`, `includeTranscript?`) — video + transcript
- [ ] R-ADAPT-06: Implement `substack` adapter (`search_substack`, `publications`, `maxPosts?`)
- [ ] R-ADAPT-07: Implement `bluesky` adapter (`search_bluesky`, `sort?`)
- [ ] R-ADAPT-08: Implement `telegram` adapter (`search_telegram`, `channel`, `maxMessages?`)
- [ ] R-ADAPT-09: Implement `mastodon` adapter (`search_mastodon`)
- [ ] R-ADAPT-10: Implement `vk` adapter (`search_vk`)
- [ ] R-ADAPT-11: Implement `preprints` adapter (`search_preprints`) — arXiv/bioRxiv/medRxiv
- [ ] R-ADAPT-12: Implement `datasets` adapter (`search_datasets`) — Zenodo/Figshare/OSF
- [ ] R-ADAPT-13: Implement `osm` adapter (`search_osm`, `location?`)
- [ ] R-ADAPT-14: Implement `sec_filings` adapter (`search_sec_filings`, `filingType?`)
- [ ] R-ADAPT-15: Implement `wayback` adapter (`resurrect_dead_link`)
- [ ] R-ADAPT-16: Implement `crossref/openalex` adapter (`verify_citations`, `references`)
- [ ] R-ADAPT-17: Implement `academic` adapters (`find_counter_arguments`, `validate_bibliography`, `bibliography`)
- [ ] R-ADAPT-18: Implement `citation` adapter (`format_citations`, `doi`, `format` — BibTeX/APA/MLA/Chicago/RIS)
- [ ] R-ADAPT-19: Implement `multi-platform` adapter (`detect_trends`, `platforms`)
- [ ] R-ADAPT-20: Implement `content` adapter (`score_reliability`, `urls` — rule-based source quality scoring)

## Pipeline / Core Wiring (R5-R7, R9, R10, R15)
- [ ] R-PIPE-01: Wire providers into `execute_research` / `execute_quick_research` / `execute_deep_research` — currently `ResearchPipeline::new(vec![])` (empty); should include `DuckDuckGoProvider` + optional `BraveProvider` + `MockProvider` for tests
- [ ] R-PIPE-02: Add contradiction detection to pipeline (`pipeline.rs` returns `Vec::new` always); wire `deep_research.rs` `detect_contradictions()` into pipeline or deep mode
- [ ] R-PIPE-03: Add sub-question generation to pipeline for `Mode::Deep` — wire `deep_research.rs` `generate_sub_questions()`
- [ ] R-PIPE-04: Fix provider name in evidence packet — `provider: "pipeline"` should be actual provider name (`provider.name()`)
- [ ] R-PIPE-05: Add cancellation token propagation through pipeline — `errors.rs` `CancellationToken` exists but `pipeline.rs` never checks `is_cancelled()` during loops
- [ ] R-PIPE-06: Add timeout per provider (not just whole future) — architecture §Key Design Rule 5 (Cancellation and Timeouts)
- [ ] R-PIPE-07: Enforce bounded result count at provider level (`SearchQuery::max_results`) — architecture §Context Protection (max 2-5 sources)
- [ ] R-PIPE-08: Implement web-content sanitization defense (R9b) — `sanitize.rs` exists (`strip_html`, `strip_control_chars`, `cap_and_truncate`) but pipeline doesn't enforce it at ACP/MCP boundary; never pass raw HTML across boundary
- [ ] R-PIPE-09: Add token budget estimation before LLM call (R15) — architecture §Context Protection; cap at 5 sources with "and N more" marker

## Evidence Packet / Data Structures (R8, evidence.rs stub)
- [ ] R-EVID-01: Fill `evidence.rs` stub — export/re-export `ResearchResult`, `Source`, `Finding`, `Contradiction` from `mod.rs`; add serialization helpers for MCP tool return
- [ ] R-EVID-02: Add provenance preservation in evidence packet — every `Source` must store `url`, `provider`, `timestamp`, `query`; memory promotion (`knowledge::promote_research_findings`) must preserve full provenance, not just URL
- [ ] R-EVID-03: Add `limitations` population — pipeline always returns `Vec::new`; should include timeout notes, provider failures, source count limits

## 9-Tier Cascade (R10 — agent/decision.rs + loop_runner.rs)
- [ ] R-CASC-01: Implement Tier 4 Skills (`search_skills` / `execute_skill`) in `check_internal_sources()` — architecture §9-Tier Confidence Cascade
- [ ] R-CASC-02: Implement Tier 5 Reflections (`list_reflections_by_status`) in cascade
- [ ] R-CASC-03: Implement Tier 6 Workflows/Plans (`list_workflows` / `get_plan`) in cascade
- [ ] R-CASC-04: Implement Tier 7 World Model (`list_world_entities` / `query_world`) in cascade
- [ ] R-CASC-05: Implement Tier 8 Hypotheses (`list_hypotheses`) in cascade
- [ ] R-CASC-06: Ensure cascade stops at first passing tier (`confidence >= 0.7`) — currently only checks memory/knowledge/experience; must include skills/reflections/workflows/world/hypotheses before reaching research
- [ ] R-CASC-07: Wire `trigger_research_on_failure()` to only fire after all 8 prior tiers fail — currently fires after 3 tiers fail

## Integration Points (R11, R12, R13, R16)
- [ ] R-INT-01: Wire `register_tools()` chain — `search::definitions::all()` includes research tools but `execute_research` uses empty provider list; fix before gate passes
- [ ] R-INT-02: Add `TestRequirement` entries for new adapter tools to `.agents/scripts/test_suite2/src/function_registry/search_tools.rs` and matching `id` cases in `argument_builder.rs` (per R11b — phantom tool prevention, T1-19 root cause)
- [ ] R-INT-03: Integrate memory promotion gate (`R12`) — `knowledge::promote_research_findings()` exists but pipeline doesn't enforce `confidence >= 0.7 AND outcome = solved`; add outcome tracking
- [ ] R-INT-04: Integrate experience recording (`R13`) — `experience::record_research()` exists but `pipeline.rs` never calls it; `failover.rs` references it but isn't called by pipeline; wire `record_research(query, sources, mode, duration, outcome)` after every research operation
- [ ] R-INT-05: Add provider failover (`R14`) — `failover.rs` `try_providers()` exists but pipeline never uses it; wire `BraveProvider` as secondary on `DuckDuckGoProvider` failure
- [ ] R-INT-06: Implement end-to-end verification (`R16`) — 13-step verification path (get_workflow → search_memory → research → tiers 1-8 → provider → rank → extract → evidence packet → LLM → citations → experience → memory promotion → provider failure → cancellation); add `test_suite2` tests for full flow

## Adapter-Specific Wiring (per architecture §Per-Source Adapters)
- [ ] R-ADAPT-21: Ensure `SearchProvider` trait abstraction holds — `duckduckgo.rs` and `brave.rs` implement it; all new adapters must implement `search()`, `name()`, `supports()` without changing pipeline
- [ ] R-ADAPT-22: Wire `JINA_API_KEY` / `BRAVE_API_KEY` config paths (`config.rs` `env_key`) — architecture §Prerequisite (install keys before R3); `jina.rs` and `brave.rs` already use `env_key()` but pipeline doesn't initialize providers with keys

---
# T2-- Reach v0.0.2.

# Gaps — Architecture v0.0.2 / v0.0.2.1 vs Codebase (wiring + missing subsystems)

> Source: `robot_architecture/RoBoT Architecture v0.0.2.md` (v0.0.2 spec, 33 chapters + appendices) + `.agents/t2_PLAN.md` (gap notes L124-156) + `.agents/PLAN.md` (CoObOpLoop/research gaps) + live `src/` inspection.
> Every entry is an actionable task: file/module/function + what is missing + wiring dependency.
> T2 gaps (existing subsystems to upgrade) are listed first; T3 gaps (missing subsystems) follow.

## T2 Gaps — Existing Subsystems (from `.agents/t2_PLAN.md` L124-156 + `src/` inspection)

### Data Contracts & Pipeline Wiring (Chapter 05 + Chapter 3.3/3.4)
- [ ] G-T2-05-01: Add `contract_validator/` module — contracts (`data_contracts/`, 13 files) exist but validation is manual; no runtime enforcement that incoming objects match `Observation`, `ContextPacket`, `MemoryRecord`, `ExperienceRecord`, `Plan`, `Decision`, `ExecutionResult`, `Reflection`, `LearningUpdate` schemas. Wiring: `data_contracts/` -> `database/` -> `pipeline/`.
- [ ] G-T2-05-02: Wire contracts into pipeline execution — `pipeline/mod.rs` defines `contract_for_step()` and `LifecycleStep`/`CognitiveStage` enums, but `execute_full_pipeline()` pushes stages with `"stage_complete"` strings; contracts are not actively consumed. Per `t2_PLAN.md` L152.
- [ ] G-T2-05-03: Implement `pipeline/execute_full_pipeline()` real processing — must invoke `context_engine/`, `memory/`, `experience/`, `planner/`, `execution/`, `reflection/`, `learning/`. Per `t2_PLAN.md` L150-156.

### Context Engine (Chapter 07) — Partial
- [ ] G-T2-07-01: Complete `context_engine/` lifecycle (`create` -> `retrieve` -> `update` -> `expire` -> `archive`). `agent/context.rs` and `bridge/mcp/context.rs` partial. Per `t2_PLAN.md` L127-128.
- [ ] G-T2-07-02: Wire `context_engine/` -> `memory/` (store/retrieve) -> `retrieval_pipeline/` (new, ch 16) -> `prompt_construction/` (new, ch 17). Per `t3_PLAN.md` L650, L656, L662.

### Memory Engine (Chapter 08) — Partial
- [ ] G-T2-08-01: Implement explicit `semantic_memory/` vs `procedural_memory/` split — `memory/` (16 files) flat. Per `t2_PLAN.md` L134-135.
- [ ] G-T2-08-02: Add forgetting mechanism (`forgetting_policy.rs`) using `confidence_system/` (ch 19, missing). Per `t2_PLAN.md` L135.
- [ ] G-T2-08-03: Wire `memory/` -> `memory_hierarchy/` (new, ch 14) -> `database/` (hierarchy tables missing). Per `t2_PLAN.md` L134.

### Experience Engine (Chapter 09) — Partial
- [ ] G-T2-09-01: Complete experience-to-learning promotion — `experience/` (100 files) very complete; promotion to `learning/` manual. Per `t2_PLAN.md` L126-127.
- [ ] G-T2-09-02: Wire `experience/` -> `learning/` -> `cooboploop/` using `confidence_system/` (ch 19). Per `t2_PLAN.md` L126.
- [ ] G-T2-09-03: Complete `experience_graph` traversal/reasoning. Per `t2_PLAN.md` L126.

### Learning Engine (Chapter 10) — Partial
- [ ] G-T2-10-01: Wire `learning/` -> `database/` (`learning_updates` schema missing). Per `t2_PLAN.md` L128.
- [ ] G-T2-10-02: Integrate `strategic_learning/` (new, ch 18) with `cooboploop/` strategic objectives. Per `t2_PLAN.md` L128.
- [ ] G-T2-10-03: Wire `learning/` pipeline from `experience/` -> `learning/` -> `memory/`/`knowledge/` (`reference_full_pipeline()` stub). Per `t2_PLAN.md` L127.

### Planning Engine (Chapter 11) — Partial
- [ ] G-T2-11-01: Implement real planning logic (decomposition, dependency analysis, resource estimation) — `planner/` (7 files) has contracts but no real logic. Per `t2_PLAN.md` L128-129.
- [ ] G-T2-11-02: Wire `planner/` -> `execution/` (ch 12) -> `tool_engine/` (ch 13) — plan steps must become `execution_request/` objects. Per `t2_PLAN.md` L129.

### Execution Engine (Chapter 12) — Critical Gap (7 files, mostly stubs)
- [ ] G-T2-12-01: Implement `ExecutionLifecycle` state machine (`execution/lifecycle.rs`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-02: Implement `ExecutionGraph` wired to `workflows/` (`execution/graph.rs`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-03: Implement `Scheduler` delegation (`execution/scheduler.rs` -> `bridge/app/scheduler.rs`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-04: Implement `ParallelExecution` (`execution/` -> `bridge/app/initialization/workers.rs`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-05: Implement `ResourceManager` (`database/` resource table). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-06: Implement `Checkpointing` (`database/` checkpoint table). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-07: Implement `LongRunningJobs` tracking (`execution/` -> `bridge/app/initialization/job_queue.rs`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-08: Implement `ResultVerification` (`execution/` verifies against `expected_results`). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-09: Complete `execution/` -> `experience/` integration (`execution_result_to_experience()` exists but no `LearningUpdate` produced; `integrated_execution()` creates result but no learning update). Per `t2_PLAN.md` L129, L154.
- [ ] G-T2-12-10: Implement `ExecutionStateMachine` persistence (`database/` state table). Per `t2_PLAN.md` L129.
- [ ] G-T2-12-11: Wire `execution/` -> `agent/loop_runner.rs` (`integrated_execution()` not invoked by `cooboploop/`). Per `t2_PLAN.md` L129.

### Tool Engine (Chapter 13) — Critical Gap (4 files, very minimal)
- [ ] G-T2-13-01: Implement `ToolRegistry` fully (`tools/` 4 files; `bridge/acp/registry.rs` separate; no unified registry). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-02: Implement parameter validation pipeline (`execute_tool()` stub). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-03: Implement isolation/sandboxing (`execution/isolation.rs` exists but `tools/` has no sandboxing; `run_isolated()` not wired to tool execution). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-04: Implement health monitoring (`tools/` no `tool_health_monitoring`). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-05: Implement retry/timeout policies (`execution/` has `RetryPolicy` but `tools/` has none wired). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-06: Implement permission system (`security/` partial; `tools/` no `permission_system` wired; `is_authorized` not enforced). Per `t2_PLAN.md` L130.
- [ ] G-T2-13-07: Wire `tools/` -> `execution/` (ch 12) -> `bridge/mcp/handler.rs`. Per `t2_PLAN.md` L130.

### Memory Hierarchy (Chapter 14) — MISSING
- [ ] G-T2-14-01: Create `memory_hierarchy/` module (`MemoryLayer` enum not present; directory missing). Per `t2_PLAN.md` L134, `t3_PLAN.md` L640-646.
- [ ] G-T2-14-02: Implement `PromotionGate` (`min_age`, `min_confidence`, `min_access_count`). Per `t2_PLAN.md` L134.
- [ ] G-T2-14-03: Implement promotion/demotion pipeline (`promote_record()` hardcoded; `archive_record()` basic). Per `t2_PLAN.md` L134.
- [ ] G-T2-14-04: Wire `memory_hierarchy/` -> `memory/` -> `database/` (hierarchy tables missing). Per `t3_PLAN.md` L644.

### Context Lifecycle (Chapter 15) — MISSING
- [ ] G-T2-15-01: Create `context_lifecycle/` module (directory missing; `agent/context.rs` partial). Per `t2_PLAN.md` L135, `t3_PLAN.md` L646-652.
- [ ] G-T2-15-02: Implement full lifecycle (`create` -> `retrieve` -> `update` -> `expire` -> `archive`). Per `t2_PLAN.md` L135.
- [ ] G-T2-15-03: Wire `context_lifecycle/` -> `context_engine/` -> `memory/` (archive) -> `database/` (context table with expiry). Per `t3_PLAN.md` L650.

### Retrieval Pipeline (Chapter 16) — MISSING
- [ ] G-T2-16-01: Create `retrieval_pipeline/` module (directory missing). Per `t2_PLAN.md` L135, `t3_PLAN.md` L652-658.
- [ ] G-T2-16-02: Implement retrieval stages (`query_understanding`, `query_expansion`, `hybrid_retrieval`, `ranking_engine`, `re-ranking`, `context_filtering`). Per `t3_PLAN.md` L652.
- [ ] G-T2-16-03: Wire `retrieval_pipeline/` -> `memory/` + `knowledge/` + `experience/` -> `prompt_construction/` (ch 17, new). Per `t3_PLAN.md` L656.

### Prompt Construction (Chapter 17) — MISSING
- [ ] G-T2-17-01: Create `prompt_construction/` module (directory missing; logic scattered). Per `t2_PLAN.md` L135, `t3_PLAN.md` L658-664.
- [ ] G-T2-17-02: Implement prompt assembly pipeline (`prompt_layers`, `prompt_assembly_process`, `information_selection`, `context_compression`, `token_budget_management`, `dynamic_prompt_templates`). Per `t3_PLAN.md` L658.
- [ ] G-T2-17-03: Wire `prompt_construction/` -> `retrieval_pipeline/` (ch 16) -> `context_engine/` (ch 7) -> `agent/loop_runner.rs`. Per `t3_PLAN.md` L662.

### Strategic Learning (Chapter 18) — MISSING
- [ ] G-T2-18-01: Create `strategic_learning/` module (`cooboploop/` partial; no dedicated module). Per `t3_PLAN.md` L664-670.
- [ ] G-T2-18-02: Implement strategic updates integration with `learning/` and `execution/`. Per `t3_PLAN.md` L668.
- [ ] G-T2-18-03: Wire `strategic_learning/` -> `cooboploop/` -> `learning/` (ch 10) -> `execution/` (ch 12, feedback loop 12.34). Per `t3_PLAN.md` L668.

### Confidence System (Chapter 19) — Partial
- [ ] G-T2-19-01: Complete `confidence_system/` module (`data_contracts/confidence.rs` partial; no full propagation). Per `t2_PLAN.md` L136-137, `t3_PLAN.md` L670-676.
- [ ] G-T2-19-02: Wire confidence propagation to `memory/`, `knowledge/`, `retrieval_pipeline/`, `execution/`, `tool_engine/`. Per `t3_PLAN.md` L674.
- [ ] G-T2-19-03: Implement `ConfidenceHistory` persistence (`database/` `confidence_history` table missing). Per `t3_PLAN.md` L674.

### Knowledge Graph (Chapter 20) — Partial
- [ ] G-T2-20-01: Complete graph traversal/reasoning (`knowledge/` 7 files; `find_path`, `get_subgraph`, `find_linked_concepts` partial). Per `t2_PLAN.md` L137-138, `t3_PLAN.md` L676-682.
- [ ] G-T2-20-02: Implement `graph_extraction_pipeline` and `entity_resolution`. Per `t3_PLAN.md` L680.
- [ ] G-T2-20-03: Implement `graph_verification` and `graph_compression`. Per `t3_PLAN.md` L680.
- [ ] G-T2-20-04: Wire `knowledge/` -> `world_model/` -> `database/` (graph tables partial). Per `t3_PLAN.md` L680.

### Storage Architecture (Chapter 21) — Partial
- [ ] G-T2-21-01: Create `storage_architecture/` module (`database/` 19 files; no storage-layer abstraction). Per `t2_PLAN.md` L138-139, `t3_PLAN.md` L682-688.
- [ ] G-T2-21-02: Implement hybrid storage model fully (relational + vector + graph + object). Per `t3_PLAN.md` L686.
- [ ] G-T2-21-03: Wire `storage_architecture/` -> `database/` -> `memory/` + `experience/` + `knowledge/` (storage policies hot/warm/cold feeding `memory_hierarchy/`). Per `t3_PLAN.md` L686.

### Database Design (Chapter 22) — Partial
- [ ] G-T2-22-01: Complete schema tables (`MemoryEmbeddings`, `KnowledgeGraph` nodes/edges, `ExperienceSchema`, `WorkflowSchema`, `SkillSchema`, `LessonSchema`, `LearningSchema`, `ConfidenceHistory`, `ConversationSchema`, `PlanningSchema`, `ExecutionSchema`, `AIModelSchema`, `ToolSchema`, `ArchitectureTraceSchema`, `DiagnosticsSchema` missing/stubbed). Per `t2_PLAN.md` L139-140, `t3_PLAN.md` L688-694.
- [ ] G-T2-22-02: Add migrations for ch 12-23, 25-30 tables (`database/migrations/` partial). Per `t3_PLAN.md` L692.

### Background Workers (Chapter 23) — Partial
- [ ] G-T2-23-01: Complete dedicated worker types (`memory_worker`, `experience_worker`, `learning_worker`, `knowledge_graph_worker`, `maintenance_worker`). Per `t2_PLAN.md` L140-141, `t3_PLAN.md` L694-700.
- [ ] G-T2-23-02: Implement scheduling fully (`Immediate`, `Scheduled`, `Resource-Based`). Per `t2_PLAN.md` L141.
- [ ] G-T2-23-03: Implement SQLite coordination (`database/` `worker_tasks`, `worker_status`, `worker_history` missing). Per `t2_PLAN.md` L141.
- [ ] G-T2-23-04: Wire `background_workers/` (new) -> `bridge/app/initialization/workers.rs` -> `execution/` (long-running jobs) -> `observability/` (ch 27). Per `t3_PLAN.md` L698.

### Security and Trust (Chapter 25) — Partial
- [ ] G-T2-25-01: Create `security/` module (`agent/safety_gate/` exists; `security/` directory missing). Per `t2_PLAN.md` L142-143, `t3_PLAN.md` L706-712.
- [ ] G-T2-25-02: Implement `PermissionArchitecture`, `MemoryProtection`, `RollbackAndRecovery`. Per `t3_PLAN.md` L710.
- [ ] G-T2-25-03: Implement full `AuditSystem` (pipeline `actor`, `action`, `target`, `confidence_change`, `reason` -> DB audit table). Per `t3_PLAN.md` L710.
- [ ] G-T2-25-04: Implement `TrustEvaluationPipeline`, `RiskClassification`, `ReputationSystem`. Per `t3_PLAN.md` L710.
- [ ] G-T2-25-05: Wire `security/` -> `agent/safety_gate/` -> `tool_engine/` -> `execution/` -> `database/` (encrypted fields) -> `observability/` (audit). Per `t3_PLAN.md` L710.

### Self-Improvement (Chapter 26) — Partial
- [ ] G-T2-26-01: Create `self_improvement/` module (`agent/safety_gate/` rollback; `experience/` corrections; `cooboploop/` loop; no dedicated module). Per `t2_PLAN.md` L143-144, `t3_PLAN.md` L712-718.
- [ ] G-T2-26-02: Implement `SelfImprovementLoop` (Experience -> Candidates -> Hypothesis -> Experimentation -> Evolution -> Consolidation). Per `t3_PLAN.md` L716.
- [ ] G-T2-26-03: Implement `HypothesisSystem`, `ControlledExperimentation`, `EvolutionBoundaries`. Per `t3_PLAN.md` L716.
- [ ] G-T2-26-04: Wire `self_improvement/` -> `learning/` -> `execution/` -> `agent/safety_gate/` (rollback) -> `developer_interface/` (ch 28, human approval). Per `t3_PLAN.md` L716.

### Observability (Chapter 27) — Partial
- [ ] G-T2-27-01: Create `observability/` module (`bridge/logging.rs` exists; `experience_recorder_diagnostics.rs` exists; directory missing). Per `t2_PLAN.md` L144-145, `t3_PLAN.md` L718-724.
- [ ] G-T2-27-02: Implement `CognitiveTraceModel` (`correlation_id` tracking; `TraceStorage` `CognitiveEvents`/`DecisionRecords` tables missing). Per `t3_PLAN.md` L722.
- [ ] G-T2-27-03: Implement `EventArchitecture` fully (`CognitiveEventType` 5 types exist; routing partial; `event_router/` missing). Per `t3_PLAN.md` L722.
- [ ] G-T2-27-04: Implement `PerformanceMonitoring`, `AnomalyDetection`. Per `t3_PLAN.md` L722.
- [ ] G-T2-27-05: Wire `observability/` -> `execution/` (traces 12.26-12.27) -> `tool_engine/` (audit) -> `database/` (metrics/log) -> `developer_interface/` (ch 28, dashboards). Per `t3_PLAN.md` L722.

### Developer Interface (Chapter 28) — Partial
- [ ] G-T2-28-01: Create `developer_interface/` module (`cli/` 11 files exists; directory missing). Per `t2_PLAN.md` L145-146, `t3_PLAN.md` L724-730.
- [ ] G-T2-28-02: Implement `CognitiveExplorer`, `MemoryManagementInterface`, `KnowledgeGraphExplorer`, `WorkerManagementInterface`, `LearningEvolutionInterface`. Per `t3_PLAN.md` L728.
- [ ] G-T2-28-03: Implement `DebuggingTools` (`TraceReplay`, `StateInspection`, `EventSearch`). Per `t3_PLAN.md` L728.
- [ ] G-T2-28-04: Implement `ControlPlaneSecurity` (auth/authz for control plane). Per `t3_PLAN.md` L728.
- [ ] G-T2-28-05: Wire `developer_interface/` -> `execution/` (approval gate 12.29) -> `self_improvement/` (approval 26) -> `configuration/` (ch 29) -> `observability/` (ch 27, dashboards). Per `t3_PLAN.md` L728.

### Configuration (Chapter 29) — Partial
- [ ] G-T2-29-01: Create `configuration/` module (`bridge/app/initialization/` partial; `cooboploop/` settings; directory missing). Per `t2_PLAN.md` L146-147, `t3_PLAN.md` L730-736.
- [ ] G-T2-29-02: Implement `SystemConfiguration` fully (`memory_enabled`, `learning_enabled`, `planning_enabled`, `workers_enabled` concepts; no full module with validation). Per `t3_PLAN.md` L734.
- [ ] G-T2-29-03: Implement `WorkerRuntimeManagement` (`memory_worker` normal, `learning_worker` low; scheduling partial). Per `t3_PLAN.md` L734.
- [ ] G-T2-29-04: Implement `StartupSequence` and `ShutdownSequence` (DB -> storage -> memory -> experience -> learning -> planning -> execution -> tool -> model -> communication -> coordination; reverse for shutdown). Per `t3_PLAN.md` L734.
- [ ] G-T2-29-05: Implement `FeatureFlags` and `HotReloading` (`RuntimeProfile` 3 profiles partial). Per `t3_PLAN.md` L734.
- [ ] G-T2-29-06: Wire `configuration/` -> `database/` (config table) -> `execution/` (budget/settings) -> `developer_interface/` (ch 28, config UI) -> `observability/` (ch 27, config audit). Per `t3_PLAN.md` L734.

### Testing (Chapter 30) — Partial
- [ ] G-T2-30-01: Create `testing/` module inside `src/` (`.agents/scripts/test_suite2/` external; `testing/` directory missing). Per `t2_PLAN.md` L147-148, `t3_PLAN.md` L736-742.
- [ ] G-T2-30-02: Implement internal validation hooks (`MemoryValidation`, `KnowledgeGraphValidation`, `ExperienceValidation`, `PlanningValidation`, `ExecutionValidation`, `ConfidenceValidation`, `LearningValidation`, `EvolutionTesting`, `BenchmarkSystem`, `ReplayTesting`, `ArchitectureTraceValidation`). Per `t3_PLAN.md` L740.
- [ ] G-T2-30-03: Wire `testing/` -> `execution/` (verification 12.31) -> `tool_engine/` (capability validation) -> `database/` (test fixtures) -> `developer_interface/` (ch 28, test runner UI). Per `t3_PLAN.md` L740.

### Deployment (Chapter 31) — Partial
- [ ] G-T2-31-01: Create `deployment/` module (directory missing in `src/`). Per `t2_PLAN.md` L148-149, `t3_PLAN.md` L742-748.
- [ ] G-T2-31-02: Implement `BootstrapProcess` fully (`BootstrapStep` 11 steps; `run_bootstrap()` partial; `graceful_shutdown()` missing). Per `t3_PLAN.md` L746.
- [ ] G-T2-31-03: Implement `EnvironmentDetection`, `HealthMonitoring`, `BackupArchitecture`, `UpdateArchitecture`, `OfflineOperation`. Per `t3_PLAN.md` L746.
- [ ] G-T2-31-04: Wire `deployment/` -> `configuration/` (ch 29) -> `database/` (deployment log) -> `developer_interface/` (ch 28, deploy UI). Per `t3_PLAN.md` L746.

## T3 Gaps — Missing Subsystems (from `.agents/t3_PLAN.md` gap analysis L501-881 + `t2_PLAN.md` L124-156)

### Cross-Chapter Wiring Gaps (from `t2_PLAN.md` L150-156)
- [ ] G-T3-WIRE-01: Pipeline contracts not actively consumed — `data_contracts/` contracts exist but `pipeline/` does not use them in execution (`execute_full_pipeline()` pushes string results, not structured contracts). Per `t2_PLAN.md` L152.
- [ ] G-T3-WIRE-02: `execution/` -> `experience/` integration partial — `execution_result_to_experience()` exists but no `LearningUpdate` produced; `integrated_execution()` creates result but no learning update. Per `t2_PLAN.md` L154.
- [ ] G-T3-WIRE-03: `execution/` -> `learning/` integration missing — `integrated_execution()` produces no `LearningUpdate`; `reference_full_pipeline()` is stub. Per `t2_PLAN.md` L154.
- [ ] G-T3-WIRE-04: `context_engine/` -> `memory/` -> `knowledge/` retrieval chain — all retrieval functions return empty; no end-to-end retrieval wired. Per `t2_PLAN.md` L155.
- [ ] G-T3-WIRE-05: `conversation/` -> `context_engine/` -> `planner/` -> `execution/` -> `reflection/` -> `learning/` full lifecycle — no end-to-end wiring exists; `pipeline/` defines stages but does not connect them to real subsystems. Per `t2_PLAN.md` L156.

### CoObOpLoop Wiring Gaps (from `.agents/PLAN.md` L5-54)
- [ ] G-T3-COO-01: Wire `loop_runner.run_cycle()` stubs (`plan()`, `execute()`, `verify()`, `record_experience()`). Per `.agents/PLAN.md` L24.
- [ ] G-T3-COO-02: Wire `post_task_evaluation.generate_objectives()` into queue. Per `.agents/PLAN.md` L25.
- [ ] G-T3-COO-03: Wire `inspection.issues_to_objectives()` into queue (add DB table `inspection_issues`). Per `.agents/PLAN.md` L26.
- [ ] G-T3-COO-04: Wire `opportunity.IntakeResult` deferred/accepted into queue (add DB table `opportunity_intake_results`). Per `.agents/PLAN.md` L27.
- [ ] G-T3-COO-05: Wire `research.ResearchManager` into loop. Per `.agents/PLAN.md` L28.
- [ ] G-T3-COO-06: Wire `self_improvement.SelfImprovementPipeline` into loop. Per `.agents/PLAN.md` L29.
- [ ] G-T3-COO-07: Wire `learning.LearningPipeline` feedback. Per `.agents/PLAN.md` L30.
- [ ] G-T3-COO-08: Implement idle evaluation (`new_goals.is_empty()` path). Per `.agents/PLAN.md` L33.
- [ ] G-T3-COO-09: Implement deliberate inactivity (`IdleState` checks before `WAIT`). Per `.agents/PLAN.md` L34.
- [ ] G-T3-COO-10: Wire strategic objectives into `GoalEvaluator.compute_priority()`. Per `.agents/PLAN.md` L37.
- [ ] G-T3-COO-11: Wire objective hierarchy (`Mission` -> `Strategic` -> `Capability` -> `Project` -> `Task` -> `Action`) into `select_objective()`. Per `.agents/PLAN.md` L38.
- [ ] G-T3-COO-12: Wire mission (`mission`) into objective selection/evaluation. Per `.agents/PLAN.md` L39.
- [ ] G-T3-COO-13: Implement hardware-aware runtime adaptation (`detect_hardware_changes()` -> `runtime_adapters`). Per `.agents/PLAN.md` L42.
- [ ] G-T3-COO-14: Implement runtime adapter replacement (`LLM/inference` switching). Per `.agents/PLAN.md` L43.
- [ ] G-T3-COO-15: Complete DB migrations (`objective_queue`, `experience_records`, `learning_updates`, `post_task_evaluations`, `inspection_issues`, `opportunity_intake_results`). Per `.agents/PLAN.md` L46.
- [ ] G-T3-COO-16: Fix `loop_runner` persistence (`ObjectiveQueue::new()` instead of persistent DB). Per `.agents/PLAN.md` L47.
- [ ] G-T3-COO-17: Fix `execute_cooboploop_evaluate_goal()` (use `self.policy_registry`). Per `.agents/PLAN.md` L50.
- [ ] G-T3-COO-18: Fix `execute_cooboploop_reprioritize_queue()` (use handler's policy registry). Per `.agents/PLAN.md` L51.
- [ ] G-T3-COO-20: Fix `execute_cooboploop_create_research_objective()` (enqueue into `ObjectiveQueue`). Per `.agents/PLAN.md` L53.
- [ ] G-T3-COO-21: Add missing `Reflect` stage wiring (`CognitiveCycleStage::Reflect` -> `LoopStage`). Per `.agents/PLAN.md` L54.
- [ ] G-T3-COO-32: Fix `execute_cooboploop_run_single_cycle()` error swallow (`.ok()` discards errors). Per `.agents/PLAN.md` L8.
- [ ] G-T3-COO-33: Add auto-restart after heartbeat (`tick_heartbeat()` -> `run_cycle()`). Per `.agents/PLAN.md` L11.
- [ ] G-T3-COO-34: Fix `step_loop()` logic (heartbeat + `run_cycle()` in same call). Per `.agents/PLAN.md` L12.
- [ ] G-T3-COO-35: Wire `plan()` to transition steps (`Pending` -> `Ready`/`InProgress`/`Completed`). Per `.agents/PLAN.md` L15.
- [ ] G-T3-COO-36: Wire `execute()` to run actual plan steps (execute `step.action`, transition status, produce real results). Per `.agents/PLAN.md` L16.
- [ ] G-T3-COO-37: Wire `verify()` to update goal status (`Active` -> `Verifying` -> `Completed`/`Failed`). Per `.agents/PLAN.md` L17.
- [ ] G-T3-COO-38: Fix `select_objective()` to include `DISCOVERED` goals. Per `.agents/PLAN.md` L20.
- [ ] G-T3-COO-39: Wire `plan()` for `ACTIVE` goals (`run_cycle()` -> `plan()` for active goal). Per `.agents/PLAN.md` L21.

### Research Engine Gaps (from `.agents/PLAN.md` L64-125)
- [ ] G-T3-R-01: Implement adapter tools (R-ADAPT-01 to R-ADAPT-20): `wikipedia`, `news`, `reddit`, `hackernews`, `youtube`, `substack`, `bluesky`, `telegram`, `mastodon`, `vk`, `preprints`, `datasets`, `osm`, `sec_filings`, `wayback`, `crossref/openalex`, `academic`, `citation`, `multi-platform`, `content`. Per `.agents/PLAN.md` L68-88.
- [ ] G-T3-R-02: Wire providers into `execute_research` (R-PIPE-01): `ResearchPipeline::new(vec![])` empty; include `DuckDuckGoProvider` + `BraveProvider` + `MockProvider`. Per `.agents/PLAN.md` L91.
- [ ] G-T3-R-03: Add contradiction detection (R-PIPE-02): `pipeline.rs` returns `Vec::new`; wire `deep_research.rs` `detect_contradictions()`. Per `.agents/PLAN.md` L92.
- [ ] G-T3-R-04: Add sub-question generation for `Mode::Deep` (R-PIPE-03): wire `deep_research.rs` `generate_sub_questions()`. Per `.agents/PLAN.md` L93.
- [ ] G-T3-R-05: Fix provider name in evidence packet (R-PIPE-04): `provider: "pipeline"` -> `provider.name()`. Per `.agents/PLAN.md` L94.
- [ ] G-T3-R-06: Add cancellation token propagation (R-PIPE-05): `pipeline.rs` never checks `is_cancelled()`. Per `.agents/PLAN.md` L95.
- [ ] G-T3-R-07: Add timeout per provider (R-PIPE-06): architecture §Key Design Rule 5. Per `.agents/PLAN.md` L96.
- [ ] G-T3-R-08: Enforce bounded result count (R-PIPE-07): `SearchQuery::max_results`; cap at 5 sources. Per `.agents/PLAN.md` L97.
- [ ] G-T3-R-09: Implement web-content sanitization defense (R-PIPE-08): `sanitize.rs` exists but pipeline doesn't enforce at ACP/MCP boundary. Per `.agents/PLAN.md` L98.
- [ ] G-T3-R-10: Add token budget estimation (R-PIPE-09): cap at 5 sources with "and N more" marker. Per `.agents/PLAN.md` L99.
- [ ] G-T3-R-11: Fill `evidence.rs` stub (R-EVID-01): export/re-export `ResearchResult`, `Source`, `Finding`, `Contradiction`; add serialization helpers. Per `.agents/PLAN.md` L102.
- [ ] G-T3-R-12: Add provenance preservation (R-EVID-02): `url`, `provider`, `timestamp`, `query` per `Source`; memory promotion preserves full provenance. Per `.agents/PLAN.md` L103.
- [ ] G-T3-R-13: Add `limitations` population (R-EVID-03): timeout notes, provider failures, source count limits. Per `.agents/PLAN.md` L104.
- [ ] G-T3-R-14: Implement Tier 4 Skills in cascade (R-CASC-01): `search_skills` / `execute_skill` in `check_internal_sources()`. Per `.agents/PLAN.md` L107.
- [ ] G-T3-R-15: Implement Tier 5 Reflections (R-CASC-02): `list_reflections_by_status`. Per `.agents/PLAN.md` L108.
- [ ] G-T3-R-16: Implement Tier 6 Workflows/Plans (R-CASC-03): `list_workflows` / `get_plan`. Per `.agents/PLAN.md` L109.
- [ ] G-T3-R-17: Implement Tier 7 World Model (R-CASC-04): `list_world_entities` / `query_world`. Per `.agents/PLAN.md` L110.
- [ ] G-T3-R-18: Implement Tier 8 Hypotheses (R-CASC-05): `list_hypotheses`. Per `.agents/PLAN.md` L111.
- [ ] G-T3-R-19: Ensure cascade stops at first passing tier (R-CASC-06): `confidence >= 0.7`; include skills/reflections/workflows/world/hypotheses before research. Per `.agents/PLAN.md` L112.
- [ ] G-T3-R-20: Wire `trigger_research_on_failure()` after all 8 tiers fail (R-CASC-07): currently fires after 3 tiers. Per `.agents/PLAN.md` L113.
- [ ] G-T3-R-21: Wire `register_tools()` chain (R-INT-01): `search::definitions::all()` includes tools but `execute_research` uses empty provider list. Per `.agents/PLAN.md` L116.
- [ ] G-T3-R-22: Add `TestRequirement` entries (R-INT-02): `.agents/scripts/test_suite2/src/function_registry/search_tools.rs` + `argument_builder.rs`. Per `.agents/PLAN.md` L117.
- [ ] G-T3-R-23: Integrate memory promotion gate (R-INT-03): `confidence >= 0.7 AND outcome = solved`. Per `.agents/PLAN.md` L118.
- [ ] G-T3-R-24: Integrate experience recording (R-INT-04): `pipeline.rs` never calls `experience::record_research()`. Per `.agents/PLAN.md` L119.
- [ ] G-T3-R-25: Add provider failover (R-INT-05): `failover.rs` `try_providers()` never used by pipeline. Per `.agents/PLAN.md` L120.
- [ ] G-T3-R-26: Implement end-to-end verification (R-INT-06): 13-step verification path; add `test_suite2` tests. Per `.agents/PLAN.md` L121.

---

# T2-- Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.
> Detailed task list: [`.agents/t2_PLAN.md`](t2_PLAN.md).

---

# T3-- Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Detailed task list: [`.agents/t3_PLAN.md`](t3_PLAN.md).

---
