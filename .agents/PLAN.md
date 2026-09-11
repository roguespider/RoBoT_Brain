# Research Engine Stub Tasks — All Complete

## Completed Tasks (per protocol)
- ✅ S1: `check_internal_sources` in decision.rs
- ✅ S2: `trigger_research_on_failure` in decision.rs
- ✅ S3: `record_research` in experience/mod.rs
- ✅ S4: `promote_research` in memory/mod.rs
- ✅ S5: `BraveProvider::search` in brave.rs
- ✅ S6: `record_failure` wired in failover.rs
- ✅ S7: 9-tier cascade wired in loop_runner.rs (between retrieval and action selection)
- ✅ S8: Removed `_use_tier_result` stub
- ✅ S9: Removed `let _ =` forbidden patterns
- ✅ S10: `promote_research_findings` in knowledge/mod.rs - gates confidence >= 0.7, creates KnowledgeItem with source="research:<url>", wired into loop_runner cascade
- ✅ S11: `test_research.py` - full 13-step research flow verification per architecture §R16

## Build Status
- `cargo check --release`: 0 errors
- Warnings: 57 (all dead-code from unregistered research tools)
- Forbidden patterns: 0

---

# T2-- Reach v0.0.2 (upgrade existing systems)

> Goal: every existing subsystem conforms to its v0.0.2 chapter and
> communicates through Data Contracts. End state = finished v0.0.2.
> Detailed task list: [`.agents/t2_PLAN.md`](t2_PLAN.md).

---

# T3-- Reach v0.0.2.1 (add missing subsystems)

> Goal: every v0.0.2.1 chapter (01-33) has a corresponding implemented module or
> documented deferral. Build in dependency order; AI Runtime/Multimodal/GUI last.
> Detailed task list: [`.agents/t3_PLAN.md`](t3_PLAN.md).

---
