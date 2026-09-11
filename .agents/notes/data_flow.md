# Data Flow Architecture

Canonical data-flow path for inputs, internal pipelines, and outputs.

## 1. Input Processing (Chapter 4.2)

External requests enter via the MCP tool layer (`src/bridge/mcp/`). Each request
undergoes three validation stages:
1. **Schema validation** — verify JSON matches the tool's input schema
2. **Authentication** — verify caller permissions (if applicable)
3. **Intent parsing** — extract goal description and parameters

Validated requests are passed to the appropriate handler, which delegates to
the Planner for goal decomposition or to direct tool execution for simple queries.

## 2. Internal Pipelines (Chapter 4.3 + Chapter 3.3)

The cognitive pipeline follows this sequence:

```
Request → Memory Retrieval → Knowledge Query → Experience Lookup →
Action Selection → Safety Gate → Execution → Experience Recording →
Learning → Knowledge Update
```

Each stage emits events to the event bus. The next stage reacts to relevant
events rather than polling for state. This event-driven design ensures loose
coupling and provides a complete audit trail of the processing pipeline.

The cognitive pipeline (Chapter 3.3) specifically defines the ordering:
Memory (fastest, highest recall) → Knowledge (validated facts) → Experience
(observed patterns) → Planning (goal decomposition) → Execution (action).

## 3. Output Generation (Chapter 4.4)

Responses are assembled through the Bridge layer (`src/bridge/mcp/`). The output
pipeline:
1. **Response assembly** — format the result according to the tool's output schema
2. **Evidence attachment** — include source citations for research/knowledge queries
3. **Audit logging** — record the response in the event log
4. **JSON serialization** — convert to MCP response format

All responses include a `correlation_id` linking back to the original request.

## 4. System Boundaries (Chapter 4.5)

The trust boundary encompasses:
- **External (untrusted):** MCP tool callers, ACP agent callers, CLI users
- **Boundary layer:** `src/bridge/` — validates all external input
- **Internal (trusted):** All `src/<subsystem>/` modules communicate via events

External callers interact with the system ONLY through:
- MCP tools (`src/bridge/mcp/handlers/`)
- ACP messages (`src/bridge/acp/`)
- CLI commands (`src/cli/`)

Direct access to internal modules from outside the `src/` tree is forbidden.
