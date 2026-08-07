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

When modifying or extending this codebase:

- **No panics**: Never use `.unwrap()`, `.expect()`, or panic macros
- **Error handling**: Use `?` operator, `match`, or `if let` for Result handling
- **No stubs**: Never use `todo!()`, `unimplemented!()`, `unreachable!()`
- **No silencing**: Never add `#[allow(*)]` attributes
- **No ignored variables**: Never use `_variable_name` for unused values
- **Name conflicts**: If renaming types, use descriptive new names (e.g., `LearningPattern` instead of `Pattern`)

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
