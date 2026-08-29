# RoBoT Brain - Agent Memory

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
- Any status claim in README, PLAN.md, CHANGELOG.md, or `.agents/*.md` that
  references test counts, warning counts, or completeness must be backed by
  a same-day gate run. If the gate was not run this session, soften the claim:
  - Instead of "0 warnings" → "pending gate verification"
  - Instead of "148/148 tests pass" → "148 tests (unverified, pending gate)"
  - Never hardcode gate counts in task notes without a date
- The single source of truth is `test_suite/test_suite_report.json`.
- When asked "is T1-NN done?" or "is X working 100%?": Do NOT read the PLAN.md
  checkbox and repeat it. Checkboxes lie. INSPECT THE CODEBASE: `grep`/`find`
  for the actual change, read the code, confirm the API exists and is wired.
- For "working 100%" claims, the done-when criteria matter (e.g. T1-10 =
  "queue survives a process restart"). Wire a real end-to-end test in
  test_suite that exercises that criterion, not just "the function exists".
- Report what is actually true, including gaps the PLAN glosses over.

## Build Commands

**test_suite auto-builds robot_brain. Never run
`cargo build -p robot_brain` or `cargo build --release -p robot_brain`
separately.**

- test_suite and robot_brain are two separate, independent projects. test_suite
  does NOT import or link robot_brain's source. It spawns robot_brain as a
  subprocess via MCP.
- When working on test_suite, NEVER touch `src/` (robot_brain's source). When
  working on robot_brain, NEVER touch `test_suite/src/`.
- Running a separate `cargo build -p robot_brain` wastes time and can mask
  discrepancies between what you built and what test_suite built.

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

#### Quality Gate (MANDATORY before any commit)

Run `cd test_suite && cargo build --release && ./target/release/test_suite --gate`.
All four metrics must pass: `tests` (100%), `compiler_warnings` (0),
`code_issues` (0), `untested_tools` (0). See README "Quality Gate" section
for the full table and the JSON-report triage recipe.

The structured report at `test_suite/test_suite_report.json` has an `issues[]`
array; each entry has `kind`/`category`/`file`/`line`/`message`/`suggested_action`.
Use `python3 -c` + `collections.Counter` to group warnings by message/file for
triage. Fix dead-code first (highest signal), then mechanical clippy lints.

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

## Build Efficiency (parallelize work)

When running builds, **start the build in the background first**, then use the
waiting time to work on other tasks (reading code, updating memory, planning).

Pattern:
1. Start `cd test_suite && cargo build --release` in background
2. While waiting, review related code, read documentation, plan next steps
3. When build completes, review results and continue

For quick compiler feedback (no test execution), prefer
`cd test_suite && cargo check --release` over full builds. This gives faster
turnaround on iterative changes.

## Post-Compile: Connect to robot_brain MCP/ACP

**IMPORTANT (User Requirement):** test_suite auto-builds robot_brain and
connects via MCP, but the AI agent MUST also connect to the running
robot_brain MCP/ACP server directly to test it. Do not rely solely on
test_suite — connect yourself as a client.

**Two ways to connect (do NOT hand-write a new MCP client):**

1. **`test_suite --probe TOOL`** — Rust, built into the test suite. Introspects a tool's live `inputSchema` (required/optional params). Fastest way to discover what a tool expects.

```bash
# Schema introspection (Rust) — discover a tool's required fields:
cd test_suite && ./target/release/test_suite --probe register_agent

# Quick smoke check — list all server tools + required fields:
cd test_suite && ./target/release/test_suite --list
```

The `robot-brain` skill (`.agents/skills/robot-brain/skill.md`) documents the tool catalog and the workflow gate. Steps after a successful build:
1. Invoke the `robot-brain` skill
2. Run `test_suite` (full suite) or `test_suite --probe TOOL` (schema lookup) for targeted verification
3. Verify key tools: `store_memory`, `search_memory`, `list_memories`, `create_plan`, `list_plans`, `create_workflow`, `start_workflow`, `query_knowledge`, `record_experience`
4. ACP tools: `route_acp_message`, `register_agent`, `list_acp_agents` (note: `list_agents` does not exist — the real tool is `list_acp_agents`)
5. Discovery: `test_suite --list` confirms all tools are available
6. Report which tools work and which fail

**Workflow gate (required before any substantive tool call):** the server returns `WORKFLOW_NOT_RETRIEVED` until `get_workflow` is called, then `MEMORY_NOT_SEARCHED` until `search_memory` is called. The Rust `TestMcpClient::new()` in `test_suite/src/main.rs` handles both automatically.

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

**Never use `#[cfg(test)]` in production source.** It causes code-quality issues
and the quality gate flags it. Tests belong in `test_suite/` (as MCP flow tests)
where they exercise the real public surface.

**Fixing unused type warnings:** Many types (like `SimpleAgent`, `AcpCapability`,
ACP message builders) are defined for testing/future use but unused in production.

1. **Move unused types to `test_suite/`** — don't expose unused types in the
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
  a fake id via `test_suite --probe TOOL` and check `is_error`.
- **Probing tip:** extract all tool schemas at once from the live server:
  ```bash
  cd test_suite && ./target/release/test_suite --list
  ```
- **Current gate state: see `test_suite/test_suite_report.json`.** Any status
  claim referencing test counts, warning counts, or completeness must be
  verified by running the gate this session. Stale counts are common — always
  re-run the gate, never trust a prior "done/GREEN" claim (Verify, Don't Trust).

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

> **The sections below have moved out of this file to reduce size:**
> - **Roadmap to v2.0 (Architecture Conformance Work)** — the P0-P4 status
>   tracker, Definition of Done, Resume Here, and verified-state snapshot —
>   moved to **`.agents/PLAN.md`** section 6 ("v0.0.1 CONFORMANCE WORK").
>   Forward planning (STAGE 1-3) is in the same file, sections 4-5.
> - **Test Suite Improvements (Diagnosability & Coverage)** — coverage gaps,
>   JSON report usage, future test work — moved to
>   **`.agents/TEST_SUITE_NOTES.md`**.
>
> This file now ends at the hard rules above. All status/narrative/
> reference lives in `.agents/` so the rules stay focused
> every session.

# STARTUP — Execute in order, no skipping

## 1. MCP workflow gate (required before any tool call)

Call `get_workflow` first — all MCP tools are blocked until this returns.

## 2. Load these files in full:
1. `AGENTS.md` — the hard rules (Incremental Workflow, Prerequisites, Build
   Commands, Post-Compile MCP connect, Strict Rust Coding Standards)
2. `.agents/PLAN.md` — the roadmap + the "Next steps to finish v0.0.1" list

## 3. Store session context in memory (Working Memory Protocol)

After reading startup files, call `store_memory` with:
- `memory_type`: "note"
- `tags`: ["startup", session date]
- `content`: Current state summary (what we're working on, gate status, next task)

This ensures session context survives across turns and can be retrieved later.
After any code change, store a note summarizing what changed.

## 4. Run the verify gate (must be green BEFORE any code change)

> **The wall (use it):** `make gate` runs test_suite, which auto-builds
> robot_brain, connects via MCP, runs all tests + code analysis, and enforces
> the quality wall. It is installed as a pre-commit hook
> (`.agents/githooks/pre-commit`) so **no commit lands unless the gate is green**.
> One-time clone setup: `make hooks` (sets `core.hooksPath = .agents/githooks`).
>
> AGENTS.md is enforced as a **HARD wall**: 0 compiler warnings, 0 code-issues
> (no `#[allow]`, no `PublicNeverCalled`, no stubs), 0 untested tools. There is
> no ratchet and no baseline to ratchet against. A non-zero count blocks the
> commit; fix it by wiring the dead-code pub API into a real caller — never by
> `#[allow]` or `_`. `git commit --no-verify` is ONLY for the one-time
> bootstrap of the wall files themselves (.agents/scripts/, .agents/githooks/, Makefile) and
> trivial doc-only edits; never for `src/` changes.

If the toolchain is not installed, install it first (see AGENTS.md
"Prerequisites"). Then either run the wall:

```bash
make gate
```

…or run test_suite by hand (the wall does exactly this):

```bash
cd test_suite && cargo build --release && ./target/release/test_suite
```

test_suite auto-builds robot_brain, spawns it as a subprocess, connects
via MCP, runs all tests + code analysis, and writes
`test_suite/test_suite_report.json`. The gate is green only when all
tests pass AND 0 warnings / 0 code-issues / 0 untested tools. If any fails,
fix the failure before doing anything else. Do not "remember" a prior pass —
actually run it this session.

**Running the gate in background:** The gate takes 17+ minutes to complete. Run it in the background using output redirection so you can continue checking tasks against the codebase while it runs:

```bash
cd test_suite && ./target/release/test_suite > test_suite_output.txt 2>&1
```

Then read the results when done: `cat test_suite/test_suite_report.json` and `cat test_suite_output.txt | tail -100`. While waiting, continue verifying PLAN.md tasks by reading actual source code (never trust checkboxes — inspect the code).

## 5. Pick the next task (in order, do not skip ahead)

**STEP 1:** You are processing AGENTS.md (STARTUP section).

**STEP 2:** Open `.agents/PLAN.md` at line 1. Find the first
`- [ ]` task marker. **That is the ONLY task you work on this session.**
Do NOT process any task after it. Do NOT skip ahead. Do NOT pick a different task.
The first unchecked task from the top IS the task — regardless of its marker.

**STEP 3:** Work on that task. When done, re-run the gate. Commit + push.

**STEP 4:** Start the next task without user confirmation.

- **Coverage gate: run it to verify.** Any status claim referencing test counts,
  warning counts, or completeness must be verified by running the gate this
  session. Do not trust prior "done/GREEN" claims.
- **TIER 1 tasks: see CHANGELOG.md for details on each task.**
- **Remaining blocker: P1 quality gate.** See PLAN.md P1-001/P1-002.
- **Self_check removal is TIER 2 work** (the APIs they exercise have no other
  callers; deleting them in TIER 1 creates dead-code warnings). Do it during
  each system's TIER 2 upgrade. 8 self_check.rs files remain.

## 6. Execute ONE change, then the gate, then stop

- Make ONE change only (one file or one tightly-coupled set).
- Re-run the full verify gate (step 2). All three must pass.
- If the gate is red, fix it before claiming done. Never claim done without
  running the gate.
- Commit + push that one change.
- Report the result (what changed, gate status, commit hash).
- STOP and report to the user for **code changes** (gate risk). For **documentation-only** changes (PLAN.md, README, AGENTS.md, CHANGELOG.md, etc.) proceed to the next task without confirmation — continue through ALL remaining tasks in PLAN.md until you hit a code-change task or the list is empty.

## 7. Periodic maintenance (check from time to time, not every session)

- **Large file refactor** (`.agents/LARGE_FILE_REFACTOR.md`): when an `.rs`
  file hits ~1000 lines mixing responsibilities, split it into a directory
  module per the pattern there. Run the candidates query occasionally.

## 8. Hard rules (from AGENTS.md — non-negotiable)

- NEVER batch multiple unrelated changes into one commit/step.
- NO `.unwrap()`, `.expect()`, `panic!()`, `assert!()`, `unreachable!()`.
- NO `todo!()`, `unimplemented!()`.
- NO `#[allow(...)]` / `#![allow(...)]` anywhere in `src/`.
- NO `#[cfg(test)]` in `src/`. Tests live in `test_suite/`, not in production source.
- NO ignored variables (`let _x = ...`, `let _ = ...`, `|_| ...`).
- NO deleting code to bypass fixes. Follow the Dead Code Resolution Protocol
  (AGENTS.md): implement if the architecture describes it, delete only if
  confirmed absent.
- The build and test suite enforce these. If the compiler or test suite flags
  a violation, the change is not done. Fix it; do not silence it.

## 9. Context Watchdog Protocol

Long tasks lose state when the context window fills. Manage this proactively.

**While working**, watch for these signals (any one is enough):
- You have completed more than ~10 turns in this session with heavy tool output.
- A single tool returned more than ~1000 lines, or cumulative tool output across recent turns exceeds several thousand lines.
- The user tells you the editor's context indicator is past ~90%.
- You are about to start a subtask that will itself be substantial (e.g. "refactor this 1000-line file").

**At any of those signals**, do NOT push further. Instead:
1. Run `store_memory` (per the Working Memory Protocol) with a checkpoint note: goal, files touched so far, current gate status, next concrete step, and any unresolved decisions.
2. Reply with a short "Checkpoint" summary in chat so session state is durable.
3. Ask the user whether to continue in this thread or open a fresh one.

**Hard halt (~90% of context full)**:
- Do not start any new code change.
- Write the full `## Handoff State Package` to `.agents/context_save.md`:
  - **Goal**: one sentence
  - **Completed**: bulleted list of files changed + commit hashes if any
  - **In progress**: what was being done when halted
  - **Next step**: the single next action (concrete, not aspirational)
  - **Gate status**: last `make gate` result, or "not run this turn"
- Delete the file after the user has read it — once the agent loads `.agents/context_save.md` at the start of a session and has acted on its contents, delete it so it does not persist as stale state.

**Context-save file as the top priority**: 
If `.agents/context_save.md` exists when you start a session, that file contains the current problem to resolve before anything else. 
Read it, execute the handoff (continue from where the previous session left off), then delete the file. 
Do not proceed to any other task until the context-save file has been resolved and removed.
