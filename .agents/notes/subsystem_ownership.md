# Subsystem Ownership Map

Each subsystem has exactly one owning module. Inter-subsystem calls go through
events, not direct imports.

| Subsystem | Owning Module Path | Primary Responsibility | v0.0.2 Chapter |
|-----------|-------------------|----------------------|----------------|
| Agent | `src/agent/` | Cognitive loop, action selection, decision flow | Chapter 12 |
| Bridge | `src/bridge/` | MCP/ACP protocol handlers and tool registry | Chapter 15 |
| CLI | `src/cli/` | Command-line interface and startup | Chapter 3 |
| CoObOpLoop | `src/cooboploop/` | Objective queue, loop runner, capability management | Chapter 16 |
| DataContracts | `src/data_contracts/` | Shared data structures and versioned types | Chapter 5 |
| Database | `src/database/` | SQLite storage, migrations, queries | Chapter 21 |
| Experience | `src/experience/` | Recording, scoring, processing experiences | Chapter 10 |
| Knowledge | `src/knowledge/` | Fact storage, confidence scoring, promotion | Chapter 9 |
| Learning | `src/learning/` | Pattern discovery, skill acquisition, generalization | Chapter 10 |
| Memory | `src/memory/` | Storage, retrieval, promotion of memory | Chapter 8 |
| Personality | `src/personality/` | Emotional state, traits, decision bias | Chapter 13 |
| Planner | `src/planner/` | Goal decomposition into plan steps | Chapter 11 |
| Research | `src/research/` | External source retrieval, evidence gathering | Chapter 13 |
| Skills | `src/skills/` | Skill registry, execution, mastery tracking | Chapter 14 |
| Workflows | `src/workflows/` | Workflow definition, execution, enforcement | Chapter 14 |
| WorldModel | `src/world_model/` | Entity/relationship modeling, reasoning | Chapter 15 |

## Ownership Rule

Any inter-subsystem call MUST go through an event emission, not a direct import.
Each subsystem owns its internal state and exposes behavior only through:
- Event emissions (for signaling state changes)
- Public API methods (for controlled access)
- MCP/ACP tools (for external callers)

Hidden cross-ownership (e.g., `src/memory/` importing `src/experience/` internals)
must be resolved by routing through the event bus or defining a shared contract
in `src/data_contracts/`.
