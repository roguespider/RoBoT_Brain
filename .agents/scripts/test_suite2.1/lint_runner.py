"""Python replacement for Rust lint analysis (clippy/check)."""

import json
import logging
import subprocess
from pathlib import Path

logger = logging.getLogger(__name__)


class LintIssue:
    def __init__(self, file_path: str, line_number: int, level: str, message: str):
        self.file_path = file_path
        self.line_number = line_number
        self.level = level
        self.message = message


class LintRunner:
    def __init__(self, project_root: Path):
        self.project_root = project_root

    def run(self) -> list[LintIssue]:
        issues: list[LintIssue] = []
        # Run cargo check (timeout 30s for fast gate — longer builds run separately)
        try:
            result = subprocess.run(
                ["cargo", "check", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Parse JSON messages from cargo check
            for line in result.stdout.splitlines():
                if line.startswith("{"):
                    try:
                        msg = json.loads(line)
                        if msg.get("reason") == "compiler-message":
                            level = msg.get("message", {}).get("level", "")
                            if level in ("error", "warning"):
                                span = msg.get("message", {}).get("spans", [{}])[0]
                                issues.append(
                                    LintIssue(
                                        span.get("file_name", "unknown"),
                                        span.get("line_start", 0),
                                        level,
                                        msg.get("message", {}).get("message", ""),
                                    )
                                )
                    except (json.JSONDecodeError, KeyError, IndexError):
                        logger.debug("Failed to parse compiler message JSON")
                # Also capture stderr lines
            if result.returncode != 0:
                for line in result.stderr.splitlines():
                    if "warning" in line.lower() or "error" in line.lower():
                        issues.append(LintIssue("unknown", 0, "warning", line))
        except subprocess.TimeoutExpired:
            logger.warning("Cargo check timed out (600s limit)")
        except OSError as e:
            logger.warning("Cargo check error: %s", e)
        # Run cargo clippy (timeout 30s for fast gate — longer builds run separately)
        try:
            result = subprocess.run(
                ["cargo", "clippy", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            # Parse JSON messages if available
            for line in result.stdout.splitlines():
                if line.startswith("{"):
                    try:
                        msg = json.loads(line)
                        if msg.get("reason") == "compiler-message":
                            level = msg.get("message", {}).get("level", "")
                            if level in ("error", "warning"):
                                span = msg.get("message", {}).get("spans", [{}])[0]
                                issues.append(
                                    LintIssue(
                                        span.get("file_name", "unknown"),
                                        span.get("line_start", 0),
                                        level,
                                        msg.get("message", {}).get("message", ""),
                                    )
                                )
                    except (json.JSONDecodeError, KeyError, IndexError):
                        logger.debug("Failed to parse clippy message JSON")
        except subprocess.TimeoutExpired:
            logger.warning("Cargo clippy timed out (600s limit)")
        except OSError as e:
            logger.warning("Clippy error: %s", e)
        return issues
