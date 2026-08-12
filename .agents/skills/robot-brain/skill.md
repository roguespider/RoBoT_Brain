---
name: robot-brain
description: Connect to RoBoT Brain MCP server for memory, knowledge, planning, and learning tools
trigger: robot_brain
---

# RoBoT Brain MCP Integration

This skill configures the agent to use RoBoT Brain as an MCP tool server, giving access to advanced memory and learning capabilities.

## MCP Configuration

Build the binary first (`cargo build --release -p robot_brain`), then configure it as an MCP server. The binary speaks MCP over stdio automatically when it receives JSON-RPC on stdin (the `server` subcommand is optional — `robot_brain` and `robot_brain server` both work).

```python
mcp_config = {
    "mcpServers": {
        "robot_brain": {
            "command": "target/release/robot_brain",
        }
    }
}
```

> **Prefer brain_tester for verification.** The unified test suite at `brain_tester/` has three modes: `brain_tester` (full suite), `brain_tester --list` (smoke check), and `brain_tester --probe TOOL` (introspect a tool's live inputSchema). For ad-hoc calls, the repo also ships a stdlib-only Python `RobotBrainClient` at `.agents/live_test/mcp_client.py` that auto-detects the binary and auto-handles the workflow gate below. Do not hand-write a new client. See "Live Testing" below.

## Workflow Gate (REQUIRED before any tool)

The server enforces a mandatory two-step gate before it accepts substantive tool calls. Skip it and every tool returns an error:

1. Call `get_workflow` with `{"purpose": "general"}` — otherwise tools return `{"code": "WORKFLOW_NOT_RETRIEVED"}`.
2. Call `search_memory` with a relevant query — otherwise tools return `{"code": "MEMORY_NOT_SEARCHED"}`.

After both, all tools work normally. The `RobotBrainClient.init()` method does this automatically; if you wire the SDK `mcp_config` above, the LLM agent must be instructed to call `get_workflow` then `search_memory` first.

## Available Tools

RoBoT Brain exposes **96 tools** via MCP including:

### Memory & Knowledge
- `store_memory` - Store new memories
- `search_memory` - Search memories by content
- `get_memory` - Get specific memory by ID
- `list_memories` - List recent memories
- `query_knowledge` - Query knowledge base
- `add_knowledge` - Add validated knowledge
- `global_search` - Search across all data

### Experience & Learning
- `record_experience` - Record action/outcome
- `list_experiences` - List experiences
- `get_experience_stats` - Get experience statistics
- `get_insights` - Get actionable insights
- `analyze_patterns` - Detect patterns
- `get_patterns` - Get detected patterns

### Planning & Workflows
- `create_plan` - Create new plan
- `get_plan` - Get plan details
- `list_plans` - List all plans
- `create_workflow` - Create workflow
- `start_workflow` - Execute workflow (⚠️ requires at least one `add_workflow_step` first; a fresh workflow returns `"Workflow ... is not valid"`)
- `list_workflows` - List workflows

### Hypothesis Testing
- `create_hypothesis` - Create testable hypothesis
- `add_evidence` - Add supporting/contradicting evidence
- `evaluate_hypothesis` - Evaluate hypothesis
- `list_hypotheses` - List hypotheses
- `extract_knowledge` - Extract validated knowledge

### Exploration
- `start_exploration` - Start exploration
- `evaluate_exploration_hypothesis` - Test hypothesis
- `promote_finding` - Promote finding to knowledge

### Skills
- `register_skill` - Register new skill
- `discover_skill` - Discover from experience
- `list_skills` - List all skills
- `execute_skill` - Execute skill
- `update_skill_mastery` - Update mastery

### ACP Messaging
- `route_acp_message` - Route ACP message
- `register_agent` - Register agent
- `list_acp_agents` - List registered agents

## Usage Examples

> All examples assume the workflow gate (`get_workflow` then `search_memory`) has already been satisfied. When driving an LLM agent via the SDK `mcp_config`, instruct the agent to call those two tools first.

### Search Memory
```
Use robot_brain's global_search to find information about Rust programming
```

### Record Experience
```
Record an experience that I successfully fixed a memory leak in the exploration module
```
The `outcome` field is case-sensitive and must be a PascalCase enum variant: `Success`, `Failure`, `Partial`, `Timeout`, or `Interrupted` (lowercase `success` is rejected).

### Create Plan
```
Create a plan to refactor the planner module into smaller modules
```

### Manage Knowledge
```
Add knowledge: "When refactoring large Rust files, extract logical groups to separate modules"
```

### Test Hypothesis
```
Create hypothesis: "Using module-based refactoring reduces compilation time for large files"
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ROBOT_BRAIN_PATH` | Path to robot_brain binary | Auto-detected |
| `LLM_API_KEY` | API key for LLM | Required |
| `LLM_MODEL` | Model name | `anthropic/claude-sonnet-4-5-20250929` |

## Example Script

See `examples/robot_brain_agent.py` for a complete integration example:

```bash
export LLM_API_KEY="your-key"
python examples/robot_brain_agent.py -m "Search memory for architecture patterns"
```

## Live Testing

The fastest way to verify the server works after compiling is `brain_tester` (Rust, built into the test suite):

```bash
# Full end-to-end suite (387 tests + coverage gate + code analysis)
cd brain_tester && cargo build --release && ./target/release/brain_tester

# Quick smoke check — list all server tools + required fields
./target/release/brain_tester --list

# Introspect one tool's live inputSchema (required/optional params)
./target/release/brain_tester --probe register_agent
```

For ad-hoc tool calls, the Python `RobotBrainClient` (`.agents/live_test/mcp_client.py`) is still available:

```bash
# Ad-hoc tool calls via the reusable client:
#   from mcp_client import RobotBrainClient
#   with RobotBrainClient() as c:
#       c.init()                       # handles initialize + workflow gate
#       r = c.call("store_memory", {"content": "hi", "memory_type": "note"})
#       print(r.text_json())
```

`mcp_client.py` is stdlib-only (no dependencies), auto-detects the binary via `ROBOT_BRAIN_PATH` or relative path, and runs the `get_workflow`→`search_memory` gate inside `init()`.
