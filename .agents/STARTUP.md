# STARTUP — Do this every session, in order, no skipping

> This file is the call to action. Read it FIRST, then AGENTS.md, then
> PLAN.md. Do not start any code change until steps 1-3 are done.

## 1. Read these in full (do not skim)

1. This file (`.agents/STARTUP.md`)
2. `AGENTS.md` — the hard rules (Incremental Workflow, Prerequisites, Build
   Commands, Post-Compile MCP connect, Strict Rust Coding Standards)
3. `.agents/PLAN.md` — the roadmap + the "Next steps to finish v0.0.1" list

## 2. Run the verify gate (must be green BEFORE any code change)

If the toolchain is not installed, install it first (see AGENTS.md
"Prerequisites"). Then:

```bash
cargo build --release -p robot_brain          # must finish 0 warnings
python3 .agents/live_test/live_test_all.py     # must be 54/54
cd test_suite && cargo build --release && ./target/release/test_suite  # 333/333, 0 code-quality
```

All three must pass. If any fails, fix the failure before doing anything else.
Do not "remember" a prior pass — actually run them this session.

## 3. Pick the next task (in order, do not skip ahead)

Open `.agents/PLAN.md`. Find the FIRST unchecked `- [ ]` increment starting from
TIER 1 (T1-01 ...). Work tiers in order: TIER 1 (finish v0.0.1) → TIER 2 (reach
v0.0.2) → TIER 3 (reach v0.0.2.1). Each increment is one ~10-15 min change.

- **Next increment right now:** `T1-01` — remove `src/planner/self_check.rs`
  (wire a planner API into an MCP tool, then delete the self_check). See
  `.agents/PLAN.md` section 4 (1A).
- **Right after self_check removal (T1-01..08): do 1E (T1-19..29) to close the
  coverage gate.** The gate is currently red ONLY because of coverage gaps
  (91/91 tests pass, 0 code issues) — 50 untested tools + 6 phantom embedding
  tools. Closing 1E makes the suite exit 0, so every later increment's verify
  step is honest.
- Do not start TIER 2 until TIER 1 is fully checked off.

## 4. Execute ONE change, then the gate, then stop

- Make ONE change only (one file or one tightly-coupled set).
- Re-run the full verify gate (step 2). All three must pass.
- If the gate is red, fix it before claiming done. Never claim done without
  running the gate.
- Commit + push that one change.
- Report the result (what changed, gate status, commit hash).
- STOP and report to the user. Do not start the next task without confirmation.

## 5. Periodic maintenance (check from time to time, not every session)

- **Large file refactor** (`.agents/LARGE_FILE_REFACTOR.md`): when an `.rs`
  file hits ~1000 lines mixing responsibilities, split it into a directory
  module per the pattern there. Run the candidates query occasionally.

## 6. Hard rules (from AGENTS.md — non-negotiable)

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
