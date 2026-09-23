"""
test_planner.py — Planning Engine functional tests.

Tests: Create, add steps, complete, dependencies, cancel.
"""

import pytest
from helpers import assert_success


class TestPlanningCRUD:
    """Planning Engine CRUD tests."""

    @pytest.mark.asyncio
    async def test_create_plan(self, client):
        """create_plan creates a plan."""
        result = await client.call_tool("create_plan", {"goal": "test planning goal"})
        assert assert_success(result)

    @pytest.mark.asyncio
    async def test_list_plans(self, client):
        """list_plans returns plans."""
        result = await client.call_tool("list_plans", {})
        assert "plans" in result or "data" in result or "items" in result

    @pytest.mark.asyncio
    async def test_get_plan(self, client):
        """get_plan retrieves a specific plan."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "specific plan to retrieve"}
        )
        plan_id = create_result["id"]
        get_result = await client.call_tool("get_plan", {"plan_id": plan_id})
        assert assert_success(get_result)


class TestPlanSteps:
    """Plan step management tests."""

    @pytest.mark.asyncio
    async def test_add_plan_step(self, client):
        """add_plan_step adds a step to a plan."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "plan with steps"}
        )
        plan_id = create_result["id"]
        step_result = await client.call_tool(
            "add_plan_step",
            {
                "plan_id": plan_id,
                "description": "test step description",
                "action": "test action",
            },
        )
        assert assert_success(step_result)
        assert "step_id" in step_result

    @pytest.mark.asyncio
    async def test_complete_plan_step(self, client):
        """complete_step marks a step as done."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "plan to complete"}
        )
        plan_id = create_result["id"]
        step_result = await client.call_tool(
            "add_plan_step",
            {
                "plan_id": plan_id,
                "description": "step to complete",
                "action": "complete action",
            },
        )
        step_id = step_result["step_id"]
        complete_result = await client.call_tool(
            "complete_step",
            {
                "plan_id": plan_id,
                "step_id": step_id,
                "outcome": "completed successfully",
            },
        )
        assert assert_success(complete_result)

    @pytest.mark.asyncio
    async def test_fail_plan_step(self, client):
        """fail_step marks a step as failed."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "plan with failing step"}
        )
        plan_id = create_result["id"]
        step_result = await client.call_tool(
            "add_plan_step",
            {
                "plan_id": plan_id,
                "description": "step to fail",
                "action": "fail action",
            },
        )
        step_id = step_result["step_id"]
        fail_result = await client.call_tool(
            "fail_step",
            {
                "plan_id": plan_id,
                "step_id": step_id,
                "error": "test failure reason",
            },
        )
        assert assert_success(fail_result)


class TestPlanDependencies:
    """Plan dependency tests."""

    @pytest.mark.asyncio
    async def test_add_step_dependency(self, client):
        """add_step_dependency links steps."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "plan with dependencies"}
        )
        plan_id = create_result["id"]
        step1 = await client.call_tool(
            "add_plan_step",
            {
                "plan_id": plan_id,
                "description": "first step",
                "action": "first action",
            },
        )
        step2 = await client.call_tool(
            "add_plan_step",
            {
                "plan_id": plan_id,
                "description": "second step",
                "action": "second action",
            },
        )
        dep_result = await client.call_tool(
            "add_step_dependency",
            {
                "plan_id": plan_id,
                "step_id": step2["step_id"],
                "depends_on": step1["step_id"],
            },
        )
        assert assert_success(dep_result)


class TestPlanLifecycle:
    """Plan lifecycle tests."""

    @pytest.mark.asyncio
    async def test_start_plan(self, client):
        """start_plan activates a plan."""
        create_result = await client.call_tool("create_plan", {"goal": "plan to start"})
        plan_id = create_result["id"]
        start_result = await client.call_tool("start_plan", {"plan_id": plan_id})
        assert assert_success(start_result)

    @pytest.mark.asyncio
    async def test_cancel_plan(self, client):
        """cancel_plan cancels a plan."""
        create_result = await client.call_tool(
            "create_plan", {"goal": "plan to cancel"}
        )
        plan_id = create_result["id"]
        cancel_result = await client.call_tool("cancel_plan", {"plan_id": plan_id})
        assert assert_success(cancel_result)

    @pytest.mark.asyncio
    async def test_list_plans_status_filter(self, client):
        """list_plans with status filter works."""
        result = await client.call_tool("list_plans", {"status": "active"})
        assert "plans" in result or "data" in result
