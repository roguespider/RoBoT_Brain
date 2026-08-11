#!/usr/bin/env bash
#
# scripts/gate.sh — the verify gate as a hard wall.
#
# Runs the three required checks IN ORDER and aborts on the first failure:
#   1. cargo build --release -p robot_brain      (build OK, 0 new warnings)
#   2. python3 .agents/live_test/live_test_all.py (connect to robot_brain; 54/54)
#   3. test_suite                                  (tests pass; exit 0)
#
# Installed as a pre-commit hook (see .githooks/pre-commit) so that NO commit
# lands unless all three pass. This is the brick wall: it removes the option
# of skipping the connect step or committing a red gate.
#
# Run by hand: `make gate` or `scripts/gate.sh`.
set -euo pipefail

# Resolve the repo root regardless of where the script is invoked from.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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

# ---- 1. Build robot_brain ----------------------------------------------------
step "1/3" "cargo build --release -p robot_brain"
if ! cargo build --release -p robot_brain; then
    fail "build failed"
fi
ok "build"

# ---- 2. Live test: actually connect to robot_brain --------------------------
# This is the step that cannot be skipped. It spawns robot_brain as a
# subprocess and exercises every tool category. If the binary does not start
# or any tool errors, the gate is red.
step "2/3" "live_test_all.py (connect to robot_brain)"
if [ ! -x "target/release/robot_brain" ]; then
    fail "robot_brain binary missing - run cargo build first"
fi
if ! python3 .agents/live_test/live_test_all.py; then
    fail "live test did not pass (expected 54/54)"
fi
ok "live test (54/54)"

# ---- 3. test_suite -----------------------------------------------------------
# Builds test_suite in its own directory (separate project) and runs it.
#
# AGENTS.md is a hard wall: 0 compiler warnings, 0 code-issues, 0 untested
# tools. The suite exits 1 if any test fails OR if has_issues() (warnings/
# code-quality). We split the two concerns:
#   - Test pass count is a HARD wall: any failing test aborts the gate.
#   - Quality counts (warnings/code-issues/untested) are a HARD wall at 0:
#     any non-zero count aborts the gate. Fix by wiring the dead-code pub
#     API into a real caller; never #[allow] or `_`.
step "3/3" "test_suite"
(
    cd test_suite
    if ! cargo build --release; then
        fail "test_suite build failed"
    fi
    # Run the suite. It may exit 1 due to has_issues() even when all tests
    # pass; we parse the JSON report instead of trusting the exit code.
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
if ! python3 scripts/gate_quality.py "$REPORT"; then
    fail "quality wall violated (fix by wiring dead-code pub APIs into real callers)"
fi
ok "test_suite (passed=$PASS, 0 warnings/0 issues/0 untested)"

printf "\n${GRN}=== GATE GREEN - commit permitted ===${RST}\n"
