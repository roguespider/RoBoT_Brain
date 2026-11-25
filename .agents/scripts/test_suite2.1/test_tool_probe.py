#!/usr/bin/env python3
"""
Comprehensive tool probe — tests ALL tools and generates detailed report.

Tests:
- Which tools work vs fail
- Ingest system (file types)
- MCP/ACP connection
- Search system
- All 200+ tools

Output: Results integrated into test_output.txt by main.py
"""

import logging
import sys
import time
from pathlib import Path

# Add current dir to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parent))

from mcp_client import RobotBrainClient

logger = logging.getLogger(__name__)


async def _safe_call_tool(client, tool_name, params):
    """Call a tool safely, catching OSError and returning (success, result_or_error).

    Only catches OSError (connection/IO errors). Other exceptions propagate.
    """
    try:
        result = await client.call_tool(tool_name, params)
        return True, result
    except OSError as e:
        return False, f"OSError: {e}"


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
    result = defaults.get(tool_name, {})
    return result if isinstance(result, dict) else {}


async def probe_all_tools() -> dict:
    """Probe all tools and return results.

    HALTS on first tool failure — a failing tool means the binary is unusable.
    """
    logger.info("[INFO] Starting comprehensive tool probe...")
    logger.info("[INFO] Connecting to robot_brain...")

    # Connect
    client = await RobotBrainClient.connect()
    logger.info("[OK] Connected to robot_brain")

    # Initialize workflow gate
    logger.info("[INFO] Initializing workflow gate...")
    await client.get_workflow("general")
    await client.search_memory_first("test")
    logger.info("[OK] Workflow gate initialized")

    # List all tools
    logger.info("[INFO] Listing all tools...")
    tools = await client.list_tools()
    tool_names = [t["name"] for t in tools]
    logger.info("[INFO] Found %d tools", len(tool_names))

    # Probe each tool — HALT on first failure
    results: dict = {
        "total_tools": len(tool_names),
        "working": [],
        "failed": [],
        "skipped": [],
        "by_category": {},
    }

    logger.info("\n[INFO] Probing each tool...\n")

    for tool_name in tool_names:
        defaults = get_tool_defaults(tool_name)

        start = time.time()
        success, result_or_error = await _safe_call_tool(client, tool_name, defaults)
        elapsed = time.time() - start

        if success:
            result = result_or_error
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
            logger.info(status)
        else:
            error_msg = str(result_or_error)[:200]
            results["failed"].append(
                {
                    "name": tool_name,
                    "error": error_msg,
                }
            )
            status = f"[FAIL] {tool_name}: {error_msg[:100]}"
            logger.info(status)
            await client.stop()
            # HALT: binary is unusable, must fix the failing tool
            logger.error(
                "[HALT] Tool '%s' failed — binary is unusable. Fix before continuing.",
                tool_name,
            )
            raise RuntimeError(f"Tool probe HALT: {tool_name} failed with: {error_msg}")

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
    logger.info("\n[INFO] Testing ingest system...")
    results: dict = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Create test files
    test_dir = Path(__file__).resolve().parent / "test_ingest_files"
    test_dir.mkdir(exist_ok=True)

    # Supported file types from file_collector.rs:
    # TEXT: txt, md, rst, csv, log, xml, html, htm
    # CODE: rs, toml, yaml, yml, env, gitignore, dockerfile,
    # py, js, ts, java, c, cpp, h, hpp, go, rb, php, sql,
    # sh, bash, zsh, ps1, bat, cmd
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
            logger.info("[OK] %s file ingest", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} file ingest (FAILED)")
            logger.info("[FAIL] %s file ingest: %s", ext, e)

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
            logger.info("[OK] %s code ingest", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} code ingest (FAILED)")
            logger.info("[FAIL] %s code ingest: %s", ext, e)

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
            logger.info("[OK] %s script ingest", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} script ingest (FAILED)")
            logger.info("[FAIL] %s script ingest: %s", ext, e)

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
            logger.info("[OK] %s config ingest", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} config ingest (FAILED)")
            logger.info("[FAIL] %s config ingest: %s", ext, e)

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
            logger.info("[OK] %s subtitle ingest", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} subtitle ingest (FAILED)")
            logger.info("[FAIL] %s subtitle ingest: %s", ext, e)

    # --- IMAGE FILES (metadata only) ---
    # Create minimal valid files for each image type
    image_files: list[tuple[str, bytes | str]] = [
        ("test.png", b"\x89PNG\r\n\x1a\n" + b"\x00" * 100),  # PNG header
        ("test.jpg", b"\xff\xd8\xff\xe0" + b"\x00" * 100),  # JPEG header
        (
            "test.svg",
            (
                '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">'
                '<rect width="50" height="50"/></svg>'
            ),
        ),
    ]

    for filename, file_content in image_files:
        file_path = test_dir / filename
        if isinstance(file_content, bytes):
            file_path.write_bytes(file_content)
        else:
            file_path.write_text(str(file_content), encoding="utf-8")
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
            logger.info("[OK] %s image ingest (metadata)", ext)
        except OSError as e:
            results["failed"].append({"type": ext, "error": str(e)[:100]})
            results["tested"].append(f"{ext} image ingest (FAILED)")
            logger.error("[FAIL] %s image ingest: %s", ext, e)

    # Cleanup
    for f in test_dir.iterdir():
        if f.is_file():
            try:
                f.unlink()
            except FileNotFoundError:
                logger.debug("File not found during cleanup: %s", f)
    try:
        test_dir.rmdir()
    except OSError:
        logger.debug("Failed to remove test directory: %s", test_dir)

    return results


async def test_search_system(client: RobotBrainClient) -> dict:
    """Test search system."""
    logger.info("\n[INFO] Testing search system...")
    results: dict = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Test search_memory
    try:
        await client.call_tool("search_memory", {"query": "test"})
        results["working"].append({"tool": "search_memory"})
        results["tested"].append("search_memory")
        logger.info("[OK] search_memory")
    except OSError as e:
        results["failed"].append({"tool": "search_memory", "error": str(e)[:100]})
        results["tested"].append("search_memory (FAILED)")
        logger.info("[FAIL] search_memory: %s", e)

    # Test global_search
    try:
        await client.call_tool(
            "global_search",
            {
                "query": "test",
                "limit": 10,
            },
        )
        results["working"].append({"tool": "global_search"})
        results["tested"].append("global_search")
        logger.info("[OK] global_search")
    except OSError as e:
        results["failed"].append({"tool": "global_search", "error": str(e)[:100]})
        results["tested"].append("global_search (FAILED)")
        logger.info("[FAIL] global_search: %s", e)

    # Test ranked_search
    try:
        await client.call_tool(
            "ranked_search",
            {
                "query": "test",
                "limit": 10,
            },
        )
        results["working"].append({"tool": "ranked_search"})
        results["tested"].append("ranked_search")
        logger.info("[OK] ranked_search")
    except OSError as e:
        results["failed"].append({"tool": "ranked_search", "error": str(e)[:100]})
        results["tested"].append("ranked_search (FAILED)")
        logger.info("[FAIL] ranked_search: %s", e)

    # Test search_similar (requires embedding)
    try:
        await client.call_tool(
            "search_similar",
            {
                "query": "test",
            },
        )
        results["working"].append({"tool": "search_similar"})
        results["tested"].append("search_similar")
        logger.info("[OK] search_similar")
    except OSError as e:
        results["failed"].append({"tool": "search_similar", "error": str(e)[:100]})
        results["tested"].append("search_similar (FAILED)")
        logger.info("[FAIL] search_similar: %s", e)

    return results


async def test_mcp_acp_connection(client: RobotBrainClient) -> dict:
    """Test MCP/ACP connection."""
    logger.info("\n[INFO] Testing MCP/ACP connection...")
    results: dict = {
        "tested": [],
        "working": [],
        "failed": [],
    }

    # Test list_acp_agents
    try:
        await client.call_tool("list_acp_agents", {})
        results["working"].append({"tool": "list_acp_agents"})
        results["tested"].append("ACP list_agents")
        logger.info("[OK] ACP list_acp_agents")
    except OSError as e:
        results["failed"].append({"tool": "list_acp_agents", "error": str(e)[:100]})
        results["tested"].append("ACP list_acp_agents (FAILED)")
        logger.info("[FAIL] ACP list_acp_agents: %s", e)

    # Test acp_registry
    try:
        await client.call_tool("acp_registry", {})
        results["working"].append({"tool": "acp_registry"})
        results["tested"].append("ACP registry")
        logger.info("[OK] ACP registry")
    except OSError as e:
        results["failed"].append({"tool": "acp_registry", "error": str(e)[:100]})
        results["tested"].append("ACP registry (FAILED)")
        logger.info("[FAIL] ACP registry: %s", e)

    # Test acp_router
    try:
        await client.call_tool("acp_router", {})
        results["working"].append({"tool": "acp_router"})
        results["tested"].append("ACP router")
        logger.info("[OK] ACP router")
    except OSError as e:
        results["failed"].append({"tool": "acp_router", "error": str(e)[:100]})
        results["tested"].append("ACP router (FAILED)")
        logger.info("[FAIL] ACP router: %s", e)

    return results


async def main():
    """Run comprehensive tool probe."""
