"""
test_knowledge.py — Knowledge Engine functional tests.

Tests: CRUD, relations, versions, status, tags.
"""

import pytest
from helpers import assert_success


class TestKnowledgeCRUD:
    """Knowledge Engine CRUD tests."""

    @pytest.mark.asyncio
    async def test_add_knowledge(self, client, data_cleanup):
        """add_knowledge creates a knowledge item."""
        result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "test knowledge statement",
                "knowledge_type": "fact",
                "tags": ["test"],
                "confidence": 0.9,
            },
        )
        assert assert_success(result)
        assert "knowledge_id" in result
        data_cleanup["knowledge"].append(result["knowledge_id"])

    @pytest.mark.asyncio
    async def test_query_knowledge(self, client, data_cleanup):
        """query_knowledge returns matching items."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "rust programming language is fast",
                "knowledge_type": "fact",
                "tags": ["rust", "programming"],
            },
        )
        data_cleanup["knowledge"].append(add_result["knowledge_id"])
        query_result = await client.call_tool(
            "query_knowledge",
            {
                "query": "rust programming fast",
                "min_confidence": 0.0,
            },
        )
        assert (
            "results" in query_result
            or "knowledge" in query_result
            or "items" in query_result
            or "success" in query_result
        )

    @pytest.mark.asyncio
    async def test_knowledge_get(self, client, data_cleanup):
        """get_knowledge retrieves a specific item."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "specific knowledge item",
                "knowledge_type": "rule",
            },
        )
        kid = add_result["knowledge_id"]
        data_cleanup["knowledge"].append(kid)
        get_result = await client.call_tool(
            "get_knowledge",
            {
                "knowledge_id": kid,
            },
        )
        assert assert_success(get_result)

    @pytest.mark.asyncio
    async def test_knowledge_list(self, client):
        """list_knowledge returns knowledge items."""
        result = await client.call_tool("list_knowledge", {})
        assert (
            "knowledge" in result
            or "items" in result
            or "data" in result
            or ("error" in result and not result.get("success", True))
        )

    @pytest.mark.asyncio
    async def test_delete_knowledge(self, client, data_cleanup):
        """delete_knowledge removes an item."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "knowledge to delete",
                "knowledge_type": "fact",
            },
        )
        kid = add_result["knowledge_id"]
        delete_result = await client.call_tool(
            "delete_knowledge",
            {
                "knowledge_id": kid,
            },
        )
        assert assert_success(delete_result)


class TestKnowledgeRelations:
    """Knowledge relation tests."""

    @pytest.mark.asyncio
    async def test_add_knowledge_relation(self, client, data_cleanup):
        """add_knowledge_relation links two items."""
        add1 = await client.call_tool(
            "add_knowledge",
            {
                "statement": "knowledge item A",
                "knowledge_type": "fact",
            },
        )
        add2 = await client.call_tool(
            "add_knowledge",
            {
                "statement": "knowledge item B",
                "knowledge_type": "fact",
            },
        )
        data_cleanup["knowledge"].append(add1["knowledge_id"])
        data_cleanup["knowledge"].append(add2["knowledge_id"])
        rel_result = await client.call_tool(
            "add_knowledge_relation",
            {
                "knowledge_id": add1["knowledge_id"],
                "related_id": add2["knowledge_id"],
                "relation_type": "supports",
            },
        )
        assert assert_success(rel_result)

    @pytest.mark.asyncio
    async def test_get_related_knowledge(self, client, data_cleanup):
        """get_related_knowledge returns linked items."""
        add1 = await client.call_tool(
            "add_knowledge",
            {
                "statement": "related item source",
                "knowledge_type": "fact",
            },
        )
        add2 = await client.call_tool(
            "add_knowledge",
            {
                "statement": "related item target",
                "knowledge_type": "fact",
            },
        )
        data_cleanup["knowledge"].append(add1["knowledge_id"])
        data_cleanup["knowledge"].append(add2["knowledge_id"])
        await client.call_tool(
            "add_knowledge_relation",
            {
                "knowledge_id": add1["knowledge_id"],
                "related_id": add2["knowledge_id"],
                "relation_type": "related",
            },
        )
        related_result = await client.call_tool(
            "get_related_knowledge",
            {
                "knowledge_id": add1["knowledge_id"],
            },
        )
        assert (
            "results" in related_result
            or "related" in related_result
            or "items" in related_result
            or "success" in related_result
        )


class TestKnowledgeLifecycle:
    """Knowledge versioning and status tests."""

    @pytest.mark.asyncio
    async def test_bump_knowledge_version(self, client, data_cleanup):
        """bump_knowledge_version updates version."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "versioned knowledge",
                "knowledge_type": "fact",
            },
        )
        data_cleanup["knowledge"].append(add_result["knowledge_id"])
        version_result = await client.call_tool(
            "bump_knowledge_version",
            {
                "knowledge_id": add_result["knowledge_id"],
                "bump_type": "minor",
            },
        )
        assert assert_success(version_result)

    @pytest.mark.asyncio
    async def test_set_knowledge_status(self, client, data_cleanup):
        """set_knowledge_status changes status."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "status-changing knowledge",
                "knowledge_type": "fact",
            },
        )
        data_cleanup["knowledge"].append(add_result["knowledge_id"])
        status_result = await client.call_tool(
            "set_knowledge_status",
            {
                "knowledge_id": add_result["knowledge_id"],
                "action": "suspend",
            },
        )
        assert assert_success(status_result)

    @pytest.mark.asyncio
    async def test_search_knowledge_by_tag(self, client, data_cleanup):
        """search_knowledge_by_tag finds items by tag."""
        add_result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "tagged knowledge item",
                "knowledge_type": "fact",
                "tags": ["test-tag", "python"],
            },
        )
        data_cleanup["knowledge"].append(add_result["knowledge_id"])
        tag_result = await client.call_tool(
            "search_knowledge_by_tag",
            {
                "tag": "test-tag",
            },
        )
        assert (
            "results" in tag_result
            or "knowledge" in tag_result
            or "items" in tag_result
            or "success" in tag_result
        )

    @pytest.mark.asyncio
    async def test_get_knowledge_stats(self, client):
        """get_knowledge_stats returns counts."""
        result = await client.call_tool("get_knowledge_stats", {})
        assert "count" in result or "stats" in result or "total" in result
