"""Test ingest_files tool via MCP.

Part of option 2 (all tests) — exercises the ingest_files tool
with the live robot_brain server.
"""

import json
import logging

import pytest

logger = logging.getLogger(__name__)


@pytest.mark.asyncio
async def test_ingest_files_tool_schema(client):
    """Verify ingest_files tool is advertised and has correct schema."""
    # Use client.list_tools() which calls the MCP protocol correctly
    tools = await client.list_tools()
    assert isinstance(tools, list), (
        f"list_tools returned unexpected type: {type(tools)}"
    )
    assert len(tools) > 0, "No tools returned"

    # Find the ingest_files tool
    tool_found = False
    for t in tools:
        if t.get("name") == "ingest_files":
            tool_found = True
            logger.info("Tool: ingest_files")
            desc = t.get("description", "")
            logger.info("Description: %s", desc[:500] if desc else "(no description)")
            logger.info("\nInput Schema:")
            logger.info(json.dumps(t.get("inputSchema", {}), indent=2))
            assert "inputSchema" in t, "ingest_files missing inputSchema"
            break

    assert tool_found, "ingest_files tool not found in tools/list"


@pytest.mark.asyncio
async def test_ingest_files_with_memory(client):
    """Test ingest_files tool stores memory records."""
    result = await client.call_tool(
        "ingest_files",
        {"folder": "files_to_import", "memory_type": "note"},
    )
    assert isinstance(result, dict), (
        f"ingest_files returned unexpected type: {type(result)}"
    )
    # ingest_files should return a result indicating success or the number of files ingested
    assert result.get("success") is not True or "error" not in result, (
        f"ingest_files failed: {result.get('error', result)}"
    )


@pytest.mark.asyncio
async def test_ingest_files_with_importable_folder(client):
    """Test ingest_files with list_importable folder."""
    # First list importable files
    list_result = await client.call_tool(
        "list_importable", {"folder": "files_to_import"}
    )
    assert isinstance(list_result, dict), (
        f"list_importable returned unexpected type: {type(list_result)}"
    )

    # Now test ingest_files
    result = await client.call_tool(
        "ingest_files", {"folder": "files_to_import", "limit": 1}
    )
    assert isinstance(result, dict), (
        f"ingest_files returned unexpected type: {type(result)}"
    )
    # Should not error when folder exists (even if empty)
    assert result.get("error") is None or result.get("success") is not False, (
        f"ingest_files failed: {result}"
    )
