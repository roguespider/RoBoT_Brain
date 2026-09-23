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

        f.write(
            "=== End Report (JSON for AI agent: .agents/scripts/test_output.json) ===\n"
        )

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

    print(
        "[DONE] Python test suite complete — see test_output.txt (human) and test_output.json (AI)"
    )
    print(
        f"[PASS] Code issues: {len(code_issues)}, Lint issues: {len(lint_issues)}, Registry: {len(registry.get_all())}"
    )
    print(f"[INFO] Human-readable report written to: {output_path}")


if __name__ == "__main__":
    main()
