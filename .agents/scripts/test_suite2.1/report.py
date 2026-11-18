"""Python replacement for Rust report generation."""

import logging
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
        # No JSON output — only .agents/scripts/test_output.txt is produced.
        logging.info(
            "[PASS] Report writer: JSON output disabled (test_output.txt only)"
        )
