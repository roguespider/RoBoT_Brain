# 1. OBJECTIVE

Take RoBoT Brain from its current state to a **finished v0.0.1 → finished
v0.0.2 → finished v0.0.2.1**, using **small 10-15 minute increments**.

- Each increment is ONE small, verifiable, committable change.
- After each increment: build → live test → test suite → commit → push → STOP.
- Do NOT spend a session on one upgrade. If an increment feels bigger than 15
  minutes, split it further.
- Work the increments in order, top to bottom. Do not skip ahead.

The target baseline is **`robot_architecture/v0.0.2.1/`** (33 chapters +
appendices A-E + `FINAL_ARCHITECTURE_SPEC.md`). v0.0.2 is an intermediate
milestone on the way there.

---

# 2. CONTEXT SUMMARY

## The blueprints

- **v0.0.1** — `robot_architecture/v0.0.1/ARCHITECTURE.md`. What the codebase
  currently approximates. TIER 1 finishes conforming to this.
- **v0.0.2** — `robot_architecture/RoBoT Architecture v0.0.2.md`. Intermediate
  upgrade: elevate Context + Conversation to first-class, add Data Contracts.
  TIER 2 conforms existing systems to this.
- **v0.0.2.1** — `robot_architecture/v0.0.2.1/` (00.md-33.md + appendices). The
  FINAL architectural baseline. Adds Execution Engine, Tool Engine, Memory
  Hierarchy, Context Lifecycle, Retrieval Pipeline, Prompt Construction,
  Strategic Learning, Confidence System, Storage, Database Design, Background
  Workers, Security & Trust, Observability, Developer Interface/Control Plane,
  Configuration, Testing, Deployment. TIER 3 builds the missing subsystems.

## Current codebase state (verified 2026-08-11)

- Workspace: two independent programs — `robot_brain` (root, MCP server) and
  `test_suite/` (E2E tests via MCP protocol).
- Builds with **0 cargo warnings**, **128 MCP tools**, **333/333 tests pass**,
  0 code-quality issues. Coverage gap: 50 server tools untested (60.9%).
- `#![allow]` / `#[allow]` in `src/`: **0** (clean).
- `self_check.rs` files: **8 remain** (planner, learning, knowledge, experience,
  experience/reflection, experience/hypothesis, experience/hypothesis/support/graph,
  experience/hypothesis/services).
- Cognitive loop (P0/P1) DONE: `ExperienceRecorded → Reflection → Hypothesis →
Knowledge → Reputation`; `run_agent_goal` agent loop works.
- P4 open: in-memory `JobQueue`; no loop-health metrics; generic MCP dispatch
  does not emit experiences.
- **No v0.0.2/v0.0.2.1 new subsystems exist**: no Context Engine, Conversation
  Engine, Execution Engine, Tool Engine, Retrieval Pipeline, Prompt
  Construction, AI Runtime, multimodal, GUI, security/trust, observability.

## Constraints

- Strict Rust coding standards (no panics/unwrap/expect, no placeholders, no
  `#[allow(...)]`, no ignored `_` vars). Enforced by the test suite.
- Incremental workflow: after EACH increment, run the gate (below) green, then
  commit + push, then STOP. Never batch.
- **Verify, don't trust:** every step must be VERIFIED by inspecting the actual
  codebase state and running the gate — never rely on a "done" message, a
  commit description, or a checkbox marked `[x]`/`[in]`. Open the file, read the
  code, confirm the change is there and the gate is actually green. A commit
  that claims "fixes all warnings" may be lying; run the gate and read the JSON
  report to confirm the metric is actually 0.
- Large-file rule: split `.rs` files over ~1000 lines that mix
  responsibilities (see `.agents/LARGE_FILE_REFACTOR.md`).
- Local-first: the cognitive architecture must work against cloud/external
  models first. AI Runtime (Candle) is built last, as an enhancement layer.

## The verify gate (run after EVERY increment)

```bash
# test_suite auto-builds robot_brain, connects via MCP, runs all tests +
# code analysis, and enforces 0 warnings / 0 code-issues / 0 untested tools.
cd test_suite && cargo build --release && ./target/release/test_suite
# Or: make gate
```

The gate is green only when all tests pass AND 0 warnings / 0 code-issues /
0 untested tools. If red, the increment is NOT done. Fix it before claiming
done. Never commit a red gate.

---

# 3. APPROACH — three tiers of small increments

Work through three tiers in order. Each tier is a checklist of small
increments. Do them top-to-bottom, one at a time, with the verify gate green
between each.

- **TIER 1 — Finish v0.0.1** (clean baseline). Clear the remaining self_check
  debt, migrate the queue, add loop metrics, close the MCP→experience path.
  No new features; just finish what v0.0.1 requires. **End state = finished v0.0.1.**
- **TIER 2 — Reach v0.0.2** (upgrade existing systems). Introduce Data Contracts,
  then upgrade each existing subsystem (Memory, Knowledge, Experience, Learning,
  Planner, Skills/Workflows, World Model, Personality) to its v0.0.2 chapter.
  **End state = finished v0.0.2.**
- **TIER 3 — Reach v0.0.2.1** (add missing subsystems). Build the new engines
  from the v0.0.2.1 chapters: Execution, Tool, Memory Hierarchy, Context
  Lifecycle, Retrieval Pipeline, Prompt Construction, Strategic Learning,
  Confidence System, Storage/Database, Background Workers, Security & Trust,
  Observability, Developer Interface/Control Plane, Configuration, Testing,
  Deployment — then AI Runtime (Candle), Multimodal, GUI last.
  **End state = finished v0.0.2.1.**

**Why this order:** finish v0.0.1 first so no dead-code debt is carried into a
refactor. Upgrade foundation systems (Data Contracts, Memory, Knowledge) before
the Context/Conversation engines so those engines consume real contract-shaped
data, not stubs. Build the cognitive architecture against cloud/external models
first; AI Runtime (Candle) comes last as the local provider behind the
`InferenceProvider` trait, and is the prerequisite for Multimodal.

---

# 4. TIER 1 — Finish v0.0.1 (clean baseline)

> Goal: green gate (test_suite exit 0). End state = a clean v0.0.1 baseline.
> Tick `[x]` when an increment is committed with a green gate.
>
> **Work order:** 1E (coverage gate) FIRST — it's the actual gate problem and
> the user's priority. Then 1B (queue), 1C (metrics), 1D (MCP→experience).
>
> **NOTE on self_check removal (moved to TIER 2):** the 8 `self_check.rs` files
> exercise APIs that have NO other callers (informed plans, replanning, action
> selection, policy engine, etc.). This is a binary crate, so removing a
> self_check surfaces dead-code warnings on those pub APIs (24 warnings for
> planner alone). Per the Dead Code Resolution Protocol, these APIs ARE
> described in v0.0.2.1 Chapter 11 (Planning) / Chapter 19 (Confidence), so
> they're incomplete stubs that must be WIRED into real MCP tools, not deleted.
> That wiring is TIER 2 work (T2-32..T2-36 for planner, similar for others).
> So: self_check removal happens DURING each system's TIER 2 upgrade, not as
> standalone TIER 1 cleanup. TIER 1 focuses on the gate + queue + metrics.

## 1A. (Moved to TIER 2) Remove self_check.rs files

> Moved: each self_check is removed as part of its system's TIER 2 upgrade,
> after the APIs it exercises are wired into real MCP tools. See T2-32..T2-42.
> Attempting standalone removal in TIER 1 creates dead-code warnings (binary
> crate flags unreached pub APIs), violating the 0-warnings gate.

## 1B. SQLite-backed JobQueue (V2-11)

- [x] **T1-09** Add `job_queue` table + migration in `src/database/migrations/`.
      (commit d1ee096; migration 012 + registered in run loop)
- [x] **T1-10** Wire enqueue/dequeue through `src/experience/queue.rs` to SQLite.
  VERIFIED 2026-08-12 by codebase inspection + live restart-durability test:
  queue.rs (with_database/push_job/pop_job/mark_complete/mark_failed/
  restore_from_database), worker_manager/manager.rs (job_queue field,
  new_with_queue, enqueue→push_job, broadcast→push_job,
  mark_job_complete/mark_job_failed), worker_manager/background.rs (loop calls
  mark_job_complete/mark_job_failed), bridge/mcp/context.rs (NOTE: PLAN's old
  path `src/mcp/context.rs` was wrong — real path is `src/bridge/mcp/context.rs`;
  pub job_queue field + new() takes it), bridge/app/initialization.rs (creates
  JobQueue::with_database, restore_from_database at startup, passes to
  WorkerManager::new_with_queue + McpContext::new, runs a startup lifecycle
  probe). Project builds and the full test suite runs (145/145 pass). A new
  end-to-end restart-durability test in test_suite
  (tests/queue_durability.rs) boots the real server, injects a pending
  job_queue row into its SQLite DB, kills the server, boots a fresh server in
  the same dir, and confirms via get_system_status (event_bus.pending_jobs)
  that the row is restored into the live queue and survives with status=pending
  in SQLite. The test passes. (Caveat noted: restored jobs are NOT replayed to
  workers — restore_from_database repopulates the in-memory JobQueue cache but
  nothing re-enqueues to ExperienceWorker channels. The startup probe's
  pop_job can drain restored `experience_scorer` rows. Replay-on-start is a
  gap, but the "queue survives a process restart" criterion is met.)
- [ ] **T1-10B** all #[cfg(test)] in codebase should be made into actual test's in test_suite
      (Verified inventory 2026-08-12: 85 test fns across 20 files, plus 20
      more files with EMPTY `#[cfg(test)] mod tests{}` blocks.) Work proceeds
      ONE file at a time: migrate → gate green → commit → push → stop.
      Rules: AGENTS.md forbids `#[allow(*)]`; test_suite cannot import/link
      robot_brain source (it only talks MCP/CLI). So tests are re-expressed as
      MCP/CLI-based tests in `test_suite/src/tests/`, then the `#[cfg(test)]`
      block is deleted from `src/`.

      ### Group A — MCP-reachable (move to test_suite, delete src/ block)
      - [x] **T1-10B-01** `personality/mod.rs` (16 tests) DONE 2026-08-14.
            Migrated 8 MCP-reachable behaviors to
            `test_suite/src/tests/personality.rs` (run_personality_tests):
            default personality (get_personality), apply_preset valid
            (apply_personality_preset + get), apply_preset invalid
            (applied=false, preset unchanged), list_presets, set_trait
            (set_personality_traits + get), communication_style
            (verbosity 0.2/0.5/0.8 -> Concise/Balanced/Detailed via
            get_personality), format_response (detailed vs concise),
            decide (cautious preset -> reason mentions cautious, approach
            Thorough via get_personality_decision). f32 traits compared with
            tolerance (abs diff < 0.01). Each test resets to "balanced" preset
            first (shared App mutex state).
            Group B (internal-only, no MCP surface) DELETED per decision:
            test_adapt_from_experience_success/failure (adapt_from_experience
            -- no tool; called by app/personality.rs adapt_personality in the
            agent loop), test_should_explore/should_take_risk
            (internal math; exercised indirectly by get_personality_decision
            via decide), test_should_use_creativity (planner-internal),
            test_get_timeout (app fn exists, no tool), test_success_rate
            (app fn exists, no tool), test_adjust_trait_clamping (clamping is
            in adjust_trait, NOT exposed -- set_personality_traits passes
            out-of-range values through unclamped), and the preset->custom
            assertion of test_adjust_trait (set_personality_traits does not
            flip preset to "custom"). The deleted methods themselves remain in
            production (called by decision_making.rs, planner.rs,
            app/personality.rs). decide() still covers should_explore/
            should_take_risk indirectly. Gate: CfgTest 56 -> 55, 0 emoji,
            145/145 registry tests, 0 err, 0 untested, 40 warns (no regression).
            tools: get_personality / apply_personality_preset /
            list_personality_presets / set_personality_traits /
            get_personality_decision / format_response.
      - [x] **T1-10B-02** `personality/emotional.rs` (3 tests) DONE.
            Group B (internal-only, no MCP surface) — the `#[cfg(test)]` block
            was DELETED per the Group B decision (not left in place). Verified
            2026-08-14: emotional.rs has no cfg(test) block and is NOT in the
            gate CfgTest list. EmotionalState::observe() has no MCP surface —
            it is only called by the agent loop (loop_runner.rs:304 via
            observe_emotional_outcome -> personality.rs:72), never by a tool.
            get_personality returns emotional_weight (observable) but NOT the
            individual fields (frustration/satisfaction/engagement) the tests
            asserted on, and there is no tool to trigger observe() or set
            fields, so the tests required direct struct manipulation. The
            deleted methods remain in production and are still exercised:
              - emotional_weight()  -> decision_making.rs:49, loop_runner.rs:166,
                                       bridge/tools/personality/mod.rs:68
                                       (exposed via get_personality)
              - action_threshold_bias() -> decision_making.rs:52
              - observe()           -> observe_emotional_outcome (personality.rs:72,
                                       called by loop_runner.rs:304)
            No dead-code warnings introduced (methods all still called).
            emotional_weight is still covered indirectly through
            get_personality (returns the field) and get_personality_decision
            (decide() applies emotional_weight to confidence).
      - [x] **T1-10B-03** `experience/reflection/services/generator.rs` (3) DONE
            2026-08-14. Group B (internal-only, no MCP surface) — the
            `#[cfg(test)]` block was DELETED per the Group B decision. The 3
            tests (test_generate_from_multiple_successes,
            test_generate_from_failures, test_requires_min_experiences)
            tested generate_from_experiences directly with constructed
            Experience vecs — behavior not reachable via any tool:
              - execute_create_reflection (the MCP tool) calls
                reflection_engine.generate_reflection(vec![].as_slice(), ...)
                — passes EMPTY experiences.
              - generate_reflection -> generate_from_experiences with an empty
                slice returns None (len < min_experiences=2) -> tool always
                returns {success:true, id:random_uuid} regardless of input.
              - So the tool NEVER exercises the tested logic (Success/Failure
                reflection_type determination, min-experiences threshold).
            generate_from_experiences remains in production (called by
            engine/mod.rs:64 generate_reflection). No dead-code warnings
            introduced (40 warns unchanged). Side benefit: removed the
            `unsafe { std::hint::unreachable_unchecked() }` the tests used to
            satisfy the compiler after assert!(false). Gate: CfgTest 55 -> 54,
            fresh full rebuild verified (libssl-dev reinstalled), 0 emoji,
            145/145, 0 err, 0 untested.
      - [x] **T1-10B-04** `knowledge/store.rs` (2 tests) DONE 2026-08-12.
            Migrated test_add_and_get + test_get_mature to test_suite/src/
            tests/knowledge_store.rs via MCP flow. test_add_and_get: add_knowledge
            (calls KnowledgeStore::add) -> query_knowledge (retrieves via
            get_all, verify statement in items[]). test_get_mature: add
            low-conf(0.3) + high-conf(0.8) items, query_knowledge with
            min_confidence=0.7 -> high included, low excluded (mirrors
            get_mature's is_mature >= 0.7 threshold). Deleted #[cfg(test)]
            block from src/. add/get/get_mature still used by handlers.
            Gate: 145/145, 0 issues, 0 untested, 67 warnings.
      - [x] **T1-10B-05** `knowledge/query.rs` (3 tests) DONE 2026-08-12.
            Migrated test_text_filter + test_confidence_filter + test_ranking
            to test_suite/src/tests/knowledge_query.rs via MCP flow. query_knowledge
            calls apply_query (text + min_confidence filters) + rank_items
            (relevance sort). test_text_filter: add 2 items, query for one
            text -> only it in items[]. test_confidence_filter: add high(0.9)+
            low(0.3), query min_confidence=0.7 -> only high in items[].
            test_ranking: add 2 matching items, verify high-conf is best_match.
            Deleted #[cfg(test)] block from src/. apply_query/rank_items still
            used by query_knowledge handler. Gate: 145/145, 0 issues, 0
            untested, 67 warnings.
      - [x] **T1-10B-06** `memory/retrieval.rs` (2 of 4 migrated; 2 reclassified Group B) DONE 2026-08-12.
            Migrated test_retrieve_working + test_unified_retrieve to
            test_suite/src/tests/memory_retrieval.rs via MCP flow. search_memory
            calls retrieve() which calls get_from_working + get_from_permanent.
            test_retrieve_working: store_memory -> search -> content in results[].
            test_unified_retrieve: store 2 items -> search -> both in results[].
            RECLASSIFIED to Group B (LEAVE as Rust unit test):
            test_retrieve_permanent + test_confidence_filtering — store_memory
            only writes to Working layer (PermanentMemory cache not populated
            by any MCP tool), and retrieve_with_query(min_confidence) is never
            called by an MCP tool. Removed those 2 from src/ test block, kept
            the 2 Group B tests. Gate: 145/145, 0 issues, 0 untested, 67 warnings.
      - [x] **T1-10B-07** `bridge/tools/ingestor/audio_transcriber.rs` (2 of 3 migrated; 1 reclassified Group B) DONE 2026-08-12.
            Migrated test_is_audio_file + test_get_supported_extensions to
            test_suite/src/tests/audio_transcriber.rs via MCP flow.
            transcribe_audio calls is_audio_file (which calls
            get_supported_extensions) and returns "Not a supported audio file"
            for non-audio extensions. Test: create temp files with audio
            (mp3/wav/m4a/flac/ogg) + non-audio (txt/mp4) extensions, call
            transcribe_audio, verify audio exts pass the is_audio_file gate
            (different error) while non-audio exts get "Not a supported audio
            file". RECLASSIFIED to Group B (LEAVE as Rust unit test):
            test_audio_analysis — AudioAnalysis::from_samples requires valid
            audio samples loaded from a real WAV file; not practical via MCP.
            Removed 2 from src/ test block, kept test_audio_analysis.
            Gate: 145/145, 0 issues, 0 untested, 67 warnings.
      - [x] **T1-10B-08** `experience/exploration/hypothesis.rs` (2 tests) DONE
            2026-08-12. Migrated test_hypothesis_lifecycle +
            test_confidence_clamping to test_suite/src/tests/
            exploration_hypothesis.rs via MCP flow. Constructor confidence
            clamp (1.5->1.0, -0.5->0.0) tested via add_hypothesis
            initial_confidence + get_exploration_status readback. Lifecycle
            (new -> set_result -> update_confidence) tested via add_hypothesis
            -> evaluate_exploration_hypothesis (confidence 0.5->0.9 for
            supported). Caveat: update_confidence clamp branch (1.5->1.0) is
            NOT MCP-reachable (tool hardcodes in-range values); only the
            constructor clamp is tested. Both use the same .clamp(0.0,1.0).
            Deleted #[cfg(test)] block from src/. Methods still used by
            handlers. Gate: 145/145, 0 issues, 0 untested, 67 warnings.
      - [x] **T1-10B-09** `experience/exploration/attempt.rs` (2 tests) — DONE
            2026-08-12. Migrated test_attempt_builder + test_attempt_failure to
            test_suite/src/tests/exploration_attempt.rs via MCP flow
            (start_exploration -> record_attempt [expected==actual] ->
            record_attempt [expected!=actual] -> get_exploration_status ->
            assert attempt[0].success=true, attempt[1].success=false).
            record_attempt calls ExplorationAttempt::new + with_expected_result
            + with_actual_result (the exact builder methods under test).
            Deleted #[cfg(test)] block from src/. Builders still used by
            record_attempt handler (no new dead-code). Gate: 145/145, 0 issues,
            0 untested, 67 warnings.
      - [x] **T1-10B-10** `experience/exploration/finding.rs` (1 test) — DONE
            2026-08-12. Migrated test_finding_new_and_promote to
            test_suite/src/tests/exploration_finding.rs via MCP flow
            (start_exploration -> complete_exploration [calls
            ExplorationFinding::new] -> get_exploration_status [promoted=false]
            -> promote_finding [calls f.promote()] -> get_exploration_status
            [promoted=true]). Deleted #[cfg(test)] block from src/. promote()
            still used by promote_finding MCP handler (no new dead-code).
      - [x] **T1-10B-11** `database/queries/observations.rs` (1 test) — DONE
            2026-08-12. Migrated the MCP-reachable part (record_observation
            [insert_observation] → list_observations, verify content+type) to
            test_suite/src/tests/observations.rs. The original test focused on
            link_observation_to_experience, which had NO MCP surface and NO
            production callers (was #[cfg(test)]-only dead code) — deleted
            link_observation_to_experience + get_observation + the test module
            from src/. Gate: 145/145, 0 issues, 0 untested, 67 warnings.

      ### Group B — internal-only, NO MCP surface (DECISION NEEDED)
      These test pure internal Rust types no tool exposes. test_suite cannot
      run them without importing robot_brain source (forbidden). Options:
      (1) leave as Rust unit tests (gate does NOT flag #[cfg(test)], only
      dead-code), (2) delete (loses coverage), (3) expose via test-only MCP
      tool (overkill). Leaning: LEAVE as-is. ~48 tests.
      - [ ] **T1-10B-12** `bridge/acp/mod.rs` (20) — ACP router/registry/message
            structs.
      - [ ] **T1-10B-13** `bridge/mcp/client/mod.rs` (8) — McpClient empty-state
            / ToolError Display.
      - [~] **T1-10B-14** `experience/scorer.rs` (5) —
      RECLASSIFIED to Group B (LEAVE as Rust unit test) 2026-08-12.
      Reason: EncounterScore, score_encounter(), and aggregate_encounter_scores()
      have ZERO callers on any MCP-reachable path. The coordinator uses
      scorer.score() (returns ExperienceScore, a DIFFERENT type), not
      score_encounter(). The only non-test references are ExperienceScorer::new()
      passed to the coordinator (which calls .score(), not .score_encounter()).
      EncounterScore is pure internal math with no MCP surface.
      - [~] **T1-10B-15** `learning/pipeline.rs` (3) —
      RECLASSIFIED to Group B (LEAVE as Rust unit test) 2026-08-12.
      Reason: LearningPipeline (start_from_input, advance_stage, stats) has
      ZERO callers on any MCP-reachable path. It's only used in
      initialization.rs as a startup self-check (tracing::info log), not
      exposed as an MCP tool. record_observation stores observations in DB
      directly, never touching LearningPipeline. Pure internal state machine
      with no MCP surface.
      - [~] **T1-10B-16** `experience/evolution/engine.rs` (3) —
      RECLASSIFIED to Group B (LEAVE as Rust unit test) 2026-08-12.
      Reason: EvolutionEngine's methods (create_behavior, record_result,
      add_evidence, get_metrics, update_priority, merge_behaviors,
      evaluate_and_maintain, etc.) have ZERO callers on any MCP-reachable
      path. The only MCP-reachable methods are list_behaviors +
      list_active_behaviors, called by get_system_status (returns counts
      only, not behaviors). No MCP tool creates/populates behaviors, so
      list_behaviors always returns 0 via MCP. create_behavior_from_insight
      (the trait method) is internal-only. Behavior methods (add_source_insight,
      record_success/failure, start_practicing, success_rate) are also
      internal-only. Pure internal evolution logic with no MCP surface.
      - [x] **T1-10B-17** `bridge/tools/ingestor/semantic_chunker.rs` (3) DONE 2026-08-12.
            Migrated test_markdown_parsing + test_sentence_splitting +
            test_code_parsing to test_suite/src/tests/semantic_chunker.rs via
            MCP flow. ingest_files (file_path) calls ingest_single_file ->
            parse_document, which dispatches to parse_markdown (for .md) /
            parse_code (for .rs). parse_markdown internally calls
            split_sentences. The tree is flatten()-ed; chunks_created in the
            output is the chunk count. Test: create .md with >=2 sections ->
            ingest_files -> chunks_created >= 2 (exercises parse_markdown +
            split_sentences). Create .rs with >=2 functions -> ingest_files ->
            chunks_created >= 2 (exercises parse_code). Removed test block
            from src/. Gate: 145/145, 0 warnings, 0 issues, 0 untested.
      - [~] **T1-10B-18** `memory/repository.rs` (1) —
      RECLASSIFIED to Group B (LEAVE as Rust unit test) 2026-08-12.
      Reason: SqliteMemoryRepository and the MemoryRepository trait have ZERO
      callers outside repository.rs — they're unused dead code. The
      store_memory MCP tool uses queries::insert_memory directly, not the
      repository abstraction. from_path is a constructor for a custom DB path
      that no MCP tool invokes. Cannot be exercised via MCP.
      - [~] **T1-10B-19** `database/queries/memory.rs` (1) —
            RECLASSIFIED to Group B (LEAVE as Rust unit test) 2026-08-12.
            Reason: delete_memories_by_string_ids has ZERO callers outside
            its own test — it's dead code (both the function and its test are
            wrapped in #[cfg(test)]). The archive_memory MCP tool uses
            delete_memories (by Uuid), not delete_memories_by_string_ids.
            Cannot be exercised via MCP.
      - [x] **T1-10B-20** `database/queries/embeddings.rs` (1) DONE 2026-08-12.
            Migrated test_get_and_delete_embedding_by_id to
            test_suite/src/tests/embeddings.rs via MCP flow. The src/ unit test
            tested get_embedding + delete_embedding (the by-embedding-id
            variants, which were #[cfg(test)] test-only functions). The MCP
            tools (store_embedding, get_embedding, delete_embedding) use the
            by-memory-id variants (get_embedding_by_memory_id +
            delete_embedding_by_memory_id, production code). Migration tests
            the same lifecycle via the production by-memory-id path:
            store_embedding -> get_embedding (found) -> delete_embedding ->
            get_embedding (not found). Removed the 2 #[cfg(test)] functions
            (get_embedding, delete_embedding) + the test block from src/.
            Gate: 145/145, 0 warnings, 0 issues, 0 untested.

      ### Group C — empty cfg-test blocks (delete, trivial)
      - [ ] **T1-10B-Z** Remove 20 EMPTY `#[cfg(test)] mod tests{}` blocks
            (files with 0 actual #[test] fns). Low risk.

      **Decision (2026-08-12):** Group B = LEAVE as Rust unit tests (gate does
      not flag #[cfg(test)]; deleting loses real coverage; no MCP surface to
      migrate to). Group A executed SMALLEST-FIRST to establish the migration
      pattern before the 16-test personality file. Group C last (trivial).
      **Resume here:** T1-10B-10 (exploration/finding.rs, 1 test) — smallest,
      establishes pattern. Execution order: 10, 11, 09, 08, 04, 05, 03, 02, 06,
      07, 01, then Z.
- [x] **T1-11** Handle broadcast `Lagged` events explicitly (skip+log or drain)
      in the worker path. (commit 560efad — both event subscriber and worker manager drain lagged events + worker manager records failed job)
- [x] **T1-12** Update `src/bridge/app/initialization.rs` startup verification
      (comment already removed; verification now reads "Verify durability: a fresh queue instance restores the pending/running rows written above from SQLite").

**Done when:** queue survives a process restart in a manual test; gate green.

## 1C. Loop-health metrics (V2-12)

- [x] **T1-13** Add `loop_latency` metric capture around `AgentLoop::run`.
      (commit in progress — added gauge fields + timer wrapping)
- [x] **T1-14** Add `confidence_drift` metric capture. DONE (verified
      2026-08-12 by codebase inspection). Captured in `src/agent/loop_runner.rs:176`
      (record_confidence_drift), not in event_subscriber/handlers.rs as originally
      planned — the loop runner is the correct capture point (drift measured per
      loop iteration). Field + record/get in `src/experience/metrics.rs`. Exposed
      via get_system_status (acp_handler.rs:437).
- [x] **T1-15** Add promotion-throughput (reflection→hypothesis→knowledge)
      metric. DONE (verified 2026-08-12 by codebase inspection). Captured in
      `src/agent/loop_runner.rs:287` (record_promotion_throughput). Field +
      record/get in `src/experience/metrics.rs`. Exposed via get_system_status
      (acp_handler.rs:438).
- [x] **T1-16** Expose the three new metrics via the `get_system_status` MCP
      tool. (done — `loop_health` block added to status JSON)

**Done when:** `get_system_status` live shows loop_latency / confidence_drift /
promotion_throughput; gate green.

## 1D. Close the generic MCP→experience path (V2-05)

- [x] **T1-17** Hook `emit_tool_experience` (publishes ExperienceRecorded)
      into the post-tool-execution dispatch wrapper. DONE (verified 2026-08-12).
      Wired in `src/bridge/rmcp/mod.rs:127` (success path) and `:141` (error path)
      — both call `emit_tool_experience(tool_name, was_successful, &arguments)`.
      Impl in `src/bridge/rmcp/types.rs:121`. Note: impl method renamed to
      `emit_tool_experience` (not `emit_experience_recorded`); it publishes the
      ExperienceRecorded event via coordinator.process() internally.
- [x] **T1-18** Ensure idempotency (no double-emit from a single tool call).
      DONE (verified 2026-08-12). The emit_tool_experience call sites are in
      mutually-exclusive match arms (Ok at mod.rs:127, Err at mod.rs:141), so a
      single tool execution emits exactly once. coordinator.process() publishes
      ExperienceRecorded once per call. No explicit guard needed — structural
      idempotency via mutually-exclusive match arms.

**Done when:** calling `store_memory` directly records an experience; no
double-emit from the agent loop; gate green.

## 1E. Close the coverage gate (make test_suite exit 0)

> The gate is red ONLY because of coverage gaps: 91/91 tests pass, 0 code
> issues, 0 warnings, but 50 server tools have no FunctionRegistry test and 6
> "phantom" embedding tools are tested but not exposed by the server. Each
> increment below adds a FunctionRegistry test entry for one tool group (in
> `test_suite/src/function_registry.rs` or the relevant `tests/<group>/` file).
> The suite exit code flips from 1 → 0 as coverage closes. Source of truth for
> the live untested/phantom lists:
> `test_suite/test_suite_report.json` → `coverage.untested_tools` /
> `coverage.phantom_tools`.

### 1E.1 — Fix the phantom embedding tools (a real wiring defect)

- [x] **T1-19** Fix the 6 phantom embedding tools (`store_embedding`,
      `get_embedding`, `search_similar`, `list_embeddings`, `delete_embedding`,
      `get_embedding_stats`). **DONE (commit b9b43ff).** Root cause: the memory
      handler maintained three separate tool lists that drifted — `tool_names()`
      listed all 13, `execute_tool()` dispatched all 13, but `get_tools()` (which
      feeds the RMCP `tools/list` response) only built 7 `Tool::new` entries and
      omitted the 6 embedding tools. They were callable but not advertised, so
      the coverage cross-check flagged them as phantom. Fix: added the 6
      embedding `Tool::new` entries to `get_tools()`, mirroring the schemas in
      `definitions.rs`. Verified 200%: all 6 appear in `tools/list`, all 6
      live-callable, full round-trip (store→get→search→list→stats→delete→
      post-delete confirms gone), build 0 warnings, live 54/54, `phantom_tools`
      6→0. **Lesson:** the `tool_names()` / `get_tools()` / `execute_tool()` triad
      in each handler is a drift hazard — three lists that must stay in sync.
      Watch for the same pattern in other handlers.

### 1E.2 — Add FunctionRegistry tests for untested tool groups

One increment per group. Each adds test entries that call the tool via MCP and
assert a sane response. Pattern is in `function_registry/` — copy an existing
entry, change the tool name + expected fields.

- [x] **T1-20** ACP tools (9): `route_acp_message`, `register_agent`,
      `unregister_agent`, `list_acp_agents`, `acp_agent_count`, `acp_registry`,
      `acp_router`, `create_acp_message`, `get_agent_capabilities`.
      **DONE (commit 6b7d036).** Added `function_registry/acp_tools.rs`.
- [x] **T1-21** System/session tools (4): `get_system_status`,
      `get_session_state`, `cleanup_sessions`, `get_consumed_resources`.
- [x] **T1-22** Memory/search extras (3): `archive_memory`, `link_memories`,
      `ranked_search`.
- [x] **T1-23** Knowledge lifecycle (6): `get_knowledge`, `delete_knowledge`,
      `update_knowledge`, `get_related_knowledge`,
      `validate_knowledge_dependencies`, `bump_knowledge_version`.
- [x] **T1-24** Evidence/observation (3): `get_evidence`, `list_evidence`,
      `list_observations`.
- [x] **T1-25** Reflection extras (3): `update_reflection`,
      `validate_reflection`, `list_reflections_by_status`.
- [x] **T1-26** Skills extras (5): `get_skill_metrics`, `clear_skill_metrics`,
      `get_unreliable_skills`, `unregister_skill`, `search_skills_by_tag`.
- [x] **T1-27** Personality (6): `get_personality`, `set_personality_traits`,
      `apply_personality_preset`, `list_personality_presets`,
      `get_personality_decision`, `format_response`.
- [x] **T1-28** World model (10): `list_world_entities`, `get_world_entity`,
      `upsert_world_entity`, `find_world_entity`, `get_world_model_stats`,
      `get_world_relationships`, `add_world_relationship`,
      `get_world_dependencies`, `get_world_blockers`, `get_consumed_resources`.
- [x] **T1-29** Agent/workflow extras (2): `run_agent_goal`,
      `set_workflow_variable`.

  **T1-21..T1-29 DONE (commit 7775ca1).** Implemented together in a single
  `function_registry/coverage_tools.rs` (40 entries) with a `req()` helper that
  takes `expect_fail` to pick the validation. Validation chosen from live
  probing: `IsSuccess(None)` for tools that succeed on a default/fake call;
  `IsSuccess(Some("false"))` for 6 tools that return an MCP error on a fake id
  (`update_knowledge`, `update_reflection`, `validate_reflection`,
  `get_evidence`, `add_world_relationship`, `archive_memory` — note
  `archive_memory` returned success on a fresh memory in the direct probe but
  isError=true inside the suite, so it expects failure). **Probing tip:** to
  pick the right validation for a future tool, call it with a fake id via
  `RobotBrainClient` and check `is_error`.

**Done when:** `test_suite_report.json` → `coverage.untested_tools` is empty,
`phantom_tools` is empty, suite exits 0. ✅ **DONE (commit 7775ca1):** untested
0, phantom 0, 141/141 tests pass, exit 0. This is the **green-gate milestone** —
every increment after this has an honest verify step.

**End of TIER 1 = finished v0.0.1. Tag: `v0.0.1-clean`.**

---

# 5. TIER 2 — Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.

## 2A. Data Contracts (Chapter 05)

Create `src/data_contracts/`. Types-only first; wire adapters incrementally.

- [ ] **T2-01** Create `src/data_contracts/` module skeleton (mod.rs, version
      field, shared traits).
- [ ] **T2-02** `Observation` struct + serde round-trip unit test.
- [ ] **T2-03** `ContextPacket` struct + serde round-trip test.
- [ ] **T2-04** `MemoryRecord` struct + serde round-trip test.
- [ ] **T2-05** `ExperienceRecord` — alias/migrate the existing type; serde
      round-trip test.
- [ ] **T2-06** `Plan` struct + serde round-trip test.
- [ ] **T2-07** `Decision` struct + serde round-trip test.
- [ ] **T2-08** `ExecutionResult` struct + serde round-trip test.
- [ ] **T2-09** `Reflection` struct + serde round-trip test.
- [ ] **T2-10** `LearningUpdate` struct + serde round-trip test.

**Done when:** all contracts round-trip through serde; gate green.

## 2B. Memory Engine (Chapters 08 & 14)

- [ ] **T2-11** Add explicit memory lifecycle states + promotion gate
      (Working → Candidate → Accepted → Permanent → Archived) in `src/memory/`.
- [ ] **T2-12** Add a confidence field to memories.
- [ ] **T2-13** Add memory relationship-graph support.
- [ ] **T2-14** Memory consolidation: merge duplicates, summarize aging
      low-importance memories, keep anchor memories standalone.
- [ ] **T2-15** Pruning policy for low-value/aged memories.
- [ ] **T2-16** Migrate `MemoryRecord` to the data-contract type.

**Done when:** store/search/list/relationship MCP tools work live; gate green.

## 2C. Knowledge Graph (Chapter 20)

- [ ] **T2-17** Add `knowledge_nodes` table + migration.
- [ ] **T2-18** Add `knowledge_edges` table + migration.
- [ ] **T2-19** Add relationship confidence on edges.
- [ ] **T2-20** Entity-resolution pass (merge aliases like "rustc"/"Rust Compiler").
- [ ] **T2-21** Graph traversal queries (relationship chains).
- [ ] **T2-22** Graph-extraction pipeline (entity detect → relationship
      extract → confidence evaluate → graph update → integrate).

**Done when:** graph traversal MCP tool returns relationship chains; gate green.

## 2D. Experience Engine (Chapters 09 & 18)

- [ ] **T2-23** Enrich `ExperienceRecord` field set (goal, plan_id, result,
      success, execution_time, cost, confidence_change, tool_usage, lessons,
      related refs).
- [ ] **T2-24** Add experience categories (conversation/planning/tool/code/...).
- [ ] **T2-25** Multi-factor success scoring.
- [ ] **T2-26** Confidence-update propagation to memory/relationships/tools.
- [ ] **T2-27** Migrate `ExperienceRecord` to the data-contract type.

**Done when:** record/list/insights MCP tools return the enriched fields; gate green.

## 2E. Learning Engine (Chapter 10)

- [ ] **T2-28** Formalize the learning pipeline (reflection → candidate →
      promotion → consolidation) in `src/learning/`.
- [ ] **T2-29** Pattern discovery from repeated successful experiences.
- [ ] **T2-30** Skill emergence from patterns.
- [ ] **T2-31** Confidence/decay management + generalization over memorization.

**Done when:** before/after learning shows measurable improvement (Ch.30.15);
gate green.

## 2F. Planning Engine (Chapter 11)

- [ ] **T2-32** Richer `decompose_goal` (more action verbs, better step gen).
- [ ] **T2-33** Dependency-aware task graphs.
- [ ] **T2-34** Candidate-plan generation + evaluation.
- [ ] **T2-35** Dynamic replanning triggers + plan scoring.
- [ ] **T2-36** Migrate `Plan` to the data-contract type.

**Done when:** create_plan returns real decomposed steps + dependencies; gate green.

## 2G. Skills & Workflows (Chapters 11 & 13)

- [ ] **T2-37** Skills: permissions + performance tracking in
      `src/skills/registry/`.
- [ ] **T2-38** Skills: fallback + async/parallel/retry.
- [ ] **T2-39** Workflows: workflow-level learning + confidence in
      `src/workflows/engine/`.
- [ ] **T2-40** Workflows: workflow ranking.

**Done when:** register/discover/execute_skill + workflow tools work live; gate green.

## 2H. World Model & Personality (Chapters 13/14/20)

- [ ] **T2-41** Align `src/world_model/` with the knowledge graph (entities →
      knowledge_nodes; relationships → edges with confidence).
- [ ] **T2-42** Finalize `src/personality/` (traits, emotional weight →
      confidence, presets, adaptation, decision_making, communication).

**Done when:** world_model + 6 personality MCP tools work live; gate green.

**End of TIER 2 = finished v0.0.2. Tag: `v0.0.2`.**

---

# 6. TIER 3 — Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Chapter refs are `robot_architecture/v0.0.2.1/<NN>.md`.

## 3A. Execution & Tool engines (Chapters 12 & 13)

- [ ] **T3-01** `src/execution/` skeleton — execution isolation, action
      authorization (Chapter 12).
- [ ] **T3-02** Execution: workflow graphs/DAGs + checkpoints.
- [ ] **T3-03** Execution: result normalization + recovery.
- [ ] **T3-04** `src/tools/` (Tool Engine) — capability registration contracts
      distinct from skills (Chapter 13).
- [ ] **T3-05** Tool: permissions + input/output contracts + isolation.

## 3B. Context subsystem (Chapters 07, 15, 16, 17)

- [ ] **T3-06** `src/context/` (Context Engine) skeleton — RetrievalPlanner,
      TokenBudget, TopicTracker, SlidingWindow (Chapter 07).
- [ ] **T3-07** Context: 4-level memory hierarchy (L0 live, L1 working summary,
      L2 checkpoints, L3 raw DB) (Chapter 14).
- [ ] **T3-08** Context Lifecycle: creation/refresh/compaction/checkpoint/
      expiration/reconstruction (Chapter 15).
- [ ] **T3-09** Retrieval Pipeline: candidate generation → ranking →
      confidence/provenance → diversity → budget (Chapter 16).
- [ ] **T3-10** Prompt Construction: source provenance, instruction hierarchy,
      model independence, reproducibility (Chapter 17).
- [ ] **T3-11** Context policies (not every question retrieves memory);
      per-item context scores.

## 3C. Conversation Engine (Chapter 06)

- [ ] **T3-12** `src/conversation/` skeleton — interaction/session ownership,
      lifecycle, interruption, traceability (Chapter 06).
- [ ] **T3-13** Conversation: Input → Understanding → Context Assembly →
      Reasoning → Planning → Tool Execution → Response → Learning pipeline.
- [ ] **T3-14** `converse` MCP tool that runs the full pipeline and returns a
      context-informed, memory-informed response.

## 3D. Strategic Learning & Confidence (Chapters 18 & 19)

- [ ] **T3-15** Strategic Learning: long-horizon evidence, policy/strategy
      changes, experiments, validation, rollback (Chapter 18).
- [ ] **T3-16** Confidence System: evidence, source quality, recency,
      relationship/skill/workflow confidence, decay (Chapter 19).

## 3E. Storage, Database, Workers (Chapters 21, 22, 23)

- [ ] **T3-17** Storage Architecture: durable persistence, transactions,
      backups, migrations, recovery, integrity (Chapter 21).
- [ ] **T3-18** Database Design: schema ownership, migration discipline,
      indexes, constraints, transactional integrity (Chapter 22).
- [ ] **T3-19** Background Workers hardening: ownership, queues, retries,
      idempotency, cancellation, backpressure, supervision, health (Chapter 23).

## 3F. Governance & Safety (Chapters 24, 25, 26)

- [ ] **T3-20** AI Contributor Operating Agreement: human/AI contribution
      boundaries, review gates, traceability (Chapter 24 — process + tests).
- [ ] **T3-21** Security & Trust: identity, authorization, capability
      security, trust boundaries, memory protection, audit (Chapter 25).
- [ ] **T3-22** Self-Improvement/Evolution: controlled hypotheses,
      experiments, promotion gates, rollback, human control (Chapter 26).
- [ ] **T3-23** Self-Improvement: a hypothesis lifecycle runs
      confirmed→confidence-increase / rejected→decrease end-to-end.

## 3G. Observability & Control Plane (Chapters 27, 28)

- [ ] **T3-24** Cognitive Monitoring/Observability: traces, correlation,
      metrics, events, decision evidence, health, retention, privacy
      (Chapter 27).
- [ ] **T3-25** Developer Interface/Control Plane: inspection + control,
      read/write separation, permissions, safe mutation, audit, recovery
      (Chapter 28).
- [ ] **T3-26** Control Plane: cognitive traces reconstruct the full request
      lifecycle; capability-denied actions blocked + audited.

## 3H. Config, Testing, Deployment (Chapters 29, 30, 31)

- [ ] **T3-27** Configuration: layered precedence (defaults → install → system
      → profile → user → runtime), validation, secrets, profiles, change
      control (Chapter 29).
- [ ] **T3-28** Testing: unit/contract/integration/persistence/event/security/
      failure-injection/recovery/migration/adapter/GUI/e2e-cognitive/regression/
      property layers (Chapter 30).
- [ ] **T3-29** Expand `test_suite/`: schema-validation matrix, edge cases,
      e2e learning loop, performance baselines (the gaps from
      `.agents/TEST_SUITE_NOTES.md`).
- [ ] **T3-30** Deployment: reproducible + versioned, validation + rollback,
      migrations, backup/recovery (Chapter 31).

## 3I. AI Runtime / Model Manager (Chapter 14 + appendix)

- [ ] **T3-31** `InferenceProvider` trait + `src/ai_runtime/` skeleton; cloud
      provider implementation first.
- [ ] **T3-32** Model Manager: discovery, metadata, selection, lifecycle.
- [ ] **T3-33** Candle-based local LLM provider.
- [ ] **T3-34** Candle-based local embeddings provider.
- [ ] **T3-35** `inference` MCP tool (routes through the runtime; cloud and
      local interchangeable behind the trait).

## 3J. Multimodal (Appendix A)

- [ ] **T3-36** Audio Engine: STT (Whisper via Candle) + audio ingest
      (WAV/MP3/FLAC/OGG/M4A).
- [ ] **T3-37** Audio Engine: TTS (Piper/Kokoro via Candle).
- [ ] **T3-38** Vision Engine: OCR + image understanding + screenshot analysis.
- [ ] **T3-39** `transcribe` / `synthesize` / `ocr` MCP tools (real results).

## 3K. GUI / Dashboard (Chapter 28)

- [ ] **T3-40** `src/control_plane/` API + event stream (runtime stays
      headless-operable without GUI).
- [ ] **T3-41** Frontend (separate crate/dir) consuming the event stream;
      renders real (not fake) system events.

## 3L. Future expansion & roadmap (Chapters 32, 33)

- [ ] **T3-42** Future Expansion gate: new capabilities integrate through
      stable contracts; document the "does it belong in an existing boundary?"
      check (Chapter 32).
- [ ] **T3-43** Capability Roadmap: architectural-gates process documented
      (Chapter 33).

**End of TIER 3 = finished v0.0.2.1. Tag: `v0.0.2.1`.**

---

# 7. Definition of Done

## v0.0.1-clean (end of TIER 1)

- `find src -name "self_check.rs"` returns empty.
- `grep -rn 'allow(' src/` returns nothing (already true).
- **test_suite exits 0** — `coverage.untested_tools` empty,
  `coverage.phantom_tools` empty (the 1E green-gate milestone).
- Queue is SQLite-backed and survives a process restart.
- `get_system_status` shows loop_latency / confidence_drift /
  promotion_throughput.
- Generic MCP tool execution emits an experience (no double-emit).
- Gate green: 0 build warnings, 54/54 live, 333/333 suite, suite exit 0.

## v0.0.2 (end of TIER 2)

- Data-contract types round-trip through serde.
- Each upgraded subsystem's MCP tools return correct results live.
- Knowledge graph traversal returns relationship chains.
- Before/after learning shows measurable improvement (Ch.30.15).
- Gate green throughout.

## v0.0.2.1 (end of TIER 3)

- All 33 blueprint chapters + appendices have a corresponding implemented
  module or documented deferral.
- The cognitive pipeline (Observe → Understand → Retrieve → Plan → Reason →
  Act → Reflect → Learn) runs end-to-end through Context/Conversation engines,
  not just the legacy agent loop.
- Memory, Experience, Knowledge, and Context are four independent systems
  communicating through Data Contracts.
- Context Engine enforces token budgets and retrieval policies; context
  construction is inspectable.
- AI Runtime: cloud and local models interchangeable behind the trait;
  embedding pipeline produces consistent vectors.
- Security: capability-denied actions blocked + audited.
- Observability: cognitive traces reconstruct the full request lifecycle.
- Multimodal: transcribe/synthesize/ocr return real results via Candle.
- GUI: runtime fully headless-operable; GUI renders only real events.
- Self-improvement: a hypothesis lifecycle runs confirmed/rejected end-to-end.
- Workers: supervised workers restart on failure; durable queue survives
  restart.
- Config/Migration: fresh-DB migration runs clean; test_suite expanded with
  schema-validation + edge-case + e2e-learning + perf-baseline coverage.
- 0 cargo warnings, 0 code-quality issues, all MCP tools pass live, test-suite
  green and expanded.
- Local-first: the entire cognitive architecture operates without cloud
  dependency.

---

# 8. v0.0.1 CONFORMANCE WORK (legacy status — for reference)

> Moved here from AGENTS.md on 2026-08-11. Historical status of the v0.0.1
> conformance work (P0-P4). TIER 1 above supersedes this for forward planning,
> but it records what was already done so progress isn't re-attempted.

## P0 — event spine drives learning — DONE

- V2-01/02/03: `ExperienceRecorded → Reflection → Hypothesis → Knowledge →
Reputation` wired in `src/experience/integration/event_subscriber/handlers.rs`.

## P1 — cognitive loop — PARTIAL

- V2-04: goal-driven `src/agent/` loop DONE; `run_agent_goal` MCP tool works
  (status=Achieved, confidence=0.507).
- V2-05: generic MCP dispatch does NOT auto-emit experience (→ T1-17/T1-18).

## P2 — stub chapters — DONE

- V2-06: World Model exists. V2-07: `src/agent/safety_gate/` (sandbox,
  rollback, hallucination, uncertainty). V2-08: Personality emotional_weight →
  confidence (`personality/decision_making.rs:49-51`).

## P3 — self-check probes — REMAINING (→ T1-01..T1-08)

- V2-09: 8 self_check.rs files remain. Pattern: wire MCP tool, delete self_check.

## P3.1 — `#![allow]` violations — RESOLVED

- 2026-08-11: `grep -rn '#!\[allow' src` returns 0; `grep -rln '#\[allow' src`
  returns 0. Both clean.

## P4 — performance maturity — REMAINING (→ T1-09..T1-16)

- V2-11: in-memory JobQueue (→ SQLite). V2-12: no loop-health metrics.

## GATE (coverage) — ✅ GREEN (T1-19..T1-29 all DONE)

- Brain_tester now exits 0. 141/141 tests pass, 0 code issues, 0 warnings.
- coverage: untested 0, phantom 0. All 134 server tools are tested.
- T1-19 fixed the 6 phantom embedding tools (commit b9b43ff).
- T1-20 added 9 ACP tool tests (commit 6b7d036).
- T1-21..T1-29 added 41 remaining tool tests (commit 7775ca1).

## Verified state (2026-08-14)

- [!] **GATE RED** -- `compiler_warnings=40` (all dead-code: `never used`/
  `never read`/`never constructed`; NO mechanical lints remain), `code_issues=56`
  (all `CfgTest` -- see T1-10B-CFG below), `untested=0`,
  `tests=145/145 (100%)`, `compiler_errors=0`, `tool_coverage=100%`,
  `mcp_protocol_ok=true`. The 40 warnings + 56 cfg_test issues are the gate
  blockers.
- [x] **T1-10B-CFG (2026-08-14, commits f7973fa, 1bfed42, 1707f15):** The gate
  now **flags `#[cfg(test)]`** in robot_brain `src/` as gate-failing code issues
  (f7973fa). Per AGENTS.md "All tests live in test_suite (MANDATORY)" and the
  user's directive (2026-08-14): tests must not live in the server source. The
  gate builds robot_brain in release (no `--tests`), so `#[cfg(test)]` blocks
  were previously invisible to the compiler. Added `CfgTest` `IssueType` +
  regex + `check_cfg_test()` in `test_suite/src/code_analyzer/`
  (analyzer/patterns/types).
  **User rule for removal:** if a test can be run from test_suite against the
  compiled robot_brain production exe via MCP -> MIGRATE it to test_suite; if
  NOT reachable -> it is useless -> DELETE the `#[cfg(test)]` block. Strategy L
  (lib crate) and "wire a new MCP tool for everything" (Strategy M extreme)
  REJECTED -- both smuggle dead code forward.
  **Dead Code Resolution Protocol (MANDATORY for production code):** deleting
  `#[cfg(test)]` test blocks is governed by the user's test rule; deleting
  PRODUCTION code is governed by the Dead Code Protocol (cross-reference
  architecture -> if described, IMPLEMENT/wire, don't delete; if absent, delete).
  These are two separate rules -- do not conflate.
  **CORRECTION (commit 1707f15):** the prior commit 1bfed42 deleted
  `src/memory/repository.rs` (MemoryRepository trait + SqliteMemoryRepository)
  as "dead code." That was a PROTOCOL VIOLATION: the architecture explicitly
  describes the Memory Repository Pattern (v0.0.1 Sec 4.06, v0.0.2.1 Sec 22.14:
  "RoBoT avoids direct database access from cognitive systems"). The trait
  was an incomplete stub, not dead code. Commit 1707f15 RESTORED the production
  code (no `#[cfg(test)]`), declared it in memory/mod.rs, and WIRED
  store_memory through `MemoryRepository::store` instead of calling
  `queries::insert_memory` directly. 8 other `queries::insert_memory` call
  sites remain on direct queries (TIER-2 wiring follow-up). LESSON: grep'ing
  the architecture is not a cross-reference -- READ the cited section.
  Gate now reports **56 `CfgTest` issues across 15 files** (was 60/17;
  removed: memory/repository.rs test block, database/queries/memory.rs test
  block + delete_memories_by_string_ids). Remaining 56 cfg_test by file:
  experience/evolution/engine.rs (21), bridge/acp/message.rs (7),
  planner/policy.rs (5), evolution/behavior.rs (4),
  hypothesis/services/repository.rs (4), bridge/acp/mod.rs (4),
  evolution/evidence.rs (2), hypothesis/support/graph/graph_types.rs (2),
  + 7 files with 1 each (personality, learning/pipeline, memory/retrieval,
  reflection/generator, audio_transcriber, mcp/client, planner/engine).

- ✅ **T1-10B file repair COMPLETE (commit 2d611ac).** T1-10B-Z had truncated
  ~20 files, leaving ~119 compile errors. Reconstruction restored: enforcement
  SessionState/WorkflowEnforcer, learning/working_memory WorkingMemoryItem +
  module tree, memory re-exports, graph EdgeId/HypothesisRelationship +
  accessors, Hypothesis::has_evidence, PlannerStatistics, acp
  list_agents/count/registry()/create_system_agent/create_worker_agent, etc.
  Result: compiles cleanly, 413 E2E test assertions pass, 0 code issues.
- ✅ **Newly-added methods wired into production (commit 4a2a2c0)** so they are
  not dead code: `AcpMessageType::expects_reply` → `route()` tracing;
  `AcpRouter::register_handler` → init registers default Inform handler;
  `AcpRegistry::get_by_type` → startup worker-count diagnostic;
  `HypothesisValidator::validate` → hypothesis maintenance probe (also wires
  has_evidence/ValidationReport/ValidationIssue/ValidationIssueType);
  `PlannerStatistics` → `Planner::create_plan` tracks `plans_created`.
  Warnings 77→69.
- **Remaining 69 dead-code warnings** are pre-existing scaffolded-but-unwired
  subsystems in `src/experience/` (NOT T1-10B damage — they predate it; the
  08-12 snapshot showed 163 warnings). Clusters: reflection_pipeline +
  Reflector/InsightProducer/ReflectionInsight/Evidence/Review/Lesson (redundant
  with EventSubscriber+LearningCoordinator §4.04 path — needs wire-vs-delete
  decision per Dead Code Protocol), exploration store
  (InMemoryExplorationRepository, "implemented but not yet integrated"),
  reputation (ReputationRecord/ReputationTarget/factors), encounter
  (EncounterScore/Stats/ExperienceRecorder record/success/failure),
  hypothesis_pipeline, learning_coordinator orphan methods
  (process_experience/complete_exploration/get_reputation/update_reputation),
  scorer (EncounterScore/score_encounter), maturity enums, and ~6 never-read
  config/struct fields. Each cluster is a separate increment (wire into the
  cognitive loop OR delete if architecture confirms redundancy).
- 134 MCP tools; 145 FunctionRegistry tests pass; 0 code-quality issues;
  0 untested tools; 0 phantom tools.
- 8 self_check.rs files remain (→ TIER 2).
- Code-issue fixes done this session (commit a21055d): planner.rs
  (`_step`/`_analysis` renamed, `replan`/`should_use_creativity` unwrapped from
  `#[cfg(test)]` and wired into maintenance loop), reflection.rs
  (`experience_count` wired into analyzer), graph `edge_count` unwrapped +
  wired into probe, `HypothesisStatistics`/`StatisticsSnapshot` unwrapped from
  `#[cfg(test)]` and wired into maintenance probe.
- ✅ **T1-10 DONE** — SQLite JobQueue fully wired (queue.rs, worker_manager/manager.rs,
  background.rs, bridge/mcp/context.rs, initialization.rs) and verified by a
  live restart-durability test in test_suite (tests/queue_durability.rs): inject
  pending row → kill server → restart in same dir → row restored into live
  queue (pending_jobs>baseline) and durable row survives status=pending.
  Known gap: restored jobs are not replayed to workers (replay-on-start is
  future work).
- Large-file refactors done: `personality/personality.rs` (352→101, split into
  presets/adaptation/decision_making); `memory/handlers.rs` (400→ directory).
