# Test Suite Improvements (Diagnosability & Coverage)

> Moved here from AGENTS.md on 2026-08-11. Reference material on how the
> test_suite reports work. Consult when diagnosing test failures or coverage
> gaps; not needed at session start.

## Architecture — test_suite2 (replaced legacy Rust test_suite)

The legacy Rust `test_suite/` at the repo root has been **replaced** by
`.agents/scripts/test_suite2/` — a **Python-based** functional test suite that
tests the **compiled** robot_brain binary via the MCP protocol. The Rust
component is retained only for the **coverage gate** (code-quality, lint,
tool-coverage cross-check) and is now located at
`.agents/scripts/test_suite2/` (see `gate.sh` line 28).

### Build + run

```bash
# The verify gate (preferred — builds robot_brain, runs both Rust and Python tests):
make gate
# Or directly:
./.agents/scripts/gate.sh

# Python functional tests only:
python3 -m pytest .agents/scripts/test_suite2/ -v
# Or via the orchestrator:
.agents/scripts/test_suite2/test_runner.sh

# Legacy Rust coverage gate only (from test_suite2 dir):
cd .agents/scripts/test_suite2 && cargo build --release && ./target/release/test_suite
```

### Output

- Rust report: `.agents/scripts/test_suite2/test_suite_report.json`
- Python output: `.agents/scripts/test_suite2/python_test_output.txt`

### Exit-code semantics

- Exit 0 ONLY when fully clean (all Python tests pass, Rust coverage gate green,
  0 code-quality issues, 0 lint errors, 0 lint warnings, no coverage gaps).
- Exit 1 if anything needs review.
- CI can gate on the exit code.

### The 5 success criteria (must all be true for exit 0)

1. All tests pass (no failures).
2. No code-quality issues (no `#[allow(*)]`, `unimplemented!()`, `todo!()`).
3. All functions work end-to-end.
4. All sub-functions complete.
5. MCP Workflow Integration: agent correctly discovers and uses workflows.

### Test execution order (5 phases)

1. **Code Analysis** — source code quality check (regex patterns in Rust suite).
2. **Lint Analysis** — clippy + cargo check (Rust suite).
3. **Comprehensive Tests** — FunctionRegistry-based tool tests (Rust suite).
4. **Traditional Tests** — individual tool category tests (Python suite).
5. **MCP Workflow Integration** — agent workflow usage validation (Python suite).

### Code-analyzer patterns (what the Rust suite flags)

- `#[allow(...)]` / `#![allow(...)]` annotations
- `unimplemented!()` / `todo!()` macros
- `panic!()` with stub-like messages
- Early-return stubs
- Placeholder return patterns
- `underscore_prefix` — `_variable` patterns

### Tool coverage cross-check

After `tools/list`, the Rust suite diffs server-exposed tool names against the
FunctionRegistry's tested tool names. Produces:
- **untested tools** (server exposes, no test) — counted in the verdict.
- **phantom tools** (registry tests, server doesn't expose) — registration gap.

## What changed (from legacy Rust test_suite to test_suite2)

1. **Relocated from repo root to `.agents/scripts/test_suite2/`**
   The old `test_suite/` at the repo root has been removed. All test infrastructure
   now lives under `.agents/scripts/test_suite2/`. The `Makefile` `suite` target
   still references `test_suite/` — update it to `test_suite2` when the Rust
   port is complete.

2. **Python-first for functional testing**
   Python/pytest now handles functional, end-to-end, and integration tests.
   Tests communicate with robot_brain via stdio MCP protocol — no source access.
   `mcp_client.py` provides the MCP client; `conftest.py` provides pytest fixtures.

3. **Rust retained for coverage gate only**
   The Rust binary (`test_suite` at `.agents/scripts/test_suite2/`) handles:
   code-quality analysis, lint checks, FunctionRegistry-based tool coverage
   cross-check, and schema probing (`--probe`).

4. **Shell orchestrator (`test_runner.sh`)**
   The Python orchestrator runs the full functional test suite: builds
   robot_brain if needed, starts the server, runs all pytest phases, stops
   the server, and writes output to `python_test_output.txt`.

5. **Machine-readable JSON report** (Rust suite)
   Full report serialized to `test_suite_report.json`: summary, coverage,
   consolidated issues, all results, lint/code issues. Enables run-to-run
   diffing and CI gating.

6. **Consolidated issues view** (Rust suite)
   One table grouping every problem kind: failing tests, error tests, untested
   tools, phantom tools, compiler errors/warnings, code-quality issues.

7. **Non-zero exit code on any issue** (gate.sh)
   The gate checks all four metrics: `passed`, `failed+errors`,
   `compiler_warnings`, `code_issues`, `untested_tools`. Any non-zero count
   causes the gate to fail.

## Current coverage gaps (from Rust suite FunctionRegistry)

These are tools the server exposes but the FunctionRegistry does not test:

- **ACP tools**: `route_acp_message`, `register_agent`, `unregister_agent`,
  `list_acp_agents`, `acp_agent_count`, `acp_registry`, `acp_router`,
  `create_acp_message`, `get_agent_capabilities`
- **Evidence/Observation**: `get_evidence`, `list_evidence`, `list_observations`
- **Knowledge**: `get_knowledge` (only `query_knowledge`/`add_knowledge` tested)
- **Workflow**: `set_workflow_variable`
- **Memory**: `archive_memory`, `link_memories`
- **Search**: `ranked_search`
- **System**: `get_system_status`

**Phantom tools** (registry tests but server doesn't expose): embedding tools
(`store_embedding`, `get_embedding`, `search_similar`, `list_embeddings`,
`delete_embedding`, `get_embedding_stats`) — registration wiring gap.

## How to use the new outputs

```bash
# Run the full gate:
make gate

# Run Python tests only:
python3 -m pytest .agents/scripts/test_suite2/ -v

# Read Rust report:
python3 -c "import json; d=json.load(open('.agents/scripts/test_suite2/test_suite_report.json')); print(json.dumps(d['summary'], indent=2))"

# Diff two runs:
python3 -c "import json; d=json.load(open('test_suite_report.json')); print(d.get('coverage', {}).get('untested_tools', []))"
```

## Still not tested (future work)

- **Schema-validation matrix**: every tool — missing/extra/wrong-type fields.
- **Edge cases**: malformed JSON, boundary values, Unicode, empty strings,
  large payloads, concurrent calls, timeouts.
- **End-to-end learning loop**: `record_experience` -> `validate_hypothesis` ->
  `promote_to_knowledge`.
- **State isolation**: tests share one server instance; no per-test rollback.
- **Performance baselines**: durations reported but never gated.
