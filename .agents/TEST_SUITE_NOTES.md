# Test Suite Improvements (Diagnosability & Coverage)

> Moved here from AGENTS.md on 2026-08-11. Reference material on how the
> test_suite reports work. Consult when diagnosing test failures or coverage
> gaps; not needed at session start.

The test suite was upgraded to surface previously-invisible problems and make
failure diagnosis faster. All improvements are implemented and verified.

## Contract — what the suite enforces (for upgrade work)

> This is the durable contract I rely on during upgrades. If a change breaks any
> of these, the gate is red and the change is not done.

### Build + run (the gotcha)

`test_suite/` is a SEPARATE independent project (own `Cargo.toml`/`Cargo.lock`),
NOT a workspace member. `cargo build -p test_suite` / `cargo run --package
test_suite` from the repo root FAILS with "package ID did not match". Build and
run it from its own directory:

```bash
cd test_suite && cargo build --release && ./target/release/test_suite
```

Outputs: `test_suite/test_suite_output.txt` (text) and
`test_suite/test_suite_report.json` (machine-readable).

### Exit-code semantics

- Exit 0 ONLY when fully clean (all tests pass, 0 code-quality issues, 0 lint
  errors, 0 lint warnings, no coverage gaps).
- Exit 1 if anything needs review (failing tests, error tests, untested tools,
  phantom tools, compiler errors/warnings, code-quality issues).
- CI can gate on the exit code.

### The 5 success criteria (must all be true for exit 0)

1. All tests pass (no failures).
2. No code-quality issues (no `#[allow(*)]`, `unimplemented!()`, `todo!()`).
3. All functions work end-to-end.
4. All sub-functions complete.
5. MCP Workflow Integration: agent correctly discovers and uses workflows.

### Test execution order (5 phases)

1. Code Analysis — source code quality check (regex patterns, see below).
2. Lint Analysis — clippy + cargo check.
3. Comprehensive Tests — FunctionRegistry-based tool tests.
4. Traditional Tests — individual tool category tests.
5. MCP Workflow Integration — agent workflow usage validation.

### Code-analyzer patterns (what it flags)

- `#[allow(...)]` / `#![allow(...)]` annotations (regex:
  `#\s*\[\s*allow\s*\([^)]*\)`). NOTE: the analyzer's regex catches outer
  `#[allow]` but the verdict/exit-code path relies on `cargo build`/clippy to
  catch inner `#![allow]`. Run both.
- `unimplemented!()` / `todo!()` macros.
- `panic!()` with stub-like messages.
- Early-return stubs (functions that only return Ok/Err immediately).
- Placeholder return patterns.
- Fallback regex is "." (always valid) via `get_fallback_regex()`. The
  `CodePatterns` struct compiles: `allow_annotation`, `dead_code_allow`,
  `unimplemented`, `todo`, `panic`, `underscore_prefix`.

### Tool coverage cross-check

After `tools/list`, diffs server-exposed tool names against the
FunctionRegistry's tested tool names. Produces:
- **untested tools** (server exposes, no test) — counted in the verdict.
- **phantom tools** (registry tests, server doesn't expose) — registration gap.

This is why the suite exits non-zero at 60.9% coverage even with 333/333 tests
passing. Closing coverage gaps is part of upgrade work (see PLAN.md T3-29).

## What changed

1. **Server stderr capture** (`src/main.rs`)
   - `TestMcpClient` now pipes `stderr` (previously only stdout/stdin).
   - A background task streams server `tracing` logs into a 500-line ring
     buffer (`ServerLogBuffer`).
   - On any non-passing `TestResult`, the runner attaches the 15 most recent
     server log lines plus any lines mentioning that tool name
     (`runner.rs` -> `TestResult.server_logs`).
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
     "81.2% coverage - 18 server tools untested".

3. **Machine-readable JSON report** (`src/test_results/json_report.rs`)
   - Full report serialized to `test_suite_report.json` alongside the text
     output: summary, coverage, consolidated issues, all results, lint/code
     issues.
   - Enables run-to-run diffing, CI gating, and tooling to filter/group
     (e.g. "newly failing since last run", "new warnings").

4. **Consolidated issues view** (`src/test_results/display/consolidated.rs`)
   - One table grouping every problem kind: failing tests, error tests,
     untested tools, phantom tools, compiler errors/warnings, code-quality
     issues - each with category, tool/file:line, message, severity, and a
     suggested action.
   - Previously these were scattered across separate sections of a 1300+ line
     text file.

5. **Non-zero exit code on any issue**
   - `has_issues()` now includes coverage gaps, lint errors, and lint warnings
     (not just test failures).
   - Exit code is 1 if anything needs review; 0 only when fully clean.
   - CI can gate on the exit code.

## Current coverage gaps surfaced by the cross-check

These are tools the server exposes but the `FunctionRegistry` does not test
(see `test_suite_report.json` -> `coverage.untested_tools` for the live list):

- **ACP tools**: `route_acp_message`, `register_agent`, `unregister_agent`,
  `list_acp_agents`, `acp_agent_count`, `acp_registry`, `acp_router`,
  `create_acp_message`, `get_agent_capabilities` - tested separately in
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
`delete_embedding`, `get_embedding_stats`) - these are registered as MCP tools
in the registry but the server's `tools/list` does not return them, indicating
a registration wiring gap in robot_brain.

## How to use the new outputs

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

## Still not tested (future work)

- **Schema-validation matrix**: every tool - missing/extra/wrong-type fields.
- **Edge cases**: malformed JSON, boundary values, Unicode, empty strings,
  large payloads, concurrent calls, timeouts.
- **End-to-end learning loop**: `record_experience` -> `validate_hypothesis` ->
  `promote_to_knowledge` (overlaps with v2.0 P0).
- **State isolation**: tests share one server instance; no per-test rollback.
- **Performance baselines**: durations reported but never gated.
