# Appendix B. Database Schemas

## Purpose

This appendix defines the logical database architecture for RoBoT. It describes the core entities, relationships, indexing strategies, versioning approach, and data organization used throughout the cognitive architecture.

The database is designed to support:

- Long-term memory
- Working memory
- Knowledge graphs
- Experience tracking
- Learning
- Planning
- Conversation history
- AI model management
- Tool execution
- Architecture tracing
- System diagnostics

The schemas presented here define the conceptual data model rather than implementation-specific SQL. Physical schemas may evolve through migrations while preserving the logical architecture.

---

# Design Principles

The database architecture follows these principles:

- Normalize where practical
- Preserve historical information
- Never overwrite important knowledge
- Prefer immutable event history
- Use relationships instead of duplication
- Maintain explainability
- Version all schema changes
- Optimize for retrieval
- Keep AI-generated information traceable

SQLite serves as the primary embedded database.

---

# High-Level Database Architecture

```text
                     SQLite Database
                           │
 ┌───────────────┬──────────────┬───────────────┐
 │               │              │               │
 ▼               ▼              ▼               ▼
Memory      Knowledge      Experience    Conversation
 │               │              │               │
 └──────┬────────┴───────┬──────┴───────────────┘
        ▼                ▼
     Learning       Planning
        │                │
        └──────┬─────────┘
               ▼
      Diagnostics & Tracing
```

---

# Schema Versioning

Every schema change is versioned.

Migration metadata includes:

- Schema version
- Migration identifier
- Timestamp
- Description
- Rollback information (when practical)

No schema should be modified without a migration.

---

# Core Tables

The database is organized into logical domains.

```text
Core

• system_metadata
• schema_versions
• configuration

Memory

• memories
• memory_embeddings
• memory_relationships

Knowledge

• knowledge_nodes
• knowledge_edges

Experience

• experiences
• skills
• lessons
• workflows

Learning

• hypotheses
• promotions
• confidence_history

Conversation

• conversations
• messages

Planning

• goals
• plans
• tasks

Execution

• executions
• execution_steps

Models

• ai_models
• model_usage

Tools

• tools
• tool_calls

Tracing

• architecture_traces
• trace_events

Diagnostics

• metrics
• health_reports
```

---

# System Metadata

Stores information about the installation.

Suggested fields:

- Installation ID
- Creation timestamp
- Architecture version
- Current schema version
- Application version
- Last startup
- Last shutdown
- Database UUID

Only one logical record should exist.

---

# Configuration

Configuration values stored in the database should complement, not replace, external configuration files.

Examples include:

- User preferences
- Runtime overrides
- Learned settings
- Feature flags

---

# Memory Schema

The Memory Engine centers around the `memories` table.

Conceptual fields:

- Memory ID
- Memory type
- Content
- Summary
- Source
- Confidence
- Importance
- Created timestamp
- Updated timestamp
- Last accessed
- Access count
- Session origin
- Archived flag

Each memory represents a single durable knowledge unit.

---

# Memory Types

Supported categories include:

- Episodic
- Semantic
- Procedural
- Working (temporary)
- Imported
- Learned
- Generated
- Observation

Future categories may be added without changing retrieval APIs.

---

# Memory Embeddings

Embeddings remain separate from memory content.

Fields include:

- Embedding ID
- Memory ID
- Model identifier
- Embedding version
- Vector reference
- Created timestamp

This allows embedding models to evolve independently.

---

# Memory Relationships

Relationships connect memories.

Fields include:

- Relationship ID
- Source memory
- Target memory
- Relationship type
- Confidence
- Weight
- Evidence
- Created timestamp

Example relationship types:

- Supports
- Contradicts
- Expands
- Depends on
- Derived from
- Similar to

---

# Knowledge Graph

The graph consists of nodes and edges.

## Nodes

Suggested fields:

- Node ID
- Label
- Type
- Description
- Confidence
- Source
- Created timestamp

## Edges

Suggested fields:

- Edge ID
- Source node
- Target node
- Relationship
- Weight
- Confidence
- Evidence

Knowledge graphs remain explainable by preserving relationship metadata.

---

# Experience Schema

Experiences capture completed events.

Fields include:

- Experience ID
- Event type
- Description
- Goal
- Outcome
- Success indicator
- Confidence
- Duration
- Workflow reference
- Lesson reference
- Created timestamp

Experiences remain immutable once finalized.

---

# Workflow Schema

Workflows represent reusable sequences of actions.

Fields include:

- Workflow ID
- Name
- Description
- Version
- Success rate
- Average duration
- Confidence
- Usage count

Workflows evolve through accumulated experience.

---

# Skill Schema

Skills represent learned capabilities.

Fields include:

- Skill ID
- Name
- Category
- Description
- Confidence
- Prerequisite skill
- Experience count
- Success rate
- Last improved

Skills become increasingly refined over time.

---

# Lesson Schema

Lessons store reusable knowledge extracted from experience.

Fields include:

- Lesson ID
- Experience source
- Description
- Evidence
- Confidence
- Category
- Promotion status

Lessons bridge Experience and Memory.

---

# Learning Schema

Learning tracks cognitive improvement.

Hypothesis fields include:

- Hypothesis ID
- Description
- Supporting evidence
- Contradicting evidence
- Confidence
- Status
- Created timestamp

Learning history should remain fully auditable.

---

# Confidence History

Confidence changes are historical records.

Fields include:

- Record ID
- Entity type
- Entity ID
- Previous confidence
- New confidence
- Reason
- Timestamp

Historical confidence supports explainability.

---

# Conversation Schema

Conversations remain separate from memory.

Conversation fields:

- Conversation ID
- Session ID
- Started
- Ended
- Summary

Message fields:

- Message ID
- Conversation ID
- Role
- Content
- Timestamp
- Token count
- Model used

Messages may later become memories through the Learning Engine.

---

# Planning Schema

Planning records future intentions.

Goal fields:

- Goal ID
- Description
- Priority
- Status
- Confidence
- Created timestamp
- Completion timestamp

Plan fields:

- Plan ID
- Goal ID
- Strategy
- Estimated complexity
- Success prediction

Task fields:

- Task ID
- Plan ID
- Description
- Order
- Status
- Dependencies
- Result

---

# Execution Schema

Execution records actual work.

Execution fields:

- Execution ID
- Plan ID
- Started
- Finished
- Outcome
- Result summary

Step fields:

- Step ID
- Execution ID
- Action
- Status
- Duration
- Tool reference

Planning and execution remain separate to preserve intent versus outcome.

---

# AI Model Schema

The Model Manager maintains installed AI models.

Fields include:

- Model ID
- Name
- Family
- Version
- Runtime
- Device support
- Quantization
- Location
- Checksum
- Installation date
- Validation status

This schema supports Candle and future runtime backends.

---

# Model Usage

Model usage supports diagnostics.

Fields include:

- Usage ID
- Model ID
- Task type
- Duration
- Tokens
- Memory usage
- Device
- Timestamp

---

# Tool Schema

Registered MCP tools.

Fields include:

- Tool ID
- Name
- Version
- Provider
- Capabilities
- Permissions
- Status

Tool calls remain separate.

Fields include:

- Call ID
- Tool ID
- Start time
- End time
- Duration
- Status
- Error message

---

# Architecture Trace Schema

Architecture tracing provides explainability.

Trace fields:

- Trace ID
- Session ID
- Root subsystem
- Started
- Finished

Event fields:

- Event ID
- Trace ID
- Parent event
- Subsystem
- Function
- Duration
- Confidence
- Tokens
- Memory references
- Model references
- Tool references

Trace data may be archived after analysis.

---

# Diagnostics Schema

Performance metrics include:

- Metric ID
- Name
- Value
- Unit
- Timestamp

Health reports include:

- Report ID
- Component
- Status
- Description
- Timestamp

Diagnostics remain independent of operational data.

---

# Relationships

Major relationships include:

```text
Conversation
      │
      ▼
Messages
      │
      ▼
Experiences
      │
      ▼
Lessons
      │
      ▼
Memories
      │
      ▼
Knowledge Graph
      │
      ▼
Planning
      │
      ▼
Execution
```

Supporting relationships include:

- Memories ↔ Embeddings
- Experiences ↔ Skills
- Skills ↔ Workflows
- Plans ↔ Goals
- Executions ↔ Tool Calls
- AI Models ↔ Model Usage
- Traces ↔ Diagnostics

---

# Indexing Strategy

Indexes should prioritize retrieval performance.

Recommended indexes include:

- Primary identifiers
- Foreign keys
- Memory type
- Confidence
- Timestamp
- Goal status
- Workflow success
- Conversation session
- Model identifier
- Tool identifier

Composite indexes should be introduced only when profiling indicates benefit.

---

# Data Integrity

Integrity rules include:

- Foreign key enforcement
- Unique identifiers
- Transactional updates
- Cascade policies where appropriate
- Schema validation
- Migration verification

Corruption detection should run periodically.

---

# Archiving

Historical data may be archived.

Candidates include:

- Old conversations
- Architecture traces
- Diagnostics
- Temporary working memory

Permanent knowledge should never be archived automatically without policy.

---

# Backup Strategy

Backups should include:

- SQLite database
- Schema version
- Configuration
- Metadata

AI model binaries should be backed up separately only if required.

---

# Future Expansion

The database architecture supports future additions including:

- Robotics
- Sensor history
- Video understanding
- Distributed memory
- Federated knowledge
- Multi-agent collaboration
- Scientific datasets
- Simulation records
- Additional AI runtimes

Future schemas should extend existing domains rather than introduce unnecessary duplication.

---

# Success Criteria

The Database Schema Architecture is considered successful when:

- Memory, Knowledge, Experience, Learning, Planning, Conversation, and Execution are represented as independent but connected domains.
- AI Runtime, Model Manager, MCP tools, and Architecture Tracing have dedicated schema support.
- Historical information remains explainable and auditable.
- Schema evolution occurs through versioned migrations.
- Data integrity, indexing, and backups support long-term reliability.
- New cognitive capabilities can be added without redesigning existing schemas.

The database serves as the persistent foundation of the RoBoT cognitive architecture, preserving not only information but also the relationships, experiences, reasoning, and history that enable the system to learn and improve over time.

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

