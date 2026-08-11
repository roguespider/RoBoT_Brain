# Appendix A. Directory Structure

**Architecture Version:** v0.0.2.1  
**Document Role:** Supporting architectural material  
**v0.0.2.1 Focus:** directory ownership, module boundaries, deployment layout, and source-of-truth locations  

## Purpose

This appendix defines the recommended directory structure for the RoBoT project. The structure is designed to reflect the cognitive architecture described throughout this document while keeping implementation modular, maintainable, and scalable.

The directory layout emphasizes:

- Clear subsystem boundaries
- Separation of interfaces from implementations
- Minimal coupling
- Expandability
- Consistent naming
- Easy navigation
- Offline-first operation
- Support for future capabilities

The directory structure represents the architectural organization of the project rather than a strict implementation requirement. Individual files and modules may evolve over time without changing the overall organization.

---

# High-Level Project Structure

```text
RoBoT/
│
├── src/
├── config/
├── data/
├── models/
├── plugins/
├── docs/
├── scripts/
├── tests/
├── benchmarks/
├── examples/
├── tools/
├── assets/
├── logs/
├── temp/
├── exports/
├── diagnostics/
├── backups/
├── target/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
└── ARCHITECTURE.md
```

---

# Source Directory

The `src` directory contains the complete cognitive architecture.

```text
src/
│
├── app/
├── bootstrap/
├── api/
├── config/
├── runtime/
├── database/
├── storage/
├── context/
├── memory/
├── knowledge/
├── experience/
├── learning/
├── conversation/
├── planning/
├── execution/
├── tools/
├── audio/
├── vision/
├── models/
├── tracing/
├── diagnostics/
├── security/
├── deployment/
├── scheduler/
├── utilities/
├── common/
└── main.rs
```

Each subsystem should remain independently testable.

---

# Bootstrap

```text
bootstrap/

├── bootstrap_manager.rs
├── startup.rs
├── shutdown.rs
├── dependency_graph.rs
├── health_check.rs
└── mod.rs
```

Responsibilities:

- System startup
- Initialization order
- Dependency validation
- Shutdown coordination
- Runtime readiness

---

# Runtime

The runtime contains the shared execution platform used by every AI subsystem.

```text
runtime/

├── ai_runtime.rs
├── model_manager.rs
├── model_registry.rs
├── execution_scheduler.rs
├── tokenizer_manager.rs
├── resource_manager.rs
├── device_manager.rs
├── cache_manager.rs
├── streaming.rs
├── diagnostics.rs
└── mod.rs
```

The runtime abstracts Candle and any future inference backends.

---

# Database

```text
database/

├── sqlite/
├── migrations/
├── repositories/
├── queries/
├── transactions/
├── schema.rs
└── mod.rs
```

Responsibilities include:

- SQLite management
- Schema migrations
- Transactions
- Repository layer
- Data validation

---

# Storage

```text
storage/

├── blobs/
├── files/
├── cache/
├── exports/
├── backups/
├── compression/
└── mod.rs
```

Storage manages persistent resources independently from higher-level cognitive systems.

---

# Context Engine

```text
context/

├── manager.rs
├── session.rs
├── working_memory.rs
├── retrieval.rs
├── compressor.rs
├── topic_tracker.rs
├── builder.rs
├── scoring.rs
└── mod.rs
```

Responsibilities:

- Session context
- Working memory
- Context construction
- Context compression
- Token budgeting

---

# Memory Engine

```text
memory/

├── manager.rs
├── episodic.rs
├── semantic.rs
├── procedural.rs
├── retrieval.rs
├── indexing.rs
├── embeddings.rs
├── consolidation.rs
├── confidence.rs
└── mod.rs
```

Responsibilities:

- Long-term memory
- Retrieval
- Embeddings
- Consolidation
- Memory confidence

---

# Knowledge Graph

```text
knowledge/

├── graph.rs
├── nodes.rs
├── edges.rs
├── ontology.rs
├── reasoning.rs
├── search.rs
├── validation.rs
└── mod.rs
```

Responsibilities:

- Knowledge representation
- Graph traversal
- Relationship management
- Semantic reasoning

---

# Experience Engine

```text
experience/

├── coordinator.rs
├── recorder.rs
├── evaluator.rs
├── confidence.rs
├── workflows.rs
├── skills.rs
├── lessons.rs
├── hypotheses.rs
├── events/
│   ├── builders.rs
│   ├── types.rs
│   └── mod.rs
├── storage.rs
└── mod.rs
```

Responsibilities:

- Experience recording
- Workflow analysis
- Skill evolution
- Lesson extraction
- Confidence scoring

---

# Learning Engine

```text
learning/

├── learner.rs
├── promotion.rs
├── consolidation.rs
├── hypothesis.rs
├── validation.rs
├── optimizer.rs
├── scoring.rs
└── mod.rs
```

Responsibilities:

- Continuous learning
- Knowledge promotion
- Pattern discovery
- Experience integration

---

# Conversation Engine

```text
conversation/

├── manager.rs
├── orchestrator.rs
├── prompts.rs
├── responses.rs
├── intent.rs
├── history.rs
├── streaming.rs
└── mod.rs
```

Responsibilities:

- Conversation orchestration
- Prompt construction
- Response handling
- Multi-turn dialogue

---

# Planning Engine

```text
planning/

├── planner.rs
├── goals.rs
├── decomposition.rs
├── scheduler.rs
├── reasoning.rs
├── simulation.rs
├── validation.rs
└── mod.rs
```

Responsibilities:

- Goal decomposition
- Task planning
- Dependency resolution
- Simulation

---

# Execution Engine

```text
execution/

├── executor.rs
├── workflow.rs
├── dispatcher.rs
├── monitoring.rs
├── recovery.rs
├── results.rs
└── mod.rs
```

Responsibilities:

- Workflow execution
- Progress tracking
- Error recovery
- Result reporting

---

# Tool Manager

```text
tools/

├── manager.rs
├── registry.rs
├── discovery.rs
├── permissions.rs
├── mcp/
│   ├── client.rs
│   ├── server.rs
│   ├── protocol.rs
│   └── mod.rs
└── mod.rs
```

Responsibilities:

- MCP integration
- Tool registration
- Capability negotiation
- Permission enforcement

---

# Audio Engine

```text
audio/

├── input.rs
├── output.rs
├── decoder.rs
├── encoder.rs
├── preprocessing.rs
├── speech_to_text.rs
├── text_to_speech.rs
├── streaming.rs
├── formats.rs
└── mod.rs
```

Responsibilities:

- Audio processing
- Speech recognition
- Speech synthesis
- Streaming audio

The Audio Engine executes AI models through the AI Runtime.

---

# Vision Engine

```text
vision/

├── image_loader.rs
├── preprocessing.rs
├── ocr.rs
├── object_detection.rs
├── scene_analysis.rs
├── screenshot.rs
├── pipeline.rs
└── mod.rs
```

Responsibilities:

- OCR
- Image understanding
- Screenshot analysis
- Vision inference

Vision models execute through the AI Runtime.

---

# Model Definitions

```text
models/

├── language/
├── embeddings/
├── speech/
├── vision/
├── rerankers/
├── tokenizers/
└── metadata/
```

These directories store AI model metadata and configuration. Large model files should reside outside source control.

---

# Tracing

```text
tracing/

├── architecture_trace.rs
├── spans.rs
├── events.rs
├── profiler.rs
├── visualization.rs
└── mod.rs
```

Responsibilities:

- Architecture tracing
- Performance analysis
- Execution visualization
- Debugging support

---

# Diagnostics

```text
diagnostics/

├── health.rs
├── metrics.rs
├── performance.rs
├── reports.rs
├── benchmark.rs
└── mod.rs
```

Responsibilities:

- Health monitoring
- Performance metrics
- Diagnostic reporting

---

# Security

```text
security/

├── permissions.rs
├── policies.rs
├── validation.rs
├── encryption.rs
├── sandbox.rs
└── mod.rs
```

Responsibilities:

- Permission enforcement
- Security policies
- Validation
- Future sandboxing

---

# Deployment

```text
deployment/

├── installer.rs
├── updater.rs
├── backup.rs
├── recovery.rs
├── validation.rs
└── mod.rs
```

Responsibilities:

- Installation
- Updates
- Backup
- Recovery
- Deployment validation

---

# Scheduler

```text
scheduler/

├── task_queue.rs
├── priorities.rs
├── timers.rs
├── workers.rs
└── mod.rs
```

Responsibilities:

- Background jobs
- Scheduling
- Worker coordination

---

# Common

```text
common/

├── errors.rs
├── identifiers.rs
├── traits.rs
├── types.rs
├── constants.rs
├── macros.rs
└── mod.rs
```

Shared primitives used across the architecture.

---

# Configuration Directory

```text
config/

├── runtime.toml
├── models.toml
├── memory.toml
├── database.toml
├── audio.toml
├── vision.toml
├── tools.toml
├── security.toml
├── logging.toml
└── deployment.toml
```

All runtime configuration should remain external to compiled binaries.

---

# Data Directory

```text
data/

├── sqlite/
├── memories/
├── experience/
├── knowledge/
├── cache/
├── vectors/
├── indexes/
├── temporary/
└── backups/
```

Contains persistent application data.

---

# AI Model Directory

```text
models/

├── language/
├── embeddings/
├── speech/
├── vision/
├── rerankers/
├── tokenizers/
├── downloads/
└── cache/
```

Managed exclusively by the Model Manager.

---

# Plugin Directory

```text
plugins/

├── installed/
├── disabled/
├── updates/
└── manifests/
```

Supports MCP servers and future plugin architectures.

---

# Documentation

```text
docs/

├── architecture/
├── api/
├── user/
├── developer/
├── examples/
└── decisions/
```

Architectural Decision Records (ADRs) should be maintained in the `decisions` directory.

---

# Test Organization

```text
tests/

├── unit/
├── integration/
├── workflow/
├── regression/
├── performance/
├── stress/
└── fixtures/
```

This mirrors the Testing and Validation Architecture.

---

# Benchmark Organization

```text
benchmarks/

├── runtime/
├── memory/
├── planning/
├── inference/
├── database/
└── reports/
```

Benchmarking is treated as a first-class engineering activity.

---

# Design Rules

The directory structure follows several architectural rules:

1. Each cognitive subsystem owns its implementation.
2. Shared functionality belongs in `common` or `runtime`, not duplicated across subsystems.
3. AI models execute through the AI Runtime, never directly from subsystem code.
4. Configuration remains external to source code.
5. Persistent data is separated from executable code.
6. Tests mirror the production architecture.
7. Subsystems communicate through interfaces, not direct file access.
8. Future capabilities should extend the structure rather than reorganize it.

---

# Future Expansion

The directory structure is intentionally designed to support future additions without major reorganization.

Examples include:

- Robotics
- Simulation
- Distributed execution
- Federated learning
- Additional AI runtimes
- New cognitive subsystems
- Specialized reasoning engines
- Research modules

Future additions should integrate as new modules while preserving the existing organization.

---

# Success Criteria

The directory structure is considered successful when:

- The organization reflects the cognitive architecture.
- Every subsystem has a clearly defined location and responsibility.
- AI Runtime, Model Manager, and Candle-based inference remain centralized.
- Memory, Knowledge, Experience, Planning, Learning, and Conversation remain independently maintainable.
- Testing, diagnostics, deployment, and security mirror the production architecture.
- New capabilities can be added with minimal impact on existing modules.
- Developers can navigate the project intuitively without relying on undocumented conventions.

The directory structure serves as the physical manifestation of the RoBoT Architecture. As the platform evolves, new capabilities should fit naturally into this organization while preserving the modular, explainable, and extensible design principles that define the project.

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

## Final v0.0.2.1 Integration Contract

This appendix is supporting material for the final v0.0.2.1 architecture. It cannot silently redefine a normative chapter.

It must preserve:

- explicit ownership
- lifecycle and retention semantics
- identity and correlation
- provenance
- confidence and uncertainty
- failure visibility
- model/runtime independence
- controlled external effects
- observability
- versioned evolution
- human control

**Supporting focus:** physical project layout, source-of-truth locations, module ownership and deployment separation.

When an appendix conflicts with a normative chapter, the conflict is a documentation defect that must be resolved. The appendix must be updated rather than allowing two competing definitions to survive.
