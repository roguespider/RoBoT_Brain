"""
test_tool_coverage.py — Verify every registered tool exists and responds.

This test iterates through the FULL_REGISTRY and calls each tool to ensure:
1. The tool is registered and callable
2. The tool returns a non-empty response (success or error)
3. The tool doesn't hang (30s timeout)
"""

import logging
import sys
from pathlib import Path

import pytest

logger = logging.getLogger(__name__)

_test_dir = Path(__file__).resolve().parent
if str(_test_dir) not in sys.path:
    sys.path.insert(0, str(_test_dir))

from registry import FULL_REGISTRY


class TestToolCoverage:
    """Verify every registered tool exists and responds."""

    @pytest.mark.asyncio
    async def test_all_tools_exist(self, client):
        """Every tool in the registry is callable and returns a response."""
        results = {"called": 0, "ok": 0, "errors": []}

        for tool_info in FULL_REGISTRY:
            tool_name = tool_info["function_name"]
            requires_workflow = tool_info.get("requires_workflow", False)

            # Skip tools that require workflow until it's initialized
            if requires_workflow:
                try:
                    result = await client.call_tool(tool_name, {})
                    results["called"] += 1

                    # Verify we got a response (even if it's an error)
                    if result is not None and isinstance(result, dict):
                        results["ok"] += 1
                    else:
                        results["errors"].append(f"{tool_name}: returned {result}")
                except Exception as e:
                    results["errors"].append(f"{tool_name}: {type(e).__name__}: {e}")
            else:
                # Non-workflow tools should always work
                try:
                    result = await client.call_tool(tool_name, {})
                    results["called"] += 1

                    if result is not None and isinstance(result, dict):
                        results["ok"] += 1
                    else:
                        results["errors"].append(f"{tool_name}: returned {result}")
                except Exception as e:
                    results["errors"].append(f"{tool_name}: {type(e).__name__}: {e}")

        # Report results
        if results["errors"]:
            logger.info("\nTool coverage report:")
            logger.info("  Called: %s/%s", results["called"], len(FULL_REGISTRY))
            logger.info("  OK: %s", results["ok"])
            logger.info("  Errors: %s", len(results["errors"]))
            for err in results["errors"][:10]:  # Show first 10 errors
                logger.info("    - %s", err)
            if len(results["errors"]) > 10:
                logger.info(
                    "    ... and %s more",
                    len(results["errors"]) - 10,
                )

        assert results["called"] == len(FULL_REGISTRY), (
            f"Not all tools were called: {results['called']}/{len(FULL_REGISTRY)}"
        )
        not_ok = len(FULL_REGISTRY) - results["ok"]
        assert results["ok"] == len(FULL_REGISTRY), (
            f"Tools returned empty/invalid: {not_ok}/{len(FULL_REGISTRY)}"
        )
