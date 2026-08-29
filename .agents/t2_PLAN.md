# TIER 2 PLAN — Reach v0.0.2

## Purpose
Upgrade the existing subsystems to the v0.0.2 architecture in a dependency-first order.

## Execution rule for small tasks
Treat each bullet as a single 10-15 minute increment: one tiny code change, one verification run, then stop.
Sub-bullets (▸) are children of the parent task above them — complete them in order.

## 0. Architecture foundations and invariants
Set the rules that every v0.0.2 subsystem must preserve.

# pre T2-01 — Research Engine (External Knowledge Acquisition) — FIRST TASK (moved from PLAN.md)

> Source: `research_engine.txt` — full 21-section requirements document.
> Core principle: The LLM decides it needs information; the Research Engine determines how to obtain it.

## Concept / Design Rules / Phased Micro-Tasks (see `.agents/PLAN.md` archive for full text)

- [ ] Phase A: Foundation (provider-independent core) — `SearchProvider` trait, types, mock provider.
- [ ] Phase B: First provider + raw MCP tools (`web_search`, `web_open`, `quick_research`, `deep_research`).
- [ ] Phase C: Pipeline — ranking, selection, passage extraction, contradiction detection, security.
- [ ] Phase D: Cognitive integration — trigger, experience recording, memory promotion, failover.
- [ ] Phase E: Hardening — security tests, docs, final gate.

Full specification preserved in `.agents/PLAN.md` (lines 1194-1520, archive copy).

- [ ] **T2-01** Write a short v0.0.2 architecture note covering persistence, continuity, memory-first design, experience-based learning, and controlled evolution.
- [ ] **T2-02** Write a short v0.0.2 architecture note covering modularity, explainability, event-driven behavior, confidence-based decisions, and controlled evolution.
- [ ] **T2-03** Write a subsystem ownership map with one owner per subsystem and no hidden cross-ownership.
- [ ] **T2-04** Write the canonical data-flow path for inputs, internal pipelines, and outputs.
- [ ] **T2-05** Write the shared invariants for identity and correlation.
- [ ] **T2-06** Write the shared invariants for provenance, evidence, uncertainty, failure visibility, and versioned evolution.
- [ ] **T2-07** Write the v0.0.2 communication model note: event-driven coordination instead of direct implementation coupling.

## 1. Data Contracts first
These types become the shared shape for the rest of Tier 2.

- [ ] **T2-08** Create `src/data_contracts/` module skeleton with `mod.rs`.
- [ ] **T2-09** Add the shared contract version field and shared traits.
- [ ] **T2-10** Add common metadata fields for version, source, and timestamp.
- [ ] **T2-11** Add common metadata fields for correlation, confidence, and provenance.
- [ ] **T2-12** Add the `Observation` struct.
- [ ] **T2-13** Add a serde round-trip test for `Observation`.
- [ ] **T2-14** Add the `ContextPacket` struct.
- [ ] **T2-15** Add a serde round-trip test for `ContextPacket`.
- [ ] **T2-16** Add the `MemoryRecord` struct.
- [ ] **T2-17** Add a serde round-trip test for `MemoryRecord`.
- [ ] **T2-18** Add the `ExperienceRecord` alias or migration target.
- [ ] **T2-19** Add a serde round-trip test for `ExperienceRecord`.
- [ ] **T2-20** Add the `Plan` struct.
- [ ] **T2-21** Add a serde round-trip test for `Plan`.
- [ ] **T2-22** Add the `Decision` struct.
- [ ] **T2-23** Add a serde round-trip test for `Decision`.
- [ ] **T2-24** Add the `ExecutionResult` struct.
- [ ] **T2-25** Add a serde round-trip test for `ExecutionResult`.
- [ ] **T2-26** Add the `Reflection` struct.
- [ ] **T2-27** Add a serde round-trip test for `Reflection`.
- [ ] **T2-28** Add the `LearningUpdate` struct.
- [ ] **T2-29** Add a serde round-trip test for `LearningUpdate`.
- [ ] **T2-30** Add adapters that convert legacy subsystem types into shared contracts without losing provenance.

## 2. Memory Engine
Bring memory up to contract shape before upgrading higher-level consumers.

- [ ] **T2-31** Add explicit memory lifecycle states in `src/memory/`.
- [ ] **T2-32** Add the promotion gate for Working, Candidate, Accepted, Permanent, and Archived states.
- [ ] **T2-33** Add working-memory and long-term-memory concepts.
- [ ] **T2-34** Add promotion logic between working and long-term memory.
- [ ] **T2-35** Add episodic and semantic memory type distinctions.
- [ ] **T2-36** Add procedural and experience-linked memory type distinctions.
- [ ] **T2-37** Add a confidence field to memories.
- [ ] **T2-38** Make retrieval preserve the stored confidence value.
- [ ] **T2-39** Add memory provenance/source fields.
- [ ] **T2-40** Add memory relationship-graph support.
  - [ ] **▸** Define `MemoryNode` struct with id, content, type, and confidence fields in `src/memory/graph.rs`.
  - [ ] **▸** Define `MemoryEdge` struct with source_id, target_id, relationship_type, and confidence fields.
  - [ ] **▸** Add `memory_edges` table with migration (SQLite schema, create index on source_id).
  - [ ] **▸** Implement `insert_node` and `insert_edge` functions in `src/memory/graph.rs`.
  - [ ] **▸** Implement `get_connections` query that returns edges for a given node id.
  - [ ] **▸** Implement `find_path` breadth-first traversal between two node ids.
- [ ] **T2-41** Add retrieval ranking rules that prefer relevant, confident, and recent records.
  - [ ] **▸** Add `rank_score` field and `rank` method to `MemoryRecord` in `src/memory/mod.rs`.
  - [ ] **▸** Implement `rank_by_confidence` — multiply confidence by 0.4 weight.
  - [ ] **▸** Implement `rank_by_recency` — log-scaled decay from current timestamp, weight 0.3.
  - [ ] **▸** Implement `rank_by_relevance` — placeholder that returns 0.3 (to be filled by search matching).
  - [ ] **▸** Wire `rank_by_confidence`, `rank_by_recency`, `rank_by_relevance` into a combined `ranked_search` function.
- [ ] **T2-42** Add duplicate-merge consolidation.
  - [ ] **▸** Add `merge_duplicates` function that groups records by similar content hash.
  - [ ] **▸** Implement dedup — keep highest-confidence record, merge provenance fields.
  - [ ] **▸** Add `consolidated_from` field to `MemoryRecord` to track merged sources.
- [ ] **T2-43** Add summarization for aging low-importance memories.
  - [ ] **▸** Add `importance` field (score 0.0-1.0) to `MemoryRecord`.
  - [ ] **▸** Add `summarize` method placeholder in `src/memory/mod.rs`.
  - [ ] **▸** Add `summarized_into` field to track which records were merged into a summary.
- [ ] **T2-44** Keep anchor memories standalone during consolidation.
  - [ ] **▸** Add `is_anchor` boolean field to `MemoryRecord`.
  - [ ] **▸** Guard `merge_duplicates` to skip records where `is_anchor` is true.
- [ ] **T2-45** Add pruning policy for low-value or aged memories.
  - [ ] **▸** Add `prune_below_importance(importance_threshold: f32)` function in `src/memory/store.rs`.
  - [ ] **▸** Add `prune_older_than(max_age_secs: u64)` function in `src/memory/store.rs`.
  - [ ] **▸** Add `prune` entry point that combines importance + age pruning.
- [ ] **T2-46** Migrate `MemoryRecord` to the data-contract type.

## 3. Experience Engine
Upgrade the record shape before adding scoring and propagation.

- [ ] **T2-47** Add the base `ExperienceRecord` fields for goal and plan_id.
- [ ] **T2-48** Add the base `ExperienceRecord` fields for result and success.
- [ ] **T2-49** Add the base `ExperienceRecord` fields for execution_time and cost.
- [ ] **T2-50** Add the base `ExperienceRecord` fields for confidence_change and tool_usage.
- [ ] **T2-51** Add the base `ExperienceRecord` fields for lessons and related refs.
- [ ] **T2-52** Add experience categories for conversation and planning.
- [ ] **T2-53** Add experience categories for tool, execution, learning, and code.
- [ ] **T2-54** Add outcome tracking fields so experience stores what happened.
- [ ] **T2-55** Add failure-analysis fields so experience stores why it happened.
- [ ] **T2-56** Add lesson-extraction fields for reusable takeaways.
- [ ] **T2-57** Add multi-factor success scoring.
  - [ ] **▸** Define `ExperienceScore` struct with success_rate, confidence_delta, and execution_efficiency fields in `src/experience/mod.rs`.
  - [ ] **▸** Implement `calc_success_rate` — ratio of successful sub-tasks to total sub-tasks.
  - [ ] **▸** Implement `calc_confidence_delta` — final_confidence minus initial_confidence.
  - [ ] **▸** Implement `calc_efficiency` — inverse of normalized execution time.
  - [ ] **▸** Implement `compute_score` combining all three factors with configurable weights.
- [ ] **T2-58** Add confidence propagation to memory.
  - [ ] **▸** Add `propagate_confidence_to_memory(experience_id, memory_id, delta)` function in `src/experience/mod.rs`.
  - [ ] **▸** Wire the function to be called after `compute_score` in the experience pipeline.
- [ ] **T2-59** Add confidence propagation to relationships and tools.
  - [ ] **▸** Add `propagate_confidence_to_tool(tool_name, delta)` function in `src/experience/mod.rs`.
  - [ ] **▸** Add `propagate_confidence_to_relationship(source, target, delta)` function.
- [ ] **T2-60** Add experience relationships between related events.
  - [ ] **▸** Add `related_experience_ids` field to `ExperienceRecord` in `src/data_contracts/mod.rs`.
  - [ ] **▸** Add `link_experiences(id_a, id_b)` function in `src/experience/mod.rs`.
  - [ ] **▸** Add `get_related_experiences(id)` query function.
- [ ] **T2-61** Migrate `ExperienceRecord` to the data-contract type.

## 4. Knowledge Graph
Build the storage layer before traversal and extraction.

- [ ] **T2-62** Add the `knowledge_nodes` table and migration.
- [ ] **T2-63** Add the `knowledge_edges` table and migration.
- [ ] **T2-64** Add relationship confidence on knowledge edges.
- [ ] **T2-65** Add concept-relationship fields for structured understanding.
- [ ] **T2-66** Add entity resolution for aliases like "rustc" and "Rust Compiler".
  - [ ] **▸** Add `EntityResolution` struct with `canonical_id` and `aliases: Vec<String>` fields.
  - [ ] **▸** Add `resolve_entity(name) -> Option<String>` function that returns canonical id for an alias.
  - [ ] **▸** Add `register_alias(canonical_id, alias)` function to register new aliases.
- [ ] **T2-67** Add graph traversal queries for relationship chains.
  - [ ] **▸** Add `traverse_from(start_id, max_depth)` function using BFS in `src/knowledge/graph.rs`.
  - [ ] **▸** Add `find_all_paths(start_id, end_id, max_paths)` function.
  - [ ] **▸** Add `get_subgraph(node_id, radius)` function for neighborhood queries.
- [ ] **T2-68** Add discovery queries for linked concepts and supporting evidence.
  - [ ] **▸** Add `find_linked_concepts(node_id, relationship_type)` query.
  - [ ] **▸** Add `find_supporting_evidence(node_id)` query that returns edges pointing to the node.
- [ ] **T2-69** Add entity-detection logic for the graph-extraction pipeline.
  - [ ] **▸** Define `ExtractionInput` struct with `text: String` and `source: String` in `src/knowledge/extraction.rs`.
  - [ ] **▸** Define `DetectedEntity` struct with `text`, `entity_type`, `confidence`, and `position` fields.
  - [ ] **▸** Add `detect_entities(input) -> Vec<DetectedEntity>` placeholder function using simple keyword matching.
  - [ ] **▸** Add unit test for `detect_entities` with a known entity in text.
- [ ] **T2-70** Add relationship-extraction logic for the graph-extraction pipeline.
  - [ ] **▸** Define `DetectedRelationship` struct with `source_id`, `target_id`, `type_`, `confidence`, and `trigger_text` fields.
  - [ ] **▸** Add `extract_relationships(entities, text) -> Vec<DetectedRelationship>` placeholder using simple pattern matching.
  - [ ] **▸** Add unit test for `extract_relationships` with a known relationship pattern in text.
- [ ] **T2-71** Add confidence-evaluation logic for the graph-extraction pipeline.
  - [ ] **▸** Define `EvaluationCriteria` struct with `source_trustworthiness`, `text_clarity`, and `entity_count` fields.
  - [ ] **▸** Add `evaluate_confidence(entities, relationships, criteria) -> f32` function with weighted scoring.
  - [ ] **▸** Add `adjust_confidence(entities, relationships, threshold)` function to filter low-confidence results.
- [ ] **T2-72** Add graph-update and integration logic for the graph-extraction pipeline.
  - [ ] **▸** Add `apply_extractions(entities, relationships)` function that calls `insert_node`/`insert_edge`.
  - [ ] **▸** Add `run_extraction(text, source) -> (Vec<Entity>, Vec<Relationship>)` pipeline function.
  - [ ] **▸** Add MCP tool handler stub that calls `run_extraction` and returns results.

## 5. Learning Engine
Make learning explicit after experience and knowledge are contract-shaped.

- [ ] **T2-73** Formalize the learning pipeline entry in `src/learning/`.
- [ ] **T2-74** Add reflection-to-candidate promotion logic.
- [ ] **T2-75** Add candidate-to-evaluation logic.
- [ ] **T2-76** Add evaluation-to-promotion logic.
- [ ] **T2-77** Add promotion-to-consolidation logic.
- [ ] **T2-78** Add pattern discovery from repeated successful experiences.
  - [ ] **▸** Define `Pattern` struct with `frequency`, `success_rate`, `context_signature`, and `actions` fields in `src/learning/mod.rs`.
  - [ ] **▸** Add `group_by_context_signature(experiences) -> HashMap<String, Vec<ExperienceId>>` helper.
  - [ ] **▸** Add `detect_patterns(experiences, min_frequency: u32) -> Vec<Pattern>` that iterates grouped experiences.
  - [ ] **▸** Add `Pattern` store/DB table stub with `id`, `signature`, `frequency`, `success_rate` columns.
  - [ ] **▸** Add `insert_pattern(pattern)` and `get_patterns(min_success_rate)` DB functions.
- [ ] **T2-79** Add knowledge extraction from observed patterns.
  - [ ] **▸** Define `ExtractedKnowledge` struct with `pattern_id`, `rule`, `confidence`, and `applicable_context` fields.
  - [ ] **▸** Add `extract_knowledge(patterns) -> Vec<ExtractedKnowledge>` function with simple rule generation.
  - [ ] **▸** Add `ExtractedKnowledge` store/DB table stub.
- [ ] **T2-80** Add skill-improvement outputs.
  - [ ] **▸** Define `SkillImprovement` struct with `skill_id`, `metric`, `old_value`, `new_value`, and `delta` fields.
  - [ ] **▸** Add `compute_improvement(old_experience, new_experience) -> SkillImprovement` function.
- [ ] **T2-81** Add confidence-update handling for learned items.
  - [ ] **▸** Add `update_confidence(item_id, new_confidence)` function in `src/learning/mod.rs`.
  - [ ] **▸** Add `confidence_history: Vec<(timestamp, value)>` field to learned item struct.
- [ ] **T2-82** Add confidence decay handling for stale or weak learning signals.
  - [ ] **▸** Add `decay_confidence(item_id, hours_since_update, decay_rate)` function with exponential decay formula.
  - [ ] **▸** Add `get_stale_items(min_confidence, max_age_hours)` query function.
- [ ] **T2-83** Add generalization rules over memorization.
  - [ ] **▸** Define `GeneralizationRule` struct with `specific_pattern`, `general_pattern`, `confidence`, and `supporting_experiences` fields.
  - [ ] **▸** Add `detect_generalizations(patterns, min_support) -> Vec<GeneralizationRule>` function.
  - [ ] **▸** Add `apply_generalization(rule, context) -> bool` function for runtime matching.

## 6. Planning Engine
Use the data contracts to make planning more structured.

- [ ] **T2-84** Add explicit goal-creation fields and validation rules.
- [ ] **T2-85** Add richer `decompose_goal` action-verb handling.
- [ ] **T2-86** Add better step generation for `decompose_goal`.
- [ ] **T2-87** Add dependency-aware task graphs.
  - [ ] **▸** Add `dependencies: Vec<StepId>` field to `PlanStep` in `src/planner/mod.rs`.
  - [ ] **▸** Add `validate_no_cycles(steps) -> bool` function using DFS cycle detection.
  - [ ] **▸** Add `topological_sort(steps) -> Vec<StepId>` function for execution ordering.
  - [ ] **▸** Add `get_ready_steps(steps) -> Vec<StepId>` that returns steps with no unmet dependencies.
- [ ] **T2-88** Add planning-strategy selection.
  - [ ] **▸** Define `PlanningStrategy` enum with `Sequential`, `Parallel`, and `Greedy` variants in `src/planner/mod.rs`.
  - [ ] **▸** Add `select_strategy(goal) -> PlanningStrategy` function with simple heuristic.
- [ ] **T2-89** Add candidate-plan generation.
- [ ] **T2-90** Add candidate-plan evaluation.
- [ ] **T2-91** Add workflow generation from plans.
- [ ] **T2-92** Add dynamic replanning triggers.
- [ ] **T2-93** Add plan scoring.
- [ ] **T2-94** Migrate `Plan` to the data-contract type.

## 7. Execution and Tooling surfaces
Make execution and tool use explicit, authorized, and observable.

- [ ] **T2-95** Add execution-step fields for actions.
- [ ] **T2-96** Add execution-step fields for external interactions.
- [ ] **T2-97** Add execution-step fields for result handling.
- [ ] **T2-98** Add result-normalization rules.
- [ ] **T2-99** Add execution error-recovery paths.
  - [ ] **▸** Define `RecoveryStrategy` enum with `Retry`, `Fallback`, and `Abort` variants in `src/execution/mod.rs`.
  - [ ] **▸** Add `Retry` strategy implementation with configurable max_retries and backoff_ms in `src/execution/retry.rs`.
  - [ ] **▸** Add `Fallback` strategy implementation that falls back to alternative tool in `src/execution/fallback.rs`.
  - [ ] **▸** Add `execute_with_recovery(step, strategy) -> Result` orchestrator function.
  - [ ] **▸** Add `log_recovery_event(step_id, strategy, outcome)` function for observability.
- [ ] **T2-100** Add tool-registration contracts distinct from skills.
- [ ] **T2-101** Add tool permissions.
- [ ] **T2-102** Add tool authorization checks.
- [ ] **T2-103** Add tool execution isolation rules.
- [ ] **T2-104** Add external capability integration rules.

## 8. Model integration, agent communication, and coordination
Finish the cross-cutting v0.0.2 concepts that keep systems replaceable and coordinated.

- [ ] **T2-105** Add local-model integration rules under one abstraction.
- [ ] **T2-106** Add cloud-model integration rules under one abstraction.
- [ ] **T2-107** Add model-routing rules based on capability instead of provider name.
- [ ] **T2-108** Add context-handling rules for inference.
- [ ] **T2-109** Add inference-management rules for scheduling.
- [ ] **T2-110** Add inference-management rules for validation.
- [ ] **T2-111** Add inference-management rules for model selection.
- [ ] **T2-112** Add agent-communication boundaries for MCP concepts.
- [ ] **T2-113** Add agent-communication boundaries for ACP concepts.
- [ ] **T2-114** Add internal communication rules for subsystem-to-subsystem events.
- [ ] **T2-115** Add cognitive coordination rules for subsystem orchestration.
- [ ] **T2-116** Add cognitive coordination rules for decision routing.
- [ ] **T2-117** Add event communication rules that avoid exposing private implementation details.

## 9. Skills, workflows, world model, and personality
Finish the remaining v0.0.2 consumer systems last.

- [ ] **T2-118** Add skill permissions in `src/skills/registry/`.
- [ ] **T2-119** Add skill performance tracking in `src/skills/registry/`.
- [ ] **T2-120** Add skill fallback behavior.
- [ ] **T2-121** Add async, parallel, and retry behavior for skills.
- [ ] **T2-122** Add workflow-level learning in `src/workflows/engine/`.
- [ ] **T2-123** Add workflow confidence in `src/workflows/engine/`.
- [ ] **T2-124** Add workflow ranking.
- [ ] **T2-125** Add world-model entities aligned with the knowledge graph.
- [ ] **T2-126** Add world-model relationships aligned with confidence-bearing edges.
- [ ] **T2-127** Add personality traits and emotional-weight handling.
- [ ] **T2-128** Add personality presets, adaptation, decision-making, and communication rules.
- [ ] **T2-129** Add the v0.0.2 confidence coverage for knowledge, skills, relationships, workflows, and conclusions.

## Completion target
End state: finished v0.0.2. Gate stays green throughout.
