"""
test_smoke.py — Smoke tests for test_suite2.1 (Python-only test suite).

Verifies the robot_brain compiled binary responds to MCP calls.
Replaces the duplicated test_smoke.py and test_smoke_full.py from test_suite2.
"""

import pytest


class TestSmoke:
    """Basic smoke tests — does the server respond?"""

    @pytest.mark.asyncio
    async def test_list_tools(self, client):
        """Verify tools/list returns a non-empty list."""
        tools = await client.list_tools()
        assert len(tools) > 0

    @pytest.mark.asyncio
    async def test_get_workflow(self, client):
        """Verify get_workflow returns valid response."""
        result = await client.call_tool("get_workflow", {"purpose": "test"})
        assert "workflow" in result or "status" in result or "version" in result

    @pytest.mark.asyncio
    async def test_search_memory(self, client):
        """Verify search_memory returns valid response."""
        result = await client.call_tool("search_memory", {"query": "test"})
        assert "results" in result or "memories" in result or "data" in result


class TestSmokeToolList:
    """Extended smoke tests — verify tool catalog structure."""

    @pytest.mark.asyncio
    async def test_tool_count_above_threshold(self, client):
        """Verify we have a reasonable number of tools (>= 20)."""
        tools = await client.list_tools()
        assert len(tools) >= 20

    @pytest.mark.asyncio
    async def test_tool_schema_present(self, client):
        """Verify each tool has name, description, and inputSchema."""
        tools = await client.list_tools()
        for tool in tools:
            assert "name" in tool, f"Tool missing name: {tool}"
            assert "description" in tool, f"Tool missing description: {tool}"
            assert "inputSchema" in tool, f"Tool missing inputSchema: {tool}"
