"""
test_experience.py — Experience Engine functional tests.

Tests: Record, search, stats, filter, archive.
"""

import pytest
from helpers import assert_success


class TestExperienceCRUD:
    """Experience Engine CRUD tests."""

    @pytest.mark.asyncio
    async def test_record_experience(self, client, data_cleanup):
        """record_experience creates an entry."""
        result = await client.call_tool(
            "record_experience",
            {
                "title": "test experience",
                "description": "this is a test experience description",
                "experience_type": "task",
                "outcome": "Success",
            },
        )
        assert assert_success(result)
        assert "id" in result

    @pytest.mark.asyncio
    async def test_get_experience(self, client):
        """get_experience retrieves a specific entry."""
        record_result = await client.call_tool(
            "record_experience",
            {
                "title": "specific experience",
                "description": "a specific experience to retrieve",
                "experience_type": "task",
                "outcome": "Success",
            },
        )
        exp_id = record_result["id"]
        get_result = await client.call_tool("get_experience", {"id": exp_id})
        assert assert_success(get_result)

    @pytest.mark.asyncio
    async def test_list_experiences(self, client):
        """list_experiences returns recent entries."""
        result = await client.call_tool("list_experiences", {"limit": 10})
        assert "experiences" in result or "data" in result or "items" in result

    @pytest.mark.asyncio
    async def test_list_experiences_filtered(self, client):
        """list_experiences with type filter works."""
        result = await client.call_tool(
            "list_experiences",
            {
                "experience_type": "task",
                "limit": 10,
            },
        )
        assert "experiences" in result or "data" in result

    @pytest.mark.asyncio
    async def test_experience_stats(self, client):
        """get_experience_stats returns counts."""
        result = await client.call_tool("get_experience_stats", {"period": "all"})
        assert "count" in result or "stats" in result or "total" in result


class TestExperienceOperations:
    """Experience operation tests."""

    @pytest.mark.asyncio
    async def test_archive_experience(self, client):
        """archive_experience soft-deletes an entry."""
        record_result = await client.call_tool(
            "record_experience",
            {
                "title": "experience to archive",
                "description": "will be archived",
                "experience_type": "task",
                "outcome": "Success",
            },
        )
        exp_id = record_result["id"]
        archive_result = await client.call_tool(
            "archive_experience",
            {
                "experience_id": exp_id,
            },
        )
        assert assert_success(archive_result)

    @pytest.mark.asyncio
    async def test_record_observation(self, client):
        """record_observation creates an observation."""
        result = await client.call_tool(
            "record_observation",
            {
                "content": "test observation content",
                "context": "test context",
                "observation_type": "pattern",
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_create_reflection(self, client):
        """create_reflection creates a reflection."""
        result = await client.call_tool(
            "create_reflection",
            {
                "title": "test reflection",
                "content": "this is a test reflection content",
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_list_reflections(self, client):
        """list_reflections returns reflections."""
        result = await client.call_tool(
            "list_reflections_by_status", {"status": "active"}
        )
        assert "reflections" in result or "data" in result or "items" in result
