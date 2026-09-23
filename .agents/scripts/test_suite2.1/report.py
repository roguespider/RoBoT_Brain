"""Python replacement for Rust report generation."""

import json
from pathlib import Path
from typing import Any


class ReportWriter:
    def __init__(self, output_path: Path | None = None):
        if output_path is None:
            # __file__ is test_suite2.1/report.py
            # parent.parent = scripts directory
            self.output_path = (
                Path(__file__).resolve().parent.parent / "test_output.json"
            )
        else:
            self.output_path = output_path

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
        with open(self.output_path, "w", encoding="utf-8") as f:
            json.dump(report, f, indent=2)
        print(f"[PASS] Report written to {self.output_path}")
