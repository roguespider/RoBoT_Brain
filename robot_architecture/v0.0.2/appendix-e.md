# Appendix E. Development Guidelines

## Purpose

This appendix defines the development standards and engineering practices for building, maintaining, and expanding RoBoT.

The goal of these guidelines is to ensure that RoBoT remains:

- Modular
- Maintainable
- Testable
- Explainable
- Secure
- Performant
- Extensible
- Consistent with the architectural vision

These guidelines apply to all contributors, subsystems, plugins, tools, AI integrations, and future expansions.

RoBoT is designed as a long-term cognitive platform. Development decisions should prioritize architectural integrity over short-term convenience.

---

# Core Development Philosophy

The primary development principle is:

> Build the architecture first. Add capabilities second.

Features should strengthen the cognitive architecture rather than bypass it.

A new capability should answer:

- Where does this belong?
- Which subsystem owns it?
- What interface does it expose?
- How will it be tested?
- How will it be observed?
- How will it evolve?

If a feature does not have a clear architectural home, the design should be reconsidered before implementation.

---

# Development Principles

## 1. Preserve Modularity

Each subsystem has a defined responsibility.

Developers should avoid:

- Cross-subsystem implementation dependencies
- Shared hidden state
- Duplicate functionality
- Direct database access from unrelated systems

Preferred communication:

```text
Subsystem A

↓

Interface

↓

Subsystem B
```

Avoid:

```text
Subsystem A

↓

Internal implementation details

↓

Subsystem B
```

---

# 2. Respect Architectural Boundaries

The following systems maintain ownership boundaries:

| System       | Responsibility                                        |
| ------------ | ----------------------------------------------------- |
| Memory       | Stores and retrieves information                      |
| Knowledge    | Represents relationships and structured understanding |
| Experience   | Records events, outcomes, and lessons                 |
| Learning     | Improves capabilities from evidence                   |
| Context      | Builds relevant working information                   |
| Conversation | Handles interaction                                   |
| Planning     | Determines intended actions                           |
| Execution    | Performs actions                                      |
| AI Runtime   | Executes AI models                                    |
| Tools        | Provides external capabilities                        |
| Storage      | Manages persistence                                   |

A subsystem should not absorb responsibilities belonging to another subsystem.

---

# 3. Rust-First Development

RoBoT production code should remain Rust-first.

Rust provides:

- Memory safety
- Reliable concurrency
- Native performance
- Portable deployment
- Reduced runtime dependencies

Python and other languages may be used for:

- Research
- Prototyping
- Model preparation
- Data analysis

However, production architecture should remain centered around Rust.

---

# 4. Use Interfaces Before Implementations

New functionality should begin with an interface design.

Example:

```text
Speech Service

Interface:

transcribe(audio)

generate(text)
```

Implementation:

```text
Whisper
Candle
Kokoro
Future Models
```

The architecture should depend on capabilities, not specific technologies.

---

# 5. Keep AI Models Replaceable

AI models are resources, not architecture.

Never hardcode assumptions about:

- Specific models
- Model providers
- Model sizes
- Model families

All AI execution should pass through:

```text
Subsystem

↓

AI Runtime

↓

Model Manager

↓

Model
```

This allows future models to replace current models without architectural changes.

---

# 6. AI Runtime Rules

All AI inference must use the AI Runtime.

Do not:

- Load models directly inside subsystems
- Manage GPU memory independently
- Create isolated inference pipelines

The AI Runtime owns:

- Model loading
- Device selection
- Resource management
- Scheduling
- Monitoring
- Validation

Candle is the current primary inference foundation, but the architecture must remain runtime-independent.

---

# 7. Memory Development Guidelines

Memory is one of RoBoT's core systems.

Developers should preserve separation between:

- Working memory
- Episodic memory
- Semantic memory
- Procedural memory
- Experience records

Memory additions should consider:

- Source
- Confidence
- Timestamp
- Provenance
- Relationships
- Retrieval value

Do not store information permanently without considering:

- Trust level
- Importance
- Future usefulness

---

# 8. Experience Development Guidelines

Experience represents what happened.

Experience records should include:

- Situation
- Action
- Result
- Outcome
- Confidence
- Lessons learned

Experience should never simply duplicate memory.

Memory asks:

> What do we know?

Experience asks:

> What happened and what did we learn?

---

# 9. Confidence Requirements

Information should not be treated as equally reliable.

Developers should track confidence for:

- Memories
- Knowledge nodes
- Relationships
- Skills
- Plans
- Hypotheses
- Predictions

Confidence should be:

- Explainable
- Adjustable
- Evidence-based
- Historically tracked

Avoid hidden confidence changes.

---

# 10. Event-Driven Development

Important system actions should produce events.

Events provide:

- Observability
- Debugging
- Learning input
- Replay capability
- Audit history

Examples:

```text
MemoryCreated

PlanGenerated

ToolExecuted

ExperienceCompleted

ModelLoaded
```

Events should be:

- Immutable
- Versioned
- Structured
- Traceable

---

# 11. Architecture Trace Requirements

Complex workflows should be observable.

Developers should add tracing for:

- Major subsystem transitions
- AI inference
- Memory retrieval
- Planning decisions
- Tool execution
- Learning updates

A developer should be able to answer:

- What happened?
- Why did it happen?
- Which components participated?
- What information influenced the result?

---

# 12. Database Guidelines

Database access should occur through controlled layers.

Preferred:

```text
Subsystem

↓

Repository

↓

Database
```

Avoid:

```text
Subsystem

↓

Raw Database Queries
```

Database changes require:

- Migration
- Testing
- Documentation
- Rollback consideration

---

# 13. Error Handling Guidelines

Errors should be:

- Explicit
- Structured
- Logged
- Recoverable where possible

Avoid:

- Silent failures
- Ignored errors
- Generic messages

Errors should provide:

- What failed
- Where it failed
- Why it failed
- Recovery attempt

---

# 14. Testing Requirements

Every major feature requires testing.

Minimum expectations:

## Unit Tests

Validate individual components.

## Integration Tests

Validate subsystem communication.

## Workflow Tests

Validate complete cognitive processes.

## Regression Tests

Prevent previously fixed problems from returning.

## Performance Tests

Measure resource usage and speed.

---

# 15. Development Workflow

Recommended workflow:

```text
Design

↓

Architecture Review

↓

Interface Definition

↓

Implementation

↓

Unit Tests

↓

Integration Tests

↓

Tracing Added

↓

Documentation Updated

↓

Merge
```

Implementation should follow architecture, not redefine it accidentally.

---

# 16. Code Organization Guidelines

Code should be organized around responsibilities.

Preferred:

```text
memory/

├── retrieval.rs
├── storage.rs
├── confidence.rs
└── validation.rs
```

Avoid:

```text
memory/

└── everything.rs
```

Files should remain focused.

---

# 17. Naming Standards

Names should describe purpose.

Preferred:

- `MemoryManager`
- `ExperienceRecorder`
- `ModelRegistry`
- `ExecutionScheduler`

Avoid:

- `Helper`
- `Manager2`
- `Utils`
- `Temp`
- `NewThing`

Names should communicate architectural responsibility.

---

# 18. Documentation Requirements

Every major subsystem should document:

- Purpose
- Responsibilities
- Interfaces
- Dependencies
- Data structures
- Events
- Testing strategy
- Future expansion points

Documentation is part of the architecture.

---

# 19. Dependency Management

Dependencies should be evaluated before adoption.

Consider:

- Maintenance status
- Security
- License
- Performance
- Long-term viability
- Rust compatibility

Avoid unnecessary dependencies when a simpler internal solution exists.

---

# 20. Plugin Development Guidelines

Plugins must:

- Use defined interfaces
- Declare capabilities
- Validate permissions
- Handle failures safely
- Avoid modifying core systems

Plugins extend RoBoT.

They do not redefine RoBoT.

---

# 21. MCP Tool Guidelines

Tools should:

- Have clear descriptions
- Validate inputs
- Return structured outputs
- Report failures
- Support versioning

Tools should be treated as external capabilities, not trusted internal code.

---

# 22. Security Guidelines

Security should be considered during design.

Developers should:

- Validate external input
- Avoid unsafe execution
- Protect credentials
- Limit permissions
- Audit sensitive actions

Security should not be added only after problems occur.

---

# 23. Performance Guidelines

Optimize after measuring.

Performance improvements should focus on:

- Removing unnecessary work
- Improving algorithms
- Efficient memory usage
- Better caching
- Parallel execution

Avoid premature optimization that damages clarity.

---

# 24. AI Development Guidelines

When adding AI capabilities:

Document:

- Model purpose
- Input format
- Output format
- Expected performance
- Resource requirements
- Validation method

AI behavior should be measurable.

---

# 25. Avoid Technical Debt

Before adding shortcuts, consider:

- Will this still make sense in one year?
- Does this violate architecture?
- Does this create hidden coupling?
- Will future developers understand it?

Temporary solutions should be clearly marked.

---

# 26. Git and Version Control Guidelines

Commits should:

- Have clear descriptions
- Represent logical changes
- Avoid unrelated modifications

Preferred:

```text
Add memory confidence scoring

Fix planner retry handling
```

Avoid:

```text
Changes
```

---

# 27. Architecture Decision Records

Major changes should create a Design Decision entry.

Examples:

- Replacing a runtime
- Changing database strategy
- Adding major subsystems
- Changing communication patterns

Future developers should understand why decisions were made.

---

# 28. Development Environment

Development tools should support:

- Formatting
- Linting
- Testing
- Profiling
- Debugging
- Tracing

Recommended automation:

- Continuous integration
- Automated tests
- Benchmark tracking
- Documentation checks

---

# 29. Future-Proof Development

New development should assume:

- Models will change
- Hardware will change
- Interfaces will change
- Requirements will change

Therefore:

- Abstract replaceable components
- Preserve interfaces
- Avoid vendor lock-in
- Maintain compatibility

---

# Final Development Principle

The purpose of development is not simply to add code.

The purpose is to grow a cognitive architecture that remains understandable, reliable, and adaptable.

Every contribution should improve one or more of the following:

- Intelligence
- Memory
- Learning
- Reliability
- Explainability
- Maintainability
- Extensibility

RoBoT should be built like a long-lived platform, not a temporary application.

The code is the implementation.

The architecture is the foundation.

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

User Interface and Visualization Architecture

Chapter XX - User Interface and Visualization Architecture
XX.1 Purpose

The User Interface and Visualization Architecture defines how humans interact with RoBoT and how the internal operation of the cognitive system can be observed, debugged, and understood.

The interface layer exists to provide:

Human interaction with RoBoT
Visualization of system status
Access to memories, experiences, and learned knowledge
Monitoring of autonomous processes
Developer debugging capabilities

The interface layer does not own intelligence.

It does not:

Perform reasoning
Manage memory
Build context
Execute tools
Make decisions
Modify cognitive systems directly

The interface is a window into the system.

The cognitive architecture remains independent.

XX.2 Design Philosophy

Traditional applications are designed around screens.

RoBoT is designed around systems.

The UI should not dictate architecture.

Instead:

Cognitive Systems

Conversation Engine
|
|
Context Engine
|
|
Memory Engine
|
|
Experience Engine
|
|
Learning Engine

             |
             |
             ▼

Visualization Layer

             |
             |
             ▼

Human Operator

The interface observes and controls through defined APIs and events.

XX.3 Interface Architecture Overview
User
|
▼
User Interface
|
┌────────────┼────────────┐
| | |
▼ ▼ ▼

Conversation Memory System
Interface Explorer Monitor

                     |
                     ▼

              API Layer


                     |
                     ▼

            RoBoT Core Systems

The UI communicates through stable interfaces.

It never reaches directly into internal subsystem data.

XX.4 Primary Interface Components

The RoBoT interface is divided into several major areas.

1. Conversation Interface
   Responsibility

Provide the primary human interaction channel.

The conversation interface manages:

User input
Assistant responses
Streaming output
Attachments
Tool activity display
Session management

It does not manage:

Memory retrieval
Prompt construction
Reasoning
Learning
Conversation Flow
User Input

↓

Conversation Interface

↓

Conversation API

↓

Conversation Engine

↓

Context Engine

↓

LLM

↓

Response Stream

↓

Conversation Interface 2. Memory Explorer
Responsibility

Provide visibility into RoBoT's stored knowledge.

The Memory Explorer allows users to inspect:

Episodic memories
Semantic knowledge
User preferences
Knowledge graph relationships
Strategic knowledge
Memory confidence
Memory Visualization

Example:

Memory Card

ID:
1842

Type:
Architecture Decision

Summary:
RoBoT uses SQLite for local persistence.

Importance:
98%

Confidence:
99%

Created:
2026-07-29

Relationships:

Database System
Experience Engine
Storage Architecture
Memory Operations

The interface may request:

Search memories
View memory history
View relationships
Review confidence changes
Archive memories

The interface does not directly edit memory.

All changes pass through the Memory Engine.

3. Experience Viewer
   Responsibility

Display what RoBoT has learned from execution.

Experiences represent:

Actions performed
Outcomes observed
Successes
Failures
Lessons learned

Example:

Experience #482

Goal:

Implement SQLite transaction handling

Action:

Modified database module

Outcome:

Successful

Lessons:

Use transaction wrapper pattern

Confidence:

96%

The Experience Viewer helps answer:

"What has RoBoT actually done?"

4. Knowledge Graph Visualization
   Responsibility

Provide a visual representation of relationships between information.

Example:

Rust

|
├── SQLite
|
├── Ownership
|
└── Async Runtime

Experience Engine

|
└── Event Pipeline

The graph view helps identify:

Strong relationships
Missing connections
Knowledge clusters
Duplicate concepts
XX.5 Cognitive Activity Monitor
Purpose

The Cognitive Activity Monitor is a developer visualization system for observing RoBoT's internal operation.

It does not display private model reasoning.

It displays system activity.

The goal is debugging, performance analysis, and architecture validation.

XX.6 System Flow Visualization

The monitor displays subsystem activity.

Example:

Conversation Engine
|
▼

Context Engine
|
▼

Memory Engine
|
▼

Prompt Assembly
|
▼

LLM
|
▼

Experience Engine
|
▼

Learning Engine

Each component reports:

Current state
Processing time
Events
Errors
Data movement
XX.7 Activity States

Subsystems expose current operational states.

Example:

Green

Idle

Yellow

Processing

Blue

Waiting

Red

Error

Example:

Memory Engine

Status:
Searching

Query:

Rust SQLite transaction

Candidates:

12

Selected:

3

Duration:

42 ms
XX.8 Event-Based Observability

The visualization system depends on system events.

Subsystems publish events.

They do not know the dashboard exists.

Example:

Memory Engine

        |
        |
        ▼

System Event Bus

        |
        |
        ▼

Cognitive Monitor
XX.9 System Event Model

Common event structure:

SystemEvent

timestamp

subsystem

operation

status

duration

details

correlation_id

Examples:

MemorySearchCompleted

Results:
7

Duration:
38 ms
ContextCompressed

Before:
4200 tokens

After:
280 tokens
PromptCreated

Budget:
2048

Used:
1837
XX.10 Conversation Replay

The system should support replaying previous executions.

Purpose:

Debug failures
Understand retrieval decisions
Validate architecture

Example:

Replay Session #24

▶ Conversation Received

▶ Task Detected

▶ Context Built

▶ Memories Retrieved

▶ Prompt Created

▶ Response Generated

▶ Experience Saved

The replay system shows:

What happened
When it happened
Which systems participated

It does not attempt to reconstruct private model reasoning.

XX.11 Context Visualization

The UI should expose context assembly.

Example:

Current Prompt

System

220 tokens

User

180 tokens

Code

850 tokens

Memory

300 tokens

Tools

250 tokens

Reserve

248 tokens

Total:

1837 / 2048

The developer can inspect:

Retrieved memories
Dropped context
Compression results
Token allocation
XX.12 Memory Retrieval Visualization

Example:

Retrieval Request:

Continue Experience Engine work

Candidates:

Memory #1842

Similarity:
0.92

Memory #2017

Similarity:
0.84

Memory #1510

Similarity:
0.61

Loaded:

Top 2

This makes context decisions explainable.

XX.13 Interface Communication Rules

The UI layer follows these rules:

Rule 1

Never access subsystem internals directly.

Rule 2

All communication uses:

APIs
Events
Data contracts
Rule 3

Visualization must never affect cognitive processing.

Rule 4

Debug tools must be optional.

RoBoT must operate without the UI.

XX.14 Future Visualization Features

Future versions may include:

Cognitive Timeline

A chronological view of system activity.

12:10:01

Message Received

12:10:02

Memory Search

12:10:03

Context Compression

12:10:04

Response Generated
Memory Heat Map

Shows:

Frequently used memories
Forgotten memories
Important knowledge clusters
Learning Progress Dashboard

Shows:

New skills
Confidence changes
Policies created
Failed learning attempts
Architecture Health Monitor

Displays:

Event latency
Queue sizes
Memory growth
Retrieval performance
Context efficiency
XX.15 Implementation Roadmap
Phase 1 - Basic Interface

Implement:

Chat interface
Session management
API layer
Basic status display
Phase 2 - Memory Visualization

Implement:

Memory browser
Search
Graph visualization
Confidence display
Phase 3 - Observability Layer

Implement:

System events
Event storage
Timeline viewer
Subsystem status
Phase 4 - Cognitive Dashboard

Implement:

Context visualization
Retrieval analysis
Prompt budget display
Experience replay
XX.16 Architectural Summary

The User Interface and Visualization Architecture provides the human connection layer for RoBoT.

Its purpose is not to make RoBoT intelligent.

Its purpose is to make intelligence observable, controllable, and understandable.

The architecture follows the principle:

Systems produce events.

Events create visibility.

Visibility enables debugging.

Debugging enables evolution.

The interface is the cockpit.

The cognitive architecture is the engine.

They must remain separate so RoBoT can continue evolving without the UI becoming a limitation.

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|
GUI Design Guidelines

Chapter XX - GUI Design Guidelines
XX.1 Purpose

The RoBoT GUI provides a human interface for interacting with, observing, and managing the RoBoT cognitive architecture.

The GUI is designed around three primary goals:

Interaction

Allow users to communicate with RoBoT naturally.

Transparency

Allow users to understand system activity without exposing private model reasoning.

Control

Allow users to manage memories, tasks, tools, and system behavior.

The GUI is not responsible for intelligence.

The GUI does not:

Perform reasoning
Generate responses
Manage memory directly
Build prompts
Execute cognitive decisions

The GUI is a visualization and control layer.

XX.2 Design Philosophy

Traditional AI interfaces hide almost everything behind a chat box.

RoBoT takes a different approach.

The user should not see the AI's private reasoning.

However, the user should be able to see:

What systems are active
What information was used
How much context was consumed
What memories were retrieved
What experiences were recorded
Where failures occurred

The goal is:

Explain the operation of the system without exposing internal model reasoning.

XX.3 Core GUI Principles
Principle 1 - System Transparency

RoBoT should never feel like a black box.

Users should be able to answer:

What is RoBoT doing?
What information influenced the response?
What subsystem is currently active?
Why did a task fail?
Principle 2 - Separation of View and Intelligence

The GUI observes the architecture.

It does not become part of the architecture.

                 RoBoT Core

Conversation Engine
Context Engine
Memory Engine
Experience Engine
Learning Engine

          |
          |
          ▼

      Event System

          |
          |
          ▼

        GUI Layer

Principle 3 - Progressive Disclosure

The interface should reveal complexity gradually.

A normal user should see:

Conversation
Status
Results

A developer should be able to expand:

Context construction
Memory retrieval
Events
Performance metrics

The complexity exists when needed, not all the time.

XX.4 Main GUI Layout

The recommended layout:

┌───────────────────────────────────────────┐
│ RoBoT │
├───────────────┬───────────────────────────┤
│ │ │
│ Navigation │ Conversation │
│ │ │
│ Chat │ User │
│ Memory │ Assistant │
│ Tasks │ │
│ Experiences │ │
│ System │ │
│ │ │
├───────────────┴───────────────────────────┤
│ Cognitive Activity Monitor │
└───────────────────────────────────────────┘
XX.5 Primary GUI Sections

1. Conversation Workspace

The primary interaction area.

Provides:

Chat interaction
Streaming responses
Attachments
Tool results
Task status

Example:

User:

Continue SQLite transaction implementation

RoBoT:

Loading project context...

✓ Found active task
✓ Retrieved architecture notes
✓ Loaded database experience

Response:
...

The interface should display activity states, not hidden reasoning.

XX.6 Context Dashboard
Purpose

Show how RoBoT builds the active prompt.

The user should be able to inspect:

Current task
Active files
Retrieved memories
Token usage
Removed context

Example:

Current Context

System:
220 tokens

User:
180 tokens

Code:
850 tokens

Memory:
300 tokens

Tools:
250 tokens

Total:

1800 / 2048
Context Visualization
Question

    ↓

Task Detection

    ↓

Context Planning

    ↓

Memory Retrieval

    ↓

Compression

    ↓

Prompt Assembly

    ↓

LLM
XX.7 Memory Explorer Design
Purpose

Provide visibility into RoBoT's knowledge system.

The Memory Explorer displays:

Memory cards
Relationships
Confidence
Importance
Usage history

Example:

Memory Card

Title:

SQLite Transaction Architecture

Type:

Semantic Knowledge

Confidence:

96%

Importance:

98%

Used:

42 times

Relationships:

Database Engine
Experience Engine
Storage Layer
Memory Actions

Allowed:

Search
Inspect
Compare
Archive request
View history

Not allowed:

Direct database modification

All changes must pass through the Memory Engine.

XX.8 Experience Viewer Design

The Experience Viewer shows what RoBoT has learned through execution.

Example:

Experience #482

Goal:

Implement MCP bridge

Actions:

Created Rust module

Updated event system

Result:

Successful

Lesson:

Avoid duplicate event definitions

Confidence:

94%

This helps answer:

"What has RoBoT actually learned from doing?"

XX.9 Cognitive Activity Monitor
Purpose

The Cognitive Activity Monitor provides real-time visibility into system operation.

It displays:

Active subsystems
Events
Timing
Errors
Data movement

It does not display:

Private chain-of-thought
Hidden model reasoning

Example:

Conversation Engine

✓ Message Stored

Context Engine

✓ Building Context

Memory Engine

Searching...

Found:

7 memories

Compression

4200 tokens

↓

280 tokens

Prompt

1837 / 2048 tokens
XX.10 System Visualization

The GUI should provide a live architecture map.

Example:

              Conversation

                    ↓

              Context Engine

             ↙            ↘

       Memory              Planning


                    ↓

                  LLM


                    ↓

             Experience Engine


                    ↓

             Learning Engine

Subsystem states:

Idle

Processing

Waiting

Completed

Error
XX.11 Event Timeline

The GUI should support replaying system activity.

Example:

12:10:01

Message Received

12:10:02

Task Identified

12:10:03

Memory Retrieved

12:10:04

Context Compressed

12:10:05

Prompt Generated

12:10:08

Response Complete

12:10:09

Experience Saved

This allows debugging without guessing.

XX.12 Task Workspace

RoBoT requires a dedicated task view.

A task is larger than a single conversation.

The Task Workspace contains:

Task:

Improve Memory Retrieval

Files:

memory.rs
context.rs

Decisions:

Use compressed summaries

Constraints:

2048 token budget

Status:

Active

This supports long-running engineering tasks.

XX.13 Developer Mode

Developer Mode exposes deeper system information.

Features:

Event stream
Memory retrieval scores
Context decisions
Token allocation
Performance metrics
Error logs

Example:

Memory Retrieval

Candidate #1

Similarity:
0.92

Importance:
0.95

Confidence:
0.98

Loaded:
YES
XX.14 Visualization Rules
Rule 1

Never visualize fake intelligence.

Do not create animations implying the AI is "thinking" when no such process exists.

Rule 2

Visualize actual system events.

Good:

Memory Search Started
Memory Search Completed

Bad:

AI Brain Thinking...
Rule 3

Every visualization should have a debugging purpose.

Avoid decorative complexity.

Rule 4

The GUI must remain optional.

RoBoT must operate through:

API
CLI
MCP
Automation

without requiring the GUI.

XX.15 Future GUI Features
Memory Timeline

Shows how knowledge evolves.

Created

↓

Used

↓

Strengthened

↓

Promoted

↓

Strategic Knowledge
Learning Dashboard

Displays:

New skills
Policy creation
Confidence growth
Failed learning attempts
System Health Dashboard

Displays:

Memory size
Context efficiency
Retrieval speed
Event latency
Storage usage
Architecture Replay

Allows developers to replay previous interactions:

Question

↓

Context

↓

Memory

↓

Prompt

↓

Response

↓

Experience

↓

Learning
XX.16 Recommended Technology Direction

The GUI should be built independently from the cognitive runtime.

Recommended architecture:

Rust Core

     |

API / Event Stream

     |

Frontend Application

Possible future interfaces:

Desktop application
Web dashboard
Developer console
Remote monitoring interface
XX.17 Final Design Goal

The RoBoT GUI should feel less like a chatbot window and more like an operating console for an intelligent runtime.

The user should be able to:

Talk to RoBoT naturally
Understand what systems are active
Inspect memory and experiences
Debug failures
Monitor learning
Observe architecture health

The guiding principle:

The GUI should reveal the machine.

It should not pretend to be the machine.

End of Chapter - GUI Design Guidelines

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

Chapter 03 - AI Contributor Operating Agreement
3.1 Purpose

This document defines the rules, responsibilities, and development procedures for any AI system or human contributor modifying RoBoT.

The purpose is to ensure that all contributors preserve the architectural integrity of the system.

RoBoT is not a collection of independent features.

It is a cognitive runtime composed of interconnected subsystems with strict responsibilities.

Changes must improve the architecture, not simply make individual components function.

3.2 The Role of the AI Contributor

An AI contributor acts as:

Software engineer
Architecture assistant
Code reviewer
Documentation maintainer

The AI contributor does not act as the system architect.

The architecture direction comes from the project owner and authoritative design documents.

The AI contributor must:

Understand existing architecture before modifying code
Follow subsystem boundaries
Preserve existing contracts
Explain architectural impacts
Prefer minimal changes
3.3 Required Reading Order

Before modifying code, an AI contributor should read:

1. Vision and Philosophy

2. Core Design Principles

3. AI Contributor Operating Agreement

4. High-Level System Overview

5. Data Flow Architecture

6. Relevant Subsystem Chapter

7. Current Implementation

Code is the implementation.

Architecture is the source of truth.

3.4 Golden Rules
Rule 1

Never create a new subsystem without architectural justification.

Bad:

Need memory search.

Create MemoryHelperManager2

Good:

Existing Memory Engine owns retrieval.

Extend existing interface.
Rule 2

Every subsystem has one responsibility.

Conversation Engine:

Stores conversations.

Does not:

Retrieve memories
Build prompts
Learn patterns

Context Engine:

Builds context.

Does not:

Store permanent knowledge
Modify memories

Memory Engine:

Stores knowledge.

Does not:

Decide reasoning
Execute actions

Experience Engine:

Records outcomes.

Does not:

Create policies

Learning Engine:

Creates reusable knowledge.

Does not:

Execute tools
3.5 Before Writing Code

The AI contributor must answer:

What subsystem owns this feature?

Example:

"Where does context compression belong?"

Answer:

Context Engine.

Not:

Memory Engine.

What data enters?

Example:

User question

-

Current task

-

Retrieved memories
What data leaves?

Example:

Compressed prompt

-

Token allocation
What existing contract changes?

If none:

Proceed.

If yes:

Explain impact first.

3.6 Modification Rules

Before changing code:

Inspect existing implementation.
Identify ownership.
Identify dependencies.
Check architecture document.
Make smallest change possible.
Test.
Update documentation.
3.7 Prohibited Behaviors

An AI contributor must not:

Invent architecture

Example:

Creating:

SuperMemoryAIManager

because it is convenient.

Duplicate functionality

Example:

Existing:

ContextCompressor

Creating:

PromptOptimizer

without justification.

Bypass subsystem boundaries

Bad:

GUI
|
▼
SQLite Database

Good:

GUI

↓

API

↓

Memory Engine

↓

Database
Store temporary information permanently

Conversation logs are not automatically memories.

3.8 Coding Philosophy

RoBoT follows:

Composition over coupling

Prefer:

Engine A

↓

Interface

↓

Engine B

over:

Engine A directly controls Engine B
Events over hidden communication

Prefer:

ExperienceCompleted Event

        ↓

Learning Engine

over:

Experience Engine
directly calls Learning Engine
Data contracts over assumptions

Every subsystem communicates through defined structures.

3.9 Debugging Procedure

When something fails:

Do not immediately patch.

Follow:

Observe

↓

Identify subsystem

↓

Trace data flow

↓

Locate ownership problem

↓

Fix root cause

↓

Add test
3.10 Architecture Review Checklist

Before accepting a change:

Does it preserve subsystem ownership?

Yes / No

Does it increase coupling?

Yes / No

Does it improve scalability?

Yes / No

Does it reduce context or memory waste?

Yes / No

Can another contributor understand it?

Yes / No

3.11 Documentation Requirements

Every major feature requires:

Architecture update
Data flow explanation
Interface definition
Testing strategy
Implementation notes

Code without documentation becomes technical debt.

3.12 AI Development Workflow

The recommended workflow:

Architecture

↓

Design Discussion

↓

Implementation Plan

↓

Small Code Change

↓

Testing

↓

Review

↓

Documentation Update

↓

Commit

Never:

Prompt

↓

Generate thousands of lines

↓

Hope it works
3.13 Final Agreement

Every contributor agrees:

RoBoT is built as a long-running cognitive system.

Short-term convenience must not damage long-term architecture.

The goal is not simply:

"Make the code run."

The goal is:

"Build a maintainable cognitive runtime capable of continuous evolution."

Other chapters I think are still missing from v0.0.2

Looking at everything you have now, I would add:

Chapter Purpose
AI Contributor Operating Agreement Rules for AI coding agents
API and Interface Contracts How systems communicate
Event Bus Architecture Needed for your future dashboard
Security and Permission Architecture Tools, files, user control
Configuration Architecture Models, paths, runtime settings
Deployment Architecture Local machine, server, distributed future
Testing Strategy Already have, but should include AI contributor testing rules
Versioning and Migration Strategy How RoBoT evolves without breaking memory

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|

\# AI Agent Instructions

This document is the authoritative architecture for this repository.

Every coding assistant working on this project must:

\- Read this document before making changes.
\- Prefer these architectural rules over existing inconsistent code.
\- Modify related files together rather than in isolation.
\- Optimize for long-term maintainability.
\- Avoid placeholder implementations.
\- Keep subsystems loosely coupled.
\- Maintain compile-ready Rust whenever practical.

Before making any changes:

1\. Read ARCHITECTURE.md completely.
2\. Treat it as the authoritative specification for this repository.
3\. Follow its architecture, naming conventions, dependency rules, and design principles.
4\. If existing code conflicts with ARCHITECTURE.md, prefer the architecture unless it would introduce compilation errors.
5\. Read all files related to this subsystem before making changes.
6\. Implement the entire subsystem, not just the requested file.
7\. Keep the architecture internally consistent.
8\. When finished, summarize:

&#x20; - files modified
&#x20; - architectural improvements
&#x20; - remaining work
&#x20; - assumptions made

now summarize ARCHITECTURE.md in your own words.

List:

\- the major subsystems
\- the event flow
\- repository conventions
\- dependency rules
\- coding standards

Only after doing that should you begin modifying code.

You are the lead software engineer for this project.
Your job is NOT to answer questions.
Your job is to COMPLETE the project.

=========================================================
MISSION
===

Treat this repository as a professional open-source project.
Do not produce placeholder code unless absolutely unavoidable.
Every module should be production quality.
Always think about the entire architecture before modifying files.
If a better design requires restructuring folders or moving code, do it.
Avoid unnecessary complexity, but never sacrifice maintainability.

=========================================================
WORKFLOW
===

Before writing code:

1. Read the entire repository.
2. Understand every module.
3. Build an internal dependency graph.
4. Find architectural inconsistencies.
5. Determine the cleanest design.

Then implement.
Do NOT ask permission every few files.
Complete as much work as possible in one pass.

=========================================================
WHEN WORKING ON A SUBSYSTEM
===

For the subsystem requested:
• identify every related file
• identify missing modules
• identify duplicate logic
• identify dead code
• identify poor abstractions
• identify cyclic dependencies
• identify naming inconsistencies
Then improve everything together.
Do not only modify the requested file if neighboring files should change.

=========================================================
CODE QUALITY
===

Every public type should have documentation.
Every important function should explain WHY it exists.
Prefer traits over duplication.
Prefer composition over inheritance.
Prefer immutable data.
Avoid global state.
Avoid magic numbers.
Avoid unwrap().
Use anyhow or thiserror where appropriate.
Return meaningful Results.
Use strong typing instead of strings whenever practical.

=========================================================
RUST STYLE
===

Prefer idiomatic Rust.
Small focused modules.
Small functions.
Minimal allocations.
Iterator chains when readable.
Avoid unnecessary cloning.
Use Arc only when ownership requires it.
Keep ownership simple.

=========================================================
PROJECT GOALS
===

The project is a self-learning AI.

Core systems include:
Experience
Memory
Knowledge
Hypothesis
Reflection
Planning
Skills
Reputation
Exploration
Learning
Events
Coordinator
Repositories
SQLite persistence
MCP interface
The architecture should be event driven.
Subsystems should remain loosely coupled.

=========================================================
WHEN ADDING CODE
===

Always ask:
What else should exist?
What is missing?
What future feature will need this?
Can this become reusable?
Can this become a service?
Can this become a trait?

=========================================================
WHEN FINISHED
===

Do NOT simply say "done."

Instead produce:

1. Files modified
2. Files created
3. Architectural improvements
4. Remaining technical debt
5. Suggestions for the next subsystem
6. Any assumptions made

=========================================================
RULE
===

Optimize for completing the entire project, not minimizing code changes.
Think like the project's CTO, not a code assistant.

|==========|==========|==========|==========|==========|==========||==========|==========|==========|==========|==========|==========|
