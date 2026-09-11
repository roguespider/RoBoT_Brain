# Communication Model

Event-driven coordination between subsystems. Direct implementation coupling
is forbidden — all inter-subsystem communication goes through the event bus.

## 1. Event-Only Coordination (Chapter 16.1)

Subsystems MUST NOT call each other directly. Instead:
- Subsystems **emit events** when state changes
- Other subsystems **react to events** they care about
- This eliminates direct dependencies and enables independent evolution

Example: The Experience subsystem does NOT call the Knowledge subsystem.
Instead, it emits `ExperienceRecorded` events, and the Knowledge subsystem
reacts by evaluating whether the experience should promote to knowledge.

## 2. Event Schema (Chapter 5.2)

All events follow this schema:
```
{
  "kind": string,           // Event type identifier (e.g., "KnowledgeUpdated")
  "payload": object,        // Event-specific data (varies by kind)
  "source": string,         // Subsystem that emitted the event
  "correlation_id": string, // Links to the originating request cycle
  "timestamp": i64          // Unix timestamp in milliseconds
}
```

The `kind` field enables type-safe event routing. The `payload` field is
schema-varied but always JSON-serializable. All other fields are universal
metadata required for every event.

## 3. Event Storage (Chapter 21)

Events are stored in an append-only `events` table in the SQLite database:
- Events are never modified or deleted after insertion
- The table is indexed by `correlation_id`, `kind`, and `timestamp`
- Retention policy: all events from the last 30 days are kept; older events
  may be archived based on system policy
- Event replay: the event log enables full system reconstruction by replaying
  all events from a known good state

## 4. Entry Points (Chapter 15.1-15.3)

The ONLY entry points for external callers are:
- **MCP tools** (`src/bridge/mcp/handlers/`) — for local AI agent interactions
- **ACP messages** (`src/bridge/acp/`) — for other agents in the distributed system

All other modules are internal implementation details. External callers
MUST NOT access internal modules directly — they must go through MCP or ACP.

This ensures consistent validation, authentication, audit logging, and
error handling for all external interactions.
