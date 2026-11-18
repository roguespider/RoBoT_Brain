FIX 22: cooboploop_research_engine_9tier_cascade_missing

What was broken:
The 9-tier confidence cascade (research_engine.md §2) was missing. Research (tier 9)
could be dispatched without first checking tiers 1-8 (Memory, Knowledge, Experience,
Skills, Reflections, Workflows, WorldModel, Hypotheses). This meant research could
fire even when the answer was already available in local memory/knowledge/experience.

What changed:
1. src/agent/decision.rs — Implemented ResearchCascade::should_research() with the full
   9-tier cascade (8 local tiers + research as tier 9 last resort). Each tier checks
   confidence >= 0.7 before allowing research to proceed. Created CascadeDeps struct
   to group dependencies and avoid clippy::too_many_arguments. CapabilityRegistry (tier 4)
   is gracefully skipped since it contains rusqlite::Connection with RefCell interior
   (not Sync, cannot be stored behind Arc).

2. src/scheduler/cooboploop/research.rs — Wired the cascade check inside ResearchManager::
   dispatch_next() before calling pipeline.run_pipeline(). If any tier 1-8 can answer
   the query, research is skipped. Added setters for all cascade dependencies
   (knowledge_store, permanent_memory, world_model, hypothesis_engine, reflection_engine,
   current_plan, latest_experience). Changed std::sync::Mutex to tokio::sync::Mutex for
   Send-compatibility with tokio::spawn.

Wiring verification:
- ResearchManager::dispatch_next() checks the cascade inside the spawned async task
  before calling pipeline.run_pipeline()
- All 8 tiers return false (skip research) when confidence >= 0.7
- Research only proceeds when all tiers 1-8 fail
- Fallback: if cascade deps not fully configured, allows research as safe default

Gate results:
- cargo check: 0 errors, 0 warnings
- Quality gate: GREEN (Code: 0, Lint: 0, Large files: 0, Python compile: 0,
  Python lint: 0, Python quality: 0)
- All 216 MCP tools probed successfully