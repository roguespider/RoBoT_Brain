# RoBoT Brain - Agent Memory

## Incremental Workflow Principle (MANDATORY)

**Do one thing, verify it works, push to GitHub, then work on the next thing.**

- NEVER batch multiple unrelated changes into a single commit or session step.
- After each fix/refactor/file change: build → test → commit → push.
- Only after the push succeeds, move to the next task.
- This makes each change independently reviewable and revertable.
- If a later change breaks something, you know exactly which change caused it.

### Verify, Don't Trust (MANDATORY)

**Never rely on a "done" message — yours, a prior session's, or a commit
description. Verify each step by inspecting the actual codebase state.**

- A commit that says "fixes all warnings" may be lying. Run the gate and read
  the actual output.
- A PLAN.md checkbox marked `[x]` only means someone claimed it was done. Open
  the file, read the code, confirm the change is actually there and actually
  works.
- A task marked `[in]` (in progress) may have been abandoned mid-step. Check
  whether the described changes actually exist in the source and whether they
  compile.
- Before claiming a task is done: run the gate, read the JSON report, confirm
  the relevant metric is actually 0 (not just "I think I fixed it").
- "It compiles on my machine" is not verification. The gate is the verifier.

### Never Separately Build robot_brain (MANDATORY)

**test_suite auto-builds robot_brain. Never run
`cargo build -p robot_brain` or `cargo build --release -p robot_brain`
separately.**

- test_suite and robot_brain are two separate, independent projects.
  test_suite does NOT import or link robot_brain's source. It spawns
  robot_brain as a subprocess via MCP.
- When working on test_suite, NEVER touch `src/` (robot_brain's source). When
  working on robot_brain, NEVER touch `test_suite/src/`.
- To build + test robot_brain: `cd test_suite && cargo build --release &&
  ./target/release/test_suite`. That's it. test_suite rebuilds robot_brain
  automatically.
- Running a separate `cargo build -p robot_brain` wastes time and can mask
  discrepancies between what you built and what test_suite built.

### Quality Gate (MANDATORY before any commit)

Run `cd test_suite && cargo build --release && ./target/release/test_suite --gate`.
All four metrics must pass: `tests` (100%), `compiler_warnings` (0),
`code_issues` (0), `untested_tools` (0). See README "Quality Gate" section
for the full table and the JSON-report triage recipe.

The structured report at `test_suite/test_suite_report.json` has an `issues[]`
array; each entry has `kind`/`category`/`file`/`line`/`message`/`suggested_action`.
Use `python3 -c` + `collections.Counter` to group warnings by message/file for
triage. Fix dead-code first (highest signal), then mechanical clippy lints.

## Startup (do this every session — see `.agents/STARTUP.md` for the full call to action)

1. Read `.agents/STARTUP.md`, then this file, then `.agents/PLAN.md` — in full.
2. Run the verify gate (build + live test + test suite). Must be green before
   any code change. Do not "remember" a prior pass — run it.
3. Pick the FIRST incomplete task from `.agents/PLAN.md` "Next steps". Do not
   skip ahead.
4. ONE change → re-run gate → commit → push → STOP and report. Never batch.

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
| **test_suite** | `/test_suite/` | `test_suite` | Unified test suite (live MCP/ACP tests + coverage gate + code analysis) |

These programs **do NOT depend on each other's source code**. test_suite tests robot_brain by spawning it as a subprocess via MCP protocol.

## Build Commands

```bash
# The verify gate — test_suite auto-builds robot_brain, connects via MCP,
# runs all tests + code analysis, and enforces 0 warnings / 0 code-issues /
# 0 untested tools. This is the ONLY command needed to build + test:
cd test_suite && cargo build --release && ./target/release/test_suite
# Outputs: test_suite/test_suite_output.txt and test_suite/test_suite_report.json
#
# Or use `make gate` (runs the same thing via .agents/scripts/gate.sh).
#
# CLI modes:
#   test_suite              → full suite (default)
#   test_suite --list       → list all server tools (smoke check)
#   test_suite --probe TOOL → introspect one tool's live inputSchema
#
# Build main binary only (rarely needed — test_suite does this automatically):
#   cargo build --release -p robot_brain
```

## Post-Compile: Connect to robot_brain MCP/ACP

**IMPORTANT (User Requirement):** test_suite auto-builds robot_brain and
connects via MCP, but the AI agent MUST also connect to the running
robot_brain MCP/ACP server directly to test it. Do not rely solely on
test_suite — connect yourself as a client.

**Two ways to connect (do NOT hand-write a new MCP client):**

1. **`test_suite --probe TOOL`** — Rust, built into the test suite. Introspects a tool's live `inputSchema` (required/optional params). Fastest way to discover what a tool expects.
2. **`.agents/live_test/mcp_client.py`** — stdlib-only Python `RobotBrainClient` for ad-hoc calls. Auto-detects the binary, auto-handles the workflow gate.

```bash
# Schema introspection (Rust) — discover a tool's required fields:
cd test_suite && ./target/release/test_suite --probe register_agent

# Quick smoke check — list all server tools + required fields:
cd test_suite && ./target/release/test_suite --list

# Ad-hoc tool calls via the reusable Python RobotBrainClient:
#   from mcp_client import RobotBrainClient
#   with RobotBrainClient() as c:
#       c.init()  # initialize + workflow gate (get_workflow -> search_memory)
#       r = c.call("store_memory", {"content": "hi", "memory_type": "note"})
```

The `robot-brain` skill (`.agents/skills/robot-brain/skill.md`) documents the tool catalog and the workflow gate. Steps after a successful build:
1. Invoke the `robot-brain` skill
2. Run `test_suite` (full suite) or `test_suite --probe TOOL` (schema lookup) or use `RobotBrainClient` for targeted calls
3. Verify key tools: `store_memory`, `search_memory`, `list_memories`, `create_plan`, `list_plans`, `create_workflow`, `start_workflow`, `query_knowledge`, `record_experience`
4. ACP tools: `route_acp_message`, `register_agent`, `list_acp_agents` (note: `list_agents` does not exist — the real tool is `list_acp_agents`)
5. Discovery: `test_suite --list` confirms all tools are available
6. Report which tools work and which fail

**Workflow gate (required before any substantive tool call):** the server returns `WORKFLOW_NOT_RETRIEVED` until `get_workflow` is called, then `MEMORY_NOT_SEARCHED` until `search_memory` is called. `RobotBrainClient.init()` handles both automatically.

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

7. **NO Emoji / Plain-Text Markers Only**
   - Strictly forbidden in ALL code and `.agents/` docs: decorative emoji
     (check marks, cross marks, party popper, clipboard, warning signs, etc.)
     and the variation selector U+FE0F. Emoji caused real mojibake breakage in
     `.agents/` files (multi-byte sequences mangled when round-tripping
     through sed/git diffs/terminals), breaking automated edits.
   - **Scope:** all `.rs` files (both robot_brain `src/` and `test_suite/src/`),
     `.agents/**/*.md`, `.agents/**/*.sh`, githooks, and README. **Excluded:**
     `robot_architecture/**` (user-authored, out of scope).
   - **Permitted non-ASCII:** flow-diagram arrows `->` `|` `v` (Unicode
     U+2190-U+21FF) in doc-comments and docs. These carry meaning and are NOT
     banned. Only decorative emoji are banned.
   - **Plain-text markers to use instead:**

     | Emoji | Plain-text |
     |-------|-----------|
     | check / OK   | `[OK]` / `[PASS]` |
     | cross / X    | `[FAIL]` / `[ERR]` |
     | warning      | `[WARN]` |
     | clipboard/info | `[INFO]` |
     | party / done | `[DONE]` |
     | no-entry     | `[BLOCKED]` |
     | star / gear  | `[INFO]` |

   - **Enforcement:** the quality gate flags disallowed non-ASCII as
     gate-failing `Emoji` code issues. The detector in
     `test_suite/src/code_analyzer/analyzer.rs` (`check_emoji`) uses an
     ALLOW-list: any non-ASCII char not explicitly permitted is flagged, so
     new emoji added later are caught automatically (no banned-list to drift).
     Allowed: Arrows (U+2190-U+21FF), Box Drawing (U+2500-U+257F), and a
     small prose-punctuation set (em/en dash, curly quotes, ellipsis, bullet,
     NBSP, section sign, degree, middle dot). Scanned across both `src/` and
     `test_suite/src/`. Status markers in `.agents/*.md` use `[x]`/`[ ]` for
     task state and `[DONE]`/`[RED]`/`[PASS]`/`[FAIL]` for gate status.

### Dead Code Resolution Protocol

> **Don't add `#[cfg(test)]` to robot_brain's `src/`.** It causes code-quality
> issues and the quality gate flags it. Tests belong in `test_suite/` (as MCP
> flow tests) where they exercise the real public surface. The ONLY exception is
> Group B — internal-only/dead-code logic with no MCP surface that genuinely
> cannot be reached from test_suite. For Group B, leave the existing Rust unit
> test in place and document the reclassification in PLAN.md, but do NOT create
> NEW `#[cfg(test)]` blocks in `src/`.

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

## Large File Refactoring

Periodic maintenance task: split large `.rs` files (~1000+ lines mixing
responsibilities) into directory modules. Full pattern + candidates query + the
import-path-migration rule moved to **`.agents/LARGE_FILE_REFACTOR.md`**. Check
it from time to time; not a session-start rule.

## OpenHands MCP Integration

Reference material for wiring an OpenHands agent to RoBoT Brain as an MCP
server has moved to **`.agents/OPENHANDS_INTEGRATION.md`**. Consult it when
integrating with the OpenHands SDK; it is not needed for normal build/test/work
sessions.

## test_suite Coverage (FunctionRegistry)

The coverage gate cross-checks the server's `tools/list` against the test
suite's `FunctionRegistry` (in `test_suite/src/function_registry/`). Key facts
every session should know:

- **Adding a tool to the server's `tools/list` is NOT enough to close
  coverage.** The tool must ALSO have a `TestRequirement` entry in
  `function_registry/` (with a matching `id` case in
  `comprehensive_test/argument_builder.rs`). The cross-check diffs server tool
  names vs the registry's `function_name` fields. Standalone tests in
  `test_suite/src/tests/` do NOT count toward coverage.
- **Tool-list drift hazard:** each MCP handler maintains `tool_names()` /
  `get_tools()` / `execute_tool()` as THREE separate lists that must stay in
  sync. `get_tools()` feeds the RMCP `tools/list` response; if it omits an
  entry that the other two include, the tool is callable-but-unadvertised →
  flagged as a **phantom tool** by the cross-check (T1-19 root cause).
- **Validation choice for new registry tests:** use `IsSuccess(None)` for tools
  that succeed on a default/fake call, and `IsSuccess(Some("false"))` for tools
  that return an MCP error on a fake id. To pick correctly, probe the tool with
  a fake id via `RobotBrainClient` and check `is_error`.
- **Probing tip:** extract all tool schemas at once from the live server:
  ```python
  from mcp_client import RobotBrainClient
  with RobotBrainClient() as c:
      c.init()
      for t in c.list_tools():
          print(t['name'], t.get('inputSchema',{}).get('required',[]))
  ```
- **Current gate state (2026-08-12):** RED on warnings. 145/145 tests pass,
  0 code issues, 0 untested, 0 phantom, but **67 compiler warnings**
  (dead-code `never used`/`never read`, `collapsible_if`, async-fn
  simplification, `module_inception`, `too_many_arguments`). The warning count
  is the only gate blocker. See `.agents/PLAN.md` "Verified state" for the
  live snapshot. NOTE: the 2026-08-11 "GREEN" note above was stale — always
  re-run the gate, never trust a prior "done/GREEN" claim (Verify, Don't Trust).

## Verifying task status (MANDATORY)

When asked "is T1-NN done?" or "is X working 100%?":
- Do NOT read the PLAN.md checkbox and repeat it. Checkboxes lie.
- Do NOT trust the "Verified state" snapshot in PLAN.md — it can be stale.
- INSPECT THE CODEBASE: `grep`/`find` for the actual change, read the code,
  confirm the API exists and is wired.
- RUN THE GATE: `cd test_suite && cargo build --release &&
  ./target/release/test_suite`. Read `test_suite_report.json` for the real
  metrics (`compiler_warnings`, `code_issues`, `untested_tools`,
  `tests passed/total`, `overall_success`).
- For "working 100%" claims, the done-when criteria matter (e.g. T1-10 =
  "queue survives a process restart"). Wire a real end-to-end test in
  test_suite that exercises that criterion, not just "the function exists".
- Report what is actually true, including gaps the PLAN glosses over.

## All tests live in test_suite (MANDATORY)

- `#[cfg(test)]` modules inside robot_brain's `src/` are NOT the place for
  tests. All tests belong in `test_suite/`. (test_suite tests robot_brain by
  spawning it as a subprocess over MCP/CLI — that is the project's testing
  model.)
- When verifying a feature end-to-end, add the test under
  `test_suite/src/tests/` (e.g. `queue_durability.rs`), wire it into
  `tests/mod.rs` and dispatch it from `main.rs`. Add deps to
  `test_suite/Cargo.toml` as needed (e.g. `rusqlite` bundled, `tempfile`).
- Cross-process/restart tests: copy the server binary into a `tempfile::tempdir()`
  (the server creates `robot_brain.db` beside `current_exe`), spawn via stdio
  MCP, manipulate the DB with `rusqlite`, restart, and assert via MCP tools.
  Remember the workflow gate: a fresh client must call `get_workflow` then
  `search_memory` before any substantive tool (else `WORKFLOW_NOT_RETRIEVED` /
  `MEMORY_NOT_SEARCHED`).

---

> **The sections below have moved out of this file to reduce noise:**
> - **Roadmap to v2.0 (Architecture Conformance Work)** — the P0-P4 status
>   tracker, Definition of Done, Resume Here, and verified-state snapshot —
>   moved to **`.agents/PLAN.md`** section 6 ("v0.0.1 CONFORMANCE WORK").
>   Forward planning (STAGE 1-3) is in the same file, sections 4-5.
> - **Test Suite Improvements (Diagnosability & Coverage)** — coverage gaps,
>   JSON report usage, future test work — moved to
>   **`.agents/TEST_SUITE_NOTES.md`**.
>
> This file now ends at the hard rules above. Everything status/narrative/
> reference lives in `.agents/` so it does not bury the rules you must follow
> every session.

