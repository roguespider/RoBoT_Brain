#!/usr/bin/env python3
"""Test ingest tool to discover supported file types."""

import asyncio
import logging

from mcp_client import RobotBrainClient


async def main():
    client = await RobotBrainClient.connect()
    await client.get_workflow("general")
    await client.search_memory_first("test")

    tools = await client.list_tools()
    for t in tools:
        if t["name"] == "ingest_files":
            logging.info("Tool: ingest_files")
            logging.info("Description: %s", t.get("description", "")[:1000])
            logging.info("\nInput Schema:")
            import json

            logging.info(json.dumps(t.get("inputSchema", {}), indent=2))
            break

    await client.stop()


if __name__ == "__main__":
    asyncio.run(main())
