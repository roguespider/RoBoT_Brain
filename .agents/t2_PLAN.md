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



- [ ] **T2-04** — Write the canonical data-flow path for inputs, internal pipelines, and outputs — Chapter 4 (Data Flow Architecture).
  - **▸** Create `.agents/notes/data_flow.md`. Section 1: input processing (request → validation → intent parsing) citing Chapter 4.2.
  - **▸** Section 2: internal pipelines (Memory → Knowledge → Planning → Execution → Experience → Learning) citing Chapter 4.3 and the cognitive pipeline in Chapter 3.3.
  - **▸** Section 3: output generation (response assembly, side effects) citing Chapter 4.4.
  - **▸** Section 4: system boundaries (which modules are at the trust boundary) citing Chapter 4.5. Commit.
- [ ] **T2-05** — Write the shared invariants for identity and correlation — Chapter 5 (Data Contracts, "API boundaries") + Chapter 16 (Cognitive Coordination Layer).
  - **▸** Create `.agents/notes/invariants.md`. Section 1: identity (UUID v4 for entities, never reuse, no PII in IDs) citing Chapter 5.
  - **▸** Section 2: correlation (every event carries `correlation_id`, every response carries `reply_to`) citing Chapter 16.1.
  - **▸** Add a checklist at the bottom: "All public types MUST have an `id` and a `correlation_id` field". Commit.
- [ ] **T2-06** — Write the shared invariants for provenance, evidence, uncertainty, failure visibility, and versioned evolution — Chapter 5 (Data Contracts) + Chapter 19 (Confidence System).
  - **▸** Append to `.agents/notes/invariants.md`. Section 3: provenance (`source`, `source_kind`, `created_by`) on every record.
  - **▸** Section 4: evidence (every claim links to at least one `Evidence` record or marks itself "ungrounded").
  - **▸** Section 5: uncertainty (every numeric score has a `confidence` field 0.0-1.0) citing Chapter 19.1.
  - **▸** Section 6: failure visibility (no silent fallbacks; every error path emits an `Error` event) citing Chapter 16.2.
  - **▸** Section 7: versioned evolution (every contract has a `version: SemVer` field) citing Chapter 5.1. Commit.
- [ ] **T2-07** — Write the v0.0.2 communication model note: event-driven coordination instead of direct implementation coupling — Chapter 16 (Cognitive Coordination Layer) + Chapter 15.4 (Internal communication).
  - **▸** Create `.agents/notes/communication_model.md`. Section 1: "Subsystems MUST NOT call each other directly — they emit events and react to events" citing Chapter 16.1.
  - **▸** Section 2: event schema (kind, payload, source, correlation_id, timestamp) citing Chapter 5.2 "Event contracts".
  - **▸** Section 3: where events are stored (an append-only `events` log table) citing Chapter 21 "Storage Architecture".
  - **▸** Section 4: "MCP tools are the ONLY entry point for external callers; ACP is the ONLY entry point for other agents" citing Chapter 15.1-15.3. Commit.

---

## 1. Data Contracts first
These types become the shared shape for the rest of Tier 2. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 5 (Data Contracts).

- [ ] **T2-08** — Create `src/data_contracts/` module skeleton with `mod.rs` — Chapter 5.1 "Shared data structures".
  - **▸** Create `src/data_contracts/mod.rs` containing only `pub mod observation; pub mod context_packet; pub mod memory_record; pub mod experience_record; pub mod plan_contract; pub mod decision; pub mod execution_result; pub mod reflection; pub mod learning_update;` (forward decls, no logic).
  - **▸** Create empty `pub fn placeholder()` in each of the 9 submodules listed above (one per commit).
  - **▸** Verify: `cargo check --release` succeeds. Commit.
- [ ] **T2-09** — Add the shared contract version field and shared traits — Chapter 5.1 "Serialization formats".
  - **▸** Create `src/data_contracts/version.rs` with `pub const CONTRACT_VERSION: &str = "0.0.2";` and a `trait Versioned { fn version() -> &'static str { CONTRACT_VERSION } }`.
  - **▸** Re-export from `src/data_contracts/mod.rs`. Verify `cargo check --release`. Commit.
- [ ] **T2-10** — Add common metadata fields for version, source, and timestamp — Chapter 5.1.
  - **▸** Create `src/data_contracts/metadata.rs` with `pub struct Metadata { pub version: String, pub source: String, pub created_at: i64 }` deriving `Clone, Serialize, Deserialize`.
  - **▸** Add `impl Default for Metadata` (version=CONTRACT_VERSION, source="unknown", created_at=0). Re-export from `mod.rs`. Verify `cargo check --release`. Commit.
- [ ] **T2-11** — Add common metadata fields for correlation, confidence, and provenance — Chapter 5.1 + Chapter 19.1.
  - **▸** Add fields to `Metadata`: `pub correlation_id: String`, `pub confidence: f32` (default 0.5), `pub provenance: Vec<String>` (list of source IDs).
  - **▸** Update `Default` impl. Verify `cargo check --release`. Commit.
- [ ] **T2-12** — Add the `Observation` struct — Chapter 5.1 + Chapter 4.2 "Input processing".
  - **▸** In `src/data_contracts/observation.rs`, define `pub struct Observation { pub metadata: Metadata, pub source_kind: String, pub content: String, pub tags: Vec<String> }`.
  - **▸** Derive `Clone, Serialize, Deserialize`. Re-export from `mod.rs`. Verify `cargo check --release`. Commit.
- [ ] **T2-13** — Add a serde round-trip test for `Observation` — Chapter 5.1 "Serialization formats".
  - **▸** Add a `#[cfg(test)] mod tests` block in `observation.rs` with `test_observation_serde_roundtrip` that builds an `Observation`, calls `serde_json::to_string` then `serde_json::from_str`, asserts equality.
  - **▸** NOTE: tests in `src/` are forbidden by AGENTS.md. Move this test to `test_suite/src/tests/data_contracts_observation.rs`. Wire into `tests/mod.rs` and `main.rs`. Verify `cargo build --release` from `test_suite/`. Commit.
- [ ] **T2-14** — Add the `ContextPacket` struct — Chapter 5.1 + Chapter 7.1 "Session context".
  - **▸** In `src/data_contracts/context_packet.rs`, define `pub struct ContextPacket { pub metadata: Metadata, pub session_id: String, pub observations: Vec<Observation>, pub summary: Option<String> }`.
  - **▸** Derive `Clone, Serialize, Deserialize`. Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-15** — Add a serde round-trip test for `ContextPacket`.
  - **▸** In `test_suite/src/tests/data_contracts_context_packet.rs`, add `test_context_packet_serde_roundtrip` (build → to_json → from_json → assert equal). Wire into `tests/mod.rs` + `main.rs`. Verify build. Commit.
- [ ] **T2-16** — Add the `MemoryRecord` struct — Chapter 5.1 + Chapter 8 (Memory Engine) header.
  - **▸** In `src/data_contracts/memory_record.rs`, define `pub struct MemoryRecord { pub metadata: Metadata, pub kind: MemoryKind, pub content: String, pub importance: f32 }` where `MemoryKind` is an enum `Working | Candidate | Accepted | Permanent | Archived` deriving `Clone, Serialize, Deserialize`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-17** — Add a serde round-trip test for `MemoryRecord`.
  - **▸** In `test_suite/src/tests/data_contracts_memory_record.rs`, add `test_memory_record_serde_roundtrip` (one variant per `MemoryKind`). Wire + verify. Commit.
- [ ] **T2-18** — Add the `ExperienceRecord` alias or migration target — Chapter 5.1 + Chapter 9 (Experience Engine) header.
  - **▸** In `src/data_contracts/experience_record.rs`, define `pub struct ExperienceRecord { pub metadata: Metadata, pub goal: String, pub plan_id: Option<String>, pub outcome: String, pub success: bool, pub lessons: Vec<String> }`.
  - **▸** Add `pub type ExperienceId = String;`. Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-19** — Add a serde round-trip test for `ExperienceRecord`.
  - **▸** In `test_suite/src/tests/data_contracts_experience_record.rs`, add `test_experience_record_serde_roundtrip`. Wire + verify. Commit.
- [ ] **T2-20** — Add the `Plan` struct — Chapter 5.1 + Chapter 11 (Planning Engine) header.
  - **▸** In `src/data_contracts/plan_contract.rs`, define `pub struct Plan { pub metadata: Metadata, pub goal: String, pub steps: Vec<PlanStep> }` and `pub struct PlanStep { pub id: String, pub action: String, pub params: serde_json::Value }`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-21** — Add a serde round-trip test for `Plan`.
  - **▸** In `test_suite/src/tests/data_contracts_plan.rs`, add `test_plan_serde_roundtrip` with 2 steps. Wire + verify. Commit.
- [ ] **T2-22** — Add the `Decision` struct — Chapter 5.1 + Chapter 19.1 "Confidence scoring".
  - **▸** In `src/data_contracts/decision.rs`, define `pub struct Decision { pub metadata: Metadata, pub chosen_action: String, pub alternatives: Vec<String>, pub confidence: f32, pub rationale: String }`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-23** — Add a serde round-trip test for `Decision`.
  - **▸** In `test_suite/src/tests/data_contracts_decision.rs`, add `test_decision_serde_roundtrip`. Wire + verify. Commit.
- [ ] **T2-24** — Add the `ExecutionResult` struct — Chapter 5.1 + Chapter 12 (Execution Engine) header.
  - **▸** In `src/data_contracts/execution_result.rs`, define `pub struct ExecutionResult { pub metadata: Metadata, pub step_id: String, pub success: bool, pub output: serde_json::Value, pub error: Option<String>, pub duration_ms: u64 }`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-25** — Add a serde round-trip test for `ExecutionResult`.
  - **▸** In `test_suite/src/tests/data_contracts_execution_result.rs`, add `test_execution_result_serde_roundtrip` covering both success and error cases. Wire + verify. Commit.
- [ ] **T2-26** — Add the `Reflection` struct — Chapter 5.1 + Chapter 10 (Learning Engine) header.
  - **▸** In `src/data_contracts/reflection.rs`, define `pub struct Reflection { pub metadata: Metadata, pub experience_ids: Vec<String>, pub insights: Vec<String>, pub confidence: f32 }`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-27** — Add a serde round-trip test for `Reflection`.
  - **▸** In `test_suite/src/tests/data_contracts_reflection.rs`, add `test_reflection_serde_roundtrip`. Wire + verify. Commit.
- [ ] **T2-28** — Add the `LearningUpdate` struct — Chapter 5.1 + Chapter 10.5 "Confidence updates".
  - **▸** In `src/data_contracts/learning_update.rs`, define `pub struct LearningUpdate { pub metadata: Metadata, pub target_kind: TargetKind, pub target_id: String, pub old_confidence: f32, pub new_confidence: f32, pub reason: String }` where `TargetKind` is an enum `Memory | Knowledge | Skill | Relationship | Workflow`.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
- [ ] **T2-29** — Add a serde round-trip test for `LearningUpdate`.
  - **▸** In `test_suite/src/tests/data_contracts_learning_update.rs`, add `test_learning_update_serde_roundtrip` covering all 5 `TargetKind` variants. Wire + verify. Commit.
- [ ] **T2-30** — Add adapters that convert legacy subsystem types into shared contracts without losing provenance — Chapter 5.1 "API boundaries".
  - **▸** In `src/data_contracts/adapters.rs`, add `pub fn memory_record_from_legacy(m: &LegacyMemory) -> MemoryRecord` (read the legacy `src/memory/` types and copy fields 1:1, including source).
  - **▸** Add `pub fn experience_record_from_legacy(e: &LegacyExperience) -> ExperienceRecord` reading from `src/experience/`.
  - **▸** Add `pub fn plan_from_legacy(p: &LegacyPlan) -> Plan` reading from `src/planner/`.
  - **▸** Add a `test_suite/src/tests/data_contracts_adapters.rs` integration test: load a legacy record via the existing test helper, call the adapter, assert all fields + provenance match. Wire + verify with `make gate`. Commit.

---

## 2. Memory Engine
Bring memory up to contract shape before upgrading higher-level consumers. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 8 (Memory Engine) + Chapter 17 (Memory Architecture).

- [ ] **T2-31** — Add explicit memory lifecycle states in `src/memory/` — Chapter 8.4 "Memory lifecycle" + Chapter 17.5.
  - **▸** Create `src/memory/lifecycle.rs` with `pub enum MemoryLifecycle { Working, Candidate, Accepted, Permanent, Archived }` deriving `Clone, Copy, Serialize, Deserialize, PartialEq`.
  - **▸** Add `pub fn can_promote(from: MemoryLifecycle, to: MemoryLifecycle) -> bool` enforcing the legal transitions (Working→Candidate→Accepted→Permanent; any→Archived).
  - **▸** Add `pub fn is_terminal(s: MemoryLifecycle) -> bool` returning true only for `Archived`. Re-export from `src/memory/mod.rs`. Verify `cargo check --release`. Commit.
- [ ] **T2-32** — Add the promotion gate for Working, Candidate, Accepted, Permanent, and Archived states — Chapter 17.4 "Memory promotion".
  - **▸** In `src/memory/lifecycle.rs`, add `pub struct PromotionGate { pub min_age_secs: u64, pub min_confidence: f32, pub min_access_count: u32 }`.
  - **▸** Add `pub fn evaluate(memory: &MemoryRecord, gate: &PromotionGate) -> Option<MemoryLifecycle>` returning the next legal state when thresholds pass, else `None`.
  - **▸** Move legacy test to `test_suite/src/tests/memory_lifecycle_promotion.rs`: construct a `MemoryRecord`, vary age/confidence/accesses, assert correct promotion or denial. Wire + verify `make gate`. Commit.
- [ ] **T2-33** — Add working-memory and long-term-memory concepts — Chapter 17.2 "Working memory" + Chapter 17.3 "Permanent memory".
  - **▸** In `src/memory/mod.rs`, add `pub trait WorkingMemory { fn push(&mut self, rec: MemoryRecord); fn drain(&mut self) -> Vec<MemoryRecord>; fn len(&self) -> usize; }`.
  - **▸** Add `pub trait LongTermMemory { fn store(&self, rec: MemoryRecord) -> Result<(), MemoryError>; fn get(&self, id: &str) -> Option<MemoryRecord>; fn search(&self, q: &str) -> Vec<MemoryRecord>; }`.
  - **▸** Add `pub enum MemoryError { NotFound, StorageError(String) }` deriving thiserror-style. Verify `cargo check --release`. Commit.
- [ ] **T2-34** — Add promotion logic between working and long-term memory — Chapter 17.4 "Memory promotion".
  - **▸** In `src/memory/promotion.rs`, add `pub fn promote_to_long_term(working: &mut dyn WorkingMemory, lt: &dyn LongTermMemory, gate: &PromotionGate) -> usize` that drains working memory, evaluates each via the gate, stores accepted ones in long-term, returns count.
  - **▸** Re-export. Verify `cargo check --release`. Commit.
  - **▸** Move integration test to `test_suite/src/tests/memory_promotion.rs`: push 3 records with varying confidence, run promote, assert 2 stored in long-term. Wire + verify `make gate`. Commit.
- [ ] **T2-35** — Add episodic and semantic memory type distinctions — Chapter 8.2 "Short-term memory" + Chapter 8.3 "Long-term memory" (the type axis).
  - **▸** Add `pub enum MemoryType { Episodic, Semantic }` to `src/memory/mod.rs`.
  - **▸** Add a `memory_type: MemoryType` field to the legacy `MemoryRecord` struct (use `#[serde(default)]` so old records still load). Verify `cargo check --release`. Commit.
  - **▸** Add `test_suite/src/tests/memory_types.rs`: store one of each, retrieve, assert kind preserved. Wire + verify `make gate`. Commit.
- [ ] **T2-36** — Add procedural and experience-linked memory type distinctions — Chapter 9.5 "Experience processing" cross-link.
  - **▸** Extend `MemoryType` enum with `Procedural, ExperienceLinked` variants. Update the default. Verify `cargo check --release`. Commit.
  - **▸** Add `test_suite/src/tests/memory_types_procedural.rs`: round-trip a Procedural record. Wire + verify. Commit.
- [ ] **T2-37** — Add a confidence field to memories — Chapter 19.2 "Knowledge confidence" (memory inherits).
  - **▸** Ensure every `MemoryRecord` (both legacy and contract) carries a `confidence: f32` with default 0.5. If the legacy struct lacks it, add it with `#[serde(default)]`. Verify `cargo check --release`. Commit.
- [ ] **T2-38** — Make retrieval preserve the stored confidence value — Chapter 8.5 "Memory retrieval".
  - **▸** Audit the `search` and `get` paths in `src/memory/store.rs` (or equivalent). Confirm the returned `MemoryRecord.confidence` is read from the stored row, not overwritten by a default. Patch any code that drops the field.
  - **▸** Add `test_suite/src/tests/memory_confidence_preserved.rs`: store a record with confidence 0.83, retrieve, assert == 0.83. Wire + verify `make gate`. Commit.
- [ ] **T2-39** — Add memory provenance/source fields — Chapter 5.1 "Shared data structures" + Chapter 8.1.
  - **▸** Confirm `MemoryRecord` (legacy + contract) carries `source: String` and `source_kind: String`. If missing, add with `#[serde(default)]`. Verify `cargo check --release`. Commit.
  - **▸** Add `test_suite/src/tests/memory_provenance.rs`: store with `source="user_input"`, retrieve, assert preserved. Wire + verify. Commit.
- [ ] **T2-40** — Add memory relationship-graph support — Chapter 20.1 "Concept relationships" (memory graph mirrors).
  - **▸** Define `pub struct MemoryNode { pub id: String, pub content: String, pub node_type: String, pub confidence: f32 }` in `src/memory/graph.rs`. Derive `Clone, Serialize, Deserialize`.
  - **▸** Define `pub struct MemoryEdge { pub source_id: String, pub target_id: String, pub relationship_type: String, pub confidence: f32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `memory_edges` table with migration: `CREATE TABLE memory_edges (id TEXT PRIMARY KEY, source_id TEXT NOT NULL, target_id TEXT NOT NULL, relationship_type TEXT NOT NULL, confidence REAL NOT NULL DEFAULT 0.5); CREATE INDEX idx_memory_edges_source ON memory_edges(source_id);`. Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn insert_node(conn: &Connection, n: &MemoryNode) -> Result<(), rusqlite::Error>` and `pub fn insert_edge(conn: &Connection, e: &MemoryEdge) -> Result<(), rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn get_connections(conn: &Connection, node_id: &str) -> Result<Vec<MemoryEdge>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn find_path(conn: &Connection, from: &str, to: &str, max_depth: usize) -> Result<Option<Vec<String>>, rusqlite::Error>` using BFS. Verify `cargo check --release`. Commit.
  - **▸** Move integration test to `test_suite/src/tests/memory_graph.rs`: insert 3 nodes + 2 edges, run `get_connections` and `find_path`. Wire + verify `make gate`. Commit.
- [ ] **T2-41** — Add retrieval ranking rules that prefer relevant, confident, and recent records — Chapter 8.5 "Memory retrieval" + Chapter 19 (Confidence System).
  - **▸** Add `pub fn rank_score(&self) -> f32` to the contract `MemoryRecord` in `src/data_contracts/memory_record.rs`, calling into a default helper.
  - **▸** In `src/memory/ranking.rs`, add `pub fn rank_by_confidence(rec: &MemoryRecord) -> f32 { rec.confidence * 0.4 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn rank_by_recency(rec: &MemoryRecord, now_ts: i64) -> f32` using log-scaled decay: `(1.0 / (1.0 + (now_ts - rec.metadata.created_at).abs() as f64).ln()) as f32 * 0.3`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn rank_by_relevance(_rec: &MemoryRecord) -> f32 { 0.3 }` (placeholder per the plan; real implementation will use vector similarity once T2-139 lands). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn ranked_search(lt: &dyn LongTermMemory, q: &str) -> Vec<(MemoryRecord, f32)>` combining the three with weight sum 1.0. Verify `cargo check --release`. Commit.
  - **▸** Move integration test to `test_suite/src/tests/memory_ranking.rs`: store 3 records with different confidences/ages, run `ranked_search`, assert descending score. Wire + verify `make gate`. Commit.
- [ ] **T2-42** — Add duplicate-merge consolidation — Chapter 17.4 "Memory promotion" (dedup before promotion).
  - **▸** In `src/memory/dedup.rs`, add `pub fn merge_duplicates(records: Vec<MemoryRecord>) -> Vec<MemoryRecord>` that groups by `content_hash` (use std hash of content) and keeps the highest-confidence record per group.
  - **▸** Add provenance merging: combine the kept record's `provenance` Vec with the duplicates'. Verify `cargo check --release`. Commit.
  - **▸** Add `consolidated_from: Vec<String>` field to contract `MemoryRecord` (with `#[serde(default)]` for back-compat). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_dedup.rs`: 4 records, 2 duplicate pairs, expect 2 results with merged provenance. Wire + verify `make gate`. Commit.
- [ ] **T2-43** — Add summarization for aging low-importance memories — Chapter 7.4 "Context compression" (memory summarization mirror).
  - **▸** Add `pub importance: f32` field to `MemoryRecord` (with `#[serde(default = "default_importance")]` returning 0.5). Verify `cargo check --release`. Commit.
  - **▸** In `src/memory/summarize.rs`, add `pub fn summarize(records: Vec<MemoryRecord>) -> MemoryRecord` that concatenates content with " | " and creates a new record with combined `consolidated_from` and importance = max.
  - **▸** Add `summarized_into: Option<String>` field to `MemoryRecord`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_summarize.rs`: summarize 3 records, assert one result with all sources in `consolidated_from`. Wire + verify. Commit.
- [ ] **T2-44** — Keep anchor memories standalone during consolidation — Chapter 17.3 "Permanent memory" (anchors are permanent by definition).
  - **▸** Add `pub is_anchor: bool` field to `MemoryRecord` (default false). Verify `cargo check --release`. Commit.
  - **▸** Update `merge_duplicates` in `src/memory/dedup.rs` to skip any record where `is_anchor` is true (drop it from the input group). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_anchor.rs`: 2 duplicates, one with is_anchor=true, assert the anchor is preserved unchanged and the other is dropped. Wire + verify `make gate`. Commit.
- [ ] **T2-45** — Add pruning policy for low-value or aged memories — Chapter 17.5 "Memory lifecycle" (archived = pruned from active set).
  - **▸** In `src/memory/store.rs`, add `pub fn prune_below_importance(conn: &Connection, threshold: f32) -> Result<usize, rusqlite::Error>` returning deleted count. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn prune_older_than(conn: &Connection, max_age_secs: u64, now_ts: i64) -> Result<usize, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn prune(conn: &Connection, importance_threshold: f32, max_age_secs: u64, now_ts: i64) -> Result<usize, rusqlite::Error>` that calls both and returns the total deleted. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/memory_prune.rs`: insert 5 records (varying importance/age), run prune, assert correct count. Wire + verify `make gate`. Commit.
- [ ] **T2-46** — Migrate `MemoryRecord` to the data-contract type — Chapter 5.1 "API boundaries" + Chapter 8.1.
  - **▸** Replace `use crate::memory::MemoryRecord` with `use crate::data_contracts::memory_record::MemoryRecord` across `src/` via `sed -i` (or manual edit_file). The contract type IS the canonical type from T2-16.
  - **▸** Run `cargo check --release`. Fix any field that doesn't exist on the contract type by adding it to the contract type (preferred) or by adapting the caller (only if semantically different).
  - **▸** Run `make gate`. Fix until green. Commit. (If the legacy `src/memory/mod.rs` still re-exports an alias for back-compat, document it in a comment but do NOT keep two definitions.)

---

## 3. Experience Engine
Upgrade the record shape before adding scoring and propagation. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 9 (Experience Engine) + Chapter 18 (Experience Architecture).

- [ ] **T2-47** — Add the base `ExperienceRecord` fields for `goal` and `plan_id` — Chapter 9.1 "Experience storage" + Chapter 18.1 "Experience records".
  - **▸** Confirm the contract `ExperienceRecord` (from T2-18) has `goal: String` and `plan_id: Option<String>`. If not, add them. Verify `cargo check --release`. Commit.
- [ ] **T2-48** — Add the base `ExperienceRecord` fields for `result` and `success` — Chapter 9.3 "Outcome tracking".
  - **▸** Confirm `outcome: String` (used as `result` per the plan) and `success: bool` exist. Add if missing. Verify `cargo check --release`. Commit.
- [ ] **T2-49** — Add the base `ExperienceRecord` fields for `execution_time` and `cost` — Chapter 9.3 + Chapter 18.2.
  - **▸** Add `pub execution_time_ms: u64` and `pub cost: f32` to the contract `ExperienceRecord` with `#[serde(default)]`. Verify `cargo check --release`. Commit.
- [ ] **T2-50** — Add the base `ExperienceRecord` fields for `confidence_change` and `tool_usage` — Chapter 9.5 "Experience processing" + Chapter 18.2.
  - **▸** Add `pub confidence_change: f32` and `pub tool_usage: Vec<String>` to `ExperienceRecord`. Verify `cargo check --release`. Commit.
- [ ] **T2-51** — Add the base `ExperienceRecord` fields for `lessons` and `related refs` — Chapter 9.4 "Lessons learned" + Chapter 18.4 "Experience relationships".
  - **▸** Confirm `lessons: Vec<String>` exists. Add `pub related_experience_ids: Vec<String>` (defaulted). Verify `cargo check --release`. Commit.
- [ ] **T2-52** — Add experience categories for conversation and planning — Chapter 9.2 (categories: at minimum Conversation, Planning).
  - **▸** Add `pub enum ExperienceCategory { Conversation, Planning, ToolExecution, Learning, Code, Other }` deriving `Clone, Copy, Serialize, Deserialize, PartialEq` in `src/data_contracts/experience_record.rs`.
  - **▸** Add `pub category: ExperienceCategory` to `ExperienceRecord`. Verify `cargo check --release`. Commit.
- [ ] **T2-53** — Add experience categories for tool, execution, learning, and code — Chapter 9.2.
  - **▸** Confirm the `Other` variant covers the long tail; if a finer split is needed (e.g. `Reasoning` vs `Code`), extend the enum. Verify `cargo check --release`. Commit.
- [ ] **T2-54** — Add outcome tracking fields so experience stores what happened — Chapter 9.3 "Outcome tracking".
  - **▸** Confirm `outcome: String` and `success: bool` capture both the textual outcome and the boolean. Add `pub error_message: Option<String>` with `#[serde(default)]`. Verify `cargo check --release`. Commit.
- [ ] **T2-55** — Add failure-analysis fields so experience stores why it happened — Chapter 9.4 "Failure analysis".
  - **▸** Add `pub failure_kind: Option<String>` (e.g. "timeout", "validation", "permission_denied") with `#[serde(default)]`. Verify `cargo check --release`. Commit.
- [ ] **T2-56** — Add lesson-extraction fields for reusable takeaways — Chapter 9.4 "Lessons learned".
  - **▸** Confirm `lessons: Vec<String>` exists and is populated when the experience represents a failure. Verify `cargo check --release`. Commit.
- [ ] **T2-57** — Add multi-factor success scoring — Chapter 9.3 "Outcome tracking" + Chapter 19 (Confidence System).
  - **▸** In `src/experience/mod.rs`, define `pub struct ExperienceScore { pub success_rate: f32, pub confidence_delta: f32, pub execution_efficiency: f32 }`. Derive `Clone, Serialize, Deserialize`.
  - **▸** Implement `pub fn calc_success_rate(rec: &ExperienceRecord) -> f32` returning `if rec.success { 1.0 } else { 0.0 }` (placeholder; for multi-task experiences, divide by sub-tasks). Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn calc_confidence_delta(rec: &ExperienceRecord) -> f32 { rec.confidence_change }`. Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn calc_efficiency(rec: &ExperienceRecord) -> f32` returning `1.0 / (1.0 + (rec.execution_time_ms as f32 / 1000.0))`. Verify `cargo check --release`. Commit.
  - **▸** Implement `pub fn compute_score(rec: &ExperienceRecord, w: &ScoreWeights) -> ExperienceScore` with default weights 0.4/0.3/0.3 and a `ScoreWeights` struct. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/experience_score.rs`: build a record with execution_time=2000, success=true, confidence_change=0.2; assert score components. Wire + verify `make gate`. Commit.
- [ ] **T2-58** — Add confidence propagation to memory — Chapter 18.2 "Learning signals" + Chapter 19.3.
  - **▸** In `src/experience/mod.rs`, add `pub fn propagate_confidence_to_memory(memory: &mut MemoryRecord, delta: f32) { memory.confidence = (memory.confidence + delta).clamp(0.0, 1.0); }`. Verify `cargo check --release`. Commit.
  - **▸** Add a hook call site: when an experience is recorded, look up the `plan_id`'s referenced memory IDs and call propagate for each. Wire into the experience-recording tool handler. Verify `make gate`. Commit.
- [ ] **T2-59** — Add confidence propagation to relationships and tools — Chapter 19.3 + Chapter 13.2 "Tool permissions".
  - **▸** Add `pub fn propagate_confidence_to_tool(tool_name: &str, delta: f32) -> Result<(), ToolError>` updating the tool reputation table. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn propagate_confidence_to_relationship(source: &str, target: &str, delta: f32) -> Result<(), MemoryError>` updating the edge confidence in the memory graph. Verify `cargo check --release`. Commit.
  - **▸** Wire both into the experience pipeline after `compute_score`. Verify `make gate`. Commit.
- [ ] **T2-60** — Add experience relationships between related events — Chapter 18.4 "Experience relationships".
  - **▸** Confirm `related_experience_ids: Vec<String>` exists on the contract (from T2-51). Verify `cargo check --release`. Commit.
  - **▸** In `src/experience/mod.rs`, add `pub fn link_experiences(conn: &Connection, id_a: &str, id_b: &str) -> Result<(), rusqlite::Error>` that appends to each record's `related_experience_ids` and writes back. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn get_related_experiences(conn: &Connection, id: &str) -> Result<Vec<ExperienceRecord>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/experience_link.rs`: link two records, retrieve, assert both see each other. Wire + verify `make gate`. Commit.
- [ ] **T2-61** — Migrate `ExperienceRecord` to the data-contract type — Chapter 5.1 "API boundaries".
  - **▸** Replace legacy `src/experience/`'s `ExperienceRecord` with `use crate::data_contracts::experience_record::ExperienceRecord` everywhere in `src/`. Run `cargo check --release`, fix breakage by adding fields to the contract type (preferred).
  - **▸** Run `make gate`. Fix until green. Commit.

---

## 4. Knowledge Graph
Build the storage layer before traversal and extraction. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 20 (Knowledge Graph).

- [ ] **T2-62** — Add the `knowledge_nodes` table and migration — Chapter 20.2 "Graph storage".
  - **▸** Create migration adding `knowledge_nodes (id TEXT PRIMARY KEY, label TEXT NOT NULL, kind TEXT NOT NULL, confidence REAL NOT NULL DEFAULT 0.5, created_at INTEGER NOT NULL)`. Wire into the migration runner. Verify `cargo check --release`. Commit.
- [ ] **T2-63** — Add the `knowledge_edges` table and migration — Chapter 20.2.
  - **▸** Add migration: `knowledge_edges (id TEXT PRIMARY KEY, source_id TEXT NOT NULL, target_id TEXT NOT NULL, relationship TEXT NOT NULL, confidence REAL NOT NULL DEFAULT 0.5)`. Add index on `source_id`. Verify `cargo check --release`. Commit.
- [ ] **T2-64** — Add relationship confidence on knowledge edges — Chapter 19.4 "Relationship confidence" + Chapter 20.3.
  - **▸** Confirm the `confidence` column exists on `knowledge_edges` (from T2-63). Add a `pub fn set_edge_confidence(conn: &Connection, id: &str, c: f32) -> Result<(), rusqlite::Error>` helper. Verify `cargo check --release`. Commit.
- [ ] **T2-65** — Add concept-relationship fields for structured understanding — Chapter 20.1 "Concept relationships".
  - **▸** In `src/knowledge/graph.rs`, define `pub struct KnowledgeNode { pub id: String, pub label: String, pub kind: String, pub confidence: f32 }` and `pub struct KnowledgeEdge { pub id: String, pub source_id: String, pub target_id: String, pub relationship: String, pub confidence: f32 }`. Verify `cargo check --release`. Commit.
- [ ] **T2-66** — Add entity resolution for aliases like "rustc" and "Rust Compiler" — Chapter 20.4 "Knowledge discovery".
  - **▸** Add `pub struct EntityResolution { pub canonical_id: String, pub aliases: Vec<String> }` to `src/knowledge/resolution.rs`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn resolve_entity(name: &str, table: &HashMap<String, String>) -> Option<String>` returning the canonical id for a given alias. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn register_alias(table: &mut HashMap<String, String>, canonical_id: &str, alias: &str)`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_entity_resolution.rs`: register "rustc" → "rust_compiler", resolve both, assert same canonical id. Wire + verify `make gate`. Commit.
- [ ] **T2-67** — Add graph traversal queries for relationship chains — Chapter 20.4.
  - **▸** In `src/knowledge/graph.rs`, add `pub fn traverse_from(conn: &Connection, start_id: &str, max_depth: usize) -> Result<Vec<KnowledgeEdge>, rusqlite::Error>` using BFS. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn find_all_paths(conn: &Connection, start: &str, end: &str, max_paths: usize) -> Result<Vec<Vec<String>>, rusqlite::Error>` (DFS with path cap). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn get_subgraph(conn: &Connection, node_id: &str, radius: usize) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>), rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_traversal.rs`: build a 4-node graph, assert BFS/DFS/subgraph correctness. Wire + verify `make gate`. Commit.
- [ ] **T2-68** — Add discovery queries for linked concepts and supporting evidence — Chapter 20.4.
  - **▸** Add `pub fn find_linked_concepts(conn: &Connection, node_id: &str, relationship: &str) -> Result<Vec<KnowledgeNode>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn find_supporting_evidence(conn: &Connection, node_id: &str) -> Result<Vec<KnowledgeEdge>, rusqlite::Error>` returning edges pointing TO the node. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_discovery.rs`. Wire + verify. Commit.
- [ ] **T2-69** — Add entity-detection logic for the graph-extraction pipeline — Chapter 20.4.
  - **▸** In `src/knowledge/extraction.rs`, define `pub struct ExtractionInput { pub text: String, pub source: String }` deriving Clone. Verify `cargo check --release`. Commit.
  - **▸** Define `pub struct DetectedEntity { pub text: String, pub entity_type: String, pub confidence: f32, pub position: usize }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn detect_entities(input: &ExtractionInput) -> Vec<DetectedEntity>` using simple capitalized-noun matching (regex `r"\b[A-Z][a-zA-Z]+\b"`). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_entity_detection.rs`: input "Rust was designed by Graydon", expect at least "Rust" and "Graydon". Wire + verify `make gate`. Commit.
- [ ] **T2-70** — Add relationship-extraction logic for the graph-extraction pipeline — Chapter 20.4.
  - **▸** Define `pub struct DetectedRelationship { pub source_id: String, pub target_id: String, pub type_: String, pub confidence: f32, pub trigger_text: String }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn extract_relationships(entities: &[DetectedEntity], text: &str) -> Vec<DetectedRelationship>` using a simple "X is Y" or "X uses Y" pattern. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_relationship_extraction.rs`: input "Rust uses Cargo", expect a relationship between "Rust" and "Cargo" with type "uses". Wire + verify `make gate`. Commit.
- [ ] **T2-71** — Add confidence-evaluation logic for the graph-extraction pipeline — Chapter 20.3 + Chapter 19.1.
  - **▸** Define `pub struct EvaluationCriteria { pub source_trustworthiness: f32, pub text_clarity: f32, pub entity_count: u32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn evaluate_confidence(entities: &[DetectedEntity], relationships: &[DetectedRelationship], criteria: &EvaluationCriteria) -> f32` with weighted scoring (0.4/0.3/0.3). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn adjust_confidence(entities: &mut Vec<DetectedEntity>, relationships: &mut Vec<DetectedRelationship>, threshold: f32)` filtering entries below the threshold. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/knowledge_confidence_eval.rs`. Wire + verify. Commit.
- [ ] **T2-72** — Add graph-update and integration logic for the graph-extraction pipeline — Chapter 20.4.
  - **▸** Add `pub fn apply_extractions(conn: &Connection, entities: &[DetectedEntity], relationships: &[DetectedRelationship]) -> Result<usize, rusqlite::Error>` that calls `insert_node` and `insert_edge`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn run_extraction(conn: &Connection, text: &str, source: &str) -> Result<(Vec<DetectedEntity>, Vec<DetectedRelationship>), KnowledgeError>` orchestrating detect → extract → evaluate → adjust. Verify `cargo check --release`. Commit.
  - **▸** Register the MCP tool `extract_knowledge` calling `run_extraction` and returning the (entities, relationships) JSON. Verify `make gate`. Commit.

---

## 5. Learning Engine
Make learning explicit after experience and knowledge are contract-shaped. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 10 (Learning Engine) + Chapter 18.2 (Learning signals).

- [ ] **T2-73** — Formalize the learning pipeline entry in `src/learning/` — Chapter 10.1 "Learning pipeline".
  - **▸** Create `src/learning/mod.rs` with `pub mod pipeline; pub mod patterns; pub mod extraction; pub mod improvement; pub mod confidence; pub mod generalization;`.
  - **▸** Create each submodule file with one `pub fn placeholder()` returning `Ok(())`. Verify `cargo check --release`. Commit.
- [ ] **T2-74** — Add reflection-to-candidate promotion logic — Chapter 10.5 "Confidence updates" (promotion gate analogy).
  - **▸** In `src/learning/pipeline.rs`, add `pub fn reflection_to_candidate(r: &Reflection) -> Option<LearningUpdate>` returning a `LearningUpdate` with target_kind=Knowledge if `r.confidence >= 0.6` and `r.insights` is non-empty. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_reflection_promote.rs`. Wire + verify. Commit.
- [ ] **T2-75** — Add candidate-to-evaluation logic — Chapter 10.1.
  - **▸** Add `pub fn candidate_to_evaluation(c: &LearningUpdate) -> EvaluationCriteria` synthesizing criteria from the source. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_candidate_eval.rs`. Wire + verify. Commit.
- [ ] **T2-76** — Add evaluation-to-promotion logic — Chapter 10.5.
  - **▸** Add `pub fn evaluation_to_promotion(c: &LearningUpdate, score: f32) -> Option<LearningUpdate>` returning a new update with `new_confidence = old_confidence + score * 0.1` if score >= 0.7, else None. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_eval_promote.rs`. Wire + verify. Commit.
- [ ] **T2-77** — Add promotion-to-consolidation logic — Chapter 10.1 (consolidation writes back to memory).
  - **▸** Add `pub fn promotion_to_consolidation(update: &LearningUpdate) -> Result<(), LearningError>` that applies the update to the memory/knowledge store. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_consolidation.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-78** — Add pattern discovery from repeated successful experiences — Chapter 10.2 "Pattern discovery".
  - **▸** In `src/learning/patterns.rs`, define `pub struct Pattern { pub id: String, pub frequency: u32, pub success_rate: f32, pub context_signature: String, pub actions: Vec<String> }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn group_by_context_signature(experiences: &[ExperienceRecord]) -> HashMap<String, Vec<String>>`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn detect_patterns(experiences: &[ExperienceRecord], min_frequency: u32) -> Vec<Pattern>` iterating the groups. Verify `cargo check --release`. Commit.
  - **▸** Add migration: `learning_patterns (id TEXT PRIMARY KEY, signature TEXT NOT NULL, frequency INTEGER NOT NULL, success_rate REAL NOT NULL)`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn insert_pattern(conn: &Connection, p: &Pattern) -> Result<(), rusqlite::Error>` and `pub fn get_patterns(conn: &Connection, min_success_rate: f32) -> Result<Vec<Pattern>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_patterns.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-79** — Add knowledge extraction from observed patterns — Chapter 10.3 "Knowledge extraction".
  - **▸** In `src/learning/extraction.rs`, define `pub struct ExtractedKnowledge { pub pattern_id: String, pub rule: String, pub confidence: f32, pub applicable_context: String }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn extract_knowledge(patterns: &[Pattern]) -> Vec<ExtractedKnowledge>` generating a rule string `"when context_signature=X with frequency>=N, expect success_rate>=Y"`. Verify `cargo check --release`. Commit.
  - **▸** Add migration: `extracted_knowledge (id TEXT PRIMARY KEY, pattern_id TEXT NOT NULL, rule TEXT NOT NULL, confidence REAL NOT NULL, applicable_context TEXT NOT NULL)`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_extraction.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-80** — Add skill-improvement outputs — Chapter 10.4 "Skill improvement".
  - **▸** In `src/learning/improvement.rs`, define `pub struct SkillImprovement { pub skill_id: String, pub metric: String, pub old_value: f32, pub new_value: f32, pub delta: f32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn compute_improvement(skill_id: &str, metric: &str, old: f32, new: f32) -> SkillImprovement`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_skill_improvement.rs`. Wire + verify. Commit.
- [ ] **T2-81** — Add confidence-update handling for learned items — Chapter 10.5 "Confidence updates" + Chapter 19.5.
  - **▸** In `src/learning/confidence.rs`, add `pub fn update_confidence(item_id: &str, new_confidence: f32) -> Result<(), LearningError>` clamping to [0,1] and recording in history. Verify `cargo check --release`. Commit.
  - **▸** Add `pub confidence_history: Vec<(i64, f32)>` field to `ExtractedKnowledge`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_confidence_update.rs`. Wire + verify. Commit.
- [ ] **T2-82** — Add confidence decay handling for stale or weak learning signals — Chapter 10.5.
  - **▸** Add `pub fn decay_confidence(item_id: &str, hours_since_update: f64, decay_rate: f32) -> f32` returning `current * (0.5_f32).powf((hours_since_update as f32) * decay_rate)`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn get_stale_items(conn: &Connection, min_confidence: f32, max_age_hours: u64) -> Result<Vec<String>, rusqlite::Error>`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_decay.rs`. Wire + verify. Commit.
- [ ] **T2-83** — Add generalization rules over memorization — Chapter 10.2 "Pattern discovery" (generalization).
  - **▸** In `src/learning/generalization.rs`, define `pub struct GeneralizationRule { pub specific_pattern: String, pub general_pattern: String, pub confidence: f32, pub supporting_experiences: Vec<String> }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn detect_generalizations(patterns: &[Pattern], min_support: u32) -> Vec<GeneralizationRule>` (placeholder: cluster by first token of context_signature, emit a rule if cluster size >= min_support). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn apply_generalization(rule: &GeneralizationRule, context: &str) -> bool` returning true if `context` matches `general_pattern` (substring for now). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/learning_generalization.rs`. Wire + verify `make gate`. Commit.

---

## 6. Planning Engine
Use the data contracts to make planning more structured. Source: `robot_architecture/RoBoT Architecture v0.0.2.md` Chapter 11 (Planning Engine).

- [ ] **T2-84** — Add explicit goal-creation fields and validation rules — Chapter 11.1 "Goal creation".
  - **▸** In `src/planner/mod.rs`, define `pub struct Goal { pub id: String, pub description: String, pub priority: u8, pub deadline: Option<i64> }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn validate_goal(g: &Goal) -> Result<(), PlanError>` enforcing: description non-empty, priority in 0..=10, deadline (if Some) in the future. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_goal_validation.rs`. Wire + verify. Commit.
- [ ] **T2-85** — Add richer `decompose_goal` action-verb handling — Chapter 11.2 "Task decomposition".
  - **▸** Add `pub enum ActionVerb { Create, Read, Update, Delete, Execute, Analyze, Communicate, Wait, Other(String) }` to `src/planner/mod.rs`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn parse_action_verb(s: &str) -> ActionVerb` mapping known verbs to variants and unknown strings to `Other(s.to_string())`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_action_verb.rs`. Wire + verify. Commit.
- [ ] **T2-86** — Add better step generation for `decompose_goal` — Chapter 11.2.
  - **▸** Add `pub fn generate_steps(goal: &Goal) -> Vec<PlanStep>` producing a skeleton (id, action from parsed verb, params={}). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_step_generation.rs`. Wire + verify. Commit.
- [ ] **T2-87** — Add dependency-aware task graphs — Chapter 11.2 + Chapter 11.4.
  - **▸** Add `pub dependencies: Vec<String>` field to `PlanStep` (with `#[serde(default)]`). Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn validate_no_cycles(steps: &[PlanStep]) -> bool` using DFS with a recursion stack. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn topological_sort(steps: &[PlanStep]) -> Option<Vec<String>>` returning None on cycles. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn get_ready_steps(steps: &[PlanStep], completed: &[String]) -> Vec<PlanStep>` returning steps whose dependencies are all in `completed`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_dag.rs`: 3 steps A→B→C, plus a cycle test, plus a ready-steps test. Wire + verify `make gate`. Commit.
- [ ] **T2-88** — Add planning-strategy selection — Chapter 11.3 "Planning strategies".
  - **▸** Define `pub enum PlanningStrategy { Sequential, Parallel, Greedy }` in `src/planner/mod.rs`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn select_strategy(goal: &Goal) -> PlanningStrategy` with heuristic: if priority >= 8 → Greedy; if `goal.description.contains("in parallel")` → Parallel; else Sequential. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_strategy.rs`. Wire + verify. Commit.
- [ ] **T2-89** — Add candidate-plan generation — Chapter 11.3.
  - **▸** Add `pub fn generate_candidates(goal: &Goal, n: usize) -> Vec<Plan>` calling `generate_steps` with `n` different strategy choices. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_candidates.rs`. Wire + verify. Commit.
- [ ] **T2-90** — Add candidate-plan evaluation — Chapter 11.5 "Plan evaluation".
  - **▸** Add `pub fn evaluate_candidate(plan: &Plan) -> f32` returning `1.0 / (1.0 + plan.steps.len() as f32)` (shorter plans score higher, placeholder). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_evaluation.rs`. Wire + verify. Commit.
- [ ] **T2-91** — Add workflow generation from plans — Chapter 11.4 "Workflow generation".
  - **▸** Add `pub fn plan_to_workflow(plan: &Plan) -> Workflow` mapping each step to a workflow step in order. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_to_workflow.rs`. Wire + verify `make gate`. Commit.
- [ ] **T2-92** — Add dynamic replanning triggers — Chapter 11.5.
  - **▸** Add `pub enum ReplanTrigger { StepFailed(String), ConfidenceBelow(f32), ExternalChange(String) }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn should_replan(trigger: &ReplanTrigger, ctx: &PlanContext) -> bool` returning true in all cases (placeholder). Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_replan.rs`. Wire + verify. Commit.
- [ ] **T2-93** — Add plan scoring — Chapter 11.5 + Chapter 19 (Confidence).
  - **▸** Add `pub struct PlanScore { pub feasibility: f32, pub confidence: f32, pub cost_estimate: f32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn score_plan(plan: &Plan, eval: f32) -> PlanScore`. Verify `cargo check --release`. Commit.
  - **▸** Move test to `test_suite/src/tests/planner_score.rs`. Wire + verify. Commit.
- [ ] **T2-94** — Migrate `Plan` to the data-contract type — Chapter 5.1 "API boundaries".
  - **▸** Replace legacy `Plan` references in `src/` with `use crate::data_contracts::plan_contract::Plan`. Run `cargo check --release`. Add missing fields to the contract type. Verify `make gate`. Commit.

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
- [ ] **T2-108** — Add context-handling rules for inference — Chapter 14.4 "Context handling".
  - **▸** Add `pub struct InferenceContext { pub system: String, pub messages: Vec<ChatMessage>, pub max_tokens: u32 }`. Verify `cargo check --release`. Commit.
  - **▸** Add `pub fn truncate_context(ctx: &InferenceContext, budget: u32) -> InferenceContext`. Verify `cargo check --release`. Commit.
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
