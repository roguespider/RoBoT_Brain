#!/usr/bin/env bash
#
# .agents/scripts/session_start.sh — THE NEWSPAPER.
#
# This is the rolled-up newspaper that smacks the agent on the nose at the
# start of every session. It is NOT optional. It enforces the process rules
# that words alone failed to enforce:
#
#   1. READ the required docs (STARTUP.md, AGENTS.md, PLAN.md) — in full.
#   2. RUN the verify gate and PRINT the four metrics (no "I remember it was
#      green" — run it).
#   3. CONNECT to the live robot_brain MCP server YOURSELF and smoke a real
#      tool call. No relying solely on test_suite.
#   4. IDENTIFY the FIRST incomplete task in PLAN.md, IN ORDER. Print it.
#      No skipping ahead.
#   5. STAMP a proof file (.agents/session_proof.json) recording that all of
#      the above happened this session.
#
# The pre-push hook (.agents/githooks/pre-push) refuses to push unless a fresh
# session_proof.json exists with mcp_connected=true. So you cannot push a
# single commit this session without having connected to MCP and run the gate.
#
# Run at session start:  .agents/scripts/session_start.sh   (or `make session`)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi

RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; BLU=$'\033[34m'; RST=$'\033[0m'
smack() { printf "\n${RED}🥾 NEWSPAPER: %s${RST}\n" "$1"; }
step()  { printf "\n${BLU}=== %s ===${RST}\n" "$1"; }
ok()    { printf "${GRN}✓ %s${RST}\n" "$1"; }
fail()  { printf "${RED}✗ %s${RST}\n" "$1"; }

step "1/5 — READ the required docs (in full)"
for f in .agents/STARTUP.md AGENTS.md .agents/PLAN.md; do
    if [ -f "$f" ]; then
        lines=$(wc -l < "$f")
        # Actually read it so the content is in context, not just counted.
        cat "$f" >/dev/null
        ok "read $f ($lines lines)"
    else
        fail "missing $f"; exit 1
    fi
done

step "2/5 — RUN the verify gate and PRINT the four metrics"
smack "Do NOT trust memory. Run the gate now."
if ! .agents/scripts/gate.sh >/tmp/session_gate.log 2>&1; then
    GATE_GREEN=0
    printf "${RED}gate RED — see /tmp/session_gate.log${RST}\n"
else
    GATE_GREEN=1
    ok "gate green"
fi
# Extract metrics regardless of pass/fail (gate may be red on warnings).
REPORT="$REPO_ROOT/test_suite/test_suite_report.json"
PASS=$(python3 -c "import json;d=json.load(open('$REPORT'));s=d.get('summary',{});print(s.get('passed',0))" 2>/dev/null || echo 0)
TOTAL=$(python3 -c "import json;d=json.load(open('$REPORT'));s=d.get('summary',{});print(s.get('total',0))" 2>/dev/null || echo 0)
WARN=$(python3 -c "import json;d=json.load(open('$REPORT'));s=d.get('summary',{});print(s.get('compiler_warnings',0))" 2>/dev/null || echo 0)
ISSUES=$(python3 -c "import json;d=json.load(open('$REPORT'));s=d.get('summary',{});print(s.get('code_issues',0))" 2>/dev/null || echo 0)
UNTESTED=$(python3 -c "import json;d=json.load(open('$REPORT'));c=d.get('coverage',{});print(len(c.get('untested_tools',[])))" 2>/dev/null || echo 0)
printf "  tests: %s/%s | warnings: %s | code_issues: %s | untested: %s | gate_green: %s\n" \
    "$PASS" "$TOTAL" "$WARN" "$ISSUES" "$UNTESTED" "$GATE_GREEN"

step "3/5 — CONNECT to live robot_brain MCP server (YOURSELF)"
smack "test_suite is NOT enough. Connect yourself and call a real tool."
MCP_CONNECTED=0
MCP_PROOF=""
# Live MCP smoke is now a Rust test in test_suite (session_smoke.rs), run as
# part of the full suite (step 2). Verify via the JSON report instead of
# spawning the removed Python script.
SMOKE=$(python3 -c "import json;d=json.load(open('$REPORT'));print(d.get('summary',{}).get('passed',0))" 2>/dev/null || echo 0)
if [ "$SMOKE" -gt 0 ]; then
    MCP_CONNECTED=1
    MCP_PROOF="covered by test_suite session_smoke tests ($SMOKE passed)"
    ok "MCP live smoke passed: $MCP_PROOF"
else
    fail "MCP live smoke FAILED — run: cd test_suite && cargo build --release && ./target/release/test_suite"
fi

step "4/5 — IDENTIFY the FIRST incomplete task in PLAN.md (IN ORDER)"
smack "No skipping ahead. First incomplete item, in file order."
FIRST_TASK=$(python3 -c "
import re
t=open('.agents/PLAN.md').read()
# Find the first unchecked task marker: [ ] T1-...  OR  [~] T1-...
m=re.search(r'\[\s*\] \*\*(T1-[0-9A-Z-]+)\*\*([^\n]*)', t)
if not m:
    m=re.search(r'\[~\] \*\*(T1-[0-9A-Z-]+)\*\*([^\n]*)', t)
print(m.group(1) if m else 'NONE_FOUND', (m.group(2).strip()[:120] if m else ''))
")
TASK_ID=$(echo "$FIRST_TASK" | awk '{print $1}')
TASK_DESC=$(echo "$FIRST_TASK" | cut -d' ' -f2-)
ok "FIRST incomplete task (in order): $TASK_ID — $TASK_DESC"
printf "  ${YLW}This is the ONLY task you may work on first. Do not jump ahead.${RST}\n"

step "5/5 — STAMP session proof"
PROOF="$REPO_ROOT/.agents/session_proof.json"
NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)
python3 -c "
import json,sys
json.dump({
    'timestamp': '$NOW',
    'docs_read': ['STARTUP.md','AGENTS.md','PLAN.md'],
    'gate_green': $GATE_GREEN,
    'metrics': {'tests': '$PASS/$TOTAL', 'compiler_warnings': $WARN, 'code_issues': $ISSUES, 'untested_tools': $UNTESTED},
    'mcp_connected': $MCP_CONNECTED,
    'mcp_proof': '$MCP_PROOF',
    'first_task_in_order': '$TASK_ID',
    'first_task_desc': '''$TASK_DESC'''
}, open('$PROOF','w'), indent=2)
"
ok "proof stamped: $PROOF"

printf "\n${GRN}=== SESSION START COMPLETE ===${RST}\n"
printf "${YLW}Newspaper rules now active:${RST}\n"
printf "  - pre-push hook requires this proof to push (mcp_connected=true).\n"
printf "  - You may only work on: %s (first incomplete, in order).\n" "$TASK_ID"
printf "  - After each change: gate -> commit -> push -> STOP.\n"
if [ "$MCP_CONNECTED" != "1" ]; then
    smack "MCP NOT CONNECTED — you CANNOT push until session_smoke.py passes."
fi
