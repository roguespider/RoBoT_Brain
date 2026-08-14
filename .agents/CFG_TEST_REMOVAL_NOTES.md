# #[cfg(test)] Removal Notes — for review

> Status: **PROPOSAL — not yet executed.** Awaiting your decision.
> Scope: every `#[cfg(test)]` in `robot_brain/src/` (60 attrs across 17 files).
> Goal: zero `#[cfg(test)]` in `src/`, per your directive. No test code lives
> in production source.

## The two flavors of `#[cfg(test)]` here

**Flavor 1 — whole test modules** (`#[cfg(test)] mod tests { #[test] fn ... }`).
Real test functions that assert internal logic. ~58 `#[test]` fns total.

**Flavor 2 — field/method-level test-only helpers** (`#[cfg(test)] pub fn ...` /
`#[cfg(test)] use ...`). Constructors/mutators that exist ONLY so tests can poke
private state (e.g. `record_success`, `with_config`, `with_ttl`, `add_source_insight`).
Zero `#[test]` fns of their own — they're scaffolding consumed by Flavor 1 modules.
These are the worst smell: production source carries methods that only tests call.

## Two global strategies (pick one — applies to ALL files)

### Strategy L — make robot_brain a lib+bin crate
Add `[lib]` + `src/lib.rs` re-exporting modules to `Cargo.toml`. Then `test_suite`
adds a `path = ".."` dependency on the `robot_brain` lib and runs ALL of these as
**plain Rust integration tests** in `test_suite/src/tests/` — no MCP gymnastics,
no fake tool calls, direct access to internal types.
- **Pro:** solves the entire class in one move; field-level helpers become normal
  `pub` methods tested directly; the dead-code "test-only helper" smell vanishes.
- **Con:** **violates the repo's stated principle** that robot_brain and test_suite
  are "two separate, independent programs that do NOT depend on each other's source."
  Linking the lib means test_suite depends on robot_brain's source. The subprocess-
  over-MCP testing model was a deliberate choice. You'd be reversing that.
- **My read:** cleanest mechanically, but it's an architecture decision, not a
  mechanical fix. Only choose this if you're OK retiring the "independent projects"
  boundary.

### Strategy M — wire each internal type onto an MCP-reachable path, migrate as MCP flow tests
For each file: expose the tested logic through a real tool (or extend an existing
tool's response), re-express the unit test as a `test_suite` MCP flow test, then
delete the `src/` block. test_suite stays a pure subprocess client.
- **Pro:** preserves the architecture; tests exercise the real public surface.
- **Con:** per-file work; some items are pure internal math with no natural MCP
  surface (contrived to wire); the field-level helpers either become real `pub`
  methods with production callers or get deleted with their dead types.
- **My read:** correct per AGENTS.md Dead Code Protocol, but slow and some cases
  are forced.

## Per-file notes (Flavor 2 helpers marked HELPER; test modules marked MODULE)

### MODULE — `src/personality/mod.rs` (16 tests) — T1-10B-01
Tests: default personality, apply/invalid preset, trait getters, decision logic.
**Reachable NOW** via MCP: `get_personality`, `apply_personality_preset`,
`list_personality_presets`, `set_personality_traits`, `get_personality_decision`.
- **Best removal (Strategy M):** migrate all 16 to `test_suite/src/tests/personality.rs`
  as MCP flow tests calling the 5 real tools. The tools already exist and return
  the asserted fields (preset, traits). Caveat: a few tests assert on
  `adapt_from_experience` / `success_rate` / `decay` / `reset` / serialization —
  assess per-test: if a tool exposes the field, migrate; if internal-only, either
  wire a tool or (Strategy L) test directly. This is the highest-value, most
  natural migration — do it first.

### MODULE — `src/bridge/acp/mod.rs` (20 tests) — T1-10B-12
Tests: ACP agent id, broadcast, message creation/reply/TTL/type.
**Reachable NOW** via MCP: `register_agent`, `list_acp_agents`, `route_acp_message`,
`create_acp_message`, `get_agent_capabilities`.
- **Best removal (Strategy M):** migrate to `test_suite/src/tests/acp.rs` as MCP
  flow tests. `create_acp_message` + `route_acp_message` + `list_acp_agents` cover
  message creation/reply/routing. TTL assertions: check if `create_acp_message`
  exposes TTL in its response; if not, extend it (small, justified — TTL is a real
  message property). 20 is a lot but the surface is real.

### MODULE — `src/bridge/mcp/client/mod.rs` (8 tests) — T1-10B-13
Tests: McpClient empty state, ToolError Display variants.
- `connect_mcp_server` / `call_tool` / `list_tools` MCP tools exist, BUT they test
  the empty/error state of the client registry, which is hard to force via MCP
  (you'd call `call_tool` on a non-existent server).
- **Best removal:** the ToolError Display tests are pure formatting — (Strategy L)
  test directly, OR (Strategy M) call `call_tool` with a fake server name and
  assert the error text matches a Display variant. The empty-state tests are
  awkward over MCP; leaning Strategy L for those, or delete (low value — asserting
  an empty HashMap is empty).

### MODULE — `src/learning/pipeline.rs` (3 tests) — T1-10B-15
Tests: LearningPipeline start/advance/stats. **ZERO MCP callers** — only used as
a startup self-check log in initialization.rs. `record_observation` stores to DB
directly, never touches LearningPipeline.
- **Best removal:** this is genuinely internal-only state machine. Per Dead Code
  Protocol: check `robot_architecture/` — if the learning-pipeline stages are a
  documented feature, wire `LearningPipeline` into `record_observation` (so
  recording an observation advances the pipeline) and migrate the test as an MCP
  flow (`record_observation` → assert stats via a status tool). If architecture is
  silent, **delete LearningPipeline + its tests** (it's dead code — the startup
  self-check is the only caller). My lean: delete, unless architecture describes it.

### MODULE — `src/experience/evolution/engine.rs` (3 tests) + HELPER attrs (14) — T1-10B-16
Tests: EvolutionEngine create_behavior/record_result/metrics. **ZERO MCP callers**
for the mutating methods — only `list_behaviors`/`list_active_behaviors` are
MCP-reachable (via `get_system_status`, returns counts only). The 14 HELPER attrs
are test-only constructors/mutators (`with_config`, `get_behavior`, `create_behavior`,
`record_result`, `add_evidence`, etc.).
- **Best removal:** hardest case. The evolution engine has rich internal logic but
  no tool populates behaviors (list_behaviors always returns 0 via MCP). Options:
  (M) add an MCP tool that creates/records behaviors (justified if evolution is a
  documented v0.0.1 feature) then migrate; (L) test the engine directly as a lib;
  or delete the untested mutating methods if evolution is post-v0.0.1. The HELPER
  attrs must become real `pub` with production callers OR be deleted — they cannot
  stay `#[cfg(test)]`. My lean: (L) for the engine logic + delete helpers that have
  no production future, because wiring a full evolution CRUD tool is out of scope
  for v0.0.1 cleanup.

### HELPER — `src/experience/evolution/behavior.rs` (4 attrs)
`add_source_insight`, `record_success`, `record_failure`, `recalculate_confidence`.
Test-only mutators. Same story as engine.rs — no production caller.
- **Best removal:** these ARE the real behavior-state machine (success/failure
  tracking). Per Dead Code Protocol, if evolution is documented, wire them into a
  production path (e.g. the agent loop calls `record_success`/`record_failure`
  after an action) and drop `#[cfg(test)]`. If not, delete with the behavior type.
  My lean: wire `record_success`/`record_failure` into `loop_runner` (it already
  knows success/failure) — that's the natural caller and makes them real.

### HELPER — `src/experience/evolution/evidence.rs` (2 attrs)
Test-only constructors on EvolutionEvidence.
- **Best removal:** same cluster as engine/behavior. Wire into a production path
  or delete with the evidence type. (Strategy L) lets them stay as `pub` tested
  directly.

### HELPER — `src/bridge/acp/message.rs` (7 attrs)
`with_ttl` + 6 test-only message constructors/fields.
- **Best removal:** (Strategy M) make `with_ttl` a real `pub` constructor (TTL is a
  real message property — not test-only) and expose it via `create_acp_message`;
  migrate the message tests to the ACP MCP flow. The other 6: assess each — if the
  field is real, make it `pub`; if test-fixture-only, delete.

### HELPER — `src/experience/hypothesis/services/repository.rs` (4 attrs)
Test-only repository helpers.
- **Best removal:** check if the hypothesis repository is MCP-reachable
  (`create_hypothesis`/`add_evidence`/`evaluate_hypothesis`/`list_hypotheses` exist).
  If so, (M) migrate; the helpers become real `pub`. If the repository abstraction
  is unused (store_memory uses queries directly, per T1-10B-18 pattern), delete it.

### HELPER — `src/experience/hypothesis/support/graph/graph_types.rs` (2 attrs)
Test-only graph node/edge helpers.
- **Best removal:** pure data-structure helpers. (Strategy L) test directly, or
  delete if the graph types are unused on any MCP path. Check architecture for the
  knowledge-graph feature.

### HELPER — `src/planner/engine/planner.rs` (1 attr) + `src/planner/policy.rs` (5 attrs)
Planner/policy test-only helpers.
- **Best removal:** `create_plan`/`get_plan`/`list_plans` MCP tools exist. (Strategy M)
  migrate planner tests as plan-flow tests; make the helpers real `pub`. Policy
  attrs: assess per-attr — if a policy is reachable via plan creation, migrate;
  else (L) or delete.

### MODULE — `src/experience/reflection/services/generator.rs` (3 tests) — T1-10B-03
Tests: generate_from_multiple_successes/failures, min-experiences threshold.
**RECLASSIFIED Group B** because `execute_create_reflection` passes empty
experiences → always returns success:true, so the tool NEVER exercises the tested
logic.
- **Best removal:** the tool is effectively a stub. (Strategy M) fix the tool to
  accept real experiences (or pull them from `list_experiences`) so it exercises
  `generate_from_experiences`, THEN migrate the 3 tests as reflection-flow tests.
  This is a real bug-fix (stub tool), not just test relocation. My lean: fix the
  tool — it's the correct outcome and unblocks the migration.

### MODULE — `src/memory/repository.rs` (1 test) + HELPER (1 attr) — T1-10B-18
Tests `SqliteMemoryRepository::from_path`. **ZERO non-test callers** — dead code.
`store_memory` uses `queries::insert_memory` directly, not the repository abstraction.
- **Best removal:** **delete both** the test and the `from_path` helper + the
  `SqliteMemoryRepository`/`MemoryRepository` trait if they have no production
  caller. This is dead code (Dead Code Protocol: architecture silent → delete).
  Confirmed no MCP surface.

### MODULE — `src/memory/retrieval.rs` (2 tests) — T1-10B-06 (partial)
Tests: retrieve_permanent, confidence_filtering.
- **Best removal:** (Strategy M) `search_memory`/`ranked_search`/`get_memory` MCP
  tools exist. Migrate as memory-search flow tests: store a memory, search, assert
  ranking/confidence. The 2 tests assert retrieval + confidence filtering which
  `ranked_search` exposes. Natural migration.

### MODULE — `src/database/queries/memory.rs` (1 test) + HELPER (1 attr) — T1-10B-19
Tests `delete_memories_by_string_ids`. **ZERO non-test callers** — dead code.
`archive_memory` uses `delete_memories` (by Uuid), not the by-string-ids variant.
- **Best removal:** **delete both** the function and its test. Dead code, no MCP
  surface. (Same pattern as T1-10B-20 embeddings, which was already done.)

### MODULE — `src/bridge/tools/ingestor/audio_transcriber.rs` (1 test) — T1-10B-07 (partial)
Tests audio analysis. Already partially migrated (2 of 3).
- **Best removal:** (Strategy M) the remaining test exercises audio analysis
  internals. If `ingest_files` on an audio file triggers the analyzed path, migrate
  as an ingest flow test; else (L) test directly or delete if the audio path is
  stubbed. Check whether audio ingest is a real v0.0.1 feature.

## Summary recommendation (my opinion, yours to override)

- **For items with a real MCP surface** (personality, acp, memory/retrieval,
  reflection/generator, embeddings-done): **Strategy M** — migrate as MCP flow
  tests. This is what the plan already does and it's correct.
- **For dead code with zero callers** (memory/repository.rs, queries/memory.rs
  by-string-ids): **delete** the code + tests. No coverage lost — it was never
  called.
- **For internal-only state machines with no MCP surface but real logic**
  (evolution engine/behavior/evidence, learning/pipeline, graph_types,
  mcp/client empty-state): this is the actual decision point. Either
  **(L) make robot_brain a lib crate** so test_suite links and tests them
  directly (cleanest, but reverses the "independent projects" principle), OR
  **wire them onto a production path** (e.g. evolution `record_success` into the
  agent loop) and migrate — but that's feature work, not cleanup.
- **The field-level HELPER attrs are the clearest win regardless of strategy:**
  none of them should be `#[cfg(test)]`. Each is either a real method that needs a
  production caller (make it `pub`, wire it) or test scaffolding for a dead type
  (delete it with the type).

## What I will NOT do until you decide

I will not touch any of these. The newspaper wall (pre-commit/pre-push) now blocks
any commit/push unless the gate is green, so even if I jumped ahead I couldn't ship
it. Tell me: Strategy L (lib crate) vs Strategy M (wire + migrate) vs a per-file
mix, and I'll execute in PLAN order, one file at a time, gate green after each.
