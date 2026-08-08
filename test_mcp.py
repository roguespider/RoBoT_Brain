#!/usr/bin/env python3
"""Quick test script for robot_brain MCP connection."""
import asyncio
import json
import sys

# Try to use the MCP SDK
try:
    from mcp import ClientSession, StdioServerParameters
    from mcp.client.stdio import stdio_client
except ImportError:
    print("MCP SDK not installed. Installing...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "mcp", "-q"])
    from mcp import ClientSession, StdioServerParameters
    from mcp.client.stdio import stdio_client


async def test_robot_brain():
    """Test the robot_brain MCP server."""
    server_path = "/workspace/project/RoBoT_Brain/target/release/robot_brain"
    
    server_params = StdioServerParameters(command=server_path)
    
    print("Connecting to robot_brain MCP server...")
    
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            # Initialize
            print("Initializing session...")
            await session.initialize()
            print("✅ Session initialized")
            
            # List tools
            print("\nListing tools...")
            tools = await session.list_tools()
            print(f"✅ Found {len(tools.tools)} tools:")
            for tool in tools.tools[:10]:
                print(f"   - {tool.name}: {tool.description[:50]}...")
            if len(tools.tools) > 10:
                print(f"   ... and {len(tools.tools) - 10} more")
            
            # MUST call get_workflow first due to workflow enforcement
            print("\n\nCalling get_workflow (required by workflow enforcement)...")
            result = await session.call_tool("get_workflow", {"purpose": "general"})
            print(f"✅ get_workflow: {result.content[0].text[:300]}...")
            
            # Test a simple tool - get_system_status
            print("\n\nTesting get_system_status...")
            result = await session.call_tool("get_system_status", {})
            print(f"✅ get_system_status: {result.content[0].text[:200]}...")
            
            # Test search_memory
            print("\nTesting search_memory...")
            result = await session.call_tool("search_memory", {"query": "test", "limit": 3})
            print(f"✅ search_memory: {result.content[0].text[:300]}...")
            
            # Test list_memories
            print("\nTesting list_memories...")
            result = await session.call_tool("list_memories", {"limit": 5})
            print(f"✅ list_memories: {result.content[0].text[:300]}...")
            
            # Test global_search
            print("\nTesting global_search...")
            result = await session.call_tool("global_search", {"query": "architecture", "limit": 3})
            print(f"✅ global_search: {result.content[0].text[:300]}...")
            
            # Test list_plans
            print("\nTesting list_plans...")
            result = await session.call_tool("list_plans", {})
            print(f"✅ list_plans: {result.content[0].text[:300]}...")
            
            # Test create_plan
            print("\nTesting create_plan...")
            result = await session.call_tool("create_plan", {"goal": "Test plan from MCP integration"})
            print(f"✅ create_plan: {result.content[0].text[:300]}...")
            
            # Test list_workflows
            print("\nTesting list_workflows...")
            result = await session.call_tool("list_workflows", {})
            print(f"✅ list_workflows: {result.content[0].text[:300]}...")
            
            # Test list_skills
            print("\nTesting list_skills...")
            result = await session.call_tool("list_skills", {})
            print(f"✅ list_skills: {result.content[0].text[:300]}...")
            
            # Test list_hypotheses
            print("\nTesting list_hypotheses...")
            result = await session.call_tool("list_hypotheses", {"limit": 5})
            print(f"✅ list_hypotheses: {result.content[0].text[:300]}...")
            
            # Test get_insights
            print("\nTesting get_insights...")
            result = await session.call_tool("get_insights", {})
            print(f"✅ get_insights: {result.content[0].text[:300]}...")
            
            # Test list_acp_agents
            print("\nTesting list_acp_agents...")
            result = await session.call_tool("list_acp_agents", {})
            print(f"✅ list_acp_agents: {result.content[0].text[:300]}...")
            
            print("\n" + "=" * 60)
            print("All MCP tests passed! ✅")
            print("=" * 60)


if __name__ == "__main__":
    try:
        asyncio.run(test_robot_brain())
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
