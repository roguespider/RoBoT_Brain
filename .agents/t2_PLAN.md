# TIER 2 PLAN — Reach v0.0.2

## Purpose
Upgrade the existing subsystems to the v0.0.2 architecture in a dependency-first order.

## Execution rule for small tasks
Treat each bullet as a single 10-15 minute increment: one tiny code change, one verification run, then stop.
Sub-bullets (▸) are children of the parent task above them — complete them in order.

**pre T2-01 uses 5-10 minute increments.** Each `▸` sub-bullet below is ONE tiny, buildable pass for an AI agent:
a small function/struct, a single wiring call, or one test — followed by `cargo check --release` (or the gate for wired
tools). Do NOT batch two sub-bullets. If a later task needs the same breakup, apply the same pattern.

## Convention
- Each `▸` is independently committable: code change -> build/check -> commit -> push -> next.
- A struct + its fields = one pass; its serde round-trip test = a separate pass.
- Tool registration is one pass PER TOOL (not per file) so each lands separately.

## 0. Architecture foundations and invariants
Set the rules that every v0.0.2 subsystem must preserve.

# pre T2-01 — Research Engine (External Knowledge Acquisition) — FIRST TASK (moved from PLAN.md)

> Source: `.agents/research_engine.md` — the live 21-section spec. Its R1-R16 implementation order, evidence-packet
> contracts, and Definition of Done are canonical for this task. Core principle: the LLM decides it needs
> information; the Research Engine determines how to obtain it.
>
> Dependencies: Phase 0 (R0) adds the HTTP client (`reqwest` + an HTML parser) to `Cargo.toml` — currently missing,
> and no provider works without it. Verify-Task note: `src/data_contracts/` currently exists but is EMPTY, and there is
> NO existing Jina code in `src/` (the "existing Jina integration pattern" claim in research_engine.md is stale).

## Phase 0: Foundation and dependencies
- [ ] **R0** Add the HTTP stack to `Cargo.toml` (one dep per pass, each followed by `cargo check --release`).
      - [ ] **▸** Add `reqwest` to `[dependencies]`; verify it compiles.
      - [ ] **▸** Add an HTML-parsing crate (e.g. `scraper`); verify it compiles.
      - [ ] **▸** Gate the network deps behind an `http` feature flag (so no-HTTP builds stay optional); verify.
      - [ ] **▸** Add a `JINA_API_KEY` / `BRAVE_API_KEY` config reader (env var helper returning `Option<String>`); verify.
      - [ ] **▸** Add a TODO-free config note to `README.md` (doc) — no code.
- [ ] **R1** Create the `src/research/` module skeleton (compile-clean, no logic).
      - [ ] **▸** Create `src/research/provider.rs` declaring `SearchSource` enum (Web, News, Reddit, HN, Wikipedia); verify.
      - [ ] **▸** Define `SearchQuery` struct { query, source, max_results, language, region }; verify.
      - [ ] **▸** Define `SearchResult` struct { title, url, snippet, relevance, source }; verify.
      - [ ] **▸** Define `SearchResults` struct { results, provider, query, retrieved_at } + serde derives; verify.
      - [ ] **▸** Define `trait SearchProvider: Send + Sync` { search, name, supports }; verify.
      - [ ] **▸** Create `src/research/errors.rs` with a minimal `ResearchError` enum (all 6 variants, no logic yet); verify.
      - [ ] **▸** Create `src/research/mod.rs` that `pub mod`s the submodules; add `pub mod research` to `src/lib.rs`; verify.
- [ ] **R2** Wire the provider abstraction + a build-check for the trait.
      - [ ] **▸** Implement `Display` / `std::error::Error` for `ResearchError`; verify.
      - [ ] **▸** Add a `From<anyhow::Error>` / `From<reqwest::Error>` conversion so `?` works; verify.
- [ ] **R2b** Add a deterministic mock provider (no network in CI / gate).
      - [ ] **▸** Define `MockProvider` (fixed 2-3 canned `SearchResult`s) in `src/research/mock.rs`; impl the trait; verify.
      - [ ] **▸** Add a unit-style `search` smoke test that calls `MockProvider::search()`; verify.

## Phase B: Core pipeline
- [ ] **R3** Implement the DuckDuckGo adapter (primary, no API key).
      - [ ] **▸** `duckduckgo.rs`: `reqwest::get` the search URL and capture the raw HTML body; verify compile.
      - [ ] **▸** Parse the HTML result block (title + URL) into `SearchResult`; verify.
      - [ ] **▸** Extract the snippet text from each result; verify.
      - [ ] **▸** Implement `SearchProvider` for `DuckDuckGo` (name="duckduckgo", supports Web, error on timeout); verify.
      - [ ] **▸** Add a live-off test using a captured HTML fixture (no network); verify.
- [ ] **R4** Implement the Jina processing layer (API-key gated; net-new).
      - [ ] **▸** `jina.rs`: add an authenticated request helper (Bearer `JINA_API_KEY`); verify compile.
      - [ ] **▸** Add the `extract(url)` -> clean-text function; verify.
      - [ ] **▸** Add a `rerank(results)` relevance sort helper; verify.
      - [ ] **▸** Expose via `SearchProvider` where applicable and gate on key presence; verify.
- [ ] **R5** Build pipeline orchestration (search -> rank -> extract -> compare -> evidence).
      - [ ] **▸** `pipeline.rs`: add `run_pipeline(query, mode) -> ResearchResult` signature + wire the research refs; verify.
      - [ ] **▸** Bound raw results: keep max 10 from `search()`; verify.
      - [ ] **▸** Rank + retain top 5 (relevance desc); verify.
      - [ ] **▸** Wrap the per-search call in `tokio::time::timeout`; verify.
      - [ ] **▸** Extract up to top-5 sources to clean text via the extraction layer; verify.
      - [ ] **▸** Assemble a `ResearchResult` evidence packet; verify.
- [ ] **R6** Implement quick mode.
      - [ ] **▸** `quick_research.rs`: `run_quick(query)` flow calling the pipeline with 1-3 searches; verify.
      - [ ] **▸** Enforce the <5s total timeout; verify.
      - [ ] **▸** Add a quick-mode evidence-packet smoke test; verify.
- [ ] **R7** Implement deep mode.
      - [ ] **▸** `deep_research.rs`: generate 1-3 sub-questions from a seed query (keyword split helper); verify.
      - [ ] **▸** Run the multi-iteration search loop (2-5 iterations, bounded); verify.
      - [ ] **▸** Compare overlapping sources for consistency; verify.
      - [ ] **▸** Detect contradictions (conflicting findings on the same claim) + optional resolution; verify.
      - [ ] **▸** Add a deep-mode basic smoke test; verify.
- [ ] **R8** Implement the evidence packet types + serde.
      - [ ] **▸** `evidence.rs`: `Source` struct { title, url, provider, query_used, retrieved_at, relevance, content }; verify.
      - [ ] **▸** `Finding` struct { statement, source_url, confidence }; verify.
      - [ ] **▸** `Contradiction` struct { claim_a, claim_b, source_a_url, source_b_url, resolution }; verify.
      - [ ] **▸** `ResearchResult` struct { question, queries, sources, findings, contradictions, limitations, confidence, retrieved_at }; verify.
      - [ ] **▸** Add serde `Serialize`/`Deserialize` on all evidence types; verify.
      - [ ] **▸** Add a serde round-trip test for `ResearchResult`; verify.

## Phase C: Security, errors, and context protection
- [ ] **R9** Error handling (never panic).
      - [ ] **▸** `errors.rs`: fill the `ResearchError` variants + a `Cancelled` signal path; verify.
      - [ ] **▸** Add a `with_timeout(fut, dur)` helper wrapping `tokio::time::timeout`; verify.
      - [ ] **▸** Wire cancellation via a shared `CancellationToken`-style flag into long fetches; verify.
- [ ] **R9b** Web-content sanitization (prompt-injection defense, DoD #12) — one tiny fn per pass.
      - [ ] **▸** `sanitize.rs`: `strip_html` (remove tags) helper; verify.
      - [ ] **▸** `strip_control_chars` + escape embedded text; verify.
      - [ ] **▸** `cap_and_truncate` (max 5 sources, "and N more" marker); verify.
      - [ ] **▸** Apply sanitization inside the extraction path so no raw HTML reaches the evidence packet; verify.
      - [ ] **▸** Add unit tests covering injection-looking content (script tags, prompt text) staying inert; verify.
- [ ] **R9c** Provider failover (graceful degradation, design rule #6).
      - [ ] **▸** Add a `try_providers(vec![providers]) -> Result<SearchResults>` fallback-orchestration fn; verify.
      - [ ] **▸** DuckDuckGo -> (optional) Brave chain, collecting per-provider errors; verify.
      - [ ] **▸** Emit a single structured `AllProvidersFailed` surface (wrap into `ResearchError`); verify.

## Phase D: Cognitive integration — DEFERRED until Memory/Experience engines are contract-shaped
> Requires `src/data_contracts/` (T2-08..T2-30) AND the Memory Engine (T2-31..T2-46) AND the Experience Engine
> (T2-47..T2-61). Do NOT start this phase before those land — it calls into subsystems that are not built yet.
- [ ] **R10** Wire the 9-tier confidence cascade into `src/agent/decision.rs` (starts after the contract-shaped dependency phase).
      - [ ] **▸** Add `check_internal_sources(query) -> tier_result` that runs tiers 1-8 in order; verify.
      - [ ] **▸** Stop at the first tier returning confidence >= 0.7; verify.
      - [ ] **▸** When all 8 fail, return a `NeedResearch` signal; verify.
      - [ ] **▸** Trigger `research()` from the cascade; verify.
      - [ ] **▸** Plumb the 0.7 threshold through the cascade as a named constant; verify.
- [ ] **R12** Memory promotion (`src/memory/`) — after R10 rebuilds the gate on contract-shaped memory.
      - [ ] **▸** Add a provenance field path for research-derived memories (url/provider/timestamp); verify.
      - [ ] **▸** Implement the promotion gate (confidence >= 0.7 AND outcome=solved); verify.
      - [ ] **▸** Wire promotion into the existing `store_memory` / `get_embedding` surface; verify.
- [ ] **R13** Experience recording (`src/experience/`).
      - [ ] **▸** Add a `record_research(query, sources_used, mode, duration, outcome)` helper; verify.
      - [ ] **▸** Set `experience_type="research"` + context { query, sources[], mode, duration }; verify.
      - [ ] **▸** Call it from the pipeline exit paths (success + failover); verify.

## Phase E: Tool registration, coverage gate, hardening
- [ ] **R11** Add raw MCP tools to `src/bridge/tools/search/mod.rs` — ONE tool per pass.
      - [ ] **▸** `web_search` (input schema + execute fn) + `all()` entry; verify.
      - [ ] **▸** `web_open`; verify.
      - [ ] **▸** `web_extract`; verify.
      - [ ] **▸** `research`; verify.
      - [ ] **▸** `quick_research`; verify.
      - [ ] **▸** `deep_research`; verify.
      - [ ] **▸** `find_error_resolution` (error-focused search, refactored only if a prior tool exists); verify.
      - [ ] **▸** Wire all new tools into the `register_tools()` Phase 6 (Search) chain in `src/bridge/tools/mod.rs`; verify.
      - [ ] **▸** Sync the THREE lists (`tool_names()`, `get_tools()`, `execute_tool()`) — a missing `get_tools()` entry is a phantom tool (T1-19); verify against `tools/list`.
      - [ ] **▸** Scope note: the ~20 long-tail per-source adapters (Wikipedia, Reddit, HN, arXiv, SEC, OSM, Bluesky, Telegram…) are POST-MVP plain `SearchProvider` impls — out of scope here.
- [ ] **R11b** Add test_suite coverage so "0 untested tools" stays green.
      - [ ] **▸** Add `TestRequirement` entries for `web_search`, `web_open`, `web_extract` to `function_registry/search_tools.rs`; verify.
      - [ ] **▸** Add `TestRequirement` entries for `research`, `quick_research`, `deep_research`, `find_error_resolution`; verify.
      - [ ] **▸** Add the matching id cases to `comprehensive_test/argument_builder.rs`; verify.
      - [ ] **▸** `cd test_suite && ./target/release/test_suite --probe <TOOL>` per tool to pick `IsSuccess(None)` vs `IsSuccess(Some("false"))`; verify coverage count.
- [ ] **R15** Context protection enforcement.
      - [ ] **▸** Enforce the hard 5-source cap at the pipeline exit; verify.
      - [ ] **▸** Add a token-budget estimate before the LLM call; verify.
      - [ ] **▸** Emit the "... and N more sources" truncation marker when capped; verify.
- [ ] **R16** End-to-end gate verification.
      - [ ] **▸** Wire the 13-step verification path (workflow gate -> research -> cascade -> evidence -> memory -> failover -> cancel) from research_engine.md into a test_suite flow test; verify.
      - [ ] **▸** Run the FULL gate: `cd test_suite && cargo build --release && ./target/release/test_suite`; ensure tests=100%, warnings=0, code-issues=0, untested-tools=0.
      - [ ] **▸** Fix any gate failures before claiming done (Verify, Don't Trust).

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
