# RoBoT Brain - Agent Memory

This file has two sections:
- **Above STARTUP**: Hard rules, conventions, build commands, coding standards
- **STARTUP section**: Session-start workflow (steps 1-9, execute in order)
- **Below STARTUP**: Objective, operating rules, context summary, approach

When asked to follow AGENTS.md: read the full file top-to-bottom, then follow STARTUP steps.

**Entry point for a new session:** call `get_workflow` (MCP tool) first, then read `.agents/PLAN.md` and `.agents/TEST_SUITE_NOTES.md` before any code work.

## Working Memory Protocol (MANDATORY)
- Your working memory is the built-in memory system — use `store_memory` for thinking/reasoning/analysis and `search_memory` for retrieval.
- Never rely on implicit chat buffer or conversation context between turns. Before every substantive action, store your current state in memory.
- Store session context with `memory_type: "note"` and relevant tags so it survives across turns and can be retrieved later.
- Always call `store_memory` before responding — do not rely on implicit conversation context.

## Incremental Workflow Principle (MANDATORY)

**Do one thing, verify it works, push to GitHub, then work on the next thing.**

- NEVER batch multiple unrelated changes into a single commit or session step.
- After each fix/refactor/file change: build → test → commit → push.
- Only after the push succeeds, move to the next task.
- This makes each change independently reviewable and revertable.
- If a later change breaks something, you know exactly which change caused it.

### Verify, Don't Trust

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
- Any status claim in README, PLAN.md, CHANGELOG.md, or `.agents/*.md` that
  references test counts, warning counts, or completeness must be backed by
  a same-day gate run. If the gate was not run this session, soften the claim:
  - Instead of "0 warnings" → "pending gate verification"
  - Instead of "148/148 tests pass" → "148 tests (unverified, pending gate)"
  - Never hardcode gate counts in task notes without a date
- The single source of truth is `.agents/scripts/test_suite2/test_suite_report.json`.
- When asked "is T1-NN done?" or "is X working 100%?": Do NOT read the PLAN.md
  checkbox and repeat it. Checkboxes lie. INSPECT THE CODEBASE: `grep`/`find`
  for the actual change, read the code, confirm the API exists and is wired.
- For "working 100%" claims, the done-when criteria matter (e.g. T1-10 =
  "queue survives a process restart"). Wire a real end-to-end test in
  test_suite2 that exercises that criterion, not just "the function exists".
- Report what is actually true, including gaps the PLAN glosses over.

## Build Commands

**test_suite2 auto-builds robot_brain. Never run
`cargo build -p robot_brain` or `cargo build --release -p robot_brain`
separately.**

- test_suite2 and robot_brain are two separate, independent projects. test_suite2
  does NOT import or link robot_brain's source. It spawns robot_brain as a
  subprocess via MCP.
- When working on test_suite2, NEVER touch `src/` (robot_brain's source). When
  working on robot_brain, NEVER touch `.agents/scripts/test_suite2/src/`.
- Running a separate `cargo build -p robot_brain` wastes time and can mask
  discrepancies between what you built and what test_suite2 built.

```bash
# The verify gate — test_suite2 auto-builds robot_brain, connects via MCP,
# runs all tests + code analysis, and enforces 0 warnings / 0 code-issues /
# 0 untested tools. This is the ONLY command needed to build + test:
pwsh .agents/scripts/make.ps1 gate
# On Unix: ./agents/scripts/gate.sh
# Outputs: .agents/scripts/test_suite2/test_suite_report.json
#
# CLI modes:
#   pwsh .agents/scripts/make.ps1 gate  → full gate
#   ./agents/scripts/test_suite2/target/release/test_suite → full suite
#   ./agents/scripts/test_suite2/target/release/test_suite --list → list tools
#   ./agents/scripts/test_suite2/target/release/test_suite --probe TOOL → introspect tool
#
# Build main binary only (rarely needed — test_suite2 does this automatically):
#   cargo build --release -p robot_brain
`````

#### Clean Rebuild (use when changing test_suite2 source)

When modifying test_suite2 source code (`src/tests/*.rs`, `Cargo.toml`, etc.),
run a clean rebuild before the gate to avoid stale incremental artifacts:

```powershell
# Windows
pwsh .agents/scripts/make.ps1 clean
```

This removes `__pycache__/`, `.pytest_cache/`, and `target/` from test_suite2,
then runs `cargo clean && cargo build --release`.

**When to use:**
- Changed test source code
- Added new dependencies to `Cargo.toml`
- Gate shows stale/failing tests after source change
- `cargo check` passes but `cargo build` fails

**Normal gate runs:** skip — the gate auto-skips rebuild when source is unchanged.

#### Quality Gate (MANDATORY before any commit)

Run `pwsh .agents/scripts/make.ps1 gate` (Windows) or `./agents/scripts/gate.sh` (Unix).
All four metrics must pass: `tests` (100%), `compiler_warnings` (0),
`code_issues` (0), `untested_tools` (0). See README "Quality Gate" section
for the full table and the JSON-report triage recipe.

The structured report at `.agents/scripts/test_suite2/test_suite_report.json` has an `issues[]`
array; each entry has `kind`/`category`/`file`/`line`/`message`/`suggested_action`.
Use `python3 -c` + `collections.Counter` to group warnings by message/file for
triage. Fix dead-code first (highest signal), then mechanical clippy lints.

**Gate execution pattern (MANDATORY):** The gate takes ~5 minutes. Run it **once at session start**,
then **in the background** while you work. Only re-trigger when you have made actual code changes
since the last run.

```bash
# Start gate in background at session beginning:
./.agents/scripts/test_suite2/target/release/test_suite > .agents/scripts/test_suite2/test_suite_output.txt 2>&1 &

# While it runs, inspect PLAN.md tasks by reading source code.
# Never trust checkboxes — always verify in code.

# When you need fresh results after code changes:
# Wait for the background process to finish, then re-run:
cat .agents/scripts/test_suite2/test_suite_report.json
```

**Rules:**
- Do NOT run the gate twice in the same session unless you changed files since the last run.
- The gate is a control point, not a continuous loop. 1-2 runs per session is normal.
- If the gate is already running, inspect the existing report and continue working.
- Always read the JSON report after the gate finishes — never trust a "passed" claim.

**When gate is red (triage order):**
1. Fix dead-code first (`pub fn` never called, unused types)
2. Fix compiler warnings (clippy lints, unnecessary borrows)
3. Fix actual bugs
4. Never use `#[allow]` or `_` to silence warnings — fix the root cause

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
| **test_suite2** | `.agents/scripts/test_suite2/` | `test_suite` | Unified test suite (live MCP/ACP tests + coverage gate + code analysis) |

These programs **do NOT depend on each other's source code**. test_suite2 tests robot_brain by spawning it as a subprocess via MCP protocol.

## Build Efficiency (parallelize work)

When running builds, **start the build in the background first**, then use the
waiting time to work on other tasks (reading code, updating memory, planning).

Pattern:
1. Start `cd .agents/scripts/test_suite2 && cargo build --release` in background
2. While waiting, review related code, read documentation, plan next steps
3. When build completes, review results and continue

For quick compiler feedback (no test execution), use the fastest option available:
1. Zed Editor's diagnostics (fastest, no build required)
2. `cd .agents/scripts/test_suite2 && cargo check --release` (faster than a full build)
3. `cd .agents/scripts/test_suite2 && cargo build --release` (full build, slowest)

Prefer option 1, fall back to 2, only use 3 when you need link-time checks.

**For gate execution in background:** see **STARTUP #4** (line ~356).

## Post-Compile: Connect to robot_brain MCP/ACP

**IMPORTANT (User Requirement):** test_suite2 auto-builds robot_brain and
connects via MCP, but the AI agent MUST also connect to the running
robot_brain MCP/ACP server directly to test it. Do not rely solely on
test_suite2 — connect yourself as a client.

**Two ways to connect (do NOT hand-write a new MCP client):**

1. **`.agents/scripts/test_suite2/target/release/test_suite --probe TOOL`** — Rust, built into the test suite. Introspects a tool's live `inputSchema` (required/optional params). Fastest way to discover what a tool expects.

```bash
# Schema introspection (Rust) — discover a tool's required fields:
cd .agents/scripts/test_suite2 && ./target/release/test_suite --probe register_agent

# Quick smoke check — list all server tools + required fields:
cd .agents/scripts/test_suite2 && ./target/release/test_suite --list
```

The `robot-brain` skill (`.agents/skills/robot-brain/skill.md`) documents the tool catalog and the workflow gate. Steps after a successful build:
1. Invoke the `robot-brain` skill
2. Run `.agents/scripts/test_suite2/target/release/test_suite` (full suite) or `.agents/scripts/test_suite2/target/release/test_suite --probe TOOL` (schema lookup) for targeted verification
3. Verify key tools: `store_memory`, `search_memory`, `list_memories`, `create_plan`, `list_plans`, `create_workflow`, `start_workflow`, `query_knowledge`, `record_experience`
4. ACP tools: `route_acp_message`, `register_agent`, `list_acp_agents` (note: `list_agents` does not exist — the real tool is `list_acp_agents`)
5. Discovery: `.agents/scripts/test_suite2/target/release/test_suite --list` confirms all tools are available
6. Report which tools work and which fail

**Workflow gate (required before any substantive tool call):** the server returns `WORKFLOW_NOT_RETRIEVED` until `get_workflow` is called, then `MEMORY_NOT_SEARCHED` until `search_memory` is called. The Rust `TestMcpClient::new()` in `.agents/scripts/test_suite2/src/main.rs` handles both automatically.

This direct testing makes it easier to identify working vs. broken functionality immediately after compilation, rather than only seeing aggregate pass/fail from the test suite.

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

7. **Fix Bugs When Found**
   - When you inspect code and find a bug (logic error, missing error handling,
     silent drop, data loss, etc.), you MUST fix it immediately.
   - Do not note it, skip it, or defer it. Write the fix, verify it compiles,
     run the gate, commit.
   - This applies even to tasks not currently on your PLAN.md checklist. A
     verified bug in production code takes priority over task order.

8. **NO Emoji / Plain-Text Markers Only**
   - Strictly forbidden in ALL code and `.agents/` docs: decorative emoji
     (check marks, cross marks, party popper, clipboard, warning signs, etc.)
     and the variation selector U+FE0F. Emoji caused real mojibake breakage in
     `.agents/` files (multi-byte sequences mangled when round-tripping
     through sed/git diffs/terminals), breaking automated edits.
   - **Scope:** all `.rs` files (both robot_brain `src/` and `.agents/scripts/test_suite2/src/`),
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
     `.agents/scripts/test_suite2/src/code_analyzer/analyzer.rs` (`check_emoji`) uses an
     ALLOW-list: any non-ASCII char not explicitly permitted is flagged, so
     new emoji added later are caught automatically (no banned-list to drift).
     Allowed: Arrows (U+2190-U+21FF), Box Drawing (U+2500-U+257F), and a
     small prose-punctuation set (em/en dash, curly quotes, ellipsis, bullet,
     NBSP, section sign, degree, middle dot). Scanned across both `src/` and
     `.agents/scripts/test_suite2/src/`. Status markers in `.agents/*.md` use `[x]`/`[ ]` for
     task state and `[DONE]`/`[RED]`/`[PASS]`/`[FAIL]` for gate status.

### Dead Code Resolution Protocol

**Never use `#[cfg(test)]` in production source.** It causes code-quality issues
and the quality gate flags it. Tests belong in `.agents/scripts/test_suite2/` (as MCP flow tests)
where they exercise the real public surface.

**Fixing unused type warnings:** Many types (like `SimpleAgent`, `AcpCapability`,
ACP message builders) are defined for testing/future use but unused in production.

1. **Move unused types to `.agents/scripts/test_suite2/`** — don't expose unused types in the
   public API; move them to test modules where they belong.
2. **Keep production traits minimal** — implement only what's actually used
   (e.g., `AcpAgent` trait only needs `id()` and `handle()` methods).
3. **Cross-reference architecture**: Check `RoBoT_Brain/robot_architecture/`
   directory for documentation about seemingly dead code.
4. **If documentation describes the feature**: The code is an incomplete stub.
   You MUST fully implement and complete the missing logic (production-ready).
5. **If documentation confirms deprecated/absent**: The code can be safely deleted.
   Clean up all associated imports and references; verify no breaking dependencies.

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

## AI Agent MCP Integration

Reference material for wiring an AI agent to RoBoT Brain as an MCP
server has moved to **`.agents/AI_AGENT_INTEGRATION.md`**. Consult it when
integrating with an MCP-compatible agent SDK; it is not needed for normal build/test/work
sessions.

## test_suite2 Coverage (FunctionRegistry)

The coverage gate cross-checks the server's `tools/list` against the test
suite's `FunctionRegistry` (in `.agents/scripts/test_suite2/src/function_registry/`). Key facts
every session should know:

- **Adding a tool to the server's `tools/list` is NOT enough to close
  coverage.** The tool must ALSO have a `TestRequirement` entry in
  `function_registry/` (with a matching `id` case in
  `.agents/scripts/test_suite2/src/comprehensive_test/argument_builder.rs`). The cross-check diffs server tool
  names vs the registry's `function_name` fields. Standalone tests in
  `.agents/scripts/test_suite2/src/tests/` do NOT count toward coverage.
- **Tool-list drift hazard:** each MCP handler maintains `tool_names()` /
  `get_tools()` / `execute_tool()` as THREE separate lists that must stay in
  sync. `get_tools()` feeds the RMCP `tools/list` response; if it omits an
  entry that the other two include, the tool is callable-but-unadvertised →
  flagged as a **phantom tool** by the cross-check (T1-19 root cause).
- **Validation choice for new registry tests:** use `IsSuccess(None)` for tools
  that succeed on a default/fake call, and `IsSuccess(Some("false"))` for tools
  that return an MCP error on a fake id. To pick correctly, probe the tool with
  a fake id via `.agents/scripts/test_suite2/target/release/test_suite --probe TOOL` and check `is_error`.
- **Current gate state: see `.agents/scripts/test_suite2/test_suite_report.json`.** Any status
  claim referencing test counts, warning counts, or completeness must be
  verified by running the gate this session. Stale counts are common — always
  re-run the gate, never trust a prior "done/GREEN" claim (Verify, Don't Trust).

## All tests live in test_suite2

- `#[cfg(test)]` modules inside robot_brain's `src/` are NOT the place for
  tests. All tests belong in `.agents/scripts/test_suite2/`. (test_suite2 tests robot_brain by
  spawning it as a subprocess over MCP/CLI — that is the project's testing
  model.)
- When verifying a feature end-to-end, add the test under
  `.agents/scripts/test_suite2/src/tests/` (e.g. `queue_durability.rs`), wire it into
  `tests/mod.rs` and dispatch it from `main.rs`. Add deps to
  `.agents/scripts/test_suite2/Cargo.toml` as needed (e.g. `rusqlite` bundled, `tempfile`).
- Cross-process/restart tests: copy the server binary into a `tempfile::tempdir()`
  (the server creates `robot_brain.db` beside `current_exe`), spawn via stdio
  MCP, manipulate the DB with `rusqlite`, restart, and assert via MCP tools.
  Remember the workflow gate: a fresh client must call `get_workflow` then
  `search_memory` before any substantive tool (see **Post-Compile** above for
  details; else `WORKFLOW_NOT_RETRIEVED` / `MEMORY_NOT_SEARCHED`).

> **Note:** Sections that moved out of this file: Roadmap → `.agents/PLAN.md`,
> Test Suite Notes → `.agents/TEST_SUITE_NOTES.md`. This file covers hard rules
> and startup workflow. All status/narrative lives in `.agents/`.

# STARTUP — Execute in order, no skipping

This section contains the session-start workflow. Follow steps 1-9 in order.
Everything before STARTUP contains the hard rules and conventions.

## 1. MCP workflow gate (required before any tool call)

Call `get_workflow` first — all MCP tools are blocked until this returns.

## 2. Load these files in full:
1. `AGENTS.md` — the hard rules (Incremental Workflow, Prerequisites, Build
   Commands, Post-Compile MCP connect, Strict Rust Coding Standards)
2. `.agents/PLAN.md` — the roadmap (TIER 2+3 tasks) + Definition of Done

## 3. Store session context in memory (Working Memory Protocol)

After reading startup files, call `store_memory` with:
- `memory_type`: "note"
- `tags`: ["startup", session date]
- `content`: Current state summary (what we're working on, gate status, next task)

This ensures session context survives across turns and can be retrieved later.
After any code change, store a note summarizing what changed.

## 4. Run the verify gate (must be green BEFORE any code change)

> **The wall:** `pwsh .agents/scripts/make.ps1 gate` runs test_suite2, which auto-builds
> robot_brain, connects via MCP, runs all tests + code analysis, and enforces
> the quality wall. It is installed as a pre-commit hook
> (`.agents/githooks/pre-commit`) so **no commit lands unless the gate is green**.
> One-time clone setup: `git config core.hooksPath .agents/githooks`.
>
> AGENTS.md is enforced as a **HARD wall**: 0 compiler warnings, 0 code-issues
> (no `#[allow]`, no `PublicNeverCalled`, no stubs), 0 untested tools. There is
> no ratchet and no baseline to ratchet against. A non-zero count blocks the
> commit; fix it by wiring the dead-code pub API into a real caller — never by
> `#[allow]` or `_`. `git commit --no-verify` is ONLY for the one-time
> bootstrap of the wall files themselves (.agents/scripts/, .agents/githooks/, Makefile)
> and trivial doc-only edits; never for `src/` changes.

If the toolchain is not installed, install it first (see AGENTS.md
"Prerequisites"). Then run the wall:

```bash
make gate
```

> **The gate command:** The full `test_suite2` command with CLI modes is in the
> **Build Commands** section above (line ~68). `make gate` runs the same thing
> via `.agents/scripts/gate.sh`.
>
> Note: `test_suite` is now at `.agents/scripts/test_suite2/` (not repo root).
> Old `test_suite/` at repo root has been removed.

The gate is green only when all tests pass AND 0 warnings / 0 code-issues /
0 untested tools. If any fails, fix the failure before doing anything else.
Do not "remember" a prior pass — actually run it this session.

**Running the gate in background:** The gate takes 17+ minutes to complete. Run it in the background using output redirection so you can continue checking tasks against the codebase while it runs:

```bash
./.agents/scripts/test_suite2/target/release/test_suite > .agents/scripts/test_suite2/test_suite_output.txt 2>&1
```

Then read the results when done: `cat .agents/scripts/test_suite2/test_suite_report.json` and `cat .agents/scripts/test_suite2/test_suite_output.txt | tail -100`. While waiting, continue verifying PLAN.md tasks by reading actual source code (never trust checkboxes — inspect the code).

## 5. Pick the next task (in order, do not skip ahead)

**STEP 1:** You are processing AGENTS.md (STARTUP section).

**STEP 2:** Open `.agents/PLAN.md` at line 1. Find the first pending task
(any task not yet deleted from PLAN.md — `[ ]`, `[?]`, `[!]`, etc. are all pending).
**That is the ONLY task you work on this session.**
Do NOT process any task after it. Do NOT skip ahead. Do NOT pick a different task.
The first pending task from the top IS the task — regardless of its marker style.

**STEP 3:** Work on that task. When done, re-run the gate. Commit + push.

**STEP 4:** Start the next task without user confirmation.

- **Coverage gate: run it to verify.** Any status claim referencing test counts,
  warning counts, or completeness must be verified by running the gate this
  session. Do not trust prior "done/GREEN" claims.

- **TIER 2 tasks:** See `.agents/t2_PLAN.md` for the detailed task list.
  Each task upgrades one existing subsystem to its v0.0.2 chapter.

- **TIER 3 tasks:** See `.agents/t3_PLAN.md` for the detailed task list.
  Each task builds one missing v0.0.2.1 subsystem.

## 6. Execute ONE change, then the gate.

- Make ONE change only (one file or one tightly-coupled set).
- Re-run the full verify gate (see **STARTUP #4** above). All four metrics must pass: tests (100%), compiler_warnings (0), code_issues (0), untested_tools (0).
- If the gate is red, fix it before claiming done. Never claim done without
  running the gate.
- Commit + push that one change.
- Report the result (what changed, gate status, commit hash).
- Report to the user for **code changes** (gate risk). For **documentation-only** changes (PLAN.md, README, AGENTS.md, CHANGELOG.md, etc.) proceed to the next task without confirmation — continue through ALL remaining tasks in PLAN.md until you hit a code-change task or the list is empty.

## 7. Periodic maintenance (check from time to time, not every session)

- **Large file refactor** (`.agents/LARGE_FILE_REFACTOR.md`): when an `.rs`
  file hits ~1000 lines mixing responsibilities, split it into a directory
  module per the pattern there. Run the candidates query occasionally.

## 8. Hard rules (from AGENTS.md — non-negotiable)

Full details in **Code Style Conventions** section (above, line ~173). Quick reference:
- NEVER batch multiple unrelated changes into one commit/step.
- NO `.unwrap()`, `.expect()`, `panic!()`, `assert!()`, `unreachable!()`.
- NO `todo!()`, `unimplemented!()`.
- NO `#[allow(...)]` / `#![allow(...)]` in `src/`.
- NO `#[cfg(test)]` in `src/` — tests live in `.agents/scripts/test_suite2/`.
- NO ignored variables (`let _x = ...`, `let _ = ...`, `|_| ...`).
- NO deleting code to bypass fixes. Follow the Dead Code Resolution Protocol (above, line ~248).
- The build and test suite enforce these. If flagged, fix it; do not silence it.

## 9. Context Watchdog Protocol

Long tasks lose state when the context window fills. Manage this proactively.

### Hard signals (any one is enough):
- The user tells you the editor's context indicator is past ~90%.
- You have completed 20+ turns or 500+ lines of output without a checkpoint.

### At hard signals, do NOT push further. Instead:
1. Run `store_memory` (per the Working Memory Protocol) with a checkpoint note: goal, files touched so far, current gate status, next concrete step, and any unresolved decisions.
2. Reply with a short "Checkpoint" summary in chat so session state is durable.
3. Ask the user whether to continue in this thread or open a fresh one.

**Persistence roles:** `store_memory` = session knowledge base (retrievable this session). `.agents/context_save.md` = cross-session handoff file (read at session start). Update both at every checkpoint.

**Checkpoint format** (store via `store_memory`, `memory_type: "note"`):
```
[CHECKPOINT] session_turns=N, cumulative_lines=M
- Goal: ...
- Files touched: ...
- Gate status: ...
- Next step: ...
```

Also update `.agents/context_save.md` with the same checkpoint info so the handoff file stays current.

### Hard halt (~90% of context full):
Use this when you cannot safely continue (hard signal triggered + no user direction). Do NOT start any new code change.
- Write (or update) the full `## Handoff State Package` to `.agents/context_save.md`:
  - **Goal**: one sentence
  - **Completed**: bulleted list of files changed + commit hashes if any
  - **In progress**: what was being done when halted
  - **Next step**: the single next action (concrete, not aspirational)
  - **Gate status**: last `make gate` result, or "not run this turn"
- **Do NOT delete context_save.md after reading.** Update it continuously as context changes.

**Context-save file as the top priority**: 
If `.agents/context_save.md` exists when you start a session, that file contains the current problem to resolve before anything else. 
Read it, execute the handoff (continue from where the previous session left off), then update it as you work.
Do not proceed to any other task until the context-save file has been resolved and removed.

# 1. OBJECTIVE
Read and apply AGENTS.md 100% end-to-end.

Take RoBoT Brain to **finished v0.0.2 → finished v0.0.2.1**, using
**small 5-10 minute increments** — see **Incremental Workflow Principle** (above,
line ~16) for the "one thing, verify, commit, push" pattern.

The target baseline is **`robot_architecture/v0.0.2.1/`** (33 chapters +
appendices A-E + `FINAL_ARCHITECTURE_SPEC.md`). First reach **finished v0.0.2**
(TIER 2), then **finished v0.0.2.1** (TIER 3).

## Mission

Resolve all known implementation bugs, complete partially implemented integrations.
Do NOT redesign the architecture unless a task explicitly requires it.

# 2. OPERATING RULES

1. Work on ONE task at a time — see **Incremental Workflow Principle** (above, line ~16).
2. Read the relevant source before modifying it — see **Working Memory Protocol** (above, line ~10).
3. Preserve existing architectural intent.
4. Do not solve a compiler warning by deleting functionality — see **Code Style Conventions** (above, line ~173).
5. Do not mark a task complete because code compiles — see **Verify, Don't Trust** (above, line ~26).
6. Every completed task must have a verification method — see **Verify, Don't Trust** (above, line ~26).
7. Run relevant tests after each change — see **Incremental Workflow Principle** (above, line ~16).
8. Update this file when a task changes state.
9. If implementation reveals an architectural conflict, FIX it according to documentation in robot_architecture folder.
10. Do not silently expand scope.
11. Task completion protocol — after a task passes the full verify gate (build + test_suite2 + gate green, end-to-end verified):
    a. Write a concise summary to `.agents/CHANGELOG.md` describing what was [ ], files changed, and verification results.
    b. Remove the task from its section in PLAN.md — delete the entire line from the file. Do not use ~~strike-through~~, do not change `[ ]` to `[x]`, do not leave stub detail. The line must be gone.
    c. Only then commit and push.
    Never write the CHANGELOG entry before the gate passes. Never let PLAN.md accumulate completed task detail.

    Important: if a task is still listed in PLAN.md, it is NOT complete -- regardless of any `[ ]` or `[x]` marker. Presence in PLAN.md means pending. The only signal of completion is removal from PLAN.md plus a CHANGELOG.md entry.

    Removing a task from PLAN.md without full end-to-end verification is not acceptable. Every removed task must be 100% complete and verified in the codebase (gate green, tests pass, no warnings). Do not remove tasks you have not actually finished.

    A task is NOT complete until its verification criteria pass. If the task cannot be safely completed:
    a. Mark it `[!]` or `[?]`.
    b. Explain why.
    c. Do NOT fabricate completion.
    d. Do NOT silently redesign another subsystem to bypass it.
12. Always make small, incremental edits — see **Incremental Workflow Principle** (above, line ~9). Never batch multiple unrelated changes into one edit. After each edit, verify it worked before proceeding. Large bulk rewrites lose information.

# 3. CONTEXT SUMMARY

## The blueprints

the location of these are important as if you had actually read agents.md you would know to check these before deleting a function

- **v0.0.2** -- `robot_architecture/RoBoT Architecture v0.0.2.md`. Intermediate
  upgrade: elevate Context + Conversation to first-class, add Data Contracts.
  TIER 2 conforms existing systems to this.
- **v0.0.2.1** -- `robot_architecture/v0.0.2.1/` (00.md-33.md + appendices). The
  FINAL architectural baseline. Adds Execution Engine, Tool Engine, Memory
  Hierarchy, Context Lifecycle, Retrieval Pipeline, Prompt Construction,
  Strategic Learning, Confidence System, Storage, Database Design, Background
  Workers, Security & Trust, Observability, Developer Interface/Control Plane,
  Configuration, Testing, Deployment. TIER 3 builds the missing subsystems.

## Current codebase state

- Workspace: two independent programs -- `robot_brain` (root, MCP server) and
  `.agents/scripts/test_suite2/` (E2E tests via MCP protocol).
- **Test count and warning count: see `.agents/scripts/test_suite2/test_suite_report.json`.** Do not
  trust prior counts — run the gate to verify.
- **Gate status:** 454/454 tests, 0 warnings, 0 issues. All known bugs
  resolved. Durable recovery verified. 0 stubs found.
- `#![allow]` / `#[allow]` in `src/`: **0** (clean).
- `self_check.rs` files: **0** (all removed/moved to TIER 2).
- **No v0.0.2/v0.0.2.1 new subsystems exist**: no Context Engine, Conversation
  Engine, Execution Engine, Tool Engine, Retrieval Pipeline, Prompt
  Construction, AI Runtime, multimodal, GUI, security/trust, observability.

## Constraints

- **Rust coding standards:** See **Code Style Conventions** (above, line ~173).
- **Incremental workflow:** See **Incremental Workflow Principle** (above, line ~16).
- **Verify, don't trust:** See **Verify, Don't Trust** (above, line ~26).
- **Large-file rule:** See **Large File Refactoring** (above, line ~276).
- **Local-first:** the cognitive architecture must work against cloud/external
  models first. AI Runtime (Candle) is built last, as an enhancement layer.
  The entire cognitive architecture operates without cloud dependency.

## The verify gate (run after EVERY increment)

See **Build Commands** section above (line ~64) for the full gate command
with CLI modes, or **STARTUP #4** (line ~371) for background execution tips.

# 4. APPROACH -- two remaining tiers of small increments

Work through two tiers in order. Each tier is a checklist of small
increments. Do them top-to-bottom, one at a time, with the verify gate green
between each.

- **TIER 2 -- Reach v0.0.2** (upgrade existing systems). Introduce Data Contracts,
  then upgrade each existing subsystem (Memory, Knowledge, Experience, Learning,
  Planner, Skills/Workflows, World Model, Personality) to its v0.0.2 chapter.
  **End state = finished v0.0.2.**
- **TIER 3 -- Reach v0.0.2.1** (add missing subsystems). Build the new engines
  from the v0.0.2.1 chapters: Execution, Tool, Memory Hierarchy, Context
  Lifecycle, Retrieval Pipeline, Prompt Construction, Strategic Learning,
  Confidence System, Storage/Database, Background Workers, Security & Trust,
  Observability, Developer Interface/Control Plane, Configuration, Testing,
  Deployment -- then AI Runtime (Candle), Multimodal, GUI last.
  **End state = finished v0.0.2.1.**

**Why this order:** no dead-code debt is carried into a refactor. Upgrade foundation systems (Data Contracts, Memory, Knowledge) before
the Context/Conversation engines so those engines consume real contract-shaped data, not stubs. Build the cognitive architecture against cloud/external models
first; AI Runtime (Candle) comes last as the local provider behind the `InferenceProvider` trait, and is the prerequisite for Multimodal.

**ONE TASK AT A TIME ENFORCEMENT**

- You MUST work on ONE task at a time. Never skip ahead.
- After completing a task: build → test → commit → push → do "Task completion protocol" → go to very first task in list.
- If no tasks remain: you are done.
- Never batch tasks. Never skip. Never assume a task is done — verify it in codebase.
