"""
test_personality.py — Personality system functional tests.

Tests: Traits, presets, decision making, emotional state.
"""

import pytest
from helpers import assert_success


class TestPersonality:
    """Personality system tests."""

    @pytest.mark.asyncio
    async def test_get_personality(self, client):
        """get_personality returns current personality state."""
        result = await client.call_tool("get_personality", {})
        assert assert_success(result) or "personality" in result

    @pytest.mark.asyncio
    async def test_set_personality_traits(self, client):
        """set_personality_traits updates individual traits."""
        result = await client.call_tool(
            "set_personality_traits",
            {
                "curiosity": 0.8,
                "caution": 0.3,
                "creativity": 0.7,
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_apply_personality_preset(self, client):
        """apply_personality_preset applies a named preset."""
        result = await client.call_tool(
            "apply_personality_preset", {"preset": "balanced"}
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_list_personality_presets(self, client):
        """list_personality_presets returns available presets."""
        result = await client.call_tool("list_personality_presets", {})
        assert "presets" in result or "data" in result

    @pytest.mark.asyncio
    async def test_get_personality_decision(self, client):
        """get_personality_decision returns personality-driven decision."""
        result = await client.call_tool(
            "get_personality_decision",
            {
                "confidence": 0.7,
                "potential_gain": 0.5,
                "potential_loss": 0.3,
                "uncertainty": 0.2,
            },
        )
        assert assert_success(result) or "decision" in result
