---
name: robot-brain
description: Connect to RoBoT Brain MCP server for memory, knowledge, planning, and learning tools
trigger: robot_brain
---

# RoBoT Brain MCP Integration

This skill configures the agent to use RoBoT Brain as an MCP tool server, giving access to advanced memory and learning capabilities.

## MCP Configuration

```python
mcp_config = {
    "mcpServers": {
        "robot_brain": {
            "command": "cargo run --release -p robot_brain",
        }
    }
}
```

## Available Tools

RoBoT Brain exposes ~89 tools via MCP including:

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
- `start_workflow` - Execute workflow
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
- `list_agents` - List registered agents

## Usage Examples

### Search Memory
```
Use robot_brain's global_search to find information about Rust programming
```

### Record Experience
```
Record an experience that I successfully fixed a memory leak in the exploration module
```

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
