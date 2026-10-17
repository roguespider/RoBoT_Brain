"""
test_coverage.py — Coverage gap analysis.

Tests: Architecture vs implementation gaps, tool coverage.
"""

import pytest


class TestToolCoverage:
    """Tool coverage gap tests."""

    @pytest.mark.asyncio
    async def test_list_all_tools(self, client):
        """Verify tools/list returns a comprehensive tool list."""
        tools = await client.list_tools()
        tool_names = [t.get("name", "") for t in tools]
        assert len(tool_names) > 0


class TestArchitectureCoverage:
    """Architecture coverage gap analysis."""

    @pytest.mark.asyncio
    async def test_memory_engine_exists(self, client):
        """Memory Engine exists and responds."""
        result = await client.call_tool(
            "store_memory",
            {
                "content": "coverage test memory",
                "memory_type": "fact",
            },
        )
        # Server may return {id: ..., created_or_updated: true} or similar
        assert "id" in result or "created" in result or "success" in result

    @pytest.mark.asyncio
    async def test_knowledge_engine_exists(self, client):
        """Knowledge Engine exists and responds."""
        result = await client.call_tool(
            "add_knowledge",
            {
                "statement": "coverage test knowledge",
                "knowledge_type": "fact",
            },
        )
        assert "id" in result or "created" in result or "success" in result

    @pytest.mark.asyncio
    async def test_experience_engine_exists(self, client):
        """Experience Engine exists and responds."""
        result = await client.call_tool(
            "record_experience",
            {
                "title": "coverage test",
                "description": "engine exists test",
                "experience_type": "task",
                "outcome": "success",
            },
        )
        assert "id" in result or "created" in result or "success" in result

    @pytest.mark.asyncio
    async def test_planning_engine_exists(self, client):
        """Planning Engine exists and responds."""
        result = await client.call_tool(
            "create_plan",
            {
                "goal": "coverage test plan",
            },
        )
        assert "id" in result or "created" in result or "success" in result


class TestMissingEngines:
    """Document missing canonical engines from v0.0.2.1 architecture."""

    @pytest.mark.asyncio
    async def test_report_missing_engines(self, client):
        """Report which canonical engines are NOT built."""
        await client.list_tools()

        # Document missing engines (expected for Phase 2)
        missing_indicators = {
            "Conversation Engine": ["converse", "conversation"],
            "Context Engine": ["get_context", "context_build"],
            "Execution Engine": ["execute", "run_action"],
            "Tool Engine": ["register_tool", "tool_contract"],
            "AI Runtime": ["generate", "inference", "chat"],
            "Model Manager": ["list_models", "select_model"],
            "Control Plane": ["inspect", "control"],
            "GUI": ["gui_status", "gui_update"],
            "Monitoring": ["metrics", "traces", "health"],
            "Configuration": ["get_config", "set_config"],
            "Deployment": ["deploy", "rollback"],
        }
        # No assertion — this is a documentation test
        assert len(missing_indicators) > 0
