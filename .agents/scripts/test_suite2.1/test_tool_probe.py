#!/usr/bin/env python3
"""
Comprehensive tool probe — tests ALL tools and generates detailed report.

Tests:
- Which tools work vs fail
- Ingest system (file types)
- MCP/ACP connection
- Search system
- All 200+ tools

Output: test_tool_probe_report.json + human-readable test_tool_probe_output.txt
"""

import asyncio
import json
import sys
import time
from pathlib import Path

# Add current dir to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parent))

from mcp_client import RobotBrainClient


def get_tool_defaults(tool_name: str) -> dict:
    """Get default parameters for common tools to test them quickly."""
    defaults = {
        # ACP tools
        "list_acp_agents": {},
        "acp_agent_count": {},
        "acp_router": {},
        "acp_registry": {},
        "get_agent_capabilities": {"agent_id": "test-agent"},
        "get_system_status": {},
        "get_workflow": {"purpose": "general"},
        "list_tools": {},
        "get_tool": {"name": "list_tools"},
        # CoObOpLoop tools
        "cooboploop_list_goals": {"limit": 10},
        "cooboploop_get_goal": {"goal_id": "test-goal"},
        "cooboploop_get_loop_status": {},
        "cooboploop_get_idle_state": {},
        "cooboploop_get_hardware_profile": {},
        "cooboploop_get_modification_boundary": {},
        "cooboploop_get_autonomous_mode": {},
        "cooboploop_get_autonomy_levels": {},
        "cooboploop_list_strategic_objectives": {},
        "cooboploop_get_objective_hierarchy": {},
        # Experience
        "list_experiences": {"limit": 10},
        "get_experience_stats": {"period": "all"},
        "get_experience": {"id": "test-id"},
        # Knowledge
        "get_knowledge": {"limit": 10},
        "get_knowledge_stats": {},
        "get_mature_knowledge": {"limit": 10},
        "query_knowledge": {"query": "test"},
        "search_knowledge_by_tag": {"tag": "test"},
        "validate_knowledge_dependencies": {},
        # Memory
        "list_memories": {"limit": 10},
        "search_memory": {"query": "test"},
        "get_memory": {"id": "test-id"},
        "get_embedding_stats": {},
        "list_embeddings": {"limit": 10},
        # Personality
        "get_personality": {},
        "list_personality_presets": {},
        # Skills
        "list_skills": {"enabled_only": False},
        "get_skill_stats": {},
        "get_unreliable_skills": {},
        # Workflow
        "list_workflows": {},
        "get_session_state": {},
        # World Model
        "get_world_model_stats": {},
        "list_world_entities": {"kind": "object"},
        "find_world_entity": {"name": "test"},
        # Exploration
        "list_hypotheses": {"limit": 10},
        "list_observations": {"limit": 10},
        "get_patterns": {},
        "get_recommendations": {"category": "test"},
        "get_reputation": {"tool_name": "list_tools"},
        # Reflection
        "list_reflections_by_status": {"status": "active"},
        "get_insights": {"limit": 10},
        # Data contracts
        "get_personality_decision": {
            "confidence": 0.8,
            "potential_gain": 10,
            "potential_loss": 5,
            "uncertainty": 0.2,
        },
    }
    return defaults.get(tool_name, {})


async def probe_all_tools() -> dict:
    """Probe all tools and return results."""
    print("[INFO] Starting comprehensive tool probe...")
    print("[INFO] Connecting to robot_brain...")

    # Connect
    client = await RobotBrainClient.connect()
    print("[OK] Connected to robot_brain")

    # Initialize workflow gate
    print("[INFO] Initializing workflow gate...")
    await client.get_workflow("general")
    await client.search_memory_first("test")
    print("[OK] Workflow gate initialized")

    # List all tools
    print("[INFO] Listing all tools...")
    tools = await client.list_tools()
    tool_names = [t["name"] for t in tools]
    print(f"[INFO] Found {len(tool_names)} tools")

    # Probe each tool
    results = {
        "total_tools": len(tool_names),
        "working": [],
        "failed": [],
        "skipped": [],
        "by_category": {},
    }

    print("\n[INFO] Probing each tool...\n")

    for tool_name in tool_names:
        defaults = get_tool_defaults(tool_name)

        try:
            start = time.time()
            result = await client.call_tool(tool_name, defaults)
            elapsed = time.time() - start

            results["working"].append(
                {
                    "name": tool_name,
                    "response_keys": list(result.keys())
                    if isinstance(result, dict)
                    else [type(result).__name__],
                    "response_preview": str(result)[:200],
                    "time_ms": round(elapsed * 1000, 1),
                }
            )
            status = f"[OK] {tool_name} ({elapsed * 1000:.0f}ms)"
            print(status)

        except Exception as e:
            error_msg = str(e)[:200]
            results["failed"].append(
                {
                    "name": tool_name,
                    "error": error_msg,
                }
            )
            status = f"[FAIL] {tool_name}: {error_msg[:100]}"
            print(status)

    await client.stop()

    # Categorize results
    results["summary"] = {
        "working_count": len(results["working"]),
        "failed_count": len(results["failed"]),
        "success_rate": f"{len(results['working']) / len(tool_names) * 100:.1f}%",
    }

    return results


async def test_ingest_system(client: RobotBrainClient) -> dict:
    """Test ingest system with all supported file types."""
    print("\n[INFO] Testing ingest system...")
    results = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Create test files
    test_dir = Path(__file__).resolve().parent / "test_ingest_files"
    test_dir.mkdir(exist_ok=True)

    # Supported file types from file_collector.rs:
    # TEXT: txt, md, rst, csv, log, xml, html, htm
    # CODE: rs, toml, yaml, yml, env, gitignore, dockerfile, py, js, ts, java, c, cpp, h, hpp, go, rb, php, sql, sh, bash, zsh, ps1, bat, cmd
    # CONFIG: css, scss, sass, less, json, jsonl, properties, conf, cfg, ini, lock
    # SUBTITLES: srt, vtt, ass
    # ARCHIVE: zip, tar, gz, tgz, tar.gz, bz2, xz, 7z, rar
    # AUDIO: mp3, wav, m4a, flac, ogg, aac, wma, opus
    # IMAGE: jpg, jpeg, png, gif, bmp, webp, ico, tiff, svg
    # VIDEO: mp4, avi, mkv, mov, wmv, flv, webm, m4v, mpeg, mpg
    # DOCUMENT: pdf, doc, docx, odt, rtf, epub

    # --- TEXT FILES ---
    text_files = [
        ("test.txt", "This is a plain text file for ingest testing."),
        ("test.md", "# Test Markdown\n\nThis is a markdown file."),
        ("test.csv", "name,age,city\nAlice,30,NYC\nBob,25,LA"),
        ("test.xml", '<?xml version="1.0"?><root><item>test</item></root>'),
        ("test.html", "<html><body><p>Test HTML</p></body></html>"),
        ("test.json", '{"key": "value", "test": true}'),
    ]

    for filename, content in text_files:
        file_path = test_dir / filename
        file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "file",
                },
            )
            results["working"].append({"type": ext, "file": filename})
            results["tested"].append(f"{ext} file ingest")
            print(f"[OK] {ext} file ingest")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} file ingest (FAILED)")
            print(f"[FAIL] {ext} file ingest: {e}")

    # --- CODE FILES ---
    code_files = [
        ("test.rs", 'fn main() { println!("Hello"); }'),
        ("test.py", "def hello(): print('Hello World')"),
        ("test.js", "function hello() { console.log('Hello'); }"),
        ("test.toml", 'name = "test"\nversion = "0.1.0"'),
        ("test.yaml", "name: test\nversion: 0.1.0"),
    ]

    for filename, content in code_files:
        file_path = test_dir / filename
        file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "code",
                },
            )
            results["working"].append({"type": ext, "file": filename})
            results["tested"].append(f"{ext} code ingest")
            print(f"[OK] {ext} code ingest")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} code ingest (FAILED)")
            print(f"[FAIL] {ext} code ingest: {e}")

    # --- SCRIPT FILES ---
    script_files = [
        ("test.sh", "#!/bin/bash\necho 'Hello'"),
        ("test.bat", "@echo off\necho Hello"),
        ("test.ps1", 'Write-Host "Hello"'),
    ]

    for filename, content in script_files:
        file_path = test_dir / filename
        file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "code",
                },
            )
            results["working"].append({"type": ext, "file": filename})
            results["tested"].append(f"{ext} script ingest")
            print(f"[OK] {ext} script ingest")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} script ingest (FAILED)")
            print(f"[FAIL] {ext} script ingest: {e}")

    # --- CONFIG FILES ---
    config_files = [
        ("test.env", "DB_HOST=localhost\nDB_PORT=5432"),
        ("test.ini", "[database]\nhost = localhost\nport = 5432"),
        ("test.conf", "server { listen 80; }"),
        ("test.properties", "db.host=localhost\ndb.port=5432"),
    ]

    for filename, content in config_files:
        file_path = test_dir / filename
        file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "file",
                },
            )
            results["working"].append({"type": ext, "file": filename})
            results["tested"].append(f"{ext} config ingest")
            print(f"[OK] {ext} config ingest")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} config ingest (FAILED)")
            print(f"[FAIL] {ext} config ingest: {e}")

    # --- SUBTITLE FILES ---
    subtitle_files = [
        ("test.srt", "1\n00:00:01,000 --> 00:00:04,000\nHello World"),
    ]

    for filename, content in subtitle_files:
        file_path = test_dir / filename
        file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "file",
                },
            )
            results["working"].append({"type": ext, "file": filename})
            results["tested"].append(f"{ext} subtitle ingest")
            print(f"[OK] {ext} subtitle ingest")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} subtitle ingest (FAILED)")
            print(f"[FAIL] {ext} subtitle ingest: {e}")

    # --- IMAGE FILES (metadata only) ---
    # Create minimal valid files for each image type
    image_files = [
        ("test.png", b"\x89PNG\r\n\x1a\n" + b"\x00" * 100),  # PNG header
        ("test.jpg", b"\xff\xd8\xff\xe0" + b"\x00" * 100),  # JPEG header
        (
            "test.svg",
            '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="50" height="50"/></svg>',
        ),
    ]

    for filename, content in image_files:
        file_path = test_dir / filename
        if isinstance(content, bytes):
            file_path.write_bytes(content)
        else:
            file_path.write_text(content, encoding="utf-8")
        ext = filename.split(".")[-1].upper()

        try:
            await client.call_tool(
                "ingest_files",
                {
                    "folder": "test_ingest_files",
                    "file_path": str(file_path),
                    "memory_type": "file",
                },
            )
            results["working"].append(
                {"type": ext, "file": filename, "note": "metadata"}
            )
            results["tested"].append(f"{ext} image ingest (metadata)")
            print(f"[OK] {ext} image ingest (metadata)")
        except Exception as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} image ingest (FAILED)")
            print(f"[FAIL] {ext} image ingest: {e}")

    # Cleanup
    for f in test_dir.iterdir():
        if f.is_file():
            try:
                f.unlink()
            except FileNotFoundError:
                pass
    try:
        test_dir.rmdir()
    except OSError:
        pass

    return results


async def test_search_system(client: RobotBrainClient) -> dict:
    """Test search system."""
    print("\n[INFO] Testing search system...")
    results = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Test search_memory
    try:
        result = await client.call_tool("search_memory", {"query": "test"})
        results["working"].append({"tool": "search_memory"})
        results["tested"].append("search_memory")
        print("[OK] search_memory")
    except Exception as e:
        results["failed"].append({"tool": "search_memory", "error": str(e)[:100]})
        results["tested"].append("search_memory (FAILED)")
        print(f"[FAIL] search_memory: {e}")

    # Test global_search
    try:
        result = await client.call_tool(
            "global_search",
            {
                "query": "test",
                "limit": 10,
            },
        )
        results["working"].append({"tool": "global_search"})
        results["tested"].append("global_search")
        print("[OK] global_search")
    except Exception as e:
        results["failed"].append({"tool": "global_search", "error": str(e)[:100]})
        results["tested"].append("global_search (FAILED)")
        print(f"[FAIL] global_search: {e}")

    # Test ranked_search
    try:
        result = await client.call_tool(
            "ranked_search",
            {
                "query": "test",
                "limit": 10,
            },
        )
        results["working"].append({"tool": "ranked_search"})
        results["tested"].append("ranked_search")
        print("[OK] ranked_search")
    except Exception as e:
        results["failed"].append({"tool": "ranked_search", "error": str(e)[:100]})
        results["tested"].append("ranked_search (FAILED)")
        print(f"[FAIL] ranked_search: {e}")

    # Test search_similar (requires embedding)
    try:
        result = await client.call_tool(
            "search_similar",
            {
                "query": "test",
            },
        )
        results["working"].append({"tool": "search_similar"})
        results["tested"].append("search_similar")
        print("[OK] search_similar")
    except Exception as e:
        results["failed"].append({"tool": "search_similar", "error": str(e)[:100]})
        results["tested"].append("search_similar (FAILED)")
        print(f"[FAIL] search_similar: {e}")

    return results


async def test_mcp_acp_connection(client: RobotBrainClient) -> dict:
    """Test MCP/ACP connection."""
    print("\n[INFO] Testing MCP/ACP connection...")
    results = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Test list_acp_agents
    try:
        result = await client.call_tool("list_acp_agents", {})
        results["working"].append({"tool": "list_acp_agents"})
        results["tested"].append("ACP list_agents")
        print("[OK] ACP list_acp_agents")
    except Exception as e:
        results["failed"].append({"tool": "list_acp_agents", "error": str(e)[:100]})
        results["tested"].append("ACP list_acp_agents (FAILED)")
        print(f"[FAIL] ACP list_acp_agents: {e}")

    # Test acp_registry
    try:
        result = await client.call_tool("acp_registry", {})
        results["working"].append({"tool": "acp_registry"})
        results["tested"].append("ACP registry")
        print("[OK] ACP registry")
    except Exception as e:
        results["failed"].append({"tool": "acp_registry", "error": str(e)[:100]})
        results["tested"].append("ACP registry (FAILED)")
        print(f"[FAIL] ACP registry: {e}")

    # Test acp_router
    try:
        result = await client.call_tool("acp_router", {})
        results["working"].append({"tool": "acp_router"})
        results["tested"].append("ACP router")
        print("[OK] ACP router")
    except Exception as e:
        results["failed"].append({"tool": "acp_router", "error": str(e)[:100]})
        results["tested"].append("ACP router (FAILED)")
        print(f"[FAIL] ACP router: {e}")

    return results


async def main():
    """Run comprehensive tool probe."""
    print("=" * 70)
    print("  COMPREHENSIVE TOOL PROBE — Testing ALL tools")
    print("=" * 70)

    # Connect and probe
    client = await RobotBrainClient.connect()
    await client.get_workflow("general")
    await client.search_memory_first("test")

    # Run all tests
    tool_results = await probe_all_tools()
    ingest_results = await test_ingest_system(client)
    search_results = await test_search_system(client)
    mcp_acp_results = await test_mcp_acp_connection(client)

    # Compile full report
    report = {
        "title": "RoBoT Brain Comprehensive Tool Probe",
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "tool_probe": tool_results,
        "ingest_system": ingest_results,
        "search_system": search_results,
        "mcp_acp_connection": mcp_acp_results,
    }

    # Write JSON report
    report_path = Path(__file__).resolve().parent / "test_tool_probe_report.json"
    with open(report_path, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    print(f"\n[INFO] JSON report written to: {report_path}")

    # Write human-readable report
    output_path = Path(__file__).resolve().parent / "test_tool_probe_output.txt"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write("=" * 70 + "\n")
        f.write("  RoBoT Brain Comprehensive Tool Probe — Human Readable Report\n")
        f.write("=" * 70 + "\n\n")

        f.write(f"Total tools discovered: {tool_results['total_tools']}\n")
        f.write(f"Working: {len(tool_results['working'])}\n")
        f.write(f"Failed: {len(tool_results['failed'])}\n")
        f.write(f"Success rate: {tool_results['summary']['success_rate']}\n\n")

        f.write("--- WORKING TOOLS ---\n")
        for tool in tool_results["working"]:
            f.write(f"  [OK] {tool['name']} ({tool['time_ms']}ms)\n")

        f.write("\n--- FAILED TOOLS ---\n")
        for tool in tool_results["failed"]:
            f.write(f"  [FAIL] {tool['name']}: {tool['error'][:100]}\n")

        f.write("\n--- INGEST SYSTEM ---\n")
        f.write(f"Tested: {len(ingest_results['tested'])}\n")
        f.write(f"Working: {len(ingest_results['working'])}\n")
        f.write(f"Failed: {len(ingest_results['failed'])}\n")
        for test in ingest_results["tested"]:
            f.write(f"  - {test}\n")

        f.write("\n--- SEARCH SYSTEM ---\n")
        f.write(f"Tested: {len(search_results['tested'])}\n")
        f.write(f"Working: {len(search_results['working'])}\n")
        f.write(f"Failed: {len(search_results['failed'])}\n")
        for test in search_results["tested"]:
            f.write(f"  - {test}\n")

        f.write("\n--- MCP/ACP CONNECTION ---\n")
        f.write(f"Tested: {len(mcp_acp_results['tested'])}\n")
        f.write(f"Working: {len(mcp_acp_results['working'])}\n")
        f.write(f"Failed: {len(mcp_acp_results['failed'])}\n")
        for test in mcp_acp_results["tested"]:
            f.write(f"  - {test}\n")

        f.write("\n" + "=" * 70 + "\n")
        f.write("  End of Report\n")
        f.write("=" * 70 + "\n")

    print(f"[INFO] Human-readable report written to: {output_path}")
    print(f"\n[INFO] Summary:")
    print(f"  Working: {len(tool_results['working'])}/{tool_results['total_tools']}")
    print(f"  Failed: {len(tool_results['failed'])}/{tool_results['total_tools']}")
    print(f"  Success rate: {tool_results['summary']['success_rate']}")

    await client.stop()

    return report


if __name__ == "__main__":
    report = asyncio.run(main())
