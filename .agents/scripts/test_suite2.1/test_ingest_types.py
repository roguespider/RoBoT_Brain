#!/usr/bin/env python3
"""Test ingest tool to discover supported file types."""

import asyncio
from pathlib import Path

from mcp_client import RobotBrainClient


async def main():
    client = await RobotBrainClient.connect()
    await client.get_workflow("general")
    await client.search_memory_first("test")

    tools = await client.list_tools()
    for t in tools:
        if t["name"] == "ingest_files":
            print("Tool: ingest_files")
            print(f"Description: {t.get('description', '')[:1000]}")
            print(f"\nInput Schema:")
            import json

            print(json.dumps(t.get("inputSchema", {}), indent=2))
            break

    await client.stop()


if __name__ == "__main__":
    asyncio.run(main())
