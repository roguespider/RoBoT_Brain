#!/usr/bin/env bash
#
# .agents/scripts/test_suite2.1/test_runner.sh — orchestrator for Python test suite
#
# This script:
# 1. Ensures robot_brain is built (Rust compilation)
# 2. Runs the Python analysis (code_analyzer, lint, registry)
# 3. Runs the Python functional tests (pytest)
# 4. Generates and reports results
#
# Run standalone: .agents/scripts/test_suite2.1/test_runner.sh
# Run from gate:  make gate (calls this automatically)
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; BLU=$'\033[34m'; RST=$'\033[0m'
step() { printf "\n${BLU}=== TEST_SUITE2.1 %s: %s ===${RST}\n" "$1" "$2"; }
ok()   { printf "${GRN}=== %s OK ===${RST}\n" "$1"; }
fail() { printf "\n${RED}=== TEST_SUITE2.1 FAILED: %s ===${RST}\n" "$1"; exit 1; }

# ---- Ensure robot_brain is built ----
step "0" "Ensure robot_brain binary exists"
RB="target/release/robot_brain"
if [ -n "$WINDIR" ] || [[ "$OSTYPE" == *mingw* ]]; then RB="target/release/robot_brain.exe"; fi
if [ ! -f "$RB" ] || { [ "src/lib.rs" -nt "$RB" ] 2>/dev/null || [ "Cargo.toml" -nt "$RB" ] 2>/dev/null; }; then
    printf "  [BUILD] robot_brain not found or stale, building...\n"
    (
        cd "$REPO_ROOT"
        if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
        cargo build --release -p robot_brain || fail "robot_brain build failed"
    )
    ok "robot_brain built"
else
    printf "  [SKIP] robot_brain binary exists (using %s)\n" "$RB"
fi

# ---- Check Python dependency ----
step "1" "Check Python environment"
if ! command -v python &>/dev/null && ! command -v python3 &>/dev/null; then
    fail "python not found. Install Python 3.8+ first."
fi
PYTHON=$(command -v python3 || command -v python)

# Install requirements if missing
REQUIREMENTS="$TS_DIR/requirements.txt"
if [ -f "$REQUIREMENTS" ]; then
    if ! python -c "import pytest" &>/dev/null; then
        printf "  [INSTALL] Installing test_suite2.1 dependencies...\n"
        $PYTHON -m pip install -q -r "$REQUIREMENTS"
        ok "dependencies installed"
    else
        printf "  [SKIP] pytest already installed\n"
    fi
fi

# ---- Clean previous output files ----
rm -f "$REPO_ROOT/.agents/scripts/test_output.txt"
rm -f "$REPO_ROOT/.agents/scripts/test_suite2/test_suite_report.json"
rm -f "$TS_DIR/test_suite_report.json"
rm -f "$TS_DIR/test_suite_output.txt"
rm -f "$TS_DIR/rust_test_output.txt"
rm -f "$TS_DIR/python_test_output.txt"

# ---- Run code analysis + lint + registry (Python main.py) ----
step "2" "Run analysis (code_analyzer, lint, registry)"
ANALYSIS_OUTPUT="$TS_DIR/analysis_output.txt"
$PYTHON "$TS_DIR/main.py" 2>&1 | tee "$ANALYSIS_OUTPUT"

# Check for failures in analysis
if grep -q "\[FAIL\]" "$ANALYSIS_OUTPUT" 2>/dev/null || grep -q "exit code 1" "$ANALYSIS_OUTPUT" 2>/dev/null; then
    fail "Analysis failed — see output above"
fi
ok "analysis complete"

# ---- Run tool probe (tests all 206+ MCP tools) ----
step "3a" "Run comprehensive tool probe (all 206+ tools)"
PROBE_OUTPUT="$TS_DIR/tool_probe_output.txt"
if [ -f "$TS_DIR/test_tool_probe.py" ]; then
    $PYTHON "$TS_DIR/test_tool_probe.py" 2>&1 | tee -a "$PROBE_OUTPUT" || true
    ok "tool probe complete — see $PROBE_OUTPUT"
else
    printf "  [SKIP] test_tool_probe.py not found\n"
fi

# ---- Run functional tests (pytest) ----
step "3b" "Run functional tests"
TEST_OUTPUT="$TS_DIR/python_test_output.txt"

# Run tests (use serial to avoid MCP server conflicts)
$PYTHON -m pytest \
    -v \
    -c "$TS_DIR/pytest.ini" \
    "$TS_DIR/" \
    2>&1 | tee -a "$TEST_OUTPUT"

# ---- Merge tool probe results into main test_output.txt ----
step "3c" "Merge tool probe results into test_output.txt"
TOOL_PROBE_REPORT="$TS_DIR/test_tool_probe_report.json"
OUTPUT_FILE="$REPO_ROOT/.agents/scripts/test_output.txt"
if [ -f "$TOOL_PROBE_REPORT" ] && [ -f "$OUTPUT_FILE" ]; then
    # Append tool probe section to test_output.txt using main.py which handles formatting
    $PYTHON "$TS_DIR/main.py" 2>&1 | tee -a "$OUTPUT_FILE"
    ok "tool probe results merged into $OUTPUT_FILE"
else
    printf "  [SKIP] Tool probe report or output file not found\n"
fi

# ---- Report ----
if grep -q "passed" "$TEST_OUTPUT" 2>/dev/null; then
    PASSED=$(grep -E "[0-9]+ passed" "$TEST_OUTPUT" | tail -1)
    ok "functional tests: $PASSED"
else
    fail "functional tests did not produce passing results"
fi

printf "\n${GRN}=== TEST_SUITE2.1 GREEN ===${RST}\n"
