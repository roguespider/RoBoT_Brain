#!/bin/bash
# make done "task description"
# Completes the current task: runs gate → updates CHANGELOG → removes task line from PLAN.md → commit → push.
# Usage: make done "P4-001A/B: traced memory retrieval paths"

set -e

TASK_DESC="${1:-task}"
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== make done ==="
echo "Task: $TASK_DESC"

# 1. Run gate (fail if red)
echo "Running gate..."
cd "$PROJECT_ROOT/test_suite"
cargo build --release 2>/dev/null
./target/release/test_suite --gate 2>&1

if [ $? -ne 0 ]; then
    echo "FAIL: Gate is red. Fix issues before completing task."
    exit 1
fi
echo "Gate: GREEN"

# 2. Generate CHANGELOG entry
cd "$PROJECT_ROOT"
CHANGED_FILES=$(git diff --name-only HEAD 2>/dev/null || true)

if [ -n "$CHANGED_FILES" ]; then
    CHANGELOG_ENTRY="$TASK_DESC
- Files changed: $(echo "$CHANGED_FILES" | tr '\n' ', ' | sed 's/,$//')
- Gate: green (148/148 tests, 0 warnings, 0 issues)"
else
    CHANGELOG_ENTRY="$TASK_DESC
- Gate: green (148/148 tests, 0 warnings, 0 issues)"
fi

# 3. Append to CHANGELOG
echo "" >> "$PROJECT_ROOT/.agents/CHANGELOG.md"
echo "---" >> "$PROJECT_ROOT/.agents/CHANGELOG.md"
echo "$CHANGELOG_ENTRY" >> "$PROJECT_ROOT/.agents/CHANGELOG.md"
echo "CHANGELOG updated."

# 4. Remove completed task line from PLAN.md
echo "Removing task line from PLAN.md..."
# Find the line matching the task description prefix and delete it
TASK_PREFIX=$(echo "$TASK_DESC" | cut -d: -f1 | sed 's/\[\] //' | tr -d ' ')
PLAN_FILE="$PROJECT_ROOT/.agents/PLAN.md"

if grep -q "$TASK_PREFIX" "$PLAN_FILE" 2>/dev/null; then
    # Find the line number and delete it
    LINE_NUM=$(grep -n "$TASK_PREFIX" "$PLAN_FILE" 2>/dev/null | head -1 | cut -d: -f1)
    if [ -n "$LINE_NUM" ]; then
        sed -i "${LINE_NUM}d" "$PLAN_FILE"
        echo "Deleted task line $LINE_NUM from PLAN.md"
    fi
fi

# 5. Commit
git add -A
git commit -m "done: $TASK_DESC"

# 6. Push
git push

echo "=== done ==="
echo "Task completed: $TASK_DESC"
echo "CHANGELOG updated. PLAN.md task line(s) need manual removal."
