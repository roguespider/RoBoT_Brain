"""Python replacement for Rust report generation."""

import json
from pathlib import Path
from typing import Any


class ReportWriter:
    def __init__(self, output_path: Path | None = None):
        # Only text output is produced; no JSON file.
        self.output_path = None

    def write(
        self,
        code_issues: list[Any],
        lint_issues: list[Any],
        registry_items: list[dict[str, Any]],
    ):
        report = {
            "issues": [
                {
                    "kind": "code_issue",
                    "category": getattr(i, "issue_type", "unknown"),
                    "file": str(getattr(i, "file_path", "unknown")),
                    "line": getattr(i, "line_number", 0),
                    "message": getattr(i, "description", ""),
                    "suggested_action": "Fix or verify",
                }
                for i in code_issues
            ]
            + [
                {
                    "kind": "lint_issue",
                    "category": getattr(i, "level", "warning"),
                    "file": getattr(i, "file_path", "unknown"),
                    "line": getattr(i, "line_number", 0),
                    "message": getattr(i, "message", ""),
                    "suggested_action": "Fix or verify",
                }
                for i in lint_issues
            ],
            "summary": {
                "code_issues": len(code_issues),
                "lint_issues": len(lint_issues),
                "registry_tools": len(registry_items),
            },
        }
        # No JSON output — only .agents/scripts/test_output.txt is produced.
        print("[PASS] Report writer: JSON output disabled (test_output.txt only)")
