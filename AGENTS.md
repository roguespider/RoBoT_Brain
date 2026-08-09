# RoBoT Brain - Agent Memory

## Prerequisites (install FIRST, before anything else)

Before building or working on this project, the following must be installed.
Do this as the very first step — the build will fail without them.

1. **Rust toolchain** (edition 2024 requires Rust 1.85+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
   . "$HOME/.cargo/env"
   ```
2. **System packages** (needed by native Rust deps: `openssl-sys`, `rusqlite`/bundled SQLite, etc.):
   ```bash
   sudo apt-get update
   sudo apt-get install -y pkg-config libssl-dev
   ```
   - `pkg-config` — required by `openssl-sys`'s build script to locate OpenSSL.
   - `libssl-dev` — OpenSSL headers + dev libs (the runtime `.so` alone is not enough).
3. **Cargo config** — `.cargo/config` already pins a Linux linker flag for libsqlite3; no action needed.

If a build fails with `Could not find directory of OpenSSL installation` or `Unable to locate package`, it means step 2 was skipped or apt lists are stale (`sudo apt-get update` first).

## Project Structure

This is a Rust workspace with **two separate, independent programs**:

| Component | Location | Binary | Purpose |
|-----------|----------|--------|---------|
| **robot_brain** | `/` (root) | `robot_brain` | Main MCP server (AI agent with tool plugins) |
| **test_suite** | `/test_suite/` | `test_suite` | End-to-end testing suite (tests robot_brain via MCP protocol) |

These programs **do NOT depend on each other's source code**. The test suite tests robot_brain by spawning it as a subprocess via MCP protocol.

## Build Commands

```bash
# Build main binary
cargo build --release -p robot_brain

# Build test binary  
cargo build --release -p test_suite

# Run test suite (outputs to test_suite/test_suite_output.txt)
./target/release/test_suite
```

## Post-Compile: Connect to robot_brain MCP/ACP

**IMPORTANT (User Requirement):** After compiling `robot_brain`, the AI agent MUST connect to the running robot_brain MCP/ACP server directly to test it. Do not rely solely on the test_suite — connect yourself as a client.

Steps after a successful `cargo build --release -p robot_brain`:
1. Invoke the `robot-brain` skill (`.agents/skills/robot-brain/skill.md`)
2. Connect to the compiled `robot_brain` binary via MCP protocol
3. Test key tools to verify what is working and what is not:
   - **MCP tools**: `store_memory`, `search_memory`, `list_memories`, `create_plan`, `list_plans`, `create_workflow`, `start_workflow`, `query_knowledge`, `record_experience`
   - **ACP tools**: `route_acp_message`, `register_agent`, `list_agents`
   - **Discovery**: `list_tools` to confirm all ~92 tools are available
4. Report which tools work and which fail

This direct testing makes it easier to identify working vs. broken functionality immediately after compilation, rather than only seeing aggregate pass/fail from the test suite.

## Handling Unused Type Warnings

Many types (like `SimpleAgent`, `AcpCapability`, ACP message builders) are defined for testing/future use but unused in production. When fixing lint warnings:

1. **Wrap test-only types in `#[cfg(test)]` modules** - keeps them available for tests without affecting production
2. **Move unused re-exports to test modules** - don't expose unused types in public API
3. **Keep production trait minimal** - implement only what's actually used (e.g., `AcpAgent` trait only needs `id()` and `handle()` methods)

This pattern reduced warnings from 480+ to ~359.

## Code Style Conventions

When modifying or extending this codebase, you **MUST** adhere to these strict constraints. Violations are critical errors that must be actively repaired.

### Strict Rust Coding Standards

1. **NO Panics or Crashes**
   - Strictly forbidden: `.unwrap()`, `.expect()`, `panic!()`, `assert!()`, `unreachable!()`
   - Use idiomatic Rust error handling: `?` operator, `match`, `if let`, `.unwrap_or_else()`, `.unwrap_or()`
   - Every `Result` and `Option` must be handled explicitly

2. **NO Placeholders or Stubs**
   - Strictly forbidden: `todo!()`, `unimplemented!()`, `unreachable!()`
   - All code blocks must be 100% complete and production-ready
   - No empty function bodies or skeleton implementations

3. **NO Code Deletion**
   - Never delete problematic code blocks or mark them as dead code to bypass fixes
   - If code is unused, follow the Dead Code Resolution Protocol below

4. **NO Compiler-Silencing Attributes**
   - Strictly forbidden: `#[allow(dead_code)]`, `#[allow(unused_variables)]`, `#[allow(unused_imports)]`, `#[allow(unused_must_use)]`, or any other `#[allow(*)]` flags
   - Fix the underlying issue instead of hiding warnings

5. **NO Ignored Variables**
   - Strictly forbidden: `let _x = ...`, `|_| ...`, `let _ = ...`
   - Every variable and result must be meaningfully utilized
   - If a value is truly unused, restructure the code to avoid binding it

6. **Name Conflicts**
   - If renaming types, use descriptive new names (e.g., `LearningPattern` instead of `Pattern`)

### Dead Code Resolution Protocol

When encountering unused, unreachable, or seemingly dead code:

1. **Cross-reference architecture**: Check `RoBoT_Brain/robot_architecture/` directory for documentation
2. **If documentation describes the feature**: The code is an incomplete stub
   - You MUST fully implement and complete the missing logic
   - Production-ready status is required
3. **If documentation confirms deprecated/absent**: The code can be safely deleted
   - Clean up all associated imports and references
   - Verify no breaking dependencies

### Enforcement

The test suite enforces these rules:
- Any `todo!()`, `unimplemented!()`, `unreachable!()` = **Test Failure**
- Any `#[allow(*)]` attribute = **Test Failure**
- Any `.unwrap()` or `.expect()` on non-test code = **Test Failure**
- Any `_variable` pattern for ignored values = **Test Failure**

## Large File Refactoring Pattern

When splitting large `.rs` files (>320 lines) into modules:

1. Create a directory with the same name as the file (e.g., `engine.rs` → `engine/`)
2. Move original file to `mod.rs` inside the new directory
3. Extract logical groups to separate files (config, types, helpers, etc.)
4. Register modules in `mod.rs` with `pub mod module_name;`
5. Re-export public types for backward compatibility

## Large Files (Needing Refactor)

Files over 320 lines that could benefit from modular structure:
- `src/bridge/acp.rs` (950 lines)
- `src/skills/registry.rs` (931 lines)
- `src/database/queries.rs` (890 lines)
- `src/bridge/app.rs` (870 lines)
- `src/bridge/tools/memory/mod.rs` (803 lines)
- `src/bridge/tools/exploration/handlers.rs` (791 lines)
- `src/personality/mod.rs` (614 lines)

### Already Refactored
- `src/experience/integration/learning_coordinator/` (1519 total → config.rs, results.rs, entry.rs, exploration.rs, hypothesis.rs, knowledge.rs, reinforcement.rs, reputation.rs, generalization.rs, mod.rs [274 lines])
- `src/skills/registry/` (983 total → types.rs, skill.rs, registry.rs, context.rs, result.rs, metrics.rs, executor.rs, mod.rs)
- `src/database/queries/` (934 total → helpers.rs, memory.rs, scheduled_tasks.rs, observations.rs, experiences.rs, embeddings.rs, relationships.rs, tests.rs, mod.rs)
- `src/bridge/app/` (944 total → state.rs, initialization.rs, scheduler.rs, personality.rs, acp.rs, mod.rs)
- `src/planner/engine/` (836 lines → planner.rs, types.rs, actions.rs, replanning.rs, mod.rs)
- `test_suite/src/code_analyzer/` (1050 lines → types.rs, patterns.rs, analyzer.rs, lint.rs, mod.rs)
- `src/bridge/acp/` (950 lines → message.rs, error.rs, channel.rs, agent.rs, registry.rs, router.rs, builder.rs, mod.rs)
- `test_suite/src/tests/rmcp/` (NEW: 650 lines → mod.rs, protocol.rs, tools.rs, sessions.rs)
- `test_suite/src/tests/acp/` (NEW: 750 lines → mod.rs, registry.rs, router.rs, agents.rs, messages.rs)
- `test_suite/src/tests/agent_simulation/` (NEW: 440 lines → mod.rs, workflows.rs, memory_agent.rs, decision_making.rs)

---

## OpenHands MCP Integration

RoBoT Brain can be used as an MCP server by **OpenHands agents** to access memory, knowledge, planning, and learning tools.

### Quick Start

```python
from openhands.sdk import LLM, Agent, Conversation
from openhands.sdk.tool import Tool
from openhands.tools.terminal import TerminalTool

# Configure MCP connection
mcp_config = {
    "mcpServers": {
        "robot_brain": {
            "command": "cargo run --release -p robot_brain",
        }
    }
}

# Create agent with robot_brain tools
agent = Agent(
    llm=LLM(model="anthropic/claude-sonnet-4-5-20250929", api_key="..."),
    tools=[Tool(name=TerminalTool.name)],
    mcp_config=mcp_config,
)

# Run conversation
conversation = Conversation(agent=agent, workspace=".")
conversation.send_message("Search memory for Rust patterns")
conversation.run()
```

### Complete Example

See `examples/robot_brain_agent.py` for a full-featured script:

```bash
export LLM_API_KEY="your-key"
python examples/robot_brain_agent.py -m "Search memory for architecture patterns"
```

### Available Tools (~89 total)

| Category | Key Tools |
|----------|-----------|
| **Memory** | `store_memory`, `search_memory`, `get_memory`, `list_memories` |
| **Knowledge** | `query_knowledge`, `add_knowledge`, `global_search` |
| **Experience** | `record_experience`, `list_experiences`, `get_insights` |
| **Planning** | `create_plan`, `get_plan`, `list_plans` |
| **Workflows** | `create_workflow`, `start_workflow`, `list_workflows` |
| **Hypothesis** | `create_hypothesis`, `add_evidence`, `evaluate_hypothesis` |
| **Exploration** | `start_exploration`, `evaluate_exploration_hypothesis` |
| **Skills** | `register_skill`, `discover_skill`, `execute_skill` |
| **ACP** | `route_acp_message`, `register_agent`, `list_agents` |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ROBOT_BRAIN_PATH` | Path to robot_brain binary | Auto-detected |
| `LLM_API_KEY` | API key for LLM | Required |
| `LLM_MODEL` | Model name | `anthropic/claude-sonnet-4-5-20250929` |

### Loading the Skill

This repo includes an OpenHands skill at `.agents/skills/robot-brain/skill.md` that documents all available tools and usage patterns. When working in an OpenHands environment, this skill is automatically loaded and provides context for using robot_brain tools.

### Tool Filtering

If you only want specific tools, use regex filtering:

```python
agent = Agent(
    ...
    filter_tools_regex="^(search_memory|store_memory|query_knowledge)$",
)
```

This allows OpenHands to use robot_brain alongside other tools, focusing on specific capabilities as needed.

---

## Roadmap to v2.0 (Architecture Conformance Work)

This section records the gap between the current `robot_brain` implementation and
`robot_architecture/v0.0.1/ARCHITECTURE.md`, derived from a wiring audit of the
live runtime. It is the work needed before the project can be called v2.0.

The architecture describes RoBoT as a **continuously self-improving cognitive
loop**: `Observe → Understand → Predict → Act → Learn → Improve`. The current
build realizes the *structure* (every subsystem module exists) and produces real
learning *when fed*, but the loop is **reactive**, not autonomous: it advances
only on MCP tool calls and scheduler ticks. The tasks below close that gap.

Tasks are ordered by impact. Each references the architecture chapter it
satisfies and the concrete file(s) to change.

### P0 — Make the §4.04 event spine actually drive learning

The centerpiece event chain is currently only partly wired:

```
ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation
```

The `WorkerManager` observers (scorer, reputation, hypothesis, metrics) DO real
work on events. But `EventSubscriber` — the component the doc calls "the main
coordinator that wires events to learning subsystems" — is currently a
metrics/reputation relay: `on_experience_recorded` re-publishes an event and
increments a counter instead of invoking the learning pipeline.

- [ ] **TASK-V2-01: Wire `EventSubscriber.on_experience_recorded` to call
  `LearningCoordinator.process_experience_full`.**
  File: `src/experience/integration/event_subscriber/handlers.rs`.
  `EventSubscriber` already holds `reflection_engine`, `hypothesis_engine`, and
  `knowledge_store` (see `mod.rs`), but NOT a `LearningCoordinator`. Either give
  it an `Arc<LearningCoordinator>` (preferred, matches §4.04 single-driver
  intent) or compose the same engines inline. The handler must run the full
  Score → Reflect → Hypothesize → Knowledge-promote path per event, not just
  re-emit. Satisfies §4.04, §5.6.
- [ ] **TASK-V2-02: Remove the redundant event echo.**
  `ExperienceCoordinator::record_experience` (in `src/experience/coordinator.rs`)
  publishes `ExperienceRecorded` on receipt of `ExperienceRecorded`, creating an
  echo loop that is only safe because handlers are idempotent. After V2-01, the
  subscriber should consume the event once and drive learning; the coordinator
  should publish `Scored`/`ExperienceRecorded` exactly once at the *input* edge
  (MCP tool / recorder), not re-echo on receipt.
- [ ] **TASK-V2-03: Make `on_reflection_completed`, `on_hypothesis_generated`,
  `on_hypothesis_validated`, `on_knowledge_updated` actually advance the next
  stage** instead of only incrementing metrics. Today these handlers mostly
  `tracing::debug!` + bump counters. Each should invoke the next subsystem in
  the chain (reflection→hypothesis, hypothesis→exploration, validation→knowledge
  update, knowledge→reputation adjust). Satisfies §4.04, §5.10.

### P1 — Close the cognitive loop (Act → New Experience)

The architecture's loop ends with `Act → New Experience → Learn`. RoBoT has no
actuators — "Action" today means "return a tool result to the MCP client." There
is no autonomous agent that decides to act and generates its own experiences.

- [ ] **TASK-V2-04: Add a goal-driven agent loop.** A component that, given a
  goal, uses the Planner → Memory retrieval → Knowledge retrieval → Experience
  retrieval → confidence evaluation → action selection path (§5.7 Decision
  Flow), then records the outcome as a new experience (closing the loop).
  This is the single biggest missing piece to realize the vision chapter. Likely
  a new `src/agent/` module driving the existing planner/workflow engines.
- [ ] **TASK-V2-05: Record outcomes of MCP tool executions as experiences
  automatically.** Today `record_experience` is a manually-invoked tool. Every
  tool execution that produces an outcome should emit an `ExperienceRecorded`
  event so the learning loop advances without the caller explicitly recording.
  Satisfies §2.04 ("Everything Important Becomes an Experience") and the loop
  closure in §5.8.

### P2 — Implement the stub architecture chapters

These chapters are placeholder bullet lists in `ARCHITECTURE.md` itself and have
no/minimal implementation:

- [ ] **TASK-V2-06: World Model (Chapter 14).** Objects, places, people, events,
  time, goals, relationships, resources. "Memory stores facts. World Model
  stores understanding." Currently no `src/world_model/` module. This is called
  out in the doc as "one of the biggest missing pieces."
- [ ] **TASK-V2-07: Safety layer (Chapter 16).** Sandboxing, permission checks,
  confidence thresholds for acting, rollback, hallucination handling,
  uncertainty reporting. Currently no safety gating on actions. Required before
  an autonomous loop (V2-04) is safe to run.
- [ ] **TASK-V2-08: Expand Personality beyond style (Chapter 13).** Current
  `src/personality/` has traits + communication style + a basic `decide()`. The
  doc wants speaking style, preferences, humor, curiosity, emotional weighting,
  interaction policies. Emotional weighting should feed confidence/decision
  scoring, not just text formatting.

### P3 — Reduce reliance on self-check probes

Much of the code surface is currently kept live only by startup self-check
probes (the lint-cleanup work), not by real runtime traffic. This signals the
production wiring is thinner than the code.

- [ ] **TASK-V2-09: Audit each `self_check.rs` and either (a) remove it because
  the path is now exercised by real wiring from P0/P1, or (b) convert it to a
  real integration test in `test_suite/`.** A self-check that exists only to
  silence dead-code warnings is a smell; the goal is that every public API is
  exercised by genuine runtime or test-suite traffic.
- [ ] **TASK-V2-10: Finish the remaining ~10 compiler warnings** (down from 118)
  across `bridge/mcp/client`, `bridge/mcp/handlers`, `bridge/mcp/context`,
  `bridge/app/state`, `bridge/tools/ingestor`, `experience/integration/
  hypothesis_pipeline`. Apply the Dead Code Resolution Protocol: implement if
  the architecture describes the feature, delete if deprecated.

### P4 — Performance & operational maturity (Chapter 17)

- [ ] **TASK-V2-11: Document and enforce threading/queue/async/cache/indexing
  strategy.** The doc lists these as future work; before v2 they need explicit
  design, especially the broadcast channel lag handling (the subscriber already
  logs `Lagged` events) and the in-memory `JobQueue` → SQLite-backed queue
  migration noted in `initialization.rs`.
- [ ] **TASK-V2-12: Add metrics/observability for the learning loop itself**
  (not just experience counts): reflection→hypothesis→knowledge promotion
  throughput, loop latency, confidence drift. The `MetricsCollector` exists but
  mostly tracks counters, not loop health.

### Definition of Done for v2.0

- The §4.04 event chain runs end-to-end on a single `ExperienceRecorded` event
  without scheduler intervention (P0).
- A goal can be given to the agent loop and it produces, acts, and records a new
  experience autonomously (P1).
- World Model + Safety gating exist and gate the autonomous loop (P2).
- No self-check exists purely to silence dead-code warnings (P3).
- The test suite passes with 0 warnings and 0 code-quality issues (P3/P4).

### Current state (as of this audit)

- Warning count: 10 (down from 118).
- test_suite: 333 passed, 0 failed, 5 skipped; 1 code-quality issue (known
  false positive on `ExperienceObserver` import).
- Event bus + WorkerManager observers: live and doing real cognitive work.
- EventSubscriber: relay only (P0 work).
- Scheduler: drives `process_experience_full` + `validate_hypothesis` +
  `promote_to_knowledge` every 2h (`LearningMaintenance`, 7200s) — this is the
  only current path by which knowledge is autonomously earned.
- No agent/actuator layer; loop is open (P1 work).

## Test Suite Improvements (Diagnosability & Coverage)

The test suite was upgraded to surface previously-invisible problems and make
failure diagnosis faster. All improvements are implemented and verified.

### What changed

1. **Server stderr capture** (`src/main.rs`)
   - `TestMcpClient` now pipes `stderr` (previously only stdout/stdin).
   - A background task streams server `tracing` logs into a 500-line ring
     buffer (`ServerLogBuffer`).
   - On any non-passing `TestResult`, the runner attaches the 15 most recent
     server log lines plus any lines mentioning that tool name
     (`runner.rs` → `TestResult.server_logs`).
   - Failed/error test detail views print these logs inline, so a bare
     "Tool returned error: X" now shows the server-side `WARN`/`ERROR` context
     that explains *why*.

2. **Tool coverage cross-check** (`src/test_results/mod.rs` `CoverageReport`,
   `src/comprehensive_test/mod.rs`, `src/test_results/display/coverage.rs`)
   - After `tools/list`, the suite diffs the server's exposed tool names
     against the `FunctionRegistry`'s tested tool names.
   - Produces two lists: **untested tools** (server exposes, no test) and
     **phantom tools** (registry tests, server doesn't expose).
   - Rendered as a dedicated report section and counted in the verdict.
   - This turned the previous misleading "100% coverage" into an honest
     "81.2% coverage — 18 server tools untested".

3. **Machine-readable JSON report** (`src/test_results/json_report.rs`)
   - Full report serialized to `test_suite_report.json` alongside the text
     output: summary, coverage, consolidated issues, all results, lint/code
     issues.
   - Enables run-to-run diffing, CI gating, and tooling to filter/group
     (e.g. "newly failing since last run", "new warnings").

4. **Consolidated issues view** (`src/test_results/display/consolidated.rs`)
   - One table grouping every problem kind: failing tests, error tests,
     untested tools, phantom tools, compiler errors/warnings, code-quality
     issues — each with category, tool/file:line, message, severity, and a
     suggested action.
   - Previously these were scattered across separate sections of a 1300+ line
     text file.

5. **Non-zero exit code on any issue**
   - `has_issues()` now includes coverage gaps, lint errors, and lint warnings
     (not just test failures).
   - Exit code is 1 if anything needs review; 0 only when fully clean.
   - CI can gate on the exit code.

### Current coverage gaps surfaced by the cross-check

These are tools the server exposes but the `FunctionRegistry` does not test
(see `test_suite_report.json` → `coverage.untested_tools` for the live list):

- **ACP tools**: `route_acp_message`, `register_agent`, `unregister_agent`,
  `list_acp_agents`, `acp_agent_count`, `acp_registry`, `acp_router`,
  `create_acp_message`, `get_agent_capabilities` — tested separately in
  `tests/acp/` but not in the `FunctionRegistry` pipeline.
- **Evidence/Observation**: `get_evidence`, `list_evidence`,
  `list_observations`.
- **Knowledge**: `get_knowledge` (only `query_knowledge`/`add_knowledge`
  tested).
- **Workflow**: `set_workflow_variable`.
- **Memory**: `archive_memory`, `link_memories`.
- **Search**: `ranked_search`.
- **System**: `get_system_status`.

**Phantom tools** (registry tests but server doesn't expose): the embedding
tools (`store_embedding`, `get_embedding`, `search_similar`, `list_embeddings`,
`delete_embedding`, `get_embedding_stats`) — these are registered as MCP tools
in the registry but the server's `tools/list` does not return them, indicating
a registration wiring gap in robot_brain.

### How to use the new outputs

```bash
# Run (from test_suite/ or repo root; paths resolve at runtime)
./target/release/test_suite

# Text report (human-readable, unchanged location)
test_suite/test_suite_output.txt

# JSON report (machine-readable, for diffing/CI)
test_suite/test_suite_report.json

# CI gating: exit code is non-zero on any issue
./target/release/test_suite && echo "clean" || echo "issues found"

# Diff two runs (example)
jq '.summary' test_suite_report.json
jq '.issues | map(.kind) | group_by(.) | map({(.[0]): length})' test_suite_report.json
```

### Still not tested (future work)

- **Schema-validation matrix**: every tool × missing/extra/wrong-type fields.
- **Edge cases**: malformed JSON, boundary values, Unicode, empty strings,
  large payloads, concurrent calls, timeouts.
- **End-to-end learning loop**: `record_experience` → `validate_hypothesis` →
  `promote_to_knowledge` (overlaps with v2.0 P0).
- **State isolation**: tests share one server instance; no per-test rollback.
- **Performance baselines**: durations reported but never gated.
