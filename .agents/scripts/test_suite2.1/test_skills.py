"""
test_skills.py — Skills registry functional tests.

Tests: Register, execute, discover, enable/disable, metrics.
"""

import pytest
from helpers import assert_success


class TestSkillsCRUD:
    """Skills CRUD tests."""

    @pytest.mark.asyncio
    async def test_register_skill(self, client):
        """register_skill adds a new skill."""
        result = await client.call_tool(
            "register_skill",
            {
                "name": "test-skill",
                "description": "a test skill for testing",
                "category": "learning",
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_list_skills(self, client):
        """list_skills returns registered skills."""
        result = await client.call_tool("list_skills", {})
        assert "skills" in result or "data" in result or "items" in result

    @pytest.mark.asyncio
    async def test_get_skill(self, client):
        """get_skill retrieves a specific skill."""
        # Register a skill first
        reg_result = await client.call_tool(
            "register_skill",
            {
                "name": "test-skill",
                "description": "a test skill",
                "category": "learning",
            },
        )
        skill_id = reg_result.get("skill_id") or reg_result.get("id", "")
        result = await client.call_tool("get_skill", {"skill_id": skill_id})
        assert assert_success(result) or "skill" in result

    @pytest.mark.asyncio
    async def test_search_skills(self, client):
        """search_skills finds skills by name/description."""
        result = await client.call_tool("search_skills", {"query": "test"})
        assert "skills" in result or "results" in result


class TestSkillsOperations:
    """Skills operation tests."""

    @pytest.mark.asyncio
    async def test_enable_disable_skill(self, client):
        """enable_disable_skill toggles skill state."""
        result = await client.call_tool(
            "enable_disable_skill",
            {
                "skill_id": "test-skill",
                "enabled": False,
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_execute_skill(self, client):
        """execute_skill runs a skill."""
        reg_result = await client.call_tool(
            "register_skill",
            {
                "name": "execute-test",
                "description": "a skill to execute",
                "category": "learning",
            },
        )
        skill_id = reg_result.get("skill_id") or reg_result.get("id", "")
        result = await client.call_tool(
            "execute_skill",
            {
                "skill_id": skill_id,
                "parameters": {},
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_discover_skill(self, client):
        """discover_skill discovers a new skill from experience."""
        result = await client.call_tool(
            "discover_skill",
            {
                "name": "discovered-skill",
                "description": "a discovered skill",
                "category": "learning",
                "source_experience_id": "00000000-0000-0000-0000-000000000001",
            },
        )
        assert assert_success(result)


class TestSkillsMetrics:
    """Skills metrics tests."""

    @pytest.mark.asyncio
    async def test_get_skill_metrics(self, client):
        """get_skill_metrics returns skill execution data."""
        result = await client.call_tool(
            "get_skill_metrics", {"skill_id": "execute-test"}
        )
        assert "metrics" in result or "data" in result

    @pytest.mark.asyncio
    async def test_clear_skill_metrics(self, client):
        """clear_skill_metrics resets skill metrics."""
        result = await client.call_tool(
            "clear_skill_metrics", {"skill_id": "execute-test"}
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_get_skill_stats(self, client):
        """get_skill_stats returns skill statistics."""
        result = await client.call_tool("get_skill_stats", {})
        assert "stats" in result or "skills" in result
