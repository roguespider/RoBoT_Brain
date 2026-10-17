"""Python replacement for Rust lint analysis (clippy/check)."""

import json
import subprocess
import sys
from pathlib import Path


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
        # Run cargo clippy
        try:
            result = subprocess.run(
                ["cargo", "clippy", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )
            # Parse JSON messages if available
            for line in result.stdout.splitlines():
                if line.startswith("{"):
                    try:
                        msg = json.loads(line)
                        if msg.get("reason") == "compiler-message":
                            issues.append(
                                LintIssue(
                                    msg.get("message", {})
                                    .get("spans", [{}])[0]
                                    .get("file_name", "unknown"),
                                    msg.get("message", {})
                                    .get("spans", [{}])[0]
                                    .get("line_start", 0),
                                    msg.get("message", {}).get("level", "warning"),
                                    msg.get("message", {}).get("message", ""),
                                )
                            )
                    except (json.JSONDecodeError, KeyError, IndexError):
                        pass
        except OSError as e:
            print(f"[WARN] Clippy error: {e}", file=sys.stderr)
        # Run cargo check
        try:
            result = subprocess.run(
                ["cargo", "check"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )
            if result.returncode != 0:
                for line in result.stderr.splitlines():
                    if "warning" in line or "error" in line:
                        issues.append(LintIssue("unknown", 0, "warning", line))
        except OSError as e:
            print(f"[WARN] Cargo check error: {e}", file=sys.stderr)
        return issues
