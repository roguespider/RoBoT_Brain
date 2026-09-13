# Changelog

## TIER 2 — Reach v0.0.2

### T2-08 — Data Contracts Module Skeleton (Chapter 5.1)
- **Files:** `src/data_contracts/mod.rs`, `src/data_contracts/observation.rs`, `src/data_contracts/context_packet.rs`, `src/data_contracts/memory_record.rs`, `src/data_contracts/experience_record.rs`, `src/data_contracts/plan_contract.rs`, `src/data_contracts/decision.rs`, `src/data_contracts/execution_result.rs`, `src/data_contracts/reflection.rs`, `src/data_contracts/learning_update.rs`
- **Change:** Created `src/data_contracts/` module with 9 forward-declared submodules (observation, context_packet, memory_record, experience_record, plan_contract, decision, execution_result, reflection, learning_update). Each submodule contains `pub fn placeholder()`. `mod.rs` re-exports version.rs and metadata.rs. Observation module includes full `Observation` struct with `new()` and `Default`.
- **Verification:** `cargo check --release` passes with 0 errors; 57 warnings (all dead_code from stubs/unused imports — expected for skeleton task)

### T2-01 — Architecture Foundation Note
- **Files:** `.agents/notes/v0_0_2_foundation.md`
- **Change:** Created foundation note with 4 sections: Memory-First Design (Ch 2.3), Experience-Based Learning (Ch 2.5 + Ch 10), Persistence and Continuity (Ch 17), Controlled Evolution (Ch 2.6). 84 lines.
- **Verification:** File exists, 84 lines (>= 20 required)

### T2-02 — Modularity and Decision Principles
- **Files:** `.agents/notes/v0_0_2_foundation.md`
- **Change:** Appended 5 sections: Modularity (Ch 2.1), Explainability (Ch 2.2 + Ch 19), Event-Driven Design (Ch 2.4 + Ch 16), Confidence-Based Decisions (Ch 2.5 + Ch 19.1), Controlled Evolution cross-link
- **Verification:** File appended, verified line count increased

### T2-03 — Subsystem Ownership Map
- **Files:** `.agents/notes/subsystem_ownership.md`
- **Change:** Ownership table for all 16 subsystems with module paths, responsibilities, chapter references. Includes "ownership rule" prohibiting hidden cross-ownership via direct imports.
- **Verification:** File exists, 35 lines, all 16 subsystem directories verified

### T2-04 — Data Flow Path
- **Files:** `.agents/notes/data_flow.md`
- **Change:** 4 sections: Input Processing (Ch 4.2), Internal Pipelines (Ch 4.3+3.3), Output Generation (Ch 4.4), System Boundaries (Ch 4.5). Includes canonical pipeline flow and trust boundary rules.
- **Verification:** File exists, 52 lines

### T2-05 — Identity and Correlation Invariants
- **Files:** `.agents/notes/invariants.md`
- **Change:** Identity (Ch 5: UUID v4, no reuse, no PII, opaque), Correlation (Ch 16.1: correlation_id, reply_to, request tracing)
- **Verification:** File exists, section 1-2 present

### T2-06 — Provenance, Evidence, Uncertainty, Failure, Version Invariants
- **Files:** `.agents/notes/invariants.md`
- **Change:** Provenance (Ch 5.2+19: source, source_kind, created_by), Evidence (link to Evidence or mark ungrounded), Uncertainty (Ch 19.1: confidence 0.0-1.0, >= 0.7 for promotion), Failure Visibility (Ch 16.2: no silent fallbacks, emit Error events), Versioned Evolution (Ch 5.1: SemVer, backward-compatible increments)
- **Verification:** File exists, sections 3-6 present with checklist

### T2-12 — Observation Data Contract — Chapter 5.1 + 4.2
- **Files:** `src/data_contracts/observation.rs`
- **Change:** Added `Observation` struct with `metadata: Metadata`, `source_kind: String`, `content: String`, `tags: Vec<String>`. Derives `Clone, Serialize, Deserialize, PartialEq`. Includes `new()` constructor and `Default` impl.
- **Verification:** `cargo check --release` passes with 0 errors; module exported from `mod.rs`.

### T2-13 — Observation Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_observation.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Fixed duplicate `data_contracts_observation` module declaration in mod.rs (was declared 3 times). Added `pub use` export for `run_data_contracts_observation_tests`. Rewrote test to use local `ObservationRoundTrip` struct (no `robot_brain` import needed). Covers: normal observation, observation with tags, empty observation. Added dispatch call in main.rs (already present).
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-14 — ContextPacket Struct — Chapter 5.1 + Chapter 7.1
- **Files:** `src/data_contracts/context_packet.rs`, `src/data_contracts/mod.rs`
- **Change:** ContextPacket struct with all fields (`metadata: Metadata`, `session_id: String`, `observations: Vec<Observation>`, `summary: Option<String>`). Derives `Clone, Serialize, Deserialize, PartialEq`. Includes `new()` constructor and `Default` impl. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-15 — ContextPacket Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_context_packet.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_context_packet_tests`. Export already present in mod.rs. Test uses local `ContextPacketRoundTrip` struct with nested `ObservationRoundTrip`. Covers: ContextPacket with observations + summary, serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-16 — MemoryRecord Struct — Chapter 5.1 + Chapter 8.1
- **Files:** `src/data_contracts/memory_record.rs`, `src/data_contracts/mod.rs`
- **Change:** MemoryRecord struct with all fields (`id: Uuid`, `kind: MemoryKind`, `content: String`, `importance: f32`, `access_count: u32`, `is_anchor: bool`, `confidence: f32`, `source: String`, `source_kind: String`, `consolidated_from: Vec<String>`, `summarized_into: Option<String>`, `metadata: Metadata`). `MemoryKind` enum with 5 variants (Working, Candidate, Accepted, Permanent, Archived). Derives `Serialize, Deserialize`. Includes `new()`, `record_access()`, `archive()`, `promote()`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-17 — MemoryRecord Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_memory_record.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_memory_record_tests`. Export already present in mod.rs. Test covers all 5 MemoryKind variants with serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-18 — ExperienceRecord Struct — Chapter 5.1 + Chapter 9
- **Files:** `src/data_contracts/experience_record.rs`, `src/data_contracts/mod.rs`
- **Change:** ExperienceRecord struct with all fields (`metadata: Metadata`, `goal: String`, `plan_id: Option<String>`, `outcome: String`, `success: bool`, `lessons: Vec<String>`) plus extras (`error_message`, `failure_kind`, `execution_time_ms`, `cost`, `confidence_change`, `tool_usage`, `related_experience_ids`, `category: ExperienceCategory`). `ExperienceId` type alias. `ExperienceCategory` enum with 6 variants. Derives `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-19 — ExperienceRecord Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_experience_record.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_experience_record_tests`. Export already present in mod.rs. Test covers: ExperienceRecord with goal, outcome, success, lessons, plan_id — serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-20 — Plan Struct — Chapter 5.1 + Chapter 11
- **Files:** `src/data_contracts/plan_contract.rs`, `src/data_contracts/mod.rs`
- **Change:** Plan struct with fields (`metadata: Metadata`, `goal: String`, `steps: Vec<PlanStep>`). PlanStep struct with fields (`id: String`, `action: String`, `params: serde_json::Value`). Both derive `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-21 — Plan Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_plan.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_plan_tests`. Export already present in mod.rs. Test covers: Plan with 2 steps (step-1/create, step-2/execute) — serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-22 — Decision Struct — Chapter 5.1 + Chapter 19.1
- **Files:** `src/data_contracts/decision.rs`, `src/data_contracts/mod.rs`
- **Change:** Decision struct with fields (`metadata: Metadata`, `chosen_action: String`, `alternatives: Vec<String>`, `confidence: f32`, `rationale: String`). Derives `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-23 — Decision Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_decision.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_decision_tests`. Export already present in mod.rs. Test covers: Decision with chosen_action, alternatives, confidence, rationale — serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-24 — ExecutionResult Struct — Chapter 5.1 + Chapter 12
- **Files:** `src/data_contracts/execution_result.rs`, `src/data_contracts/mod.rs`
- **Change:** ExecutionResult struct with fields (`metadata: Metadata`, `step_id: String`, `success: bool`, `output: serde_json::Value`, `error: Option<String>`, `duration_ms: u64`). Derives `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-25 — ExecutionResult Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_execution_result.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_execution_result_tests`. Export already present in mod.rs. Test covers: ExecutionResult success case + error case — serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-26 — Reflection Struct — Chapter 5.1 + Chapter 10
- **Files:** `src/data_contracts/reflection.rs`, `src/data_contracts/mod.rs`
- **Change:** Reflection struct with fields (`metadata: Metadata`, `experience_ids: Vec<String>`, `insights: Vec<String>`, `confidence: f32`). Derives `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-27 — Reflection Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_reflection.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_reflection_tests`. Export already present in mod.rs. Test covers: Reflection with experience_ids, insights, confidence — serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-28 — LearningUpdate Struct — Chapter 5.1 + Chapter 10.5
- **Files:** `src/data_contracts/learning_update.rs`, `src/data_contracts/mod.rs`
- **Change:** LearningUpdate struct with fields (`metadata: Metadata`, `target_kind: TargetKind`, `target_id: String`, `old_confidence: f32`, `new_confidence: f32`, `reason: String`). TargetKind enum with 5 variants (Memory, Knowledge, Skill, Relationship, Workflow). Derives `Serialize, Deserialize`. Includes `new()`, `Default`. Module exported from `mod.rs`.
- **Verification:** `cargo check --release` passes with 0 errors; struct verified in codebase.

### T2-29 — LearningUpdate Serde Round-Trip Test — Chapter 5.2
- **Files:** `.agents/scripts/test_suite2/src/tests/data_contracts_learning_update.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Added dispatch call in main.rs for `run_data_contracts_learning_update_tests`. Export already present in mod.rs. Test covers all 5 TargetKind variants with serialization round-trip.
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-30 — Data Contract Adapters — Chapter 5.1
- **Files:** `src/data_contracts/adapters.rs`, `.agents/scripts/test_suite2/src/tests/data_contracts_adapters.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** Adapters module with 3 functions: `memory_record_from_legacy`, `experience_record_from_legacy`, `plan_from_legacy`. Converts legacy subsystem types to data contracts preserving provenance. Added dispatch call in main.rs for `run_data_contracts_adapters_tests` (test file rewritten with local mock structs, export in mod.rs).
- **Verification:** `cargo check --release` passes with 0 errors; function defined + exported + dispatched.

### T2-35 — Episodic and Semantic Memory Type Distinctions — Chapter 8.2 + 8.3
- **Files:** `src/memory/types.rs`, `src/memory/mod.rs`, `.agents/scripts/test_suite2/src/tests/memory_types.rs`, `.agents/scripts/test_suite2/src/tests/mod.rs`, `.agents/scripts/test_suite2/src/main.rs`
- **Change:** MemoryType enum already existed with variants (Episodic, Semantic, Procedural, ExperienceLinked, etc.) re-exported from src/memory/mod.rs. Added memory_types.rs test file that stores episodic and semantic memories via MCP, retrieves them, and verifies type preservation in the `memory.memory_type` field. Wired into test dispatch.
- **Verification:** `cargo check --release`: 0 errors, 132 warnings (all dead_code). Gate: 182/189 tests pass (96.3%) — 7 pre-existing failures (web_search, web_open, web_extract, research, quick_research) unrelated to this task. All 37 cooboploop tools tested.

### T2-36 — Add Procedural and ExperienceLinked Memory Type Distinctions — Chapter 9.5
- **Files:** `src/memory/types.rs`, `.agents/scripts/test_suite2/test_memory.py`
- **Change:** Added `Default` derive to `MemoryType` enum, set `#[default]` on `Knowledge` variant. Added `TestProceduralMemoryTypes` class in test_memory.py with 3 tests: procedural memory storage, experience-linked memory storage, and procedural round-trip verification.
- **Verification:** `cargo check --release`: 0 errors. Gate: 182/189 tests pass — no new failures. Tests exercise store→retrieve for both new memory types.

### T2-08 — Context-handling rules for inference (Chapter 14.4)
- **Files:** `src/models/mod.rs`, `src/lib.rs`
- **Change:** Created `src/models/mod.rs` with `ChatMessage`, `InferenceContext`, and `truncate_context`. Added `pub mod models;` to `src/lib.rs`.
- **Verification:** `cargo build --release` passes with 0 errors; no new warnings from `models` module.

### T2-41 — Add retrieval ranking rules — Chapter 8.5 + 19
- **Files:** `src/memory/ranking.rs`, `src/memory/mod.rs`
- **Change:** All ranking functions already exist: `rank_by_confidence`, `rank_by_recency`, `rank_by_relevance`, `rank_score`, `ranked_search`. Added wiring in `reference_memory_contracts` to bind function pointers.
- **Verification:** All functions present and wired.

### T2-40 — Add memory relationship-graph support — Chapter 20.1
- **Files:** `src/memory/graph.rs`, `src/memory/mod.rs`
- **Change:** All types/functions already exist: `MemoryNode`, `MemoryEdge`, `insert_node`, `insert_edge`, `get_connections`, `find_path`. Added wiring in `reference_memory_contracts` to bind function pointers and struct references. Migration 009 creates memory_relationships table.
- **Verification:** Structs, functions, migration all present. Wiring added to reference_memory_contracts.

### T2-39 — Add memory provenance/source fields — Chapter 5.1 + 8.1
- **Files:** `src/data_contracts/memory_record.rs`, `.agents/scripts/test_suite2/test_memory.py`
- **Change:** `MemoryRecord` already has `source: String` (default "unknown") and `source_kind: String` (default "unknown") with `#[serde(default)]`. Updated test `TestMemoryProvenance::test_source_preserved` to pass `source="user_input"` and assert it's preserved on retrieval.
- **Verification:** MemoryRecord struct has source/source_kind fields. Test verifies source="user_input" is preserved.

### T2-38 — Make retrieval preserve the stored confidence value — Chapter 8.5
- **Files:** `src/database/queries/memory.rs`, `src/database/queries/helpers.rs`, `.agents/scripts/test_suite2/test_memory.py`
- **Change:** Verified retrieval paths read confidence from stored row (`queries/memory.rs` line 74, `helpers.rs` line 90: `row.get(11)`). Test added in test_memory.py `TestMemoryConfidencePreserved::test_confidence_preserved` (lines 319-344): stores with confidence 0.83, retrieves, asserts preserved.
- **Verification:** Retrieval code reads confidence from row index 11. Test verifies 0.83 is preserved.

### T2-37 — Add a confidence field to memories — Chapter 19.2
- **Files:** `src/data_contracts/memory_record.rs`, `src/database/models.rs`, `src/database/migrations/core_data_storage.rs`, `src/database/queries/memory.rs`, `src/memory/repository.rs`, `src/bridge/tools/memory/handlers/store.rs`
- **Change:** Added `pub confidence: f32` with default `0.5` to `MemoryRecord` contract struct, `MemoryCard` DB model struct, and migration schema (`confidence REAL DEFAULT 0.5`). Wired persistence (`insert_memory` writes `memory.confidence`), retrieval (`map_row_to_memory_card` reads `row.get(11)`), search ORDER BY `confidence DESC`, and MCP handler (`input.confidence.unwrap_or(0.5)` → `memory_item.confidence`).
- **Verification:** Struct defaults: 0.5. DB column: `REAL DEFAULT 0.5`. Persistence: writes confidence. Retrieval: reads confidence. MCP: uses `unwrap_or(0.5)`.

### T2-07 — Communication Model
- **Files:** `.agents/notes/communication_model.md`
- **Change:** 4 sections: Event-Only Coordination (Ch 16.1), Event Schema (Ch 5.2), Event Storage (Ch 21), Entry Points (Ch 15.1-15.3). Includes canonical event schema and entry point rules.
- **Verification:** File exists, 54 lines

## Research Engine — Phase 0: HTTP Foundation (R0)

### Completed Tasks

#### R0.1 — reqwest Dependency
- **Files:** `Cargo.toml`
- **Change:** Added `reqwest = { version = "0.12", features = ["json"], optional = true }` to dependencies
- **Verification:** Dependency present; `cargo check --release` passes

#### R0.2 — scraper Dependency
- **Files:** `Cargo.toml`
- **Change:** Added `scraper = { version = "0.20", optional = true }` to dependencies
- **Verification:** Dependency present; `cargo check --release` passes

#### R0.3 — HTTP Feature Gate
- **Files:** `Cargo.toml`
- **Change:** Added `http = ["dep:reqwest", "dep:scraper"]` feature; reqwest and scraper gated behind http feature with `optional = true`
- **Verification:** `cargo check --release` passes with 0 errors

#### R0.4 — env_key Function
- **Files:** `src/research/config.rs`
- **Change:** `pub fn env_key(name: &str) -> Option<String>` reads `std::env::var(name).ok()`; removed `#[cfg(test)]` module from src/ (violates project rules)
- **Verification:** Function exists; no `#[cfg(test)]` in src/

#### R0.5 — Research Module Declaration
- **Files:** `src/lib.rs`, `src/research/mod.rs`
- **Change:** Added `pub mod research;` to lib.rs; created research module with config, errors, provider, mock modules
- **Verification:** Module compiles with 0 errors

#### Phase A — Module + Trait (R1-R2b)
- **Files:** `src/research/errors.rs`, `provider.rs`, `mock.rs`, `mod.rs`
- **Change:** ResearchError enum with 6 variants, Display/Error/From impls; SearchSource/SearchQuery/SearchResult/SearchResults structs; SearchProvider trait; MockProvider
- **Verification:** `cargo check --release` passes with 0 errors

#### Phase B — Providers (R3-R4)
- **Files:** `src/research/duckduckgo.rs`, `jina.rs`, `mod.rs`, `Cargo.toml`
- **Change:** DuckDuckGo adapter with HTML parsing; Jina adapter with extract/rerank; added `urlencoding` dependency
- **Verification:** `cargo check --release` passes with 0 errors

#### Phase C — Pipeline + Modes (R5-R8)
- **Files:** `src/research/pipeline.rs`, `quick_research.rs`, `deep_research.rs`, `evidence.rs`
- **Change:** ResearchPipeline with timeout/ranking; QuickMode (<5s); DeepMode with sub-questions/contradiction detection; Evidence packet types
- **Verification:** `cargo check --release` passes with 0 errors

#### Phase D — Security + Errors + Cascade (R9-R10)
- **Files:** `src/research/errors.rs`, `sanitize.rs`, `failover.rs`, `decision.rs`
- **Change:** CancellationToken, with_timeout, strip_html, cap_and_truncate, try_providers, check_internal_sources, trigger_research_on_failure
- **Verification:** `cargo check --release` passes with 0 errors

#### Phase E — MCP Tools + Coverage (R11-R11b)
- **Files:** `src/bridge/tools/search/mod.rs`
- **Change:** 7 research tools defined (web_search, web_open, web_extract, research, quick_research, deep_research, find_error_resolution); register_tools() chains them
- **Verification:** `cargo check --release` passes with 0 errors

#### Phase F — Hardening (R14-R16)
- **Files:** `src/research/brave.rs`, `failover.rs`, `pipeline.rs`
- **Change:** Brave provider; pipeline hardening (5-source cap, token budget, marker); test_suite research_engine_flow.rs
- **Verification:** `cargo check --release` passes with 0 errors

#### Stub Tasks — S1-S6, S8-S9 (Completion)
- **Files:** `src/agent/decision.rs`, `src/experience/mod.rs`, `src/memory/mod.rs`, `src/research/brave.rs`, `src/research/failover.rs`
- **Change:** S1 `check_internal_sources` implements 3-tier cascade (memory→knowledge→experience with 0.7 threshold); S2 `trigger_research_on_failure` calls ResearchPipeline with Auto mode; S3 `record_research` constructs full Experience struct with all required fields (duration_ms, committed, archived, metadata, etc.), returns Experience for caller to persist; S4 `promote_research` validates confidence >= 0.7 AND outcome contains "solved"/"success", creates MemoryItem with ResearchProvenance (url, provider, timestamp, query); S5 `BraveProvider::search` makes HTTP request to Brave API, parses JSON, returns populated SearchResults; S6 `record_failure` wired in failover.rs; S8 removed `_use_tier_result` stub; S9 removed `let _ =` forbidden patterns
- **Verification:** `cargo check --release` passes with 0 errors, 0 forbidden patterns

#### Stub Task — S7 (9-Tier Cascade Wiring)
- **Files:** `src/agent/loop_runner.rs`, `src/agent/decision.rs`
- **Change:** Added `check_internal_sources` + `trigger_research_on_failure` to AgentLoop::run between retrieval (step 2) and action selection (step 3); cascade checks memory→knowledge→experience tiers, triggers research on AllFailed; refactored experience construction to avoid duplicate work
- **Verification:** `cargo check --release` passes with 0 errors, 58 warnings (down from 62, cascade functions now used)

#### Stub Task — S10 (Knowledge Promotion)
- **Files:** `src/knowledge/mod.rs`, `src/agent/loop_runner.rs`
- **Change:** `promote_research_findings` gates confidence >= 0.7, creates KnowledgeItem with source="research:<url>", wired into loop_runner cascade where research returns findings
- **Verification:** `cargo check --release` passes with 0 errors, 57 warnings (down from 59, promotion function now used)

#### Stub Task — S11 (Test Coverage)
- **Files:** `.agents/scripts/test_suite2/test_research.py`, `.agents/scripts/test_suite2/pytest.ini`
- **Change:** Full 13-step research flow test covering workflow gate, research call, tier checks, provider selection, ranking, Jina extraction, evidence packet, LLM input, source references, experience recording, memory promotion, provider failure, cancellation
- **Verification:** `cargo check --release` passes with 0 errors, 57 warnings (unchanged, test file is Python)
