# Chapter 07. Context Engine

## Purpose

The Context Engine is responsible for determining **what the AI should know right now**.

Unlike Memory, which stores knowledge, or Experience, which evaluates outcomes, the Context Engine continuously builds the temporary 
working state used by every reasoning cycle.

Its primary responsibility is transforming an enormous lifetime of information into a compact, relevant, structured context that 
fits within the model's reasoning window.

Without a Context Engine, every conversation eventually suffers from one of two problems:

* Too little information resulting in poor decisions.
* Too much information resulting in context overflow.

The Context Engine prevents both.

It acts as the intelligence layer between long-term storage and active reasoning.

---

# Philosophy

The AI should never attempt to remember everything simultaneously.

Humans don't.

When asked a question, the brain activates only the memories, experiences, goals, emotions, skills, and observations that matter.

RoBoT follows the same philosophy.

Every reasoning cycle begins with a nearly empty working space.

The Context Engine assembles only what is required.

When reasoning completes, that working state is discarded.

Nothing remains unless another subsystem decides it deserves permanent storage.

This makes reasoning:

* deterministic
* explainable
* scalable
* memory efficient

---

# Responsibilities

The Context Engine is responsible for:

* Building the active reasoning context
* Selecting relevant memories
* Loading current goals
* Tracking active conversations
* Managing temporary facts
* Maintaining entity references
* Compressing large conversations
* Detecting topic changes
* Maintaining conversation continuity
* Prioritizing information by relevance
* Enforcing token budgets
* Coordinating information flow between subsystems

It never owns permanent information.

It only assembles it.

---

# Position within the Architecture

```
                 User Input
                      │
                      ▼
             Conversation Engine
                      │
                      ▼
               Context Engine
      ┌───────────────┼────────────────┐
      │               │                │
      ▼               ▼                ▼
 Memory Engine   Experience Engine   Knowledge Engine
      │               │                │
      └───────────────┼────────────────┘
                      ▼
              Active Context Model
                      │
                      ▼
             Planner / Reasoner
                      │
                      ▼
                  LLM Request
```

The Context Engine is the orchestration layer connecting nearly every cognitive subsystem.

---

# Context is Not Memory

One of the biggest architectural mistakes in AI systems is confusing context with memory.

They are completely different.

| Memory         | Context                        |
| -------------- | ------------------------------ |
| Persistent     | Temporary                      |
| Large          | Small                          |
| Stored forever | Exists for one reasoning cycle |
| Indexed        | Built dynamically              |
| Searchable     | Disposable                     |
| Historical     | Immediate                      |

Memory answers:

> "What do I know?"

Context answers:

> "What matters right now?"

---

# Context Layers

The active context consists of multiple independent layers.

```
Active Context
│
├── Conversation Context
├── Working Memory
├── Active Goals
├── Retrieved Memories
├── Relevant Experiences
├── Knowledge References
├── Current Environment
├── Active Skills
├── Recent Observations
├── User Preferences
├── Planner State
├── Tool State
├── Task State
└── System Status
```

Each layer has independent limits.

No single subsystem can consume the entire context window.

---

# Context Lifecycle

Every request follows the same lifecycle.

```
User Message
      │
      ▼
Topic Detection
      │
      ▼
Goal Identification
      │
      ▼
Memory Retrieval
      │
      ▼
Experience Retrieval
      │
      ▼
Knowledge Retrieval
      │
      ▼
Context Ranking
      │
      ▼
Context Compression
      │
      ▼
Token Budget Allocation
      │
      ▼
Final Context Package
      │
      ▼
LLM
```

Once the response completes:

```
Working Context
      │
      ▼
Discard
```

Only important information is promoted elsewhere.

---

# Context Assembly Pipeline

The Context Engine performs several stages before every reasoning cycle.

### 1. Conversation Analysis

Determines:

* intent
* entities
* references
* topic
* urgency
* ambiguity
* conversation continuity

---

### 2. Planner Requirements

The planner specifies:

* required knowledge
* required skills
* required tools
* expected output
* reasoning depth

This prevents unnecessary retrieval.

---

### 3. Memory Retrieval

Memory Engine returns:

* semantic memories
* episodic memories
* procedural memories
* graph relationships
* summaries

Each memory includes:

* relevance
* confidence
* freshness
* importance

---

### 4. Experience Retrieval

Experience Engine contributes:

* successful workflows
* failed attempts
* confidence adjustments
* learned strategies
* historical outcomes

The AI learns from previous execution instead of repeating mistakes.

---

### 5. Knowledge Retrieval

Knowledge Engine contributes:

* concepts
* documentation
* APIs
* code references
* indexed files
* manuals
* structured facts

---

### 6. Context Ranking

Everything is ranked using multiple signals.

Possible scoring factors include:

```
Final Score =
Relevance
× Confidence
× Recency
× Importance
× Goal Match
× Conversation Match
× Planner Priority
× User Preference
```

Different applications may adjust weighting, but the architecture encourages multi-factor scoring rather than relying on vector 
similarity alone.

---

### 7. Deduplication

Multiple retrieved items often represent the same concept.

Duplicates are merged.

Redundant information is removed.

Relationships are preserved.

---

### 8. Compression

Large information becomes concise summaries.

Compression preserves:

* important facts
* relationships
* references
* confidence
* citations

Compression never invents information.

---

### 9. Token Budget Allocation

Every subsystem receives its own allocation.

Example:

| Section      | Tokens |
| ------------ | ------ |
| Conversation | 20%    |
| Memory       | 25%    |
| Experience   | 15%    |
| Knowledge    | 20%    |
| Planner      | 10%    |
| Tools        | 5%     |
| System       | 5%     |

Budgets are adaptive rather than fixed. A coding task may allocate more space to documentation and tools, while a personal 
conversation may prioritize conversational history and preferences.

---

# Working Memory

Working Memory is the temporary scratchpad.

It contains:

* intermediate reasoning
* partial plans
* unresolved references
* temporary calculations
* active variables
* tool outputs
* execution notes

Working Memory disappears after the task.

It is never considered permanent memory.

---

# Conversation Context

Conversation Context preserves continuity.

It tracks:

* recent dialogue
* unresolved questions
* pronouns
* references
* assumptions
* user corrections
* conversation goals

Rather than replaying the entire chat, older exchanges are progressively summarized while retaining critical references and 
decisions.

---

# Topic Tracking

The Context Engine continuously monitors discussion flow.

Possible topic states include:

```
Programming
Architecture
Debugging
Planning
Learning
General Conversation
```

Each topic maintains its own compressed history.

Switching topics does not erase previous work.

It simply changes the active context.

---

# Context Compression

Long conversations cannot fit forever.

Instead of deleting history, the engine produces progressively richer summaries.

```
Messages
     │
     ▼
Short Summary
     │
     ▼
Medium Summary
     │
     ▼
Long-term Conversation Memory
```

Information becomes increasingly compressed as it ages while remaining recoverable through Memory when needed.

---

# Entity Tracking

The Context Engine maintains active entities.

Example:

```
RoBoT
Rust
SQLite
Experience Engine
Planner
User
GitHub Repository
```

Each entity maintains references to:

* memories
* documents
* relationships
* goals
* experiences
* conversation mentions

This reduces ambiguity and supports accurate pronoun resolution.

---

# Context Windows

Instead of one giant prompt, RoBoT constructs layered context.

```
System Layer

Conversation Layer

Planner Layer

Memory Layer

Experience Layer

Knowledge Layer

Tool Layer

Execution Layer
```

Each layer can evolve independently without affecting the others.

---

# Adaptive Retrieval

Retrieval depth depends on task complexity.

Simple questions require minimal context.

Complex architectural reasoning may retrieve:

* multiple documents
* graph memories
* procedural knowledge
* historical experiences
* planning state
* tool outputs

The Context Engine dynamically adjusts retrieval depth instead of always retrieving the maximum amount of information.

---

# Context Feedback Loop

The Context Engine continuously improves future reasoning.

```
LLM Response
      │
      ▼
Execution
      │
      ▼
Experience Evaluation
      │
      ▼
Memory Update
      │
      ▼
Future Context Retrieval
```

Better experiences lead to better future context.

---

# Explainability

One design goal is that context construction should be inspectable.

During debugging, developers should be able to visualize:

* retrieved memories
* rejected memories
* scoring values
* token usage
* compression results
* planner requests
* retrieval timing
* final context composition

This transparency greatly simplifies optimization and debugging of cognitive behavior.

---

# Future Evolution

The Context Engine is intentionally modular.

Future versions may introduce:

* predictive context prefetching
* multi-agent shared context
* long-running task contexts
* streaming context updates
* multimodal context layers
* attention heat maps
* temporal reasoning layers
* contextual caching
* personalized retrieval policies

The surrounding architecture should require minimal change as these capabilities evolve.

---

# Design Principles

The Context Engine follows several core principles:

* Context is temporary.
* Memory remains independent.
* Experience influences retrieval.
* Knowledge remains authoritative.
* The planner requests information rather than receiving everything.
* Every retrieved item is ranked.
* Token budgets are enforced.
* Compression preserves meaning.
* Context should be explainable.
* Working memory is disposable.
* Conversation continuity is maintained without replaying entire histories.
* Retrieval quality is more important than retrieval quantity.

---

# Summary

The Context Engine is the cognitive assembly system of RoBoT.

Rather than storing information, it constructs the precise working environment required for each reasoning cycle.

By combining conversation state, retrieved memories, learned experience, structured knowledge, planner requirements, and system 
state into a carefully ranked and compressed context, the engine enables efficient reasoning at scale while remaining 
transparent, modular, and extensible.

This separation between **Memory**, **Experience**, **Knowledge**, and **Context** is one of the defining architectural principles 
of RoBoT v0.0.2 and provides the foundation for future capabilities such as autonomous planning, multi-agent collaboration, 
predictive retrieval, and lifelong learning.

|==========|==========|==========|==========|         Chapter 08 - Memory Engine         |==========|==========|==========|==========|

