"""
test_learning.py — Learning Engine functional tests.

Tests: Patterns, insights, recommendations, skill decay.
"""

import pytest
from helpers import assert_success


class TestLearningPatterns:
    """Learning pattern tests."""

    @pytest.mark.asyncio
    async def test_analyze_patterns(self, client):
        """analyze_patterns detects patterns in experiences."""
        result = await client.call_tool("analyze_patterns", {"lookback_days": 30})
        assert "patterns" in result or "data" in result or "results" in result

    @pytest.mark.asyncio
    async def test_get_patterns(self, client):
        """get_patterns returns detected patterns."""
        result = await client.call_tool("get_patterns", {})
        assert "patterns" in result or "data" in result

    @pytest.mark.asyncio
    async def test_get_insights(self, client):
        """get_insights returns actionable insights."""
        result = await client.call_tool("get_insights", {"limit": 10})
        assert "insights" in result or "data" in result


class TestLearningRecommendations:
    """Learning recommendation tests."""

    @pytest.mark.asyncio
    async def test_get_recommendations(self, client):
        """get_recommendations returns recommendations."""
        result = await client.call_tool("get_recommendations", {"category": "general"})
        assert "recommendations" in result or "data" in result

    @pytest.mark.asyncio
    async def test_get_skill_recommendations(self, client):
        """get_skill_recommendations returns skill suggestions."""
        result = await client.call_tool(
            "get_skill_recommendations", {"task": "testing task"}
        )
        assert "skills" in result or "recommendations" in result or "data" in result

    @pytest.mark.asyncio
    async def test_get_reputation(self, client):
        """get_reputation returns quality score for a tool."""
        result = await client.call_tool("get_reputation", {"tool_name": "store_memory"})
        assert "score" in result or "reputation" in result or "quality" in result


class TestLearningDecay:
    """Skill decay tests."""

    @pytest.mark.asyncio
    async def test_apply_skill_decay(self, client):
        """apply_skill_decay applies decay to skill mastery."""
        result = await client.call_tool("apply_skill_decay", {})
        assert assert_success(result)
