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

### T2-08 — Context-handling rules for inference (Chapter 14.4)
- **Files:** `src/models/mod.rs`, `src/lib.rs`
- **Change:** Created `src/models/mod.rs` with `ChatMessage`, `InferenceContext`, and `truncate_context`. Added `pub mod models;` to `src/lib.rs`.
- **Verification:** `cargo build --release` passes with 0 errors; no new warnings from `models` module.

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
