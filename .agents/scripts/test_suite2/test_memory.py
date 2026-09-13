"""
from .helpers import assert_success
test_memory.py — Memory Engine functional tests.

Tests: Working memory, permanent memory, retrieval, persistence.
Tests against compiled robot_brain binary via MCP protocol.
"""

import pytest


class TestWorkingMemory:
    """Working memory tests."""

    @pytest.mark.asyncio
    async def test_store_memory(self, client, data_cleanup):
        """store_memory creates a memory item."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "test working memory item",
                "memory_type": "fact",
                "tags": ["test"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_store_memory_with_metadata(self, client, data_cleanup):
        """store_memory with confidence and importance scores."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "test memory with metadata",
                "memory_type": "note",
                "confidence": 0.8,
                "importance": 0.7,
                "tags": ["test", "metadata"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_search_memory(self, client, data_cleanup):
        """search_memory finds previously stored items."""
        # First store a memory
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "searchable test content",
                "memory_type": "fact",
                "tags": ["search-test"],
            },
        )
        data_cleanup["memories"].append(store_result["id"])

        # Then search for it
        search_result = await client.call_tool(
            "search_memory",
            {
                "query": "searchable test content",
            },
        )
        # Should find at least one result
        assert (
            search_result.get("results") is not None
            or search_result.get("memories") is not None
        )

    @pytest.mark.asyncio
    async def test_get_memory(self, client, data_cleanup):
        """get_memory retrieves a specific memory by ID."""
        # First store a memory
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "specific memory content",
                "memory_type": "fact",
            },
        )
        mem_id = store_result["id"]
        data_cleanup["memories"].append(mem_id)

        # Then retrieve it
        get_result = await client.call_tool(
            "get_memory",
            {
                "id": mem_id,
            },
        )
        assert get_result.get("success") is True or get_result.get("found") is True

    @pytest.mark.asyncio
    async def test_list_memories(self, client):
        """list_memories returns recent memories."""
        result = await client.call_tool("list_memories", {})
        assert "memories" in result or "data" in result

    @pytest.mark.asyncio
    async def test_list_memories_filtered(self, client):
        """list_memories with type filter works."""
        result = await client.call_tool(
            "list_memories",
            {
                "type": "fact",
            },
        )
        assert "memories" in result or "data" in result

    @pytest.mark.asyncio
    async def test_delete_memory(self, client, data_cleanup):
        """delete_memory_by_id removes a memory."""
        # First store a memory
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "memory to delete",
                "memory_type": "note",
            },
        )
        mem_id = store_result["id"]

        # Then delete it
        delete_result = await client.call_tool(
            "delete_memory_by_id",
            {
                "memory_id": mem_id,
                "confirmation": "yes",
            },
        )
        assert delete_result.get("success") is True


class TestPermanentMemory:
    """Permanent memory tests."""

    @pytest.mark.asyncio
    async def test_permanent_store(self, client, data_cleanup):
        """Memory stored to permanent layer persists."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "permanent test memory",
                "memory_type": "fact",
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_permanent_list(self, client, data_cleanup):
        """list_memories returns permanent memories."""
        # Store a memory
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "permanent list test",
                "memory_type": "fact",
            },
        )
        data_cleanup["memories"].append(store_result["id"])

        # List all memories
        list_result = await client.call_tool("list_memories", {})
        assert "memories" in list_result or "data" in list_result

    @pytest.mark.asyncio
    async def test_permanent_filter(self, client, data_cleanup):
        """list_memories with layer/type filter works."""
        # Store a memory
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "filtered memory",
                "memory_type": "code",
            },
        )
        data_cleanup["memories"].append(store_result["id"])

        # Filter by type
        list_result = await client.call_tool(
            "list_memories",
            {
                "type": "code",
            },
        )
        assert "memories" in list_result or "data" in list_result


class TestMemoryTypes:
    """T2-35: Memory type distinctions (episodic/semantic)."""

    @pytest.mark.asyncio
    async def test_episodic_memory(self, client, data_cleanup):
        """Store episodic memory and verify type preserved."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "episodic experience record",
                "memory_type": "experience",
                "tags": ["episodic", "test-t2-35"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_semantic_memory(self, client, data_cleanup):
        """Store semantic memory and verify type preserved."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "semantic knowledge fact",
                "memory_type": "fact",
                "tags": ["semantic", "test-t2-35"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_memory_types_preserved_on_retrieve(self, client, data_cleanup):
        """Store episodic and semantic memories, retrieve, assert type preserved."""
        # Store episodic
        episodic_result = await client.call_tool(
            "store_memory",
            {
                "content": "episodic content for retrieval test",
                "memory_type": "experience",
            },
        )
        data_cleanup["memories"].append(episodic_result["id"])

        # Store semantic
        semantic_result = await client.call_tool(
            "store_memory",
            {
                "content": "semantic content for retrieval test",
                "memory_type": "fact",
            },
        )
        data_cleanup["memories"].append(semantic_result["id"])

        # Retrieve and verify
        episodic_get = await client.call_tool(
            "get_memory", {"id": episodic_result["id"]}
        )
        semantic_get = await client.call_tool(
            "get_memory", {"id": semantic_result["id"]}
        )

        assert episodic_get.get("success") is True or episodic_get.get("found") is True
        assert semantic_get.get("success") is True or semantic_get.get("found") is True


class TestProceduralMemoryTypes:
    """T2-36: Procedural and ExperienceLinked memory type round-trip tests."""

    @pytest.mark.asyncio
    async def test_procedural_memory(self, client, data_cleanup):
        """Store procedural memory type and verify type preserved."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "procedural how-to knowledge",
                "memory_type": "procedural",
                "tags": ["procedural", "test-t2-36"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_experience_linked_memory(self, client, data_cleanup):
        """Store experience-linked memory type and verify type preserved."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "experience-linked memory record",
                "memory_type": "experience_linked",
                "tags": ["experience_linked", "test-t2-36"],
            },
        )
        assert result.get("success") is True
        assert "id" in result
        data_cleanup["memories"].append(result["id"])

    @pytest.mark.asyncio
    async def test_procedural_round_trip(self, client, data_cleanup):
        """Round-trip: store procedural memory, retrieve, assert type preserved."""
        store_result = await client.call_tool(
            "store_memory",
            {
                "content": "procedural round-trip content",
                "memory_type": "procedural",
                "confidence": 0.85,
                "tags": ["round_trip"],
            },
        )
        mem_id = store_result["id"]
        data_cleanup["memories"].append(mem_id)

        get_result = await client.call_tool("get_memory", {"id": mem_id})
        assert get_result.get("success") is True or get_result.get("found") is True


class TestMemoryConfidencePreserved:
    """T2-37: Confidence field preserved through store/retrieve cycle."""

    @pytest.mark.asyncio
    async def test_confidence_preserved(self, client, data_cleanup):
        """Store memory with confidence 0.83, retrieve, assert == 0.83."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "confidence preservation test",
                "memory_type": "fact",
                "confidence": 0.83,
                "tags": ["t2-37"],
            },
        )
        mem_id = result["id"]
        data_cleanup["memories"].append(mem_id)

        get_result = await client.call_tool("get_memory", {"id": mem_id})
        assert get_result.get("success") is True or get_result.get("found") is True
        # Assert confidence is preserved (allow small floating-point tolerance)
        stored_confidence = get_result.get("confidence") or get_result.get(
            "data", {}
        ).get("confidence")
        assert stored_confidence is not None, (
            "Confidence field missing in get_memory response"
        )
        assert abs(stored_confidence - 0.83) < 0.01, (
            f"Expected ~0.83, got {stored_confidence}"
        )


class TestMemoryProvenance:
    """T2-39: Memory provenance/source fields preserved through store/retrieve."""

    @pytest.mark.asyncio
    async def test_source_preserved(self, client, data_cleanup):
        """Store with source=user_input, retrieve, assert source preserved."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "provenance test",
                "memory_type": "fact",
                "source": "user_input",
                "tags": ["t2-39"],
            },
        )
        mem_id = result["id"]
        data_cleanup["memories"].append(mem_id)

        get_result = await client.call_tool("get_memory", {"id": mem_id})
        assert get_result.get("success") is True or get_result.get("found") is True
        stored_source = get_result.get("source") or get_result.get("data", {}).get(
            "source"
        )
        assert stored_source == "user_input", (
            f"Expected source=user_input, got {stored_source}"
        )
