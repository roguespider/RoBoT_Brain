#!/usr/bin/env python3
#
# scripts/gate_quality.py — enforce AGENTS.md's zero-violation rule.
#
# Reads the test_suite JSON report and requires:
#   compiler_warnings == 0
#   code_issues        == 0   (no #[allow], no PublicNeverCalled, no stubs)
#   untested_tools     == 0
#
# This is a HARD wall, not a ratchet. AGENTS.md forbids dead code, #[allow],
# and ignored vars; any non-zero count is a violation that must be fixed by
# wiring the offending pub API into a real caller (never #[allow] or `_`).
#
# Usage: python3 scripts/gate_quality.py [path/to/test_suite_report.json]
"""Hard AGENTS.md wall: zero warnings, zero code-issues, zero untested tools."""

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_REPORT = REPO_ROOT / "test_suite" / "test_suite_report.json"

RED = "\033[31m"
GRN = "\033[32m"
RST = "\033[0m"


def fail(msg: str) -> int:
    print(f"{RED}QUALITY WALL RED: {msg}{RST}")
    return 1


def main() -> int:
    report_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_REPORT
    if not report_path.exists():
        return fail(f"test_suite report not found at {report_path}")

    with report_path.open() as f:
        report = json.load(f)

    actual = {
        "compiler_warnings": report.get("summary", {}).get("compiler_warnings", 0),
        "code_issues": report.get("summary", {}).get("code_issues", 0),
        "untested_tools": len(
            report.get("coverage", {}).get("untested_tools", []) or []
        ),
    }

    bad = []
    for key in ("compiler_warnings", "code_issues", "untested_tools"):
        got = actual.get(key, 0)
        marker = GRN if got == 0 else RED
        verdict = "OK" if got == 0 else "VIOLATION"
        print(f"  {marker}{key:<20} actual={got:<4} {verdict}{RST}")
        if got != 0:
            bad.append((key, got))

    if bad:
        msgs = [f"{k}={g}" for k, g in bad]
        print(f"\nFix the violation(s) per AGENTS.md: wire the dead-code pub API "
              f"into a real caller. Do NOT use #[allow] or `_` to silence.")
        return fail("; ".join(msgs))

    print(f"{GRN}QUALITY WALL OK (0 warnings, 0 code-issues, 0 untested tools){RST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

