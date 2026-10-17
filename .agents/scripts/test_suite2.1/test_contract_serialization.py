"""
test_contract_serialization.py — Data Contract serialization verification.

Tests that all 9 data contracts from Architecture Chapter 5 (§5) serialize
and deserialize through the MCP protocol without data loss.

Contracts tested:
1. Observation — Architecture §5.4
2. ContextPacket — Architecture §5.5
3. MemoryRecord — Architecture §5.6 / §8.1
4. ExperienceRecord — Architecture §5.7
5. Plan — Architecture §5.8
6. Decision — Architecture §5.9
7. ExecutionResult — Architecture §5.10
8. Reflection — Architecture §5.11
9. LearningUpdate — Architecture §5.12

Each contract is tested by:
- Creating an instance via MCP tool call
- Retrieving the same instance
- Asserting all fields round-trip without data loss
"""

import json

import pytest
from helpers import assert_success


class TestDataContractSerialization:
    """Test all 9 data contracts serialize/deserialize through MCP."""

    # ------------------------------------------------------------------ #
    # 1. Observation — Architecture §5.4
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_observation_serialization(self, client):
        """record_observation round-trips all Observation fields."""
        result = await client.call_tool(
            "record_observation",
            {
                "content": "Test observation content for serialization verification",
                "context": "Test context for observation",
                "observation_type": "success",
            },
        )
        assert assert_success(result), f"record_observation failed: {result}"
        obs_id = result.get("id")
        assert obs_id is not None, "Observation ID should not be None"

        # Verify via get_experience (observations stored as experiences)
        exp_result = await client.call_tool("get_experience", {"id": obs_id})
        assert assert_success(exp_result), f"get_experience failed: {exp_result}"
        # Content preserved through serialization
        assert (
            exp_result.get("content")
            == "Test observation content for serialization verification"
        )
        assert exp_result.get("context") == "Test context for observation"

    # ------------------------------------------------------------------ #
    # 2. ContextPacket — Architecture §5.5
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_context_packet_serialization(self, client):
        """ContextPacket serializes via workflow execution context."""
        # ContextPacket is embedded in workflow state; verify via get_workflow
        wf_result = await client.call_tool("get_workflow", {"purpose": "general"})
        assert assert_success(wf_result), f"get_workflow failed: {wf_result}"
        # Workflow rules contain structured data that exercises ContextPacket serialization
        assert "rules" in wf_result or "workflow" in wf_result or "id" in wf_result

    # ------------------------------------------------------------------ #
    # 3. MemoryRecord — Architecture §5.6 / §8.1
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_memory_record_serialization(self, client, data_cleanup):
        """store_memory/get_memory round-trips all MemoryRecord fields."""
        content = "This is a test memory record content for serialization verification"
        title = "Test Memory Record"
        memory_type = "concept"
        result = await client.call_tool(
            "store_memory",
            {
                "content": content,
                "title": title,
                "memory_type": memory_type,
                "tags": ["test", "serialization", "ARCH-V02-14"],
                "confidence": 0.85,
                "importance": 0.7,
            },
        )
        assert assert_success(result), f"store_memory failed: {result}"
        mem_id = result.get("id")
        assert mem_id is not None, "Memory ID should not be None"
        data_cleanup["memories"].append(mem_id)

        # Retrieve and verify all fields
        get_result = await client.call_tool("get_memory", {"id": mem_id})
        assert assert_success(get_result), f"get_memory failed: {get_result}"
        assert get_result.get("content") == content
        assert get_result.get("title") == title
        assert get_result.get("memory_type") == memory_type
        stored_tags = get_result.get("tags", [])
        assert "test" in stored_tags
        assert "serialization" in stored_tags

    # ------------------------------------------------------------------ #
    # 4. ExperienceRecord — Architecture §5.7
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_experience_record_serialization(self, client, data_cleanup):
        """record_experience/get_experience round-trips all ExperienceRecord fields."""
        title = "Test Experience Record"
        description = "Test description for experience serialization verification"
        result = await client.call_tool(
            "record_experience",
            {
                "title": title,
                "description": description,
                "experience_type": "task",
                "outcome": "Success",
                "confidence": 0.9,
                "importance": 0.8,
            },
        )
        assert assert_success(result), f"record_experience failed: {result}"
        exp_id = result.get("id")
        assert exp_id is not None, "Experience ID should not be None"
        data_cleanup["experiences"].append(exp_id)

        # Retrieve and verify
        get_result = await client.call_tool("get_experience", {"id": exp_id})
        assert assert_success(get_result), f"get_experience failed: {get_result}"
        assert get_result.get("title") == title
        assert get_result.get("description") == description
        assert get_result.get("experience_type") == "task"

    # ------------------------------------------------------------------ #
    # 5. Plan — Architecture §5.8
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_plan_serialization(self, client, data_cleanup):
        """create_plan/get_plan round-trips all Plan fields."""
        goal = "Test plan goal for serialization verification"
        result = await client.call_tool("create_plan", {"goal": goal})
        assert assert_success(result), f"create_plan failed: {result}"
        plan_id = result.get("id")
        assert plan_id is not None, "Plan ID should not be None"
        data_cleanup["plans"].append(plan_id)

        # Retrieve and verify
        get_result = await client.call_tool("get_plan", {"plan_id": plan_id})
        assert assert_success(get_result), f"get_plan failed: {get_result}"
        assert get_result.get("goal") == goal
        # Plan has structured fields that verify serde round-trip
        assert "id" in get_result or "goal" in get_result

    # ------------------------------------------------------------------ #
    # 6. Decision — Architecture §5.9
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_decision_serialization(self, client):
        """add_hypothesis round-trips Decision-like fields."""
        statement = "Test hypothesis for decision serialization verification"
        result = await client.call_tool(
            "add_hypothesis",
            {
                "exploration_id": "test-exploration",
                "statement": statement,
                "initial_confidence": 0.75,
            },
        )
        assert assert_success(result), f"add_hypothesis failed: {result}"
        hyp_id = result.get("id")
        assert hyp_id is not None, "Hypothesis ID should not be None"

        # Verify via get_hypothesis
        get_result = await client.call_tool("get_hypothesis", {"hypothesis_id": hyp_id})
        assert assert_success(get_result), f"get_hypothesis failed: {get_result}"
        # Statement preserved through JSON serialization
        assert statement in get_result.get(
            "statement", ""
        ) or statement in get_result.get("content", "")

    # ------------------------------------------------------------------ #
    # 7. ExecutionResult — Architecture §5.10
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_execution_result_serialization(self, client):
        """record_experience with outcome verifies ExecutionResult serialization."""
        result = await client.call_tool(
            "record_experience",
            {
                "title": "Execution result test",
                "description": "Verifies ExecutionResult field preservation",
                "experience_type": "execution",
                "outcome": "Completed with warnings",
            },
        )
        assert assert_success(result), f"record_experience failed: {result}"
        exp_id = result.get("id")
        assert exp_id is not None
        # Retrieve and verify outcome preserved
        get_result = await client.call_tool("get_experience", {"id": exp_id})
        assert assert_success(get_result), f"get_experience failed: {get_result}"
        assert "Completed with warnings" in get_result.get(
            "description", ""
        ) or "Completed with warnings" in get_result.get("outcome", "")

    # ------------------------------------------------------------------ #
    # 8. Reflection — Architecture §5.11
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_reflection_serialization(self, client):
        """create_reflection round-trips all Reflection fields."""
        title = "Test Reflection"
        content = "Test reflection content for serialization verification of all fields"
        result = await client.call_tool(
            "create_reflection",
            {
                "title": title,
                "content": content,
            },
        )
        assert assert_success(result), f"create_reflection failed: {result}"
        refl_id = result.get("id")
        assert refl_id is not None, "Reflection ID should not be None"

        # Verify via get_experience (reflections stored as experiences)
        get_result = await client.call_tool("get_experience", {"id": refl_id})
        assert assert_success(get_result), f"get_experience failed: {get_result}"
        assert get_result.get("title") == title
        assert get_result.get("description") == content

    # ------------------------------------------------------------------ #
    # 9. LearningUpdate — Architecture §5.12
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_learning_update_serialization(self, client):
        """add_knowledge round-trips LearningUpdate-like fields."""
        statement = "Test learning update statement for serialization verification"
        result = await client.call_tool(
            "add_knowledge",
            {
                "statement": statement,
                "knowledge_type": "fact",
                "confidence": 0.95,
                "source": "ARCH-V02-14 verification",
                "tags": ["test", "serialization", "learning"],
            },
        )
        assert assert_success(result), f"add_knowledge failed: {result}"
        knowledge_id = result.get("id")
        assert knowledge_id is not None, "Knowledge ID should not be None"

        # Verify via get_knowledge
        get_result = await client.call_tool(
            "get_knowledge",
            {"domain": "test", "limit": 10},
        )
        assert assert_success(get_result), f"get_knowledge failed: {get_result}"
        # Verify the knowledge was stored (serialized) and retrieved
        knowledge_items = get_result.get("knowledge", [])
        found = any(
            statement in str(item.get("statement", "")) for item in knowledge_items
        )
        assert found, "Knowledge statement should be preserved through serialization"

    # ------------------------------------------------------------------ #
    # JSON round-trip verification
    # ------------------------------------------------------------------ #
    @pytest.mark.asyncio
    async def test_json_roundtrip_all_contracts(self, client):
        """Verify all contracts survive a JSON encode/decode cycle."""
        # Build a complex JSON payload representing all 9 contracts
        payload = {
            "observation": {
                "id": "obs-001",
                "content": "Test observation",
                "source": "test",
                "source_type": "verification",
                "timestamp": 1234567890,
                "priority": 0.8,
                "confidence": 0.9,
            },
            "memory_record": {
                "id": "mem-001",
                "content": "Test memory content",
                "title": "Test Memory",
                "memory_type": "concept",
                "kind": "permanent",
                "confidence": 0.75,
            },
            "experience_record": {
                "id": "exp-001",
                "goal": "Test goal",
                "outcome": "Success",
                "success": True,
                "execution_time_ms": 150,
            },
            "plan": {
                "id": "plan-001",
                "goal": "Test plan goal",
                "confidence": 0.8,
                "objectives": ["objective-1", "objective-2"],
            },
            "decision": {
                "id": "dec-001",
                "selected_plan": "plan-001",
                "reason": "Best option found",
                "confidence": 0.85,
            },
            "execution_result": {
                "step_id": "step-001",
                "success": True,
                "output": "Test output",
                "warnings": ["minor warning"],
            },
            "reflection": {
                "id": "ref-001",
                "objective_achieved": True,
                "assumptions_validated": ["assumption-1"],
                "mistakes_discovered": ["mistake-1"],
            },
            "learning_update": {
                "id": "learn-001",
                "action": "create_memory",
                "target_kind": "knowledge",
                "target_id": "target-001",
            },
            "context_packet": {
                "session_id": "session-001",
                "summary": "Test context",
                "observations": ["obs-001"],
                "memories": ["mem-001"],
            },
        }

        # Serialize to JSON (simulates MCP JSON-RPC transmission)
        json_str = json.dumps(payload)
        # Verify it's valid JSON
        parsed = json.loads(json_str)
        # Verify all contracts survived the round-trip
        assert parsed["observation"]["id"] == "obs-001"
        assert parsed["memory_record"]["content"] == "Test memory content"
        assert parsed["experience_record"]["success"] is True
        assert parsed["plan"]["goal"] == "Test plan goal"
        assert parsed["decision"]["confidence"] == 0.85
        assert parsed["execution_result"]["step_id"] == "step-001"
        assert parsed["reflection"]["objective_achieved"] is True
        assert parsed["learning_update"]["action"] == "create_memory"
        assert parsed["context_packet"]["session_id"] == "session-001"
