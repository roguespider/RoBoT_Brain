# TIER 3 PLAN — Reach v0.0.2.1

## Purpose
Build the missing subsystems from the v0.0.2.1 architecture in a dependency-first order.

## Execution rule for small tasks
Treat each bullet as a single 10-15 minute increment: one tiny code change, one verification run, then stop.

## 0. Architecture-wide contract
Set the invariants that every v0.0.2.1 chapter, appendix, and runtime surface must obey.

- [ ] **T3-01** Write the ownership boundaries for interaction, control plane, cognition, state, action, and platform.
- [ ] **T3-02** Write the lifecycle boundaries for ephemeral, session, working, persistent operational, persistent knowledge, and archived state.
- [ ] **T3-03** Write the identity and correlation rules for installations and sessions.
- [ ] **T3-04** Write the identity and correlation rules for events, plans, actions, executions, tools, and learning changes.
- [ ] **T3-05** Write the provenance and evidence rules for durable information.
- [ ] **T3-06** Write the confidence rules separating confidence from source quality, recency, contradiction, uncertainty, and applicability.
- [ ] **T3-07** Write the model-independence rule so no subsystem depends on a fixed model or provider.
- [ ] **T3-08** Write the controlled-effects rule so execution and tools remain authorized and traceable.
- [ ] **T3-09** Write the observability, failure visibility, versioned evolution, human control, and compatibility rules.

## 1. Foundation chapters 01-05
Build the base architecture and shared contracts first.

- [ ] **T3-10** Summarize the vision and philosophy chapter as a persistent cognitive architecture with continuity and long-term improvement.
- [ ] **T3-11** Summarize the core design principles chapter as modularity, explainability, memory-first behavior, event-driven design, confidence, and controlled evolution.
- [ ] **T3-12** Summarize the high-level system overview chapter as subsystem relationships and a cognitive pipeline.
- [ ] **T3-13** Summarize the data-flow chapter as input processing, internal data pipelines, output generation, and system boundaries.
- [ ] **T3-14** Summarize the data-contracts chapter as shared structures, event contracts, API boundaries, serialization formats, and interoperability.
- [ ] **T3-15** Add the shared contract metadata fields for version and timestamp.
- [ ] **T3-16** Add the shared contract metadata fields for source, correlation, provenance, and confidence.

## 2. Conversation Engine (Chapter 06)
Make the user-facing interaction layer explicit and testable.

- [ ] **TC-01** `ConversationSession` struct -- fields: `session_id`,
      `turn_number`, `user_messages[]`, `agent_responses[]`, `learnings[]`.
      In-memory session tracker.
- [ ] **TC-02** `ConversationEngine` -- in-memory struct with methods:
      `start_session()`, `add_turn(session_id, user_msg, agent_resp)`,
      `get_session(session_id)`.
- [ ] **TC-03** `converse` MCP tool -- takes `message` string, finds or creates
      session, calls `run_agent_goal(message)`, stores turn, returns response.
      (This is the entry point users actually use.)
- [ ] **TC-04** Learning extraction from conversation -- simple function that
      takes (user_message, agent_response) and returns a list of extracted
      facts/learnings. Starts with keyword-based pattern matching,
      upgrades to LLM extraction later.
- [ ] **TC-05** Wire `extract_learnings` into `converse` -- each extracted
      learning is stored via existing `store_memory` (memory_type =
      `preference` or `fact`).
- [ ] **TC-06** Conversation persistence -- store conversation turns to
      SQLite (`conversation_turns` table). Simple: `session_id, turn_number,
      role (user/agent), content, timestamp`.

**Done when:** `converse` tool works end-to-end (user message -> agent loop
-> response -> conversation stored). Gate green.

## 3. Context Engine (Chapter 07)
Build the context subsystem that serves the conversation engine.

- [ ] **T3-27** Add the active session-state model.
- [ ] **T3-28** Add the working-memory model.
- [ ] **T3-29** Add the current retrieval-set model.
- [ ] **T3-30** Add context-compression logic for long or repetitive context.
- [ ] **T3-31** Add topic-tracking logic.
- [ ] **T3-32** Add relevant-information selection logic.
- [ ] **T3-33** Add token-budget enforcement.
- [ ] **T3-34** Add policy-budget enforcement.
- [ ] **T3-35** Add the context-construction step.
- [ ] **T3-36** Add the context-assembly step for prompts.

## 4. Memory Engine (Chapter 08)
Implement the durable memory system as a first-class cognitive subsystem.

- [ ] **T3-37** Add the short-term memory responsibility boundary.
- [ ] **T3-38** Add the long-term memory responsibility boundary.
- [ ] **T3-39** Add explicit memory-create operations.
- [ ] **T3-40** Add explicit memory-promote operations.
- [ ] **T3-41** Add explicit memory-demote operations.
- [ ] **T3-42** Add explicit memory-archive operations.
- [ ] **T3-43** Add explicit memory-retrieve operations.
- [ ] **T3-44** Add working-memory records.
- [ ] **T3-45** Add episodic-memory records.
- [ ] **T3-46** Add semantic-memory records.
- [ ] **T3-47** Add procedural-memory records.
- [ ] **T3-48** Add experience-linked-memory records.
- [ ] **T3-49** Add promotion rules for working or episodic information becoming durable knowledge.

## 5. Experience Engine (Chapter 09)
Record outcomes, lessons, and failures separately from memory.

- [ ] **T3-50** Add experience storage for execution history.
- [ ] **T3-51** Add experience storage for outcomes.
- [ ] **T3-52** Add outcome tracking for success and failure.
- [ ] **T3-53** Add lesson capture for successes.
- [ ] **T3-54** Add lesson capture for failures.
- [ ] **T3-55** Add failure-analysis storage.
- [ ] **T3-56** Add experience-processing flow from actions to learning.
- [ ] **T3-57** Add related-experience links for pattern analysis.

## 6. Learning Engine (Chapter 10)
Turn repeated evidence into reusable capability.

- [ ] **T3-58** Add the reflection stage in the learning pipeline.
- [ ] **T3-59** Add the candidate stage in the learning pipeline.
- [ ] **T3-60** Add the evaluation stage in the learning pipeline.
- [ ] **T3-61** Add the promotion stage in the learning pipeline.
- [ ] **T3-62** Add the consolidation stage in the learning pipeline.
- [ ] **T3-63** Add pattern discovery for repeated successful structures.
- [ ] **T3-64** Add knowledge extraction into durable knowledge.
- [ ] **T3-65** Add skill-improvement outputs.
- [ ] **T3-66** Add confidence updates from evidence and repeated behavior.

## 7. Planning Engine (Chapter 11)
Make goal handling and plan quality explicit.

- [ ] **T3-67** Add goal-creation records.
- [ ] **T3-68** Add goal-validation rules.
- [ ] **T3-69** Add task decomposition into smaller work items.
- [ ] **T3-70** Add planning-strategy selection.
- [ ] **T3-71** Add workflow-generation support from plans.
- [ ] **T3-72** Add plan-evaluation scoring.
- [ ] **T3-73** Add feedback-driven replanning.

## 8. Execution Engine (Chapter 12)
Separate authorized action from planning.

- [ ] **T3-74** Add the controlled action-execution path.
- [ ] **T3-75** Add tool usage as a distinct execution concern.
- [ ] **T3-76** Add external interaction records.
- [ ] **T3-77** Add result handling and normalization.
- [ ] **T3-78** Add error-recovery behavior for execution failures.

## 9. Tool Engine (Chapter 13)
Make tool capability registration and permissions independent from skills.

- [ ] **T3-79** Add tool-capability records.
- [ ] **T3-80** Add tool-registration flow.
- [ ] **T3-81** Add tool-permission checks.
- [ ] **T3-82** Add tool-execution flow.
- [ ] **T3-83** Add external-capability integration rules.

## 10. AI Runtime and Model Integration (Chapter 14)
Build the runtime abstraction before provider-specific implementations.

- [ ] **T3-84** Add the local-model runtime abstraction.
- [ ] **T3-85** Add the cloud-model runtime abstraction.
- [ ] **T3-86** Add model-routing by capability.
- [ ] **T3-87** Add inference context handling.
- [ ] **T3-88** Add inference scheduling.
- [ ] **T3-89** Add inference validation.
- [ ] **T3-90** Add model selection management.

## 11. Agent Communication Architecture (Chapter 15)
Make agent and protocol boundaries explicit.

- [ ] **T3-91** Add the agent protocol model.
- [ ] **T3-92** Add MCP integration boundaries.
- [ ] **T3-93** Add ACP integration boundaries.
- [ ] **T3-94** Add internal communication rules for subsystem messages.

## 12. Cognitive Coordination Layer (Chapter 16)
Orchestrate subsystems without collapsing their boundaries.

- [ ] **T3-95** Add subsystem-coordination rules.
- [ ] **T3-96** Add event communication rules.
- [ ] **T3-97** Add decision-routing rules.
- [ ] **T3-98** Add top-level orchestration rules for the cognitive pipeline.

## 13. Memory and Knowledge Systems (Chapters 17-20)
Build the durable state and knowledge layer in full.

- [ ] **T3-99** Add working-memory promotion rules.
- [ ] **T3-100** Add permanent-memory retention rules.
- [ ] **T3-101** Add archived-memory retention rules.
- [ ] **T3-102** Add experience-to-outcome links.
- [ ] **T3-103** Add experience-to-learning-signal links.
- [ ] **T3-104** Add confidence scoring for knowledge.
- [ ] **T3-105** Add confidence scoring for skills.
- [ ] **T3-106** Add confidence scoring for relationships.
- [ ] **T3-107** Add confidence scoring for workflows.
- [ ] **T3-108** Add knowledge-graph concept relationships.
- [ ] **T3-109** Add knowledge-graph storage.
- [ ] **T3-110** Add knowledge-graph confidence-bearing edges.
- [ ] **T3-111** Add knowledge discovery queries.
- [ ] **T3-112** Add knowledge-promotion rules with evidence and provenance preservation.

## 14. Storage, Database, and Workers (Chapters 21-23)
Harden persistence and execution infrastructure before adding more surface area.

- [ ] **T3-113** Add durable-persistence architecture.
- [ ] **T3-114** Add data-organization rules.
- [ ] **T3-115** Add backup strategy rules.
- [ ] **T3-116** Add SQLite architecture rules.
- [ ] **T3-117** Add schema design rules.
- [ ] **T3-118** Add indexing rules.
- [ ] **T3-119** Add migration strategy rules.
- [ ] **T3-120** Add data-integrity rules.
- [ ] **T3-121** Add worker architecture.
- [ ] **T3-122** Add task queue handling.
- [ ] **T3-123** Add worker supervision.
- [ ] **T3-124** Add memory-worker behavior.
- [ ] **T3-125** Add learning-worker behavior.
- [ ] **T3-126** Add maintenance-worker behavior.

## 15. Governance, safety, and evolution (Chapters 24-27)
Lock down the control and trust layers before exposing broader operator surfaces.

- [ ] **T3-127** Add AI contributor roles and rules.
- [ ] **T3-128** Add contribution standards and human-approval boundaries.
- [ ] **T3-129** Add identity and permission rules.
- [ ] **T3-130** Add capability-security rules.
- [ ] **T3-131** Add memory-protection rules.
- [ ] **T3-132** Add audit rules.
- [ ] **T3-133** Add trust-evaluation rules.
- [ ] **T3-134** Add the learning-versus-evolution distinction.
- [ ] **T3-135** Add the hypothesis system.
- [ ] **T3-136** Add experimentation and controlled-change rules.
- [ ] **T3-137** Add tracing and telemetry rules.
- [ ] **T3-138** Add explanation and event-monitoring rules.
- [ ] **T3-139** Add debugger visibility for architectural evidence.

## 16. Interfaces, runtime management, testing, and deployment (Chapters 28-31)
Make the operational layer explicit and safe.

- [ ] **T3-140** Add Developer Interface and Control Plane inspection rules.
- [ ] **T3-141** Add Developer Interface command rules.
- [ ] **T3-142** Add safe-mutation and recovery rules.
- [ ] **T3-143** Add memory-management tools through the control plane.
- [ ] **T3-144** Add worker-control tools through the control plane.
- [ ] **T3-145** Add debugging interfaces for trace inspection and diagnostics.
- [ ] **T3-146** Add layered configuration precedence.
- [ ] **T3-147** Add secrets and profile management.
- [ ] **T3-148** Add runtime override handling.
- [ ] **T3-149** Add unit and contract testing layers.
- [ ] **T3-150** Add integration and persistence testing layers.
- [ ] **T3-151** Add recovery, event, security, and regression testing layers.
- [ ] **T3-152** Add installation and startup deployment rules.
- [ ] **T3-153** Add upgrade, rollback, and recovery deployment rules.
- [ ] **T3-154** Add versioned release validation.

## 17. Future expansion chapters 32-33
Leave the final architectural guardrails in place.

- [ ] **T3-155** Add the stable-contract admission rule for future capabilities.
- [ ] **T3-156** Add the architectural-gate review and roadmap process.

## 18. Appendices and quarantine material
Cover the supporting architecture material so the plan matches the full documentation set.

- [ ] **T3-157** Add Appendix A directory-ownership rules.
- [ ] **T3-158** Add Appendix A source-tree coverage for the major folders.
- [ ] **T3-159** Add Appendix B schema-domain coverage for system metadata, memory, knowledge, experience, learning, conversation, planning, execution, models, tools, tracing, diagnostics, configuration, and history.
- [ ] **T3-160** Add Appendix B schema-versioning and migration discipline.
- [ ] **T3-161** Add Appendix C event identity and versioning rules.
- [ ] **T3-162** Add Appendix C event payload, metadata, and confidence rules.
- [ ] **T3-163** Add Appendix C event lifecycle rules.
- [ ] **T3-164** Add Appendix D decision-record rules.
- [ ] **T3-165** Add Appendix D supersession and status tracking.
- [ ] **T3-166** Add Appendix E architecture-first development rules.
- [ ] **T3-167** Add Appendix E modularity and interface-before-implementation rules.
- [ ] **T3-168** Add Appendix E model-replaceability and AI runtime rules.
- [ ] **T3-169** Add Appendix E review-discipline rules.
- [ ] **T3-170** Keep odd-notes explicitly non-normative.

## Completion target
End state: finished v0.0.2.1. Gate stays green throughout.
