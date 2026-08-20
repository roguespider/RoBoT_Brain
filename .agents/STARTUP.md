# STARTUP — Do this every session, in order, no skipping

> This file is the call to action. Read it FIRST, then AGENTS.md, then
> PLAN.md. Do not start any code change until steps 1-3 are done.

## 1. MCP workflow gate (required before any tool call)

Call `get_workflow` first — all MCP tools are blocked until this returns.

## 2. Read these in full (do not skim)

1. This file (`.agents/STARTUP.md`)
2. `AGENTS.md` — the hard rules (Incremental Workflow, Prerequisites, Build
   Commands, Post-Compile MCP connect, Strict Rust Coding Standards)
3. `.agents/PLAN.md` — the roadmap + the "Next steps to finish v0.0.1" list

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

## 4. Pick the next task (in order, do not skip ahead)

Open `.agents/PLAN.md`. Find the FIRST unchecked `- [ ]` increment. Work tiers
in order: TIER 1 (finish v0.0.1) → TIER 2 (reach v0.0.2) → TIER 3 (reach
v0.0.2.1). Each increment is one ~10-15 min change.

- **Coverage gate is GREEN** (section 1E done). test_suite exits 0 (141/141 tests
  pass, 0 code issues, 0 warnings, 0 untested, 0 phantom). All 134 server tools
  are covered. Commits: b9b43ff (phantom fix), 6b7d036 (ACP tests), 7775ca1
  (remaining 41 tools).
- **TIER 1 is NOT fully done — 10 tasks remain** (sections 1B, 1C, 1D):
  - **1B. SQLite JobQueue (T1-09..T1-12):** add job_queue table+migration, wire
    enqueue/dequeue to SQLite, handle Lagged events, update startup verification.
  - **1C. Loop-health metrics (T1-13..T1-16):** add loop_latency,
    confidence_drift, promotion_throughput metrics, expose via get_system_status.
  - **1D. MCP→experience hook (T1-17..T1-18):** hook emit_experience_recorded
    into post-tool-execution dispatch, ensure idempotency.
  - Do these in order. Start with T1-09 (SQLite JobQueue migration).
- **Self_check removal is TIER 2 work** (the APIs they exercise have no other
  callers; deleting them in TIER 1 creates dead-code warnings). Do it during
  each system's TIER 2 upgrade. 8 self_check.rs files remain.

## 5. Execute ONE change, then the gate, then stop

- Make ONE change only (one file or one tightly-coupled set).
- Re-run the full verify gate (step 2). All three must pass.
- If the gate is red, fix it before claiming done. Never claim done without
  running the gate.
- Commit + push that one change.
- Report the result (what changed, gate status, commit hash).
- STOP and report to the user. Do not start the next task without confirmation.

## 6. Periodic maintenance (check from time to time, not every session)

- **Large file refactor** (`.agents/LARGE_FILE_REFACTOR.md`): when an `.rs`
  file hits ~1000 lines mixing responsibilities, split it into a directory
  module per the pattern there. Run the candidates query occasionally.

## 7. Hard rules (from AGENTS.md — non-negotiable)

- NEVER batch multiple unrelated changes into one commit/step.
- NO `.unwrap()`, `.expect()`, `panic!()`, `assert!()`, `unreachable!()`.
- NO `todo!()`, `unimplemented!()`.
- NO `#[allow(...)]` / `#![allow(...)]` anywhere in `src/`.
- NO ignored variables (`let _x = ...`, `let _ = ...`, `|_| ...`).
- NO deleting code to bypass fixes. Follow the Dead Code Resolution Protocol
  (AGENTS.md): implement if the architecture describes it, delete only if
  confirmed absent.
- The build and test suite enforce these. If the compiler or test suite flags
  a violation, the change is not done. Fix it; do not silence it.
