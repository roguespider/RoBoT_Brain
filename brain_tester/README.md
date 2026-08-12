# RoBoT Brain — Unified Test Suite (`brain_tester`)

Independent project (own `Cargo.toml`/`Cargo.lock`, NOT a workspace member).
Tests the `robot_brain` MCP/ACP server end-to-end by spawning it as a subprocess
and speaking MCP over stdio. No mocking, no stubs. This is the **single**
test tool — it supersedes the old `test_suite` and the Python `live_test`.

## Build + run (from this directory)

```bash
cargo build --release && ./target/release/brain_tester
```

Do NOT run `cargo build -p brain_tester` / `cargo run --package brain_tester`
from the repo root — it fails with "package ID did not match". This project is
built from its own directory only.

## CLI modes

| Command | Purpose |
|---|---|
| `brain_tester` | Full suite: live tool tests + coverage gate + code analysis + lint (default). |
| `brain_tester --list` | Quick smoke check: list every server-advertised tool + required fields. |
| `brain_tester --probe TOOL` | Introspect one tool's live `inputSchema` — discover required/optional params. |

The `--probe` mode replaces the old Python `RobotBrainClient.list_tools()`
introspection. It lets you discover the exact parameters a tool expects without
guessing (e.g. `brain_tester --probe register_agent` reveals it requires
`agent_type` + `instance_id`).

## Outputs

- `brain_tester_output.txt` — human-readable report.
- `brain_tester_report.json` — machine-readable (summary, coverage, issues, all
  results, lint/code issues). Use for run-to-run diffing and CI gating.

## Exit code

- `0` only when fully clean (all tests pass, 0 code-quality issues, 0 lint
  errors/warnings, no coverage gaps).
- `1` if anything needs review. CI can gate on the exit code.

## Success criteria (all must hold for exit 0)

1. All tests pass (no failures).
2. No code-quality issues (no `#[allow(*)]`, `unimplemented!()`, `todo!()`).
3. All functions work end-to-end.
4. All sub-functions complete.
5. MCP Workflow Integration: agent correctly discovers and uses workflows.

## Execution order

1. Code Analysis — source code quality check (regex patterns).
2. Lint Analysis — clippy + cargo check.
3. Comprehensive Tests — FunctionRegistry-based tool tests.
4. Traditional Tests — individual tool category tests.
5. MCP Workflow Integration — agent workflow usage validation.

## Code analyzer flags

`#[allow(...)]`/`#![allow(...)]`, `unimplemented!()`, `todo!()`, `panic!()`
with stub messages, early-return stubs, placeholder returns, `_` ignored vars.

## Tool coverage cross-check

After `tools/list`, diffs server-exposed tool names against the FunctionRegistry
tested names. Produces **untested tools** (server exposes, no test) and
**phantom tools** (registry tests, server doesn't expose). Untested tools are
counted in the verdict — so the suite can exit non-zero even with all tests
passing if coverage is incomplete.

See `../.agents/TEST_SUITE_NOTES.md` for the full contract + analyzer internals +
current coverage gaps + JSON report usage.
