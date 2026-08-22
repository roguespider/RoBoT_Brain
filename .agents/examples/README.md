# RoBoT Brain Examples

This directory contains examples for integrating RoBoT Brain with an MCP-compatible AI agent.

## robot_brain_agent.py

A complete Python script that connects an AI agent to RoBoT Brain via MCP protocol.

### Prerequisites

```bash
pip install openhands-sdk openhands-tools
```

> Note: This example uses the OpenHands SDK. For other MCP-compatible agents,
> adapt the connection pattern to your SDK of choice.

### Usage

```bash
# Set environment variables
export LLM_API_KEY="your-api-key"
export LLM_MODEL="anthropic/claude-sonnet-4-5-20250929"

# Run with default message
python examples/robot_brain_agent.py

# Run with custom message
python examples/robot_brain_agent.py -m "Search memory for Rust architecture patterns"

# Run with verbose output
python examples/robot_brain_agent.py -v -m "Record an experience about refactoring the planner"

# Use custom robot_brain binary path
python examples/robot_brain_agent.py --robot-brain-path /path/to/robot_brain
```

### What It Does

1. Creates an AI agent (via OpenHands SDK) with RoBoT Brain as an MCP server
2. The agent can use all ~89 RoBoT Brain tools including:
   - Memory & knowledge management
   - Experience tracking & learning
   - Planning & workflow execution
   - Hypothesis testing & exploration
   - Skill registry & execution
   - ACP message routing

### Example Conversations

```python
# Search memory
agent = create_robot_brain_agent()
run_conversation(agent, "Search memory for any architecture patterns")

# Record experience
run_conversation(agent, "Record that I successfully refactored the planner module")

# Create plan
run_conversation(agent, "Create a plan to split the bridge module into smaller components")

# Manage knowledge
run_conversation(agent, "Add knowledge: Using module-based refactoring improves code organization")
```
