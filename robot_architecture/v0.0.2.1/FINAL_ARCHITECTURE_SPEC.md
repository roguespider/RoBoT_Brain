# RoBoT Architecture v0.0.2.1
## Final Architectural Specification and Integration Contract

**Status:** Final v0.0.2.1 architecture baseline  
**Normative level:** Architecture-wide  
**Purpose:** Establish the rules that every chapter, appendix, implementation, runtime, GUI, tool, and future extension must obey.

---

## 1. Purpose

RoBoT is a persistent cognitive architecture rather than a single model invocation.

The architecture exists to maintain continuity across interactions, preserve useful knowledge, learn from experience, pursue goals, execute controlled actions, observe results, and improve future behavior.

The v0.0.2.1 baseline consolidates the architecture into explicit boundaries for:

- cognition
- state ownership
- information lifecycle
- memory and knowledge
- experience and learning
- planning and execution
- tools and external effects
- model/runtime abstraction
- persistence
- events and correlation
- security and trust
- observability
- developer control
- GUI operation
- configuration
- deployment
- testing
- evolution

The architecture must remain understandable and replaceable even as implementations change.

---

# 2. Core Architectural Principle

> **Context is temporary. Knowledge is persistent. Experience creates learning.**

The system must never rely on a single context window as the authoritative representation of its identity, knowledge, history, or long-running state.

A model may reason over a constructed context. The architecture owns the persistent state from which that context is constructed.

---

# 3. Architectural Layers

```text
┌─────────────────────────────────────────────────────────────┐
│ Human / External Interaction                                │
│ GUI • CLI • API • MCP • Device Interfaces                   │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│ Developer / Control Plane                                   │
│ Inspection • Commands • Configuration • Permissions • Audit  │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│ Cognitive Coordination                                      │
│ Conversation • Context • Reasoning • Planning • Learning    │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│ Cognitive State                                              │
│ Memory • Knowledge • Experience • Skills • Goals • Tasks     │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│ Action and Capability                                       │
│ Execution • Tools • Permissions • External Effects          │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────┐
│ Platform                                                   │
│ AI Runtime • Model Manager • Storage • Workers • OS/Device  │
└─────────────────────────────────────────────────────────────┘
```

No lower layer may silently acquire ownership of a higher layer's persistent cognitive state.

---

# 4. Canonical Engines

| Engine / subsystem | Owns |
|---|---|
| Conversation Engine | interactions, conversation/session coordination |
| Context Engine | temporary cognitive context |
| Memory Engine | durable memory and knowledge lifecycle |
| Experience Engine | execution history and outcomes |
| Learning Engine | evidence-based learning transitions |
| Planning Engine | goals, tasks, plans and replanning |
| Execution Engine | authorized action execution |
| Tool Engine | external capabilities and tool contracts |
| AI Runtime | model invocation abstraction |
| Model Manager | model discovery, metadata, selection and lifecycle |
| Storage | durable persistence mechanisms |
| Event System | versioned event transport and history |
| Control Plane | authorized inspection and mutation |
| GUI | human-facing presentation and interaction |
| Monitoring | health, metrics, traces and architectural evidence |
| Configuration | declared runtime configuration and change control |
| Deployment | installation, startup, upgrades, rollback and recovery |

These are logical ownership boundaries. An implementation may combine several into one process while preserving the contracts.

---

# 5. State Ownership and Lifetime

Every important state category must have an owner and lifetime.

```text
Ephemeral
  model intermediate state
  temporary calculations

Interaction / Session
  active conversation state
  transient coordination

Working
  active context
  current retrieval set
  active plan state

Persistent Operational
  goals
  tasks
  plans
  experiences
  skills
  execution history

Persistent Knowledge
  semantic knowledge
  relationships
  validated lessons
  strategic knowledge

Archived
  compressed or inactive information retained for history
```

State must not cross these boundaries implicitly.

Promotion, demotion, consolidation, expiration, and archival are explicit operations.

---

# 6. Evidence and Learning Lifecycle

Information is not automatically knowledge.

```text
Observation
    ↓
Candidate / Hypothesis
    ↓
Evaluation
    ↓
Promotion
    ↓
Maintained Knowledge
    ↓
Revision / Decay / Archive
```

Experience follows a related path:

```text
Goal
  ↓
Plan
  ↓
Action
  ↓
Execution
  ↓
Outcome
  ↓
Experience
  ↓
Evaluation
  ↓
Lesson / Skill / Knowledge / Strategy
```

A failed operation is evidence, not automatically a permanent negative belief.

A successful operation is evidence, not automatically a universally valid procedure.

---

# 7. Identity and Correlation

Cross-engine operations must retain enough identity to reconstruct causal relationships.

The architecture should support identifiers for:

- installation
- instance
- user
- conversation
- session
- interaction
- event
- correlation
- goal
- task
- plan
- action
- execution
- tool invocation
- experience
- learning change
- configuration change
- deployment

Not every record requires every identifier. Required identifiers must not be discarded when needed for traceability.

---

# 8. Confidence and Provenance

Confidence is a property of a belief, relationship, skill, plan, strategy, or conclusion, not a universal truth value.

Confidence must remain distinguishable from:

- source reliability
- evidence count
- recency
- uncertainty
- contradiction
- applicability
- execution success

Important durable information should preserve provenance sufficient to answer:

- where did this information come from?
- when was it observed?
- what evidence supports it?
- what transformed it?
- what confidence was assigned?
- what later evidence challenged it?

---

# 9. Memory Architecture

Memory is selective.

The system must support:

- retention
- retrieval
- promotion
- consolidation
- correction
- contradiction
- confidence updates
- decay
- archival
- restoration

Working context must never become permanent memory merely because it appeared in a prompt.

Permanent knowledge must never be injected into every context merely because it exists.

Retrieval is a decision bounded by relevance, confidence, provenance, diversity, budget, and context capacity.

---

# 10. Context Architecture

Context is a constructed artifact.

A context build should be reproducible from:

- current interaction
- active session state
- relevant memories
- applicable experience
- current goals/tasks
- plan state
- tool availability
- policies
- configuration
- model constraints

Context construction must preserve source attribution and allow expiration or compaction.

---

# 11. Planning and Execution

Planning determines what should happen.

Execution performs an authorized action.

The two must remain separate.

```text
Goal
 ↓
Task decomposition
 ↓
Plan
 ↓
Validation
 ↓
Authorization
 ↓
Execution
 ↓
Observation
 ↓
Result
 ↓
Experience
```

A plan is not permission.

A tool being available is not authorization.

A model proposing an action is not execution.

---

# 12. Tool and External Capability Boundary

Tools are capability providers.

Every tool should have:

- identity
- version
- input contract
- output contract
- permission requirements
- risk classification
- timeout/resource policy
- provenance
- failure semantics
- result capture

External effects must pass through controlled execution.

The GUI, model, MCP layer, or developer interface must not bypass this boundary.

---

# 13. AI Runtime and Model Manager

The AI Runtime provides the stable inference contract between RoBoT and model implementations.

The Model Manager owns:

- model registration
- model metadata
- capabilities
- availability
- health
- resource requirements
- model selection
- loading/unloading
- version tracking

Model-specific APIs remain behind adapters.

Candle may be used as an implementation, but no cognitive engine should depend directly on Candle-specific types or private model state.

Replacing a model must not require rebuilding persistent cognitive state.

The runtime should support multiple model classes, including where applicable:

- language
- vision
- speech
- multimodal
- embedding
- coding
- planning
- robotics
- scientific
- simulation

---

# 14. GUI and Human Control Plane

The GUI is a first-class operational interface, not a second architecture.

```text
GUI
 │
 ▼
Control Plane API
 │
 ├── Read / Inspect
 ├── Command
 ├── Configuration
 ├── Diagnostics
 ├── Memory Management
 ├── Experience Review
 ├── Planning
 ├── Workers
 ├── Tools / Permissions
 ├── Model / Runtime
 └── Learning / Evolution
 │
 ▼
Authorized subsystem interfaces
```

### GUI responsibilities

The GUI may provide:

- system overview
- engine health
- active sessions
- conversations
- context inspection
- memory search and review
- experience history
- goals/tasks/plans
- execution history
- tool status
- worker status
- model/runtime status
- configuration views
- event/trace inspection
- learning/evolution review
- deployment status
- audit history
- alerts and failures

### GUI safety rules

The GUI must distinguish:

- read-only operations
- reversible mutations
- privileged mutations
- destructive operations
- emergency controls

A GUI action must become an authorized control-plane command. It must not write directly to subsystem databases or bypass validation.

Sensitive values must be redacted according to policy.

Every privileged mutation should produce an auditable event.

---

# 15. Developer and AI Contributor Control

AI contributors must use the same architectural boundaries as human developers where practical.

An AI contributor must not:

- silently alter architecture
- bypass validation
- directly mutate protected production state
- erase evidence of its changes
- convert speculative ideas into normative architecture
- claim a change was completed when it was not verified

Architecture changes require explicit review and validation.

---

# 16. Configuration and Runtime

Configuration has precedence and ownership.

A recommended precedence model is:

```text
Built-in defaults
    ↓
Installation configuration
    ↓
System configuration
    ↓
Profile configuration
    ↓
User configuration
    ↓
Runtime overrides
```

Higher layers may override lower layers only where the setting is declared mutable.

Secrets must not be embedded in source-controlled architecture files.

Runtime changes must be validated before activation and recorded when they affect system behavior.

---

# 17. Events

Events form the durable operational history of the architecture.

Events should provide:

- event ID
- event type
- version
- timestamp
- source
- correlation ID
- causation ID where applicable
- actor
- payload
- schema version

Important events include:

- interaction received
- context created
- memory retrieved
- plan created
- action authorized
- tool invoked
- execution completed
- execution failed
- experience recorded
- learning proposed
- learning promoted
- configuration changed
- deployment changed
- permission changed

Events are evidence. They are not automatically permanent knowledge.

---

# 18. Security and Trust

Security applies to:

- users
- AI contributors
- models
- tools
- stored information
- configuration
- runtime
- GUI
- APIs
- external systems

Authorization must be capability-based where practical.

High-risk operations should support confirmation, policy checks, or additional authorization.

Trust must be represented explicitly rather than inferred solely from origin.

---

# 19. Observability

Observability must allow reconstruction of important system behavior without requiring unrestricted access to private model reasoning.

The system should expose structured:

- events
- metrics
- health state
- traces
- execution records
- decision evidence
- failures
- configuration changes
- resource usage

Observability itself is subject to retention and privacy policies.

---

# 20. Self-Improvement

Self-improvement must be controlled.

```text
Observation
 ↓
Hypothesis
 ↓
Experiment
 ↓
Evaluation
 ↓
Candidate change
 ↓
Validation
 ↓
Promotion
 ↓
Monitoring
 ↓
Rollback if required
```

A model-generated suggestion is not automatically an approved architectural change.

Changes must be reversible where practical.

---

# 21. Storage and Database

Persistent storage must preserve the logical domains of:

- conversation
- context checkpoints
- memory
- knowledge
- graph relationships
- experience
- learning
- goals/tasks/plans
- execution
- tools
- models
- configuration
- events
- audit
- deployment

Schema changes require versioned migrations.

The database is a persistence mechanism, not the owner of cognitive policy.

---

# 22. Background Workers

Workers must have:

- explicit ownership
- queue semantics
- retry policy
- idempotency behavior
- cancellation behavior
- backpressure
- supervision
- health state
- observability

A background worker must not silently mutate unrelated domains.

---

# 23. Testing and Validation

Testing must validate architecture, not just individual functions.

Required layers include:

- unit tests
- contract tests
- integration tests
- persistence tests
- event tests
- security tests
- failure injection
- recovery tests
- migration tests
- runtime/model adapter tests
- GUI/control-plane tests
- end-to-end cognitive lifecycle tests
- regression tests
- property/invariant tests

A release is not complete because the software compiles.

The architecture must demonstrate that ownership, lifecycle, permissions, provenance, observability, and recovery still work.

---

# 24. Deployment

Deployment must establish:

- runtime
- storage
- configuration
- models
- permissions
- workers
- event infrastructure
- observability
- health checks
- migrations
- backup/recovery

Deployment must be versioned and reproducible.

Updates must support validation and rollback.

---

# 25. Future Expansion

Future capabilities must integrate through stable contracts.

Examples:

- robotics
- simulation
- distributed cognition
- multiple agents
- new model classes
- new sensors
- new interfaces
- advanced learning
- scientific reasoning

New capability does not automatically justify a new core engine.

The first question is whether the capability belongs inside an existing boundary.

---

# 26. Capability Roadmap

Future advancement should proceed through architectural gates:

1. Define capability.
2. Identify ownership.
3. Define lifecycle.
4. Define contracts.
5. Define security boundary.
6. Define persistence requirements.
7. Define observability.
8. Define tests.
9. Implement behind stable interfaces.
10. Validate integration.
11. Document the change.
12. Promote only after evidence.

---

# 27. Final Architectural Invariants

Every component must preserve:

1. explicit ownership
2. explicit lifecycle
3. stable identity
4. provenance
5. confidence/uncertainty
6. failure visibility
7. model independence
8. controlled external effects
9. observability
10. versioned evolution
11. human control
12. compatibility

If a proposed implementation violates one of these, it is not v0.0.2.1-compliant until the conflict is explicitly resolved.

---

# 28. Definition of Done for v0.0.2.1

The architecture is considered structurally complete when:

- every chapter has a defined responsibility
- every major engine has a clear owner
- cross-engine boundaries are explicit
- persistent and temporary state are separated
- memory and experience are distinct
- learning has promotion gates
- planning and execution are separate
- tools are capability boundaries
- models are behind the AI Runtime
- GUI operates through the Control Plane
- configuration has ownership and precedence
- events provide correlation and evidence
- storage has migration discipline
- security governs privileged effects
- observability can reconstruct important operations
- testing verifies architectural invariants
- deployment supports reproducibility and recovery
- future expansion has explicit gates
- appendices reflect the normative architecture
- unresolved material is quarantined as non-normative

This document is the architecture-wide contract. Detailed chapters may specialize it but must not silently contradict it.
