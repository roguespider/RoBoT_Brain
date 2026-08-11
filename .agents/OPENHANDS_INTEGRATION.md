# OpenHands MCP Integration

> Moved here from AGENTS.md on 2026-08-11. This is reference material for wiring
> an OpenHands agent to use RoBoT Brain as an MCP server. You do NOT need this
> at session start; consult it when integrating with the OpenHands SDK.

RoBoT Brain can be used as an MCP server by **OpenHands agents** to access memory, knowledge, planning, and learning tools.

## Quick Start

```python
from openhands.sdk import LLM, Agent, Conversation
from openhands.sdk.tool import Tool
from openhands.tools.terminal import TerminalTool

# Configure MCP connection
mcp_config = {
    "mcpServers": {
        "robot_brain": {
            "command": "cargo run --release -p robot_brain",
        }
    }
}

# Create agent with robot_brain tools
agent = Agent(
    llm=LLM(model="anthropic/claude-sonnet-4-5-20250929", api_key="..."),
    tools=[Tool(name=TerminalTool.name)],
    mcp_config=mcp_config,
)

# Run conversation
conversation = Conversation(agent=agent, workspace=".")
conversation.send_message("Search memory for Rust patterns")
conversation.run()
```

## Complete Example

See `examples/robot_brain_agent.py` for a full-featured script:

```bash
export LLM_API_KEY="your-key"
python examples/robot_brain_agent.py -m "Search memory for architecture patterns"
```

## Available Tools (~89 total)

| Category | Key Tools |
|----------|-----------|
| **Memory** | `store_memory`, `search_memory`, `get_memory`, `list_memories` |
| **Knowledge** | `query_knowledge`, `add_knowledge`, `global_search` |
| **Experience** | `record_experience`, `list_experiences`, `get_insights` |
| **Planning** | `create_plan`, `get_plan`, `list_plans` |
| **Workflows** | `create_workflow`, `start_workflow`, `list_workflows` |
| **Hypothesis** | `create_hypothesis`, `add_evidence`, `evaluate_hypothesis` |
| **Exploration** | `start_exploration`, `evaluate_exploration_hypothesis` |
| **Skills** | `register_skill`, `discover_skill`, `execute_skill` |
| **ACP** | `route_acp_message`, `register_agent`, `list_acp_agents` |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ROBOT_BRAIN_PATH` | Path to robot_brain binary | Auto-detected |
| `LLM_API_KEY` | API key for LLM | Required |
| `LLM_MODEL` | Model name | `anthropic/claude-sonnet-4-5-20250929` |

## Loading the Skill

This repo includes an OpenHands skill at `.agents/skills/robot-brain/skill.md` that documents all available tools and usage patterns. When working in an OpenHands environment, this skill is automatically loaded and provides context for using robot_brain tools.

## Tool Filtering

If you only want specific tools, use regex filtering:

```python
agent = Agent(
    ...
    filter_tools_regex="^(search_memory|store_memory|query_knowledge)$",
)
```

This allows OpenHands to use robot_brain alongside other tools, focusing on specific capabilities as needed.
