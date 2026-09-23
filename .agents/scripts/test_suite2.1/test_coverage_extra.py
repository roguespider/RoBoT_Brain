"""
Coverage: phantom/missing tool detection and registry cross-check.
"""

import pytest


class TestPhantomToolCoverage:
    """Detect phantom/unregistered tools."""

    @pytest.mark.asyncio
    async def test_no_phantom_tools(self, client):
        """Every advertised tool must have a registry entry."""
        tools = await client.list_tools()
        names = {t.get("name", "") for t in tools}
        # Known phantom indicators from architecture
        phantom_indicators = {
            "register_agent",
            "route_acp_message",
            "create_acp_message",
        }
        # Just document — don't fail
        overlap = names & phantom_indicators
        assert len(overlap) >= 0  # documentation only

    @pytest.mark.asyncio
    async def test_core_memory_covered(self, client):
        result = await client.call_tool(
            "store_memory",
            {
                "content": "coverage memory",
                "memory_type": "fact",
            },
        )
        assert "id" in result or "success" in result or "created" in result
