# TIER 2 PLAN — Reach v0.0.2

## Purpose
Upgrade the existing subsystems to the v0.0.2 architecture in a dependency-first order.

## Execution rule for small tasks
Treat each bullet as a single 10-15 minute increment: one tiny code change, one verification run, then stop.

## 0. Architecture foundations and invariants
Set the rules that every v0.0.2 subsystem must preserve.

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
- [ ] **T2-41** Add retrieval ranking rules that prefer relevant, confident, and recent records.
- [ ] **T2-42** Add duplicate-merge consolidation.
- [ ] **T2-43** Add summarization for aging low-importance memories.
- [ ] **T2-44** Keep anchor memories standalone during consolidation.
- [ ] **T2-45** Add pruning policy for low-value or aged memories.
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
- [ ] **T2-58** Add confidence propagation to memory.
- [ ] **T2-59** Add confidence propagation to relationships and tools.
- [ ] **T2-60** Add experience relationships between related events.
- [ ] **T2-61** Migrate `ExperienceRecord` to the data-contract type.

## 4. Knowledge Graph
Build the storage layer before traversal and extraction.

- [ ] **T2-62** Add the `knowledge_nodes` table and migration.
- [ ] **T2-63** Add the `knowledge_edges` table and migration.
- [ ] **T2-64** Add relationship confidence on knowledge edges.
- [ ] **T2-65** Add concept-relationship fields for structured understanding.
- [ ] **T2-66** Add entity resolution for aliases like "rustc" and "Rust Compiler".
- [ ] **T2-67** Add graph traversal queries for relationship chains.
- [ ] **T2-68** Add discovery queries for linked concepts and supporting evidence.
- [ ] **T2-69** Add entity-detection logic for the graph-extraction pipeline.
- [ ] **T2-70** Add relationship-extraction logic for the graph-extraction pipeline.
- [ ] **T2-71** Add confidence-evaluation logic for the graph-extraction pipeline.
- [ ] **T2-72** Add graph-update and integration logic for the graph-extraction pipeline.

## 5. Learning Engine
Make learning explicit after experience and knowledge are contract-shaped.

- [ ] **T2-73** Formalize the learning pipeline entry in `src/learning/`.
- [ ] **T2-74** Add reflection-to-candidate promotion logic.
- [ ] **T2-75** Add candidate-to-evaluation logic.
- [ ] **T2-76** Add evaluation-to-promotion logic.
- [ ] **T2-77** Add promotion-to-consolidation logic.
- [ ] **T2-78** Add pattern discovery from repeated successful experiences.
- [ ] **T2-79** Add knowledge extraction from observed patterns.
- [ ] **T2-80** Add skill-improvement outputs.
- [ ] **T2-81** Add confidence-update handling for learned items.
- [ ] **T2-82** Add confidence decay handling for stale or weak learning signals.
- [ ] **T2-83** Add generalization rules over memorization.

## 6. Planning Engine
Use the data contracts to make planning more structured.

- [ ] **T2-84** Add explicit goal-creation fields and validation rules.
- [ ] **T2-85** Add richer `decompose_goal` action-verb handling.
- [ ] **T2-86** Add better step generation for `decompose_goal`.
- [ ] **T2-87** Add dependency-aware task graphs.
- [ ] **T2-88** Add planning-strategy selection.
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
