# RoBoT Brain - Agent Memory

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

# Check compilation
cargo check -p robot_brain
cargo check -p test_suite
```

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
- `src/planner/planner.rs` (836 lines)
- `src/bridge/tools/memory/mod.rs` (803 lines)
- `src/bridge/tools/exploration/handlers.rs` (791 lines)
- `src/personality/mod.rs` (614 lines)

### Already Refactored
- `src/experience/integration/learning_coordinator/` (1519 total → config.rs, results.rs, entry.rs, exploration.rs, hypothesis.rs, knowledge.rs, reinforcement.rs, reputation.rs, generalization.rs, mod.rs [274 lines])
- `src/skills/registry/` (983 total → types.rs, skill.rs, registry.rs, context.rs, result.rs, metrics.rs, executor.rs, mod.rs)
- `src/database/queries/` (934 total → helpers.rs, memory.rs, scheduled_tasks.rs, observations.rs, experiences.rs, embeddings.rs, relationships.rs, tests.rs, mod.rs)
- `src/bridge/app/` (944 total → state.rs, initialization.rs, scheduler.rs, personality.rs, acp.rs, mod.rs)
- `test_suite/src/code_analyzer/` (1050 lines → types.rs, patterns.rs, analyzer.rs, lint.rs, mod.rs)
- `src/bridge/acp/` (950 lines → message.rs, error.rs, channel.rs, agent.rs, registry.rs, router.rs, builder.rs, mod.rs)
- `test_suite/src/tests/rmcp/` (NEW: 650 lines → mod.rs, protocol.rs, tools.rs, sessions.rs)
- `test_suite/src/tests/acp/` (NEW: 750 lines → mod.rs, registry.rs, router.rs, agents.rs, messages.rs)
- `test_suite/src/tests/agent_simulation/` (NEW: 440 lines → mod.rs, workflows.rs, memory_agent.rs, decision_making.rs)
