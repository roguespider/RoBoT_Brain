"""Python replacement for Rust report generation."""

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
        pass  # Report written inline in main.py
