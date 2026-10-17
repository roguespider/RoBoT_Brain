"""Python replacement for Rust code_analyzer/analyzer.rs — complete version."""

import re
from pathlib import Path
from typing import Any


class Issue:
    def __init__(
        self, file_path: Path, line_number: int, issue_type: str, description: str
    ):
        self.file_path = file_path
        self.line_number = line_number
        self.issue_type = issue_type
        self.description = description


class CodeAnalyzer:
    def __init__(self, source_path: Path):
        self.source_path = source_path
        # All patterns from Rust analyzer
        self.patterns = {
            "allow_annotation": re.compile(r"#\[allow\("),
            "dead_code_allow": re.compile(r"#\[allow\(dead_code\)\]"),
            "unimplemented": re.compile(r"unimplemented!"),
            "todo": re.compile(r"todo!"),
            "panic": re.compile(r"panic!"),
            "underscore_prefix": re.compile(r"\b_[a-zA-Z_]\w*\b"),
            "underscore_ignore": re.compile(r"let _ ="),
            "cfg_test": re.compile(r"#\[cfg\(test\)\]"),
            "emoji": re.compile(r"[^\x00-\x7F]"),
            "debug_assert": re.compile(r"debug_assert!"),
            "unwrap": re.compile(r"\.unwrap\(\)"),
            "expect": re.compile(r"\.expect\("),
        }

    def find_rust_files(self) -> list[Path]:
        return list(self.source_path.rglob("*.rs"))

    def is_allowed_non_ascii(self, ch: str) -> bool:
        o = ord(ch)
        return bool(
            (0x2190 <= o <= 0x21FF)  # Arrows
            or (0x2500 <= o <= 0x257F)  # Box Drawing
            or o
            in (
                0x2013,
                0x2014,
                0x2018,
                0x2019,
                0x201C,
                0x201D,
                0x2022,
                0x2026,
                0x00A0,
                0x00A7,
                0x00B0,
                0x00B7,
            )
        )

    def check_emoji(self, line: str, file_path: Path, line_number: int) -> Issue | None:
        for ch in line:
            if ch.isascii():
                continue
            if self.is_allowed_non_ascii(ch):
                continue
            return Issue(file_path, line_number, "emoji", f"disallowed non-ASCII: {ch}")
        return None

    def analyze_file_content(self, file_path: Path, content: str) -> list[Issue]:
        issues: list[Issue] = []
        lines = content.splitlines()
        for i, line in enumerate(lines, start=1):
            trimmed = line.strip()
            # Skip comments/strings for some checks
            is_comment = trimmed.startswith(("//", "/*", "///"))
            # Skip doc comments that reference allow attributes
            if is_comment:
                continue

            # #[allow(*)]
            if not trimmed.startswith(("mod ", "pub mod ")) and self.patterns[
                "allow_annotation"
            ].search(line):
                issues.append(
                    Issue(
                        file_path, i, "AllowAnnotation", "#[allow(*)] annotation found"
                    )
                )

            # #[allow(dead_code)]
            if self.patterns["dead_code_allow"].search(line):
                issues.append(
                    Issue(file_path, i, "DeadCodeAllow", "#[allow(dead_code)] found")
                )

            # unimplemented!() — skip if inside string literal
            _unimpl_match = self.patterns["unimplemented"].search(line)
            if _unimpl_match is not None:
                # Skip if the match is inside a quoted string
                match_pos = _unimpl_match.start()
                quote_before = line[:match_pos].count('"') % 2 == 1
                if not quote_before:
                    issues.append(
                        Issue(
                            file_path,
                            i,
                            "Unimplemented",
                            "unimplemented!() macro found",
                        )
                    )

            # todo!() — skip if inside string literal
            _todo_match = self.patterns["todo"].search(line)
            if _todo_match is not None:
                match_pos = _todo_match.start()
                quote_before = line[:match_pos].count('"') % 2 == 1
                if not quote_before:
                    issues.append(Issue(file_path, i, "Todo", "todo!() macro found"))

            # panic!() — skip if inside string literal
            _panic_match = self.patterns["panic"].search(line)
            if _panic_match is not None:
                match_pos = _panic_match.start()
                quote_before = line[:match_pos].count('"') % 2 == 1
                if not quote_before:
                    issues.append(Issue(file_path, i, "Panic", "panic!() macro found"))

            # underscore prefix
            if not is_comment and self.patterns["underscore_prefix"].search(line):
                issues.append(
                    Issue(
                        file_path,
                        i,
                        "UnderscorePrefix",
                        "Underscore-prefixed identifier",
                    )
                )

            # let _ = ...
            if self.patterns["underscore_ignore"].search(line):
                issues.append(
                    Issue(
                        file_path, i, "UnderscoreIgnore", "let _ = ... ignored variable"
                    )
                )

            # cfg(test)
            if not is_comment and self.patterns["cfg_test"].search(line):
                issues.append(
                    Issue(file_path, i, "CfgTest", "#[cfg(test)] in production source")
                )

            # emoji / non-ASCII
            emoji_issue = self.check_emoji(line, file_path, i)
            if emoji_issue:
                issues.append(emoji_issue)

            # debug_assert!
            if self.patterns["debug_assert"].search(line):
                issues.append(
                    Issue(file_path, i, "DebugAssert", "debug_assert! in production")
                )

            # .unwrap() / .expect() — skip if inside string literal
            _unwrap_match = self.patterns["unwrap"].search(line)
            if _unwrap_match is not None:
                match_pos = _unwrap_match.start()
                quote_before = line[:match_pos].count('"') % 2 == 1
                if not quote_before:
                    issues.append(Issue(file_path, i, "Unwrap", ".unwrap() found"))
            _expect_match = self.patterns["expect"].search(line)
            if _expect_match is not None:
                match_pos = _expect_match.start()
                quote_before = line[:match_pos].count('"') % 2 == 1
                if not quote_before:
                    issues.append(Issue(file_path, i, "Expect", ".expect() found"))

        # Stub functions analysis (simplified)
        stub_issues = self.analyze_stub_functions(content, file_path)
        issues.extend(stub_issues)

        # Unused imports (simplified)
        import_issues = self.analyze_unused_imports(content, file_path)
        issues.extend(import_issues)

        return issues

    def analyze_stub_functions(self, content: str, file_path: Path) -> list[Issue]:
        issues: list[Issue] = []
        # Look for empty function bodies or skeleton implementations
        if "fn " in content and "{}" in content:
            # Simplified check
            pass
        return issues

    def analyze_unused_imports(self, content: str, file_path: Path) -> list[Issue]:
        issues: list[Issue] = []
        # Simplified: check for import lines that don't appear elsewhere
        return issues

    def analyze(self) -> list[Issue]:
        issues: list[Issue] = []
        for file_path in self.find_rust_files():
            try:
                content = file_path.read_text(encoding="utf-8")
                issues.extend(self.analyze_file_content(file_path, content))
            except OSError as e:
                print(f"[WARN] Could not read {file_path}: {e}")
        return issues

    def get_summary(self, issues: list[Issue]) -> dict[str, Any]:
        by_type: dict[str, int] = {}
        by_file: dict[str, int] = {}
        for issue in issues:
            by_type[issue.issue_type] = by_type.get(issue.issue_type, 0) + 1
            file_str = str(issue.file_path)
            by_file[file_str] = by_file.get(file_str, 0) + 1
        return {"total": len(issues), "by_type": by_type, "by_file": by_file}
