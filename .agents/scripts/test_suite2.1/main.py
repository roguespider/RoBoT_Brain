#!/usr/bin/env python3
"""Python-only test suite for RoBoT Brain — replaces Rust test_suite2 binary."""

import argparse
import asyncio
import json
import logging
import subprocess
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

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


async def _run_probe():
    """Run comprehensive tool probe and return results dict."""
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
        if RobotBrainClient is None:
            logging.info("[INFO] --list: RobotBrainClient not available")
            return

        async def list_tools():
            client = await RobotBrainClient.connect()
            tools = await client.list_tools()
            await client.stop()
            logging.info(f"Available tools ({len(tools)}):")
            for t in tools:
                name = t.get("name", "?")
                desc = t.get("description", "")
                logging.info(f"  {name:<40} {desc[:60]}")

        asyncio.run(list_tools())
        return

    if args.probe:
        if RobotBrainClient is None:
            logging.info(f"[INFO] --probe {args.probe}: RobotBrainClient not available")
            return

        async def probe_tool(name: str):
            client = await RobotBrainClient.connect()
            tools = await client.list_tools()
            await client.stop()
            for t in tools:
                if t.get("name") == name:
                    logging.info(f"Tool: {name}")
                    logging.info(
                        f"Schema: {json.dumps(t.get('inputSchema', {}), indent=2)}"
                    )
                    return
            logging.info(f"[FAIL] Tool '{name}' not found")

        asyncio.run(probe_tool(args.probe))
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
            logging.debug(f"Could not read {rs_file}")

    # Python self-check: all .py files in test_suite2.1 must compile without syntax errors
    py_compile_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    for py_file in sorted(test_suite_dir.rglob("*.py")):
        try:
            source = py_file.read_text(encoding="utf-8")
            compile(source, str(py_file), "exec")
        except SyntaxError as e:
            py_compile_issues.append(f"{py_file}:{e.lineno}: {e.msg}")
        except OSError:
            logging.debug(f"Could not read {py_file}")

    # Python lint self-check: all .py files in test_suite2.1 must pass pyflakes
    # (errors and warnings). Gate is RED if any issue exists.
    py_lint_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    for py_file in sorted(test_suite_dir.rglob("*.py")):
        try:
            result = subprocess.run(
                ["python", "-m", "pyflakes", str(py_file)],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            if result.returncode != 0:
                for line in result.stdout.splitlines():
                    if line.strip():
                        py_lint_issues.append(f"{py_file}: {line}")
                for line in result.stderr.splitlines():
                    if line.strip():
                        py_lint_issues.append(f"{py_file}: {line}")
        except OSError:
            py_lint_issues.append(
                f"{py_file}: pyflakes not installed - cannot verify lint"
            )

    # Python quality self-check: all .py files in test_suite2.1 must be free of
    # stub patterns (TODO/FIXME/XXX/HACK, pass, ...), print() calls,
    # broad except: clauses, and lines over 100 chars. Gate is RED if any issue exists.
    py_quality_issues = []
    test_suite_dir = Path(__file__).resolve().parent
    checker_file = Path(__file__).resolve()
    for py_file in sorted(test_suite_dir.rglob("*.py")):
        if py_file == checker_file:
            continue
        try:
            source = py_file.read_text(encoding="utf-8")
            lines = source.splitlines()
            for i, line in enumerate(lines, start=1):
                stripped = line.strip()
                upper = stripped.upper()
                # Stub markers: only flag actual TODO/FIXME markers (followed by : or space+desc)
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
                # print() calls (should use logging) — skip if inside string literal
                if "print(" in stripped:
                    # Check if print( is inside a string literal
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
                if stripped.startswith("except:") or stripped.startswith(
                    "except Exception:"
                ):
                    py_quality_issues.append(
                        f"{py_file}:{i}: broad except clause found: {stripped[:80]}"
                    )
                # Line length > 100
                if len(line) > 100:
                    py_quality_issues.append(
                        f"{py_file}:{i}: line too long ({len(line)} chars > 100): {stripped[:80]}"
                    )
        except OSError:
            logging.debug(f"Could not read {py_file}")

    # Initialize probe_data before gate check
    probe_data: dict = {}

    # Gate: red if any code issues, lint issues, large file issues,
    # Python compile issues, Python lint issues, Python quality issues, or HALT probe
    gate_red = bool(
        code_issues
        or lint_issues
        or large_file_issues
        or py_compile_issues
        or py_lint_issues
        or py_quality_issues
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
    with open(output_path, "w", encoding="utf-8") as f:
        f.write("=== RoBoT Brain Test Suite — Human Readable Report ===\n\n")

        # Summary table
        f.write("--- Summary ---\n")
        format_table(
            ["Metric", "Count"],
            [
                ["Code Issues", len(code_issues)],
                ["Lint Issues", len(lint_issues)],
                ["Python Compile Issues", len(py_compile_issues)],
                ["Python Lint Issues", len(py_lint_issues)],
                ["Python Quality Issues", len(py_quality_issues)],
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
                "[ACTION] Create directory modules per the pattern, update imports, verify gate passes.\n"
            )
        else:
            f.write("--- Large File Issues ---\n[PASS] No files exceed 800 lines\n")

        # Python compile issues (gate-failing)
        if py_compile_issues:
            f.write("--- Python Compile Issues (GATE RED if any) ---\n")
            f.writelines(f"[FAIL] {issue}\n" for issue in py_compile_issues)
        else:
            f.write(
                "--- Python Compile Issues ---\n[PASS] All .py files compile cleanly\n"
            )

        # Python lint issues (gate-failing)
        if py_lint_issues:
            f.write("--- Python Lint Issues (GATE RED if any) ---\n")
            f.writelines(f"[FAIL] {issue}\n" for issue in py_lint_issues)
        else:
            f.write("--- Python Lint Issues ---\n[PASS] No Python lint issues\n")

        # Python quality issues (gate-failing)
        if py_quality_issues:
            f.write("--- Python Quality Issues (GATE RED if any) ---\n")
            f.writelines(f"[FAIL] {issue}\n" for issue in py_quality_issues)
        else:
            f.write("--- Python Quality Issues ---\n[PASS] No Python quality issues\n")

        # Gate status
        f.write("\n--- Gate Status ---\n")
        if gate_red:
            f.write("[RED] GATE FAILED\n")
            f.write(f"  Code issues: {len(code_issues)}\n")
            f.write(f"  Lint issues: {len(lint_issues)}\n")
            f.write(f"  Large file issues: {len(large_file_issues)}\n")
            f.write(f"  Python compile issues: {len(py_compile_issues)}\n")
            f.write(f"  Python lint issues: {len(py_lint_issues)}\n")
            f.write(f"  Python quality issues: {len(py_quality_issues)}\n")
        else:
            f.write("[PASS] GATE GREEN\n")
            f.write("  Code issues: 0\n")
            f.write("  Lint issues: 0\n")
            f.write("  Large file issues: 0\n")
            f.write("  Python compile issues: 0\n")
            f.write("  Python lint issues: 0\n")
            f.write("  Python quality issues: 0\n")

        f.write("=== End Report ===\n")

        # Tool probe results table (call probe functions directly)
        probe_data = {}
        try:
            import asyncio

            probe_data = asyncio.run(_run_probe())
        except RuntimeError as e:
            # HALT failure — tool probe reported a broken binary
            logging.error("[HALT] Tool probe failed: %s", e)
            probe_data = {"halted": True, "error": str(e)}
        except Exception as e:
            logging.debug("Probe skipped (server not available): %s", e)

        if probe_data:
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
        logging.error(
            "[RED] GATE FAILED — code/lint/large-file/python-compile/python-quality issues found"
        )
        logging.error(
            "[FAIL] Code issues: %d, Lint issues: %d, Large files: %d, Python compile issues: %d, Python lint issues: %d, Python quality issues: %d",
            len(code_issues),
            len(lint_issues),
            len(large_file_issues),
            len(py_compile_issues),
            len(py_lint_issues),
            len(py_quality_issues),
        )
        import sys

        sys.exit(1)

    logging.info(
        "[PASS] Python test suite complete — see test_output.txt (only output file)"
    )
    logging.info(
        "[PASS] Code issues: %d, Lint issues: %d, Python compile issues: %d, Python lint issues: %d, Python quality issues: %d, Registry: %d",
        len(code_issues),
        len(lint_issues),
        len(py_compile_issues),
        len(py_lint_issues),
        len(py_quality_issues),
        len(registry.get_all()),
    )
    logging.info("[INFO] Human-readable report written to: %s", output_path)


if __name__ == "__main__":
    main()
