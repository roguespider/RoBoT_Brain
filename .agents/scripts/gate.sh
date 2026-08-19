#!/usr/bin/env bash
#
# .agents/scripts/gate.sh — the verify gate as a hard wall.
#
# Runs test_suite, which auto-builds robot_brain, spawns it as a subprocess,
# connects via MCP, runs all tests + code analysis, and writes a JSON report.
# The gate is green only when all tests pass AND 0 warnings / 0 code-issues /
# 0 untested tools.
#
# Installed as a pre-commit hook (see .agents/githooks/pre-commit) so that NO commit
# lands unless the gate passes.
#
# Run by hand: `make gate` or `.agents/scripts/gate.sh`.
set -euo pipefail

# Resolve the repo root regardless of where the script is invoked from.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Make the Rust toolchain available (no-op if already on PATH).
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'

step() { printf "\n${YLW}=== GATE %s: %s ===${RST}\n" "$1" "$2"; }
fail() { printf "\n${RED}=== GATE RED: %s ===${RST}\n" "$1"; exit 1; }
ok()   { printf "${GRN}=== %s OK ===${RST}\n" "$1"; }

# ---- 1. test_suite (auto-builds robot_brain, connects via MCP, runs tests) -
step "1/1" "test_suite (build + connect + test + code analysis)"
(
    cd test_suite
    # Clean robot_brain before building to ensure a fresh build.
    cargo clean -p robot_brain || true
    if ! cargo build --release; then
        fail "test_suite build failed"
    fi
    # Run the suite. It may exit non-zero due to code-quality issues even
    # when all tests pass; we parse the JSON report for the final verdict.
    ./target/release/test_suite || true
)
REPORT="$REPO_ROOT/test_suite/test_suite_report.json"
if [ ! -f "$REPORT" ]; then
    fail "test_suite did not produce a report"
fi
# Hard wall: every test must pass (no failures, no errors).
PASS=$(python3 -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('passed',0))")
FAIL=$(python3 -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('failed',0)+s.get('errors',0))")
if [ "$FAIL" != "0" ]; then
    fail "test_suite has failing tests (passed=$PASS, failed=$FAIL)"
fi
# Hard wall: AGENTS.md requires 0 warnings, 0 code-issues, 0 untested tools.
# test_suite --gate reads the JSON report and enforces this (pure Rust,
# no Python dependency).
if ! ./test_suite/target/release/test_suite --gate; then
    fail "quality wall violated (fix by wiring dead-code pub APIs into real callers)"
fi
ok "test_suite (passed=$PASS, 0 warnings/0 issues/0 untested)"

printf "\n${GRN}=== GATE GREEN - commit permitted ===${RST}\n"
