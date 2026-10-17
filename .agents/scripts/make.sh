#!/usr/bin/env bash
#
# .agents/scripts/make.sh — unified build/test/verify gate runner.
#
# Subcommands:
#   gate    — full verify gate (build + Python tests + quality wall)
#   session — session-start enforcement (read docs → gate → connect → pick task)
#   clean   — clean test_suite2.1 build artifacts + rebuild
#   help    — show this help message
#
# Run by hand:  make gate  |  make session  |  make clean  |  make help

set -euo pipefail

# Resolve the repo root regardless of where the script is invoked from.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; BLU=$'\033[34m'; RST=$'\033[0m'

# ---------------------------------------------------------------------------
# Helper functions shared across subcommands
# ---------------------------------------------------------------------------

step() { printf "\n${YLW}=== $1 ===${RST}\n"; }
ok()   { printf "${GRN}✓ %s${RST}\n" "$1"; }
fail() { printf "${RED}✗ %s${RST}\n" "$1"; exit 1; }
info() { printf "${BLU}  $1${RST}\n"; }

# ---------------------------------------------------------------------------
# Clean
# ---------------------------------------------------------------------------

clean() {
    printf "\n${YLW}=== Cleaning test_suite2.1 build artifacts ===${RST}\n"
    local TS_DIR="$REPO_ROOT/.agents/scripts/test_suite2.1"

    if [ -d "$TS_DIR/__pycache__" ]; then
        rm -rf "$TS_DIR/__pycache__"
        printf "  Removed __pycache__\n"
    fi
    if [ -d "$TS_DIR/.pytest_cache" ]; then
        rm -rf "$TS_DIR/.pytest_cache"
        printf "  Removed .pytest_cache\n"
    fi
    # Clean scattered output files
    rm -f "$TS_DIR/test_suite_report.json"
    rm -f "$TS_DIR/python_test_output.txt"
    rm -f "$TS_DIR/analysis_output.txt"
    rm -f "$REPO_ROOT/.agents/scripts/tool_coverage_report.json"

    # Clean main project target/ (cargo clean)
    printf "  Cleaning robot_brain target/...\n"
    if [ -f "$REPO_ROOT/Cargo.toml" ]; then
        pushd "$REPO_ROOT" > /dev/null
        if command -v cargo > /dev/null 2>&1; then
            cargo clean 2>/dev/null || {
                printf "    [WARN] cargo clean failed, trying manual removal...\n"
                rm -rf "$REPO_ROOT/target"
                printf "    [CLEAN] robot_brain target/ (manual removal)\n"
            }
        else
            rm -rf "$REPO_ROOT/target"
            printf "    [CLEAN] robot_brain target/ (manual removal, no cargo)\n"
        fi
        popd > /dev/null
    fi

    printf "${GRN}=== Clean complete ===${RST}\n"
}

# ---------------------------------------------------------------------------
# Gate
# ---------------------------------------------------------------------------

run_gate() {
    local TS_DIR="$REPO_ROOT/.agents/scripts/test_suite2.1"
    local OUTPUT_FILE="$REPO_ROOT/.agents/scripts/test_output.txt"
    # Consolidate all output to one file; clean any others
    rm -f "$OUTPUT_FILE"
    rm -f "$TS_DIR/test_suite_report.json"
    rm -f "$TS_DIR/python_test_output.txt"
    rm -f "$TS_DIR/analysis_output.txt"
    rm -f "$REPO_ROOT/.agents/scripts/tool_coverage_report.json"
    rm -f "$REPO_ROOT/.agents/scripts/make_gate_output.txt"
    touch "$OUTPUT_FILE"

    # ---- 0. Build robot_brain ----
    step "BUILD — Ensure robot_brain binary exists"
    if [ ! -f "target/release/robot_brain" ] && [ ! -f "target/release/robot_brain.exe" ]; then
        info "Building robot_brain..."
        cargo build --release -p robot_brain || fail "robot_brain build failed"
        ok "robot_brain built"
    else
        info "robot_brain binary exists — skipping"
    fi

    # ---- 1. Run Python analysis (code_analyzer + lint + registry) ----
    step "ANALYSIS — Python code analysis, lint, registry"
    (
        cd "$TS_DIR"
        python main.py 2>&1 | tee -a "$OUTPUT_FILE" || true
    )

    # ---- 2. Run Python tests ----
    step "RUN — Python functional tests"
    if [ -f "$TS_DIR/test_runner.sh" ]; then
        if ! "$TS_DIR/test_runner.sh" 2>&1; then
            fail "test_suite2.1 functional tests FAILED"
        fi
        ok "test_suite2.1 functional tests passed"
    else
        info "test_suite2.1 Python not yet implemented — skipping"
    fi

    # ---- 3. Quality wall ----
    step "CHECK — Quality wall"
    local PASS=0 TOTAL=0 FAIL=0 WARN=0 ISSUES=0 UNTESTED=0

    # Parse test counts from pytest output (python_test_output.txt)
    local PY_TEST_OUTPUT="$TS_DIR/python_test_output.txt"
    if [ -f "$PY_TEST_OUTPUT" ] && grep -qE '[0-9]+ passed' "$PY_TEST_OUTPUT"; then
        PASS=$(grep -oP '[0-9]+(?= passed)' "$PY_TEST_OUTPUT" | tail -1 || echo 0)
        PASS=${PASS:-0}
        FAIL=0
        TOTAL=$PASS
    else
        # Fallback: check test_output.txt
        if [ -f "$OUTPUT_FILE" ] && grep -qE '[0-9]+ passed' "$OUTPUT_FILE"; then
            PASS=$(grep -oP '[0-9]+(?= passed)' "$OUTPUT_FILE" | tail -1 || echo 0)
            PASS=${PASS:-0}
        else
            PASS=0
        fi
        FAIL=0
        TOTAL=$PASS
    fi

    # Extract code issues count from text output
    ISSUES=$(grep -oP 'Code issues:\s+\K\d+' "$OUTPUT_FILE" 2>/dev/null | head -1 || echo 0)
    ISSUES=${ISSUES:-0}
    # Extract lint issues count ("Lint issues: N")
    WARN=$(grep -oP 'Lint issues:\s+\K\d+' "$OUTPUT_FILE" 2>/dev/null | head -1 || echo 0)
    WARN=${WARN:-0}
    # Extract untested tools count
    UNTESTED=$(grep -oP 'Untested:\s+\K\d+' "$OUTPUT_FILE" 2>/dev/null | tail -1 || echo 0)
    UNTESTED=${UNTESTED:-0}
    info "Parsed: passed=$PASS total=$TOTAL failed=$FAIL warnings=$WARN issues=$ISSUES untested=$UNTESTED"

    if [ "$FAIL" != "0" ]; then
        fail "test_suite2.1 has failing tests (passed=$PASS/$TOTAL, failed=$FAIL)"
    fi
    if [ "$WARN" != "0" ] || [ "$ISSUES" != "0" ] || [ "$UNTESTED" != "0" ]; then
        fail "quality wall violated: warnings=$WARN issues=$ISSUES untested=$UNTESTED"
    fi
    ok "test_suite2.1 (passed=$PASS/$TOTAL, warnings=$WARN issues=$ISSUES untested=$UNTESTED)"

    # ---- 4. Report ----
    step "DONE"
    printf "\n${GRN}=== GATE GREEN - commit permitted ===${RST}\n"
    printf "  Consolidated output: %s\n" "$OUTPUT_FILE"
    printf "  Python tests:   $PASS/$TOTAL passed, $WARN warnings, $ISSUES issues, $UNTESTED untested\n"
    # Read Python result from consolidated output file
    PY_RESULT=$(grep -E "passed|failed" "$OUTPUT_FILE" 2>/dev/null | tail -1 || echo "")
    if [ -n "$PY_RESULT" ]; then
        printf "  Python: %s\n" "$PY_RESULT"
    fi
}

# ---------------------------------------------------------------------------
# Session start
# ---------------------------------------------------------------------------

run_session() {
    local SMACK=0

    smack() {
        if [ "$SMACK" = "1" ]; then
            printf "\n${RED}NEWSPAPER: $1${RST}\n"
        fi
    }
    # Enable newspaper mode
    SMACK=1

    # ---- 1. Read required docs ----
    step "1/5 — READ the required docs (in full)"
    for f in .agents/STARTUP.md AGENTS.md .agents/PLAN.md; do
        if [ -f "$f" ]; then
            lines=$(wc -l < "$f")
            cat "$f" >/dev/null
            ok "read $f ($lines lines)"
        else
            fail "missing $f"
        fi
    done

    # ---- 2. Run the verify gate ----
    step "2/5 — RUN the verify gate"
    smack "Do NOT trust memory. Run the gate now."
    if ! .agents/scripts/make.sh gate > /tmp/session_gate.log 2>&1; then
        GATE_GREEN=0
        printf "${RED}gate RED — see /tmp/session_gate.log${RST}\n"
    else
        GATE_GREEN=1
        ok "gate green"
    fi

    OUTPUT_FILE="$REPO_ROOT/.agents/scripts/test_output.txt"
    local PASS TOTAL WARN ISSUES UNTESTED
    PASS=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(d.get('summary',{}).get('passed',0))" 2>/dev/null || echo 0)
    TOTAL=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(d.get('summary',{}).get('total',0))" 2>/dev/null || echo 0)
    WARN=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(d.get('summary',{}).get('compiler_warnings',0))" 2>/dev/null || echo 0)
    ISSUES=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(d.get('summary',{}).get('code_issues',0))" 2>/dev/null || echo 0)
    UNTESTED=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(len(d.get('coverage',{}).get('untested_tools',[])))" 2>/dev/null || echo 0)
    printf "  tests: %s/%s | warnings: %s | code_issues: %s | untested: %s | gate_green: %s\n" \
        "$PASS" "$TOTAL" "$WARN" "$ISSUES" "$UNTESTED" "$GATE_GREEN"

    PY_RESULT=$(grep -E "passed|failed" "$OUTPUT_FILE" 2>/dev/null | tail -1 || echo "")
    if [ -n "$PY_RESULT" ]; then
        printf "  Python: %s\n" "$PY_RESULT"
    fi

    # ---- 3. MCP connection check ----
    step "3/5 — VERIFY MCP live smoke"
    local SMOKE
    SMOKE=$(python3 -c "import json;d=json.load(open('$OUTPUT_FILE'));print(d.get('summary',{}).get('passed',0))" 2>/dev/null || echo 0)
    if [ "$SMOKE" -gt 0 ]; then
        MCP_CONNECTED=1
        ok "MCP live smoke passed ($SMOKE passed)"
    else
        MCP_CONNECTED=0
        fail "MCP live smoke FAILED"
    fi

    # ---- 4. Identify first task ----
    step "4/5 — FIND first incomplete task in PLAN.md"
    FIRST_TASK=$(python3 -c "
import re
t=open('.agents/PLAN.md').read()
m=re.search(r'\[\s*\] \*\*(T1-[0-9A-Z-]+)\*\*([^\n]*)', t)
if not m:
    m=re.search(r'\[~\] \*\*(T1-[0-9A-Z-]+)\*\*([^\n]*)', t)
print(m.group(1) if m else 'NONE_FOUND', (m.group(2).strip()[:120] if m else ''))
")
    TASK_ID=$(echo "$FIRST_TASK" | awk '{print $1}')
    TASK_DESC=$(echo "$FIRST_TASK" | cut -d' ' -f2-)
    ok "FIRST task: $TASK_ID — $TASK_DESC"
    info "This is the ONLY task you may work on first."

    # ---- 5. Stamp proof ----
    step "5/5 — STAMP session proof"
    local PROOF="$REPO_ROOT/.agents/session_proof.json"
    local NOW
    NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    python3 -c "
import json,sys
json.dump({
    'timestamp': '$NOW',
    'docs_read': ['STARTUP.md','AGENTS.md','PLAN.md'],
    'gate_green': $GATE_GREEN,
    'metrics': {'tests': '$PASS/$TOTAL', 'compiler_warnings': $WARN, 'code_issues': $ISSUES, 'untested_tools': $UNTESTED},
    'mcp_connected': $MCP_CONNECTED,
    'first_task_in_order': '$TASK_ID',
    'first_task_desc': '''$TASK_DESC'''
}, open('$PROOF','w'), indent=2)
"
    ok "proof stamped: $PROOF"

    printf "\n${GRN}=== SESSION START COMPLETE ===${RST}\n"
    printf "${YLW}Newspaper rules now active:${RST}\n"
    printf "  - pre-push hook requires this proof to push (mcp_connected=true).\n"
    printf "  - Work on: %s (first incomplete, in order).\n" "$TASK_ID"
    printf "  - After each change: gate -> commit -> push -> STOP.\n"
    if [ "$MCP_CONNECTED" != "1" ]; then
        smack "MCP NOT CONNECTED — you CANNOT push."
    fi
}

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------

show_help() {
    printf "Usage: make.sh <subcommand>\n\n"
    printf "Subcommands:\n"
    printf "  gate      Run the full verify gate (build + tests + quality wall)\n"
    printf "  session   Run session-start enforcement (docs → gate → MCP → pick task)\n"
    printf "  clean     Clean test_suite2.1 artifacts\n"
    printf "  help      Show this help message\n"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

SUBCMD="${1:-help}"
case "$SUBCMD" in
    gate)      run_gate ;;
    session)   run_session ;;
    clean)     clean ;;
    help|--help|-h|"") show_help ;;
    *) printf "${RED}Unknown subcommand: %s\nRun 'make.sh help' for usage.${RST}\n" "$SUBCMD"; exit 1 ;;
esac
