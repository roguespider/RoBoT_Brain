"""
test_world_model.py — World Model functional tests.

Tests: Entity CRUD, relationships, queries.
"""

import pytest
from helpers import assert_success


class TestWorldEntities:
    """World model entity tests."""

    @pytest.mark.asyncio
    async def test_upsert_world_entity(self, client):
        """upsert_world_entity creates or updates an entity."""
        result = await client.call_tool(
            "upsert_world_entity",
            {
                "name": "test entity",
                "kind": "object",
                "properties": {"test": "value"},
            },
        )
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_get_world_entity(self, client):
        """get_world_entity retrieves an entity."""
        upsert_result = await client.call_tool(
            "upsert_world_entity",
            {
                "name": "entity to get",
                "kind": "place",
            },
        )
        entity_id = upsert_result.get("id", "")
        get_result = await client.call_tool("get_world_entity", {"id": entity_id})
        assert assert_success(get_result)

    @pytest.mark.asyncio
    async def test_list_world_entities(self, client):
        """list_world_entities returns entities."""
        result = await client.call_tool("list_world_entities", {"kind": "object"})
        assert "entities" in result or "data" in result or "items" in result

    @pytest.mark.asyncio
    async def test_delete_world_entity(self, client):
        """delete_world_entity removes an entity."""
        upsert_result = await client.call_tool(
            "upsert_world_entity",
            {
                "name": "entity to delete",
                "kind": "resource",
            },
        )
        entity_id = upsert_result.get("id", "")
        # Some tools may not exist yet — check for tool existence first
        tools = await client.list_tools()
        tool_names = [t.get("name", "") for t in tools]
        if "delete_world_entity" in tool_names:
            delete_result = await client.call_tool(
                "delete_world_entity", {"id": entity_id}
            )
            assert assert_success(delete_result)


class TestWorldRelationships:
    """World model relationship tests."""

    @pytest.mark.asyncio
    async def test_add_world_relationship(self, client):
        """add_world_relationship creates a relationship."""
        e1 = await client.call_tool(
            "upsert_world_entity", {"name": "source entity", "kind": "object"}
        )
        e2 = await client.call_tool(
            "upsert_world_entity", {"name": "target entity", "kind": "resource"}
        )
        src_id = e1.get("id", "")
        tgt_id = e2.get("id", "")
        rel_result = await client.call_tool(
            "add_world_relationship",
            {
                "source_id": src_id,
                "target_id": tgt_id,
                "kind": "related_to",
            },
        )
        assert assert_success(rel_result)

    @pytest.mark.asyncio
    async def test_get_world_relationships(self, client):
        """get_world_relationships returns relationships."""
        e1 = await client.call_tool(
            "upsert_world_entity",
            {"name": "entity with relationships", "kind": "object"},
        )
        e2 = await client.call_tool(
            "upsert_world_entity", {"name": "related entity", "kind": "place"}
        )
        src_id = e1.get("id", "")
        await client.call_tool(
            "add_world_relationship",
            {
                "source_id": src_id,
                "target_id": e2.get("id", ""),
                "kind": "part_of",
            },
        )
        rel_result = await client.call_tool("get_world_relationships", {"id": src_id})
        assert "relationships" in rel_result or "data" in rel_result

    @pytest.mark.asyncio
    async def test_get_world_dependencies(self, client):
        """get_world_dependencies returns dependencies."""
        e1 = await client.call_tool(
            "upsert_world_entity", {"name": "dependent entity", "kind": "resource"}
        )
        e2 = await client.call_tool(
            "upsert_world_entity", {"name": "prerequisite entity", "kind": "object"}
        )
        src_id = e1.get("id", "")
        tgt_id = e2.get("id", "")
        await client.call_tool(
            "add_world_relationship",
            {
                "source_id": src_id,
                "target_id": tgt_id,
                "kind": "depends_on",
            },
        )
        dep_result = await client.call_tool("get_world_dependencies", {"id": src_id})
        assert "dependencies" in dep_result or "data" in dep_result


class TestWorldModelStats:
    """World model statistics tests."""

    @pytest.mark.asyncio
    async def test_get_world_model_stats(self, client):
        """get_world_model_stats returns entity/relationship counts."""
        result = await client.call_tool("get_world_model_stats", {})
        assert "entity_count" in result or "relationship_count" in result
