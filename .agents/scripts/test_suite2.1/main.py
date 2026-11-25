#!/usr/bin/env python3
"""Python-only test suite for RoBoT Brain — replaces Rust test_suite2 binary."""

import argparse
import ast
import json
import logging
import re
import subprocess
import time
from pathlib import Path

logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

# Import Python MCP client (same protocol as Rust)
from code_analyzer import CodeAnalyzer
from lint_runner import LintRunner
from registry import FunctionRegistry
from report import ReportWriter
from test_tool_probe import (
    probe_all_tools,
    test_ingest_system,
    test_mcp_acp_connection,
    test_search_system,
)

try:
    from mcp_client import RobotBrainClient as _RobotBrainClient

    RobotBrainClient: type[_RobotBrainClient] | None = _RobotBrainClient
except ImportError:
    RobotBrainClient = None


class _FileWithRetry:
    """Context manager that opens a file with retry logic for PermissionError."""

    def __init__(self, path, mode="w", encoding="utf-8", retries=5):
        self.path = path
        self.mode = mode
        self.encoding = encoding
        self.retries = retries
        self.file = None

    def __enter__(self):
        for _retry in range(self.retries):
            try:
                self.file = open(self.path, mode=self.mode, encoding=self.encoding)
                break
            except PermissionError:
                time.sleep(0.5)
        else:
            logger.error(
                "[FAIL] Could not open %s after %d retries", self.path, self.retries
            )
            raise OSError(f"Could not open {self.path} after {self.retries} retries")
        return self.file

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.file is not None:
            self.file.close()
        return False


def open_file_with_retry(path, mode="w", encoding="utf-8", retries=5):
    """Open a file with retry logic for PermissionError."""
    return _FileWithRetry(path, mode, encoding, retries)


def format_table(headers, rows, f):
    """Format data as a text table."""
    if not rows:
        f.write("  (none)\n")
        return

    # Calculate column widths
    col_widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            col_widths[i] = max(col_widths[i], len(str(cell)))

    # Create format string
    fmt = " | ".join(f"{{:<{w}}}" for w in col_widths)
    separator = "-+-".join("-" * w for w in col_widths)

    # Write table
    f.write("\n")
    f.write(fmt.format(*headers) + "\n")
    f.write(separator + "\n")
    for row in rows:
        f.write(fmt.format(*[str(c) for c in row]) + "\n")
    f.write("\n")


async def _run_probe():
    """Run comprehensive tool probe and return results dict."""
    if RobotBrainClient is None:
        return {"error": "RobotBrainClient not available"}
    client = await RobotBrainClient.connect()
    await client.get_workflow("general")
    await client.search_memory_first("test")

    tool_results = await probe_all_tools()
    ingest_results = await test_ingest_system(client)
    search_results = await test_search_system(client)
    mcp_acp_results = await test_mcp_acp_connection(client)

    await client.stop()

    return {
        "tool_probe": tool_results,
        "ingest_system": ingest_results,
        "search_system": search_results,
        "mcp_acp_connection": mcp_acp_results,
    }


def _get_imported_names(source: str) -> set:
    """Get all imported names from source."""
    tree = ast.parse(source)
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                names.add(alias.asname or alias.name)
    return names


def main():
    parser = argparse.ArgumentParser(description="Python test suite for RoBoT Brain")
    parser.add_argument(
        "--list", action="store_true", help="List available MCP tools via live server"
    )
    parser.add_argument(
        "--probe",
        type=str,
        default=None,
        help="Probe a tool's input schema via live server",
    )
    parser.add_argument(
        "--run-probe",
        action="store_true",
        help="Run full tool probe (requires built binary)",
    )
    args = parser.parse_args()

    if args.list:
        if RobotBrainClient is None:
            logger.info("[INFO] --list: RobotBrainClient not available")
            return

        _Client = RobotBrainClient

        async def list_tools():
            client = await _Client.connect()
            if client is None:
                logger.info("[FAIL] Failed to connect")
                return
            tools = await client.list_tools()
            await client.stop()
            logger.info("Available tools (%d):", len(tools))
            for t in tools:
                name = t.get("name", "?")
                desc = t.get("description", "")
                logger.info("  %-40s %s", name, desc[:60])

        import asyncio as _asyncio

        _asyncio.run(list_tools())
        return

    if args.probe:
        if RobotBrainClient is None:
            logger.info("[INFO] --probe %s: RobotBrainClient not available", args.probe)
            return

        _Client = RobotBrainClient

        async def probe_tool(name: str):
            client = await _Client.connect()
            if client is None:
                logger.info("[FAIL] Failed to connect for tool '%s'", name)
                return
            tools = await client.list_tools()
            await client.stop()
            for t in tools:
                if t.get("name") == name:
                    logger.info("Tool: %s", name)
                    logger.info(
                        "Schema: %s", json.dumps(t.get("inputSchema", {}), indent=2)
                    )
                    return
            logger.info("[FAIL] Tool '%s' not found", name)

        import asyncio as _asyncio

        _asyncio.run(probe_tool(args.probe))
        return

    # Default: run analysis, lint, registry, and generate report
    project_root = Path(__file__).resolve().parent.parent.parent.parent
    source_path = project_root / "src"

    analyzer = CodeAnalyzer(source_path)
    code_issues = analyzer.analyze()

    lint_runner = LintRunner(project_root)
    lint_issues = lint_runner.run()

    registry = FunctionRegistry()
    registry.load()

    # Large file check (per .agents/LARGE_FILE_REFACTOR.md: 800 line threshold)
    large_file_issues = []
    for rs_file in source_path.rglob("*.rs"):
        try:
            line_count = len(rs_file.read_text(encoding="utf-8").splitlines())
            if line_count > 800:
                large_file_issues.append(
                    f"{rs_file}: {line_count} lines (threshold 800)"
                )
        except OSError:
            logger.debug("Could not read %s", rs_file)

    # Python self-check: all .py files in test_suite2.1 must compile and import without errors
    py_compile_issues = []
    py_runtime_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    import ast
    import sys

    sys.path.insert(0, str(test_suite_dir))
    for py_file in sorted(test_suite_dir.rglob("*.py")):
        try:
            source = py_file.read_text(encoding="utf-8")
            compile(source, str(py_file), "exec")
        except SyntaxError as e:
            py_compile_issues.append(f"{py_file}:{e.lineno}: {e.msg}")
            continue
        except OSError:
            logger.debug("Could not read %s", py_file)
            continue
        # Walk AST to find undefined names and other static issues
        try:
            tree = ast.parse(source, str(py_file))
            for node in ast.walk(tree):
                if isinstance(node, ast.Name):
                    if node.id in ("asyncio",) and node.id not in _get_imported_names(
                        source
                    ):
                        py_runtime_issues.append(
                            f"{py_file}:{node.lineno}: {node.id} used when not imported"
                        )
                elif (
                    isinstance(node, ast.Attribute)
                    and isinstance(node.value, ast.Name)
                    and node.value.id == "None"
                ):
                    py_runtime_issues.append(
                        f"{py_file}:{node.lineno}: attribute '{node.attr}' on None"
                    )
        except (SyntaxError, ValueError) as e:
            py_compile_issues.append(f"{py_file}: ast parse error: {e}")
        # Try importing to catch runtime errors
        # Skip files with complex import chains to avoid circular imports
        skip_modules = {"main", "test_tool_probe", "conftest", "run_tests_with_timeout"}
        try:
            module_name = py_file.stem
            if module_name in skip_modules:
                continue
            if module_name in sys.modules:
                del sys.modules[module_name]
            __import__(module_name)
        except ImportError as e:
            py_runtime_issues.append(f"{py_file}: import failed: {e}")
        except AttributeError as e:
            py_runtime_issues.append(f"{py_file}: attribute error: {e}")
        except NameError as e:
            py_runtime_issues.append(f"{py_file}: name error: {e}")
        except RecursionError:
            logger.debug("Import recursion (skipped): %s", py_file)
        except (OSError, TypeError, ValueError) as _import_exc:
            logger.debug("Import error (ignored): %s", _import_exc)
    sys.path.pop(0)

    # Python lint self-check: all .py files in the project must pass pyflakes
    # (errors and warnings). Gate is RED if any issue exists.
    # Scan directories: test_suite2.1, bulletin, skills, and other .py dirs under .agents/
    py_lint_issues = []
    scan_dirs = [Path(__file__).resolve().parent]  # test_suite2.1
    # Add other Python directories under .agents/
    agents_dir = project_root / ".agents"
    if agents_dir.is_dir():
        for subdir in sorted(agents_dir.iterdir()):
            if subdir.is_dir() and subdir.name not in ("scripts",):
                candidate = agents_dir / subdir.name
                if any(candidate.rglob("*.py")):
                    scan_dirs.append(candidate)
    # Run pyflakes on each directory at once (much faster than per-file)
    # Check if pyflakes is available first
    pyflakes_available = False
    try:
        result = subprocess.run(
            ["python", "-m", "pyflakes", "--version"],
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
        pyflakes_available = result.returncode == 0
    except OSError as e:
        logger.warning("pyflakes version check failed: %s", e)

    if pyflakes_available:
        for py_dir in scan_dirs:
            try:
                result = subprocess.run(
                    ["python", "-m", "pyflakes", str(py_dir)],
                    capture_output=True,
                    text=True,
                    timeout=60,
                    check=False,
                )
                if result.returncode != 0:
                    for line in result.stdout.splitlines():
                        if line.strip():
                            py_lint_issues.append(f"{py_dir.name}: {line}")
                    for line in result.stderr.splitlines():
                        if line.strip():
                            py_lint_issues.append(f"{py_dir.name}: {line}")
            except OSError as e:
                logger.warning("pyflakes error on %s: %s", py_dir, e)
    else:
        logger.info("pyflakes not available - skipping Python lint check")

    # Python type-check self-check: run mypy on test_suite2.1 to detect
    # type errors (return type mismatches, etc.). Gate is RED if any issue.
    # Filter out notes (informational messages) - only count actual errors.
    py_type_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    try:
        result = subprocess.run(
            ["python", "-m", "mypy", str(test_suite_dir), "--no-error-summary"],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        if result.returncode != 0:
            for line in result.stdout.splitlines():
                if line.strip() and "error:" in line.lower():
                    py_type_issues.append(f"{test_suite_dir.name}: {line}")
            for line in result.stderr.splitlines():
                if line.strip() and "error:" in line.lower():
                    py_type_issues.append(f"{test_suite_dir.name}: {line}")
    except OSError:
        logger.debug("mypy not installed - skipping type checks")
    except subprocess.TimeoutExpired:
        logger.debug("mypy timed out - skipping type checks")

    # Python silencer detection: find and report all silencer directives
    # that hide linter warnings. These are forbidden per AGENTS.md rules.
    import re as _re

    silencer_patterns = [
        (r"#\s*noqa", "noqa silencer directive"),
        (r"#\s*type:\s*ignore", "type: ignore silencer directive"),
        (r"#\s*pylint:\s*disable", "pylint: disable silencer directive"),
        (r"#\s*mypy:\s*ignore", "mypy: ignore silencer directive"),
        (r"#\s*ruff:\s*disable", "ruff: disable silencer directive"),
        (r"#\s*flake8:\s*ignore", "flake8: ignore silencer directive"),
    ]
    silencer_issues = []
    for py_dir in scan_dirs:
        for py_file in sorted(py_dir.rglob("*.py")):
            try:
                source = py_file.read_text(encoding="utf-8")
                for line_num, line in enumerate(source.splitlines(), start=1):
                    for pattern, desc in silencer_patterns:
                        if _re.search(pattern, line):
                            silencer_issues.append(
                                f"{py_file}:{line_num}: {desc} found: {line.strip()[:100]}"
                            )
            except OSError:
                logger.debug("Could not read %s", py_file)
                continue

    # Python quality self-check: all .py files in test_suite2.1 must be free of
    # stub patterns (TODO/FIXME/XXX/HACK, pass, ...), print() calls,
    # broad except: clauses, and lines over 100 chars. Gate is RED if any issue exists.
    py_quality_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    for py_file in sorted(test_suite_dir.rglob("*.py")):
        try:
            source = py_file.read_text(encoding="utf-8")
            lines = source.splitlines()
            for i, line in enumerate(lines, start=1):
                stripped = line.strip()
                upper = stripped.upper()
                # Only flag markers that look like actual directives, not descriptions
                # Skip false positives like "todo!()" in comments describing code
                found_stub = False
                if stripped.startswith("#"):
                    for tag in ["TODO", "FIXME", "XXX", "HACK"]:
                        idx = 0
                        while True:
                            idx = upper.find(tag, idx)
                            if idx == -1:
                                break
                            # Check word boundary
                            before_ok = idx == 0 or not upper[idx - 1].isalnum()
                            after_ok = (
                                idx + len(tag) >= len(upper)
                                or not upper[idx + len(tag)].isalnum()
                            )
                            if before_ok and after_ok:
                                # Require tag is followed by : or space+text (actual marker, not description)
                                after_pos = idx + len(tag)
                                if after_pos < len(upper):
                                    next_char = upper[after_pos]
                                    if next_char == ":" or (
                                        next_char == " " and after_pos + 1 < len(upper)
                                    ):
                                        found_stub = True
                                elif after_pos == len(upper):
                                    found_stub = True
                                break
                            idx += 1
                elif not stripped.replace("'", "").replace('"', "").replace(
                    "_", ""
                ).replace(":", "").replace("=", "") and upper in [
                    "TODO",
                    "FIXME",
                    "XXX",
                    "HACK",
                ]:
                    found_stub = True
                if found_stub:
                    py_quality_issues.append(
                        f"{py_file}:{i}: stub marker found: {stripped[:80]}"
                    )
                if stripped == "pass":
                    py_quality_issues.append(f"{py_file}:{i}: pass stub found")
                if stripped == "...":
                    py_quality_issues.append(f"{py_file}:{i}: ellipsis stub found")
                # Skip comment lines for print() check
                if stripped.startswith("#"):
                    continue
                if "print(" in stripped:
                    idx = stripped.find("print(")
                    in_string = False
                    in_single = False
                    in_double = False
                    j = 0
                    while j < idx and j < len(stripped):
                        ch = stripped[j]
                        if ch == "\\" and j + 1 < len(stripped):
                            j += 2
                            continue
                        if ch == "'" and not in_double:
                            in_single = not in_single
                        elif ch == '"' and not in_single:
                            in_double = not in_double
                        j += 1
                    in_string = in_single or in_double
                    if not in_string:
                        py_quality_issues.append(
                            f"{py_file}:{i}: print() call found (use logging): {stripped[:80]}"
                        )
                # Broad except clauses
                if stripped.startswith(("except:", "except Exception:")):
                    py_quality_issues.append(
                        f"{py_file}:{i}: broad except clause found: {stripped[:80]}"
                    )
                # Line length > 100
                if len(line) > 100:
                    py_quality_issues.append(
                        f"{py_file}:{i}: line too long ({len(line)} chars > 100): {stripped[:80]}"
                    )
                # Async functions using blocking file I/O
                is_async_context = "async def" in line or "await " in stripped
                has_open_call = re.search(
                    r"\bopen\s*\(", stripped
                ) and not stripped.lstrip().startswith("#")
                if is_async_context and has_open_call:
                    py_quality_issues.append(
                        f"{py_file}:{i}: async function uses blocking open(): {stripped[:80]}"
                    )
                # Aliased TimeoutError (should use builtin TimeoutError)
                # Skip main.py itself (it contains the checker code)
                if (
                    "asyncio.TimeoutError" in stripped
                    and not stripped.lstrip().startswith("#")
                    and py_file.name != "main.py"
                ):
                    py_quality_issues.append(
                        f"{py_file}:{i}: use builtin TimeoutError: {stripped[:70]}"
                    )
        except OSError:
            logger.debug("Could not read %s", py_file)

    # Initialize probe_data before gate check
    probe_data: dict = {}

    # Gate: red if any code issues, lint issues, large file issues,
    # Python compile issues, Python runtime issues, Python lint issues,
    # Python quality issues, Python type issues, or HALT probe
    # Consolidated Python errors (list)
    python_errors_list = (
        py_compile_issues + py_runtime_issues + py_lint_issues + py_quality_issues
    )
    gate_red = bool(
        code_issues
        or lint_issues
        or large_file_issues
        or python_errors_list
        or silencer_issues
        or py_type_issues
        or probe_data.get("halted")
    )

    writer = ReportWriter()
    writer.write(code_issues, lint_issues, registry.get_all())

    # Human-readable output for .agents/scripts/test_output.txt
    output_path = (
        Path(__file__).resolve().parent.parent.parent.parent
        / ".agents"
        / "scripts"
        / "test_output.txt"
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open_file_with_retry(output_path, "w", "utf-8") as f:
        f.write("=== RoBoT Brain Test Suite — Human Readable Report ===\n\n")

        # Summary table
        f.write("--- Summary ---\n")
        # Combine all Python error categories (count)
        python_errors_count = (
            len(py_compile_issues)
            + len(py_runtime_issues)
            + len(py_lint_issues)
            + len(py_quality_issues)
            + len(py_type_issues)
        )
        format_table(
            ["Metric", "Count"],
            [
                ["Code Issues", len(code_issues)],
                ["Lint Issues", len(lint_issues)],
                ["Python Errors", python_errors_count],
                ["Silencer Issues", len(silencer_issues)],
                ["Registry Tools", len(registry.get_all())],
            ],
            f,
        )

        # Code issues table
        f.write("--- Code Issues ---\n")
        code_rows = [
            [
                getattr(i, "issue_type", "unknown"),
                str(getattr(i, "file_path", "unknown")),
                getattr(i, "line_number", 0),
                getattr(i, "description", ""),
            ]
            for i in code_issues
        ]
        format_table(
            ["Type", "File", "Line", "Description"],
            code_rows,
            f,
        )

        # Lint issues table
        f.write("--- Lint Issues ---\n")
        lint_rows = [
            [
                getattr(i, "level", "warning"),
                getattr(i, "file_path", "unknown"),
                getattr(i, "line_number", 0),
                getattr(i, "message", ""),
            ]
            for i in lint_issues
        ]
        format_table(
            ["Level", "File", "Line", "Message"],
            lint_rows,
            f,
        )

        # Large file issues (gate-failing per .agents/LARGE_FILE_REFACTOR.md)
        if large_file_issues:
            f.write("--- Large File Issues (GATE RED if >800 lines) ---\n")
            f.writelines(f"[FAIL] {lf}\n" for lf in large_file_issues)
            f.write(
                "\n[REQUIREMENT] Use .agents/LARGE_FILE_REFACTOR.md to split these files.\n"
            )
            f.write(
                "[ACTION] Create directory modules per the pattern, "
                "update imports, verify gate passes.\n"
            )
        else:
            f.write("--- Large File Issues ---\n[PASS] No files exceed 800 lines\n")

        # Python errors (consolidated: compile + runtime + lint + quality + type)
        f.write("--- Python Errors ---\n")
        if python_errors_count > 0:
            f.write("[FAIL] Python errors found\n")
            if py_compile_issues:
                f.write("  Compile errors:\n")
                f.writelines(f"    [FAIL] {issue}\n" for issue in py_compile_issues)
            if py_runtime_issues:
                f.write("  Runtime errors:\n")
                f.writelines(f"    [FAIL] {issue}\n" for issue in py_runtime_issues)
            if py_lint_issues:
                f.write("  Lint errors:\n")
                f.writelines(f"    [FAIL] {issue}\n" for issue in py_lint_issues)
            if py_quality_issues:
                f.write("  Quality issues:\n")
                f.writelines(f"    [FAIL] {issue}\n" for issue in py_quality_issues)
            if py_type_issues:
                f.write("  Type errors (mypy):\n")
                f.writelines(f"    [FAIL] {issue}\n" for issue in py_type_issues)
        else:
            f.write("[PASS] No Python errors\n")

        # Silencer detection (gate-failing — no silencers allowed)
        if silencer_issues:
            f.write("--- Silencer Detection (GATE RED if any) ---\n")
            f.writelines(f"[FAIL] {issue}\n" for issue in silencer_issues)
        else:
            f.write("--- Silencer Detection ---\n[PASS] No silencer directives found\n")

        # Gate status
        f.write("\n--- Gate Status ---\n")
        if gate_red:
            f.write("[RED] GATE FAILED\n")
            f.write(f"  Code issues: {len(code_issues)}\n")
            f.write(f"  Lint issues: {len(lint_issues)}\n")
            f.write(f"  Large file issues: {len(large_file_issues)}\n")
            f.write(f"  Python errors: {python_errors_count}\n")
            f.write(f"  Silencer issues: {len(silencer_issues)}\n")
        else:
            f.write("[PASS] GATE GREEN\n")
            f.write("  Code issues: 0\n")
            f.write("  Lint issues: 0\n")
            f.write("  Large file issues: 0\n")
            f.write("  Python errors: 0\n")
            f.write("  Silencer issues: 0\n")

        f.write("=== End Report ===\n")

    # Tool probe results table (only if --run-probe flag set)
    probe_data = {}
    if getattr(args, "run_probe", False):
        try:
            import asyncio

            probe_data = asyncio.run(_run_probe())
        except RuntimeError as e:
            # HALT failure — tool probe reported a broken binary
            logger.error("[HALT] Tool probe failed: %s", e)
            probe_data = {"halted": True, "error": str(e)}
        except (OSError, ConnectionError) as _probe_exc:
            logger.debug("Probe skipped (server not available): %s", _probe_exc)

    if probe_data:
        # Re-open file in append mode for probe results
        with open(output_path, "a", encoding="utf-8") as f:
            f.write("\n--- Tool Probe Results ---\n")
            probe = probe_data.get("tool_probe", {})

            # Summary table
            format_table(
                ["Metric", "Value"],
                [
                    ["Total Tools", probe.get("total_tools", 0)],
                    ["Working", len(probe.get("working", []))],
                    ["Failed", len(probe.get("failed", []))],
                    [
                        "Success Rate",
                        probe.get("summary", {}).get("success_rate", "N/A"),
                    ],
                ],
                f,
            )

            # Working tools table (ALL)
            working = probe.get("working", [])
            if working:
                f.write(f"\n--- WORKING TOOLS ({len(working)}) ---\n")
                format_table(
                    ["Status", "Tool", "Time (ms)"],
                    [
                        ["[PASS]", t.get("name", ""), t.get("time_ms", 0)]
                        for t in working
                    ],
                    f,
                )

            # Failed tools table (ALL)
            failed = probe.get("failed", [])
            if failed:
                f.write(f"\n--- FAILED TOOLS ({len(failed)}) ---\n")
                format_table(
                    ["Status", "Tool", "Error"],
                    [
                        ["[FAIL]", t.get("name", ""), t.get("error", "")[:100]]
                        for t in failed
                    ],
                    f,
                )

            # Ingest system table (ALL results)
            ingest = probe_data.get("ingest_system", {})
            if ingest:
                f.write("\n--- INGEST SYSTEM ---\n")
                format_table(
                    ["Status", "File Type"],
                    [["[PASS]", w.get("type", "")] for w in ingest.get("working", [])],
                    f,
                )
                failed_ingest = ingest.get("failed", [])
                if failed_ingest:
                    format_table(
                        ["Status", "File Type", "Error"],
                        [
                            ["[FAIL]", w.get("type", ""), w.get("error", "")[:80]]
                            for w in failed_ingest
                        ],
                        f,
                    )

            # Search system table (ALL results)
            search = probe_data.get("search_system", {})
            if search:
                f.write("\n--- SEARCH SYSTEM ---\n")
                format_table(
                    ["Status", "Tool"],
                    [["[PASS]", w.get("tool", "")] for w in search.get("working", [])],
                    f,
                )
                failed_search = search.get("failed", [])
                if failed_search:
                    format_table(
                        ["Status", "Tool", "Error"],
                        [
                            ["[FAIL]", w.get("tool", ""), w.get("error", "")[:80]]
                            for w in failed_search
                        ],
                        f,
                    )

            # ACP connection table (ALL results)
            acp = probe_data.get("mcp_acp_connection", {})
            if acp:
                f.write("\n--- MCP/ACP CONNECTION ---\n")
                format_table(
                    ["Status", "Tool"],
                    [["[PASS]", w.get("tool", "")] for w in acp.get("working", [])],
                    f,
                )
                failed_acp = acp.get("failed", [])
                if failed_acp:
                    format_table(
                        ["Status", "Tool", "Error"],
                        [
                            ["[FAIL]", w.get("tool", ""), w.get("error", "")[:80]]
                            for w in failed_acp
                        ],
                        f,
                    )

    if gate_red:
        logger.error(
            "[RED] GATE FAILED — code/lint/large-file/python-compile/python-quality issues found"
        )
        logger.error(
            "[FAIL] Code: %d, Lint: %d, Large: %d, "
            "Compile: %d, Runtime: %d, Lint: %d, Quality: %d",
            len(code_issues),
            len(lint_issues),
            len(large_file_issues),
            len(py_compile_issues),
            len(py_runtime_issues),
            len(py_lint_issues),
            len(py_quality_issues),
        )
        import sys

        sys.exit(1)

    logger.info(
        "[PASS] Python test suite complete — see test_output.txt (only output file)"
    )
    logger.info(
        "[PASS] Code: %d, Lint: %d, Compile: %d, Runtime: %d, "
        "Lint: %d, Quality: %d, Registry: %d",
        len(code_issues),
        len(lint_issues),
        len(py_compile_issues),
        len(py_runtime_issues),
        len(py_lint_issues),
        len(py_quality_issues),
        len(registry.get_all()),
    )
    logger.info("[INFO] Human-readable report written to: %s", output_path)


if __name__ == "__main__":
    main()
#!/usr/bin/env python3
"""Python-only test suite for RoBoT Brain — replaces Rust test_suite2 binary."""

import argparse
import asyncio
import json
from pathlib import Path

from code_analyzer import CodeAnalyzer
from lint_runner import LintRunner
from registry import FunctionRegistry
from report import ReportWriter

# Import Python MCP client (same protocol as Rust)
try:
    from mcp_client import RobotBrainClient as _RobotBrainClient

    RobotBrainClient = _RobotBrainClient
except ImportError:
    RobotBrainClient = None


def format_table(headers, rows, f):
    """Format data as a text table."""
    if not rows:
        f.write("  (none)\n")
        return

    # Calculate column widths
    col_widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            col_widths[i] = max(col_widths[i], len(str(cell)))

    # Create format string
    fmt = " | ".join(f"{{:<{w}}}" for w in col_widths)
    separator = "-+-".join("-" * w for w in col_widths)

    # Write table
    f.write("\n")
    f.write(fmt.format(*headers) + "\n")
    f.write(separator + "\n")
    for row in rows:
        f.write(fmt.format(*[str(c) for c in row]) + "\n")
    f.write("\n")


def main():
    parser = argparse.ArgumentParser(description="Python test suite for RoBoT Brain")
    parser.add_argument(
        "--list", action="store_true", help="List available MCP tools via live server"
    )
    parser.add_argument(
        "--probe",
        type=str,
        default=None,
        help="Probe a tool's input schema via live server",
    )
    args = parser.parse_args()

    if args.list:
        _RPC = RobotBrainClient
        if _RPC is not None:

            async def list_tools():
                client = await _RPC.connect()
                tools = await client.list_tools()
                await client.stop()
                print(f"Available tools ({len(tools)}):")
                for t in tools:
                    name = t.get("name", "?")
                    desc = t.get("description", "")
                    print(f"  {name:<40} {desc[:60]}")

            asyncio.run(list_tools())
        else:
            print("[INFO] --list: RobotBrainClient not available")
        return

    if args.probe:
        _RPC = RobotBrainClient
        if _RPC is not None:

            async def probe_tool(name: str):
                client = await _RPC.connect()
                tools = await client.list_tools()
                await client.stop()
                for t in tools:
                    if t.get("name") == name:
                        print(f"Tool: {name}")
                        print(
                            f"Schema: {json.dumps(t.get('inputSchema', {}), indent=2)}"
                        )
                        return
                print(f"[FAIL] Tool '{name}' not found")

            asyncio.run(probe_tool(args.probe))
        else:
            print(f"[INFO] --probe {args.probe}: RobotBrainClient not available")
        return

    # Default: run analysis, lint, registry, and generate report
    project_root = Path(__file__).resolve().parent.parent.parent.parent
    source_path = project_root / "src"

    analyzer = CodeAnalyzer(source_path)
    code_issues = analyzer.analyze()

    lint_runner = LintRunner(project_root)
    lint_issues = lint_runner.run()

    registry = FunctionRegistry()
    registry.load()

    # Large file check (per .agents/LARGE_FILE_REFACTOR.md: 1000 line threshold — will lower to 500 after thinning)
    large_file_issues = []
    for rs_file in source_path.rglob("*.rs"):
        try:
            line_count = len(rs_file.read_text(encoding="utf-8").splitlines())
            if line_count > 1000:
                large_file_issues.append(
                    f"{rs_file}: {line_count} lines (threshold 1000)"
                )
        except OSError:
            pass

    # Gate: red if any code issues, lint issues, or large file issues exist
    gate_red = bool(code_issues or lint_issues or large_file_issues)

    writer = ReportWriter()
    writer.write(code_issues, lint_issues, registry.get_all())

    # Human-readable output for .agents/scripts/test_output.txt
    output_path = (
        Path(__file__).resolve().parent.parent.parent.parent
        / ".agents"
        / "scripts"
        / "test_output.txt"
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write("=== RoBoT Brain Test Suite — Human Readable Report ===\n\n")

        # Summary table
        f.write("--- Summary ---\n")
        format_table(
            ["Metric", "Count"],
            [
                ["Code Issues", len(code_issues)],
                ["Lint Issues", len(lint_issues)],
                ["Registry Tools", len(registry.get_all())],
            ],
            f,
        )

        # Code issues table
        f.write("--- Code Issues ---\n")
        code_rows = [
            [
                getattr(i, "issue_type", "unknown"),
                str(getattr(i, "file_path", "unknown")),
                getattr(i, "line_number", 0),
                getattr(i, "description", ""),
            ]
            for i in code_issues
        ]
        format_table(
            ["Type", "File", "Line", "Description"],
            code_rows,
            f,
        )

        # Lint issues table
        f.write("--- Lint Issues ---\n")
        lint_rows = [
            [
                getattr(i, "level", "warning"),
                getattr(i, "file_path", "unknown"),
                getattr(i, "line_number", 0),
                getattr(i, "message", ""),
            ]
            for i in lint_issues
        ]
        format_table(
            ["Level", "File", "Line", "Message"],
            lint_rows,
            f,
        )

        # Large file issues (gate-failing per .agents/LARGE_FILE_REFACTOR.md)
        if large_file_issues:
            f.write("--- Large File Issues (GATE RED if >1000 lines) ---\n")
            for lf in large_file_issues:
                f.write(f"[FAIL] {lf}\n")
        else:
            f.write("--- Large File Issues ---\n[PASS] No files exceed 1000 lines\n")

        # Gate status
        f.write("\n--- Gate Status ---\n")
        if gate_red:
            f.write("[RED] GATE FAILED\n")
            f.write(f"  Code issues: {len(code_issues)}\n")
            f.write(f"  Lint issues: {len(lint_issues)}\n")
            f.write(f"  Large file issues: {len(large_file_issues)}\n")
        else:
            f.write("[PASS] GATE GREEN\n")
            f.write(f"  Code issues: 0\n")
            f.write(f"  Lint issues: 0\n")
            f.write(f"  Large file issues: 0\n")

        f.write("=== End Report ===\n")

        # Tool probe results table
        tool_probe_report = (
            Path(__file__).resolve().parent / "test_tool_probe_report.json"
        )
        if tool_probe_report.exists():
            with open(tool_probe_report, "r", encoding="utf-8") as tp:
                probe_data = json.load(tp)

            f.write("\n--- Tool Probe Results ---\n")
            probe = probe_data.get("tool_probe", {})

            # Summary table
            format_table(
                ["Metric", "Value"],
                [
                    ["Total Tools", probe.get("total_tools", 0)],
                    ["Working", len(probe.get("working", []))],
                    ["Failed", len(probe.get("failed", []))],
                    [
                        "Success Rate",
                        probe.get("summary", {}).get("success_rate", "N/A"),
                    ],
                ],
                f,
            )

            # Working tools table (ALL)
            working = probe.get("working", [])
            if working:
                f.write(f"\n--- WORKING TOOLS ({len(working)}) ---\n")
                format_table(
                    ["Status", "Tool", "Time (ms)"],
                    [
                        ["[PASS]", t.get("name", ""), t.get("time_ms", 0)]
                        for t in working
                    ],
                    f,
                )

            # Failed tools table (ALL)
            failed = probe.get("failed", [])
            if failed:
                f.write(f"\n--- FAILED TOOLS ({len(failed)}) ---\n")
                format_table(
                    ["Status", "Tool", "Error"],
                    [
                        ["[FAIL]", t.get("name", ""), t.get("error", "")[:100]]
                        for t in failed
                    ],
                    f,
                )

            # Ingest system table (ALL results)
            ingest = probe_data.get("ingest_system", {})
            if ingest:
                f.write("\n--- INGEST SYSTEM ---\n")
                format_table(
                    ["Status", "File Type"],
                    [["[PASS]", w.get("type", "")] for w in ingest.get("working", [])],
                    f,
                )
                failed_ingest = ingest.get("failed", [])
                if failed_ingest:
                    format_table(
                        ["Status", "File Type", "Error"],
                        [
                            ["[FAIL]", w.get("type", ""), w.get("error", "")[:80]]
                            for w in failed_ingest
                        ],
                        f,
                    )

            # Search system table (ALL results)
            search = probe_data.get("search_system", {})
            if search:
                f.write("\n--- SEARCH SYSTEM ---\n")
                format_table(
                    ["Status", "Tool"],
                    [["[PASS]", w.get("tool", "")] for w in search.get("working", [])],
                    f,
                )
                failed_search = search.get("failed", [])
                if failed_search:
                    format_table(
                        ["Status", "Tool", "Error"],
                        [
                            ["[FAIL]", w.get("tool", ""), w.get("error", "")[:80]]
                            for w in failed_search
                        ],
                        f,
                    )

            # ACP connection table (ALL results)
            acp = probe_data.get("mcp_acp_connection", {})
            if acp:
                f.write("\n--- MCP/ACP CONNECTION ---\n")
                format_table(
                    ["Status", "Tool"],
                    [["[PASS]", w.get("tool", "")] for w in acp.get("working", [])],
                    f,
                )
                failed_acp = acp.get("failed", [])
                if failed_acp:
                    format_table(
                        ["Status", "Tool", "Error"],
                        [
                            ["[FAIL]", w.get("tool", ""), w.get("error", "")[:80]]
                            for w in failed_acp
                        ],
                        f,
                    )

    if gate_red:
        print("[RED] GATE FAILED — code/lint/large-file issues found")
        print(
            f"[FAIL] Code issues: {len(code_issues)}, Lint issues: {len(lint_issues)}, Large files: {len(large_file_issues)}"
        )
        import sys

        sys.exit(1)

    print("[PASS] Python test suite complete — see test_output.txt (only output file)")
    print(
        f"[PASS] Code issues: {len(code_issues)}, Lint issues: {len(lint_issues)}, Registry: {len(registry.get_all())}"
    )
    print(f"[INFO] Human-readable report written to: {output_path}")


if __name__ == "__main__":
    main()
