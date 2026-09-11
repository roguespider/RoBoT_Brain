#!/usr/bin/env bash
#
# .agents/scripts/gate.sh — the verify gate as a hard wall.
#
# All testing infrastructure consolidated in .agents/scripts/test_suite2/:
# - Rust test_suite (code quality, coverage gate, tool introspection)
# - Python test_suite2 (functional, end-to-end, integration tests)
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

TS_DIR="$REPO_ROOT/.agents/scripts/test_suite2"
REPORT="$TS_DIR/test_suite_report.json"

# ---- 0. Build robot_brain (required for both Python and Rust tests) ----
step "0/4" "Ensure robot_brain binary exists"
if [ ! -f "target/release/robot_brain" ] && [ ! -f "target/release/robot_brain.exe" ]; then
    printf "  [BUILD] Building robot_brain...\n"
    cargo build --release -p robot_brain || fail "robot_brain build failed"
    ok "robot_brain built"
else
    printf "  [SKIP] robot_brain binary exists\n"
fi

# ---- 1. Build Rust test_suite ----
step "1/4" "Build Rust test_suite"
(
    cd "$TS_DIR"
    if git diff --quiet HEAD -- test_suite2/src 2>/dev/null || [ -n "$(git status --porcelain -- test_suite2/src 2>/dev/null)" ]; then
        printf "[REBUILD] test_suite2 source changed, rebuilding\n"
        cargo build --release || fail "test_suite2 build failed"
    else
        printf "[SKIP] test_suite2 binary up-to-date\n"
    fi
    # Build robot_brain (test_suite auto-builds it)
    if [ ! -f "$REPO_ROOT/target/release/robot_brain" ] && [ ! -f "$REPO_ROOT/target/release/robot_brain.exe" ]; then
        printf "[BUILD] robot_brain needed, building...\n"
        cargo build --release -p robot_brain || true
    fi
    ./target/release/test_suite || true
)

# ---- 2. Run Python test_suite2 ----
step "2/4" "Python functional tests"
if [ -f "$TS_DIR/test_runner.sh" ]; then
    if ! "$TS_DIR/test_runner.sh" 2>&1; then
        fail "test_suite2 functional tests FAILED"
    fi
    ok "test_suite2 functional tests passed"
else
    printf "${YLW}[SKIP] test_suite2 Python not yet implemented${RST}\n"
fi

# ---- 3. Quality wall ----
if [ ! -f "$REPORT" ]; then
    fail "test_suite did not produce a report at $REPORT"
fi
PASS=$(python -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('passed',0))")
FAIL=$(python -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('failed',0)+s.get('errors',0))")
WARN=$(python -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('compiler_warnings',0))")
ISSUES=$(python -c "import json,sys; d=json.load(open('$REPORT')); s=d.get('summary',{}); print(s.get('code_issues',0))")
UNTESTED=$(python -c "import json,sys; d=json.load(open('$REPORT')); c=d.get('coverage',{}); print(len(c.get('untested_tools',[])))")

if [ "$FAIL" != "0" ]; then
    fail "test_suite has failing tests (passed=$PASS, failed=$FAIL)"
fi
if [ "$WARN" != "0" ] || [ "$ISSUES" != "0" ] || [ "$UNTESTED" != "0" ]; then
    fail "quality wall violated: warnings=$WARN issues=$ISSUES untested=$UNTESTED"
fi
ok "test_suite (passed=$PASS, warnings=$WARN issues=$ISSUES untested=$UNTESTED)"

# ---- 4. Report summary ----
step "3/4" "Quality wall"
ok "all checks passed"

printf "\n${GRN}=== GATE GREEN - commit permitted ===${RST}\n"
printf "  Rust tests:   $PASS passed, $WARN warnings, $ISSUES issues, $UNTESTED untested\n"
printf "  Python:       see python_test_output.txt\n"

# ---- Helper: clean-rebuild test_suite2 ----
clean() {
    printf "\n${YLW}=== Cleaning test_suite2 build artifacts ===${RST}\n"
    local TS_DIR="$REPO_ROOT/.agents/scripts/test_suite2"
    if [ -d "$TS_DIR/__pycache__" ]; then
        rm -rf "$TS_DIR/__pycache__"
        printf "  Removed __pycache__\n"
    fi
    if [ -d "$TS_DIR/.pytest_cache" ]; then
        rm -rf "$TS_DIR/.pytest_cache"
        printf "  Removed .pytest_cache\n"
    fi
    if [ -d "$TS_DIR/target" ]; then
        rm -rf "$TS_DIR/target"
        printf "  Removed target/\n"
    fi
    printf "${YLW}=== Rebuilding test_suite2 from scratch ===${RST}\n"
    (cd "$TS_DIR" && cargo clean && cargo build --release)
    printf "${GRN}=== Clean rebuild complete ===${RST}\n"
}
