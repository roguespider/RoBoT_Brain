# Chapter 21. Storage Architecture

## Purpose

The Storage Architecture is the foundation that allows RoBoT to preserve, organize, retrieve, and evolve knowledge over time.

A cognitive system requires more than a database.

It requires a storage strategy that understands the difference between:

* Temporary information
* Active reasoning state
* Experiences
* Learned knowledge
* Skills
* Relationships
* System history
* Operational data

Storage is not simply where information is placed.

Storage determines how intelligence grows.

The Storage Architecture provides the persistence layer supporting:

* Memory Hierarchy
* Knowledge Graph
* Experience Engine
* Retrieval Pipeline
* Context Lifecycle
* Learning Engine
* Confidence System
* Tool Engine

---

# Design Goals

The Storage Architecture is designed to:

* Separate different types of information
* Prevent memory pollution
* Support lifelong learning
* Enable fast retrieval
* Preserve historical knowledge
* Maintain relationships
* Store confidence and provenance
* Scale from local use to large knowledge systems
* Support Rust-native implementation
* Allow future storage evolution

---

# Design Philosophy

A single database containing everything creates a storage problem.

Different types of knowledge have different lifecycles.

A temporary conversation should not live beside permanent knowledge.

A failed experiment should not overwrite a proven skill.

A tool execution log should not become a memory.

RoBoT uses layered storage.

```text
Temporary Data

        ↓

Working Data

        ↓

Experience

        ↓

Validated Knowledge

        ↓

Strategic Intelligence
```

Information moves upward only when it earns its place.

---

# Storage Architecture Overview

```text
                         RoBoT Storage Layer


                              │

        ┌─────────────────────┼─────────────────────┐

        ▼                     ▼                     ▼

 Working Storage       Knowledge Storage      Operational Storage

        │                     │                     │

        ▼                     ▼                     ▼

 Context State          Memory System          Logs

 Sessions              Knowledge Graph        Metrics

 Tasks                  Skills                Telemetry

 Cache                  Experiences            Events

                              │

                              ▼

                       Learning Consolidation

                              │

                              ▼

                     Long-Term Intelligence
```

---

# Storage Layers

RoBoT separates storage into specialized layers.

---

# 1. Working Storage

## Purpose

Stores temporary information required during active reasoning.

Working Storage is disposable.

Examples:

* Current conversation state
* Active goals
* Planner state
* Tool outputs
* Temporary calculations
* Retrieved context

---

## Characteristics

* Fast access
* Short lifespan
* Frequently updated
* Not considered trusted knowledge

Example:

```text
Current Task:

Fix MCP timeout


Temporary Findings:

Database connection issue

Possible solution:

Move initialization
```

After completion, useful information may be promoted.

Everything else disappears.

---

# 2. Session Storage

## Purpose

Maintains continuity during active interactions.

Stores:

* Conversation history
* Active objectives
* User corrections
* Current project state

Session storage allows RoBoT to pause and resume work.

---

# 3. Experience Storage

## Purpose

Stores what happened.

Experience is different from knowledge.

Knowledge:

"What is true?"

Experience:

"What happened before?"

Examples:

* Successful fixes
* Failed attempts
* Tool executions
* Debugging sessions
* Planning outcomes

---

## Experience Record

```rust
Experience

id

timestamp

goal

context

actions

tools_used

result

success

failure_reason

lessons

confidence
```

---

# 4. Semantic Memory Storage

## Purpose

Stores validated knowledge.

Examples:

* Technical concepts
* Architecture decisions
* Stable facts
* Documentation knowledge

Semantic Memory contains information considered useful beyond one event.

---

# 5. Skill Storage

## Purpose

Stores reusable capabilities.

A skill contains:

```rust
Skill

id

name

description

requirements

steps

dependencies

confidence

success_rate

history
```

---

Example:

```text
Skill:

Create Rust MCP Tool


Requirements:

Rust Async

Protocol Knowledge

Database Access


Confidence:

0.91
```

---

# 6. Knowledge Graph Storage

## Purpose

Stores relationships between information.

The graph connects:

* Memories
* Experiences
* Skills
* Tools
* Concepts
* Dependencies

---

Graph storage contains:

```rust
Node

Relationship

Confidence

Metadata

History

Provenance
```

---

# 7. Archive Storage

## Purpose

Preserves historical information.

Archive contains:

* Old versions
* Deprecated knowledge
* Previous architectures
* Historical decisions

Archive is not deleted memory.

It is historical context.

---

# 8. Operational Storage

Stores system activity.

Examples:

* Logs
* Metrics
* Diagnostics
* Events
* Performance data

Operational data helps improve the system but is not intelligence itself.

---

# Database Philosophy

RoBoT avoids the "one giant table" approach.

Instead, storage is organized around cognitive purpose.

Example:

```text
robot_brain.db


├── memory

├── experiences

├── skills

├── knowledge_nodes

├── relationships

├── confidence

├── sessions

├── events

├── telemetry

└── archive
```

---

# SQLite Foundation

The initial implementation uses SQLite because it provides:

* Local-first operation
* Reliability
* Simple deployment
* Transaction support
* Good Rust integration
* Portable storage

SQLite becomes the foundation layer.

The architecture does not prevent future migration to:

* Distributed databases
* Graph databases
* Vector databases
* Hybrid systems

---

# Hybrid Storage Model

RoBoT uses multiple storage strategies.

---

# Relational Storage

Used for structured information.

Examples:

* Metadata
* Records
* Events
* Configuration
* Relationships

---

# Vector Storage

Used for semantic similarity.

Examples:

* Document embeddings
* Memory embeddings
* Experience similarity

---

# Graph Storage

Used for relationships.

Examples:

* Dependencies
* Associations
* Knowledge paths

---

# Object Storage

Used for large artifacts.

Examples:

* Documents
* Code snapshots
* Audio
* Images
* Models

---

# Storage Flow

```text
New Information

        │

        ▼

Ingestion Layer

        │

        ▼

Classification

        │

        ▼

Temporary Storage

        │

        ▼

Evaluation

        │

        ▼

Promotion Decision

        │

        ├───────────────┐

        ▼               ▼

Memory Storage     Experience Storage

        │

        ▼

Knowledge Graph Update

        │

        ▼

Long-Term Intelligence
```

---

# Memory Promotion

Not everything becomes permanent memory.

Promotion requires evaluation.

Factors:

* Confidence
* Usefulness
* Repetition
* User importance
* Future value

Example:

Temporary observation:

```text
Build failed after dependency update
```

becomes:

Experience:

```text
Dependency updates may require compatibility checks
```

becomes:

Knowledge:

```text
Always verify dependency compatibility after major updates
```

---

# Storage Confidence

Stored information maintains confidence.

Example:

```text
Knowledge:

Cargo requires Rust toolchain


Confidence:

0.98


Evidence:

Documentation

+

Repeated execution
```

Confidence prevents weak information from becoming permanent truth.

---

# Provenance Tracking

Every stored object records origin.

Example:

```text
Source:

User Input

Created:

2026-07-29

Modified:

Learning Engine

Confidence:

0.87
```

This allows explainable memory.

---

# Versioning

Knowledge changes.

Storage preserves evolution.

Versioning tracks:

* Previous values
* Changes
* Reasons
* Confidence changes

Example:

```text
Architecture v0.0.1

        ↓

Architecture v0.0.2
```

---

# Data Lifecycle

Every object follows a lifecycle.

```text
Created

↓

Observed

↓

Evaluated

↓

Stored

↓

Used

↓

Updated

↓

Archived
```

---

# Storage Cleanup

The system performs maintenance.

Operations include:

* Removing duplicates
* Compressing old data
* Merging similar memories
* Archiving unused records
* Rebuilding indexes

Cleanup preserves intelligence while reducing storage growth.

---

# Storage Security

Storage protects:

* Integrity
* Provenance
* Permissions
* Sensitive information
* Tool access

Important operations require validation.

---

# Backup Strategy

RoBoT supports:

## Memory Backup

Stores:

* Knowledge
* Experiences
* Skills

---

## Configuration Backup

Stores:

* Settings
* Models
* Tool definitions

---

## Full Brain Snapshot

Stores:

* Entire cognitive state

Useful for:

* Migration
* Recovery
* Experimentation

---

# Storage and Learning

The Learning Engine uses storage patterns.

It learns:

* Frequently used knowledge
* Important memories
* Weak areas
* Missing relationships
* Storage optimization

Storage itself becomes a learning signal.

---

# Storage and Retrieval

Retrieval depends on storage organization.

Good storage enables:

* Faster search
* Better ranking
* Stronger relationships
* Better context assembly

Poor storage creates cognitive noise.

---

# Storage and Experience

Experience storage is intentionally separate from permanent memory.

This prevents:

* Failed attempts becoming facts
* Temporary solutions becoming rules
* Experiments contaminating knowledge

Experience informs learning.

Memory preserves understanding.

---

# Storage and Knowledge Graph

The graph acts as the connective layer.

Example:

```text
Memory Record

        │

        ▼

Knowledge Node

        │

        ▼

Relationship

        │

        ▼

Skill Dependency
```

---

# Rust Module Layout

```text
src/
└── storage/
    ├── mod.rs
    ├── database.rs
    ├── connection.rs
    ├── migrations.rs
    ├── models.rs
    ├── repository.rs
    ├── memory_store.rs
    ├── experience_store.rs
    ├── skill_store.rs
    ├── graph_store.rs
    ├── session_store.rs
    ├── archive_store.rs
    ├── vector_store.rs
    ├── cache.rs
    ├── backup.rs
    ├── versioning.rs
    └── telemetry.rs
```

---

# Future Evolution

Future versions may include:

* Distributed memory storage
* Neural database indexing
* Autonomous storage optimization
* Cross-device synchronization
* Federated knowledge sharing
* Advanced graph databases
* Hardware accelerated retrieval
* Self-organizing memory structures

---

# Summary

The Storage Architecture provides the foundation that allows RoBoT to maintain intelligence over time.

It separates temporary thoughts from permanent knowledge, experiences from facts, and operational data from cognitive information.

By combining relational storage, vector storage, graph storage, and structured memory layers, RoBoT gains a scalable foundation for lifelong learning.

The goal is not to store everything forever.

The goal is to preserve what matters, understand why it matters, and make it available when intelligence needs it.

This chapter connects the lower-level foundation pieces together:

Storage Architecture → Memory Hierarchy → Knowledge Graph → Retrieval Pipeline → Context Lifecycle → Learning

The main architectural decisions carried forward here are the ones you had been circling around earlier:

Memory and Experience are separate systems
Working Memory is disposable
Permanent knowledge must earn its place
Index-card style memory points toward deeper records
Relationships and confidence are stored, not calculated on the fly
SQLite can be the foundation without locking RoBoT into a single future database choice

This should fit cleanly after Chapter 20 because the Knowledge Graph now has a place to physically live instead of being an abstract intelligence layer floating above the rest of the system.


|==========|==========|==========|==========|        Chapter 22 - Database Design        |==========|==========|==========|==========|

Chapter 22 - Database Design
22.1 Overview

The RoBoT database system is the foundation layer that allows the cognitive architecture to persist, retrieve, organize, and evolve information over time.

Unlike traditional applications where databases primarily store records, RoBoT treats storage as part of its cognitive infrastructure.

The database is not the brain.

It is the long-term memory substrate that allows the brain systems to develop continuity.

The architecture separates different categories of knowledge because not all information has the same purpose, lifespan, or confidence requirements.

RoBoT uses a multi-layer database design:

                    RoBoT Cognitive System

                         |
                         |
                 Storage Architecture

                         |
        ---------------------------------------
        |                 |                   |
 Experience DB      Knowledge DB        Memory DB

        |                 |                   |

 Events            Concepts             Memories
 Outcomes          Skills              Facts
 Failures          Relationships       Context
 Feedback          Graph Data          Embeddings

Each storage layer has a different responsibility.

22.2 Database Philosophy

Traditional AI systems often treat memory as a single searchable archive.

RoBoT does not.

A single memory bucket creates several problems:

information becomes duplicated
outdated knowledge remains active
confidence becomes unclear
retrieval becomes noisy
learning cannot distinguish success from failure

RoBoT instead uses structured cognitive storage.

Information is stored based on:

purpose
reliability
confidence
relationship
usage history
learning value

The database answers:

"What does RoBoT know?"

The Experience System answers:

"What has RoBoT done?"

The Learning System answers:

"What did RoBoT improve from?"

The Confidence System answers:

"How much should RoBoT trust this?"

22.3 Storage Layers

RoBoT storage is divided into five major database domains.

Storage Layer

├── Working Memory
│
├── Experience Database
│
├── Knowledge Database
│
├── Memory Database
│
└── System Database
22.4 Working Memory Storage

Working Memory is temporary cognitive space.

It exists during active reasoning and conversation.

Characteristics:

short lifespan
high change rate
disposable
optimized for speed

Examples:

Current conversation
Active task
Temporary reasoning
Current plan
Recent observations

Working memory should not immediately become permanent memory.

The pipeline is:

Input
 |
 v
Working Memory
 |
 |
Evaluation
 |
 v
Experience / Memory Candidate
 |
 |
Confidence Review
 |
 v
Permanent Storage

This prevents database contamination from every interaction.

22.5 Experience Database

The Experience Database stores what RoBoT has done.

Experience is different from knowledge.

Knowledge:

"Rust uses ownership rules."

Experience:

"I attempted a Rust database migration. The first approach failed because of lifetime issues."

Experience creates improvement.

Experience Tables
experiences

Stores individual events.

CREATE TABLE experiences (
    id TEXT PRIMARY KEY,
    timestamp INTEGER,

    category TEXT,
    action TEXT,

    context TEXT,

    result TEXT,

    success BOOLEAN,

    confidence REAL,

    importance REAL
);
Experience Outcomes

RoBoT records results.

Examples:

Successful:
Implemented SQLite migration
Confidence +0.05


Failed:
Incorrect async architecture
Confidence -0.10

Failure is valuable data.

A system that only stores successes cannot improve.

Experience Relationships

Experiences connect together.

Example:

Experience A

Rust MCP implementation

        |
        |
        v

Experience B

Tokio async debugging

        |
        |
        v

Experience C

Final architecture improvement

Relationships themselves have confidence.

Example:

Rust ownership
        |
        |
confidence: 0.92
        |
        v
Memory safety improvements
22.6 Knowledge Database

The Knowledge Database contains structured understanding.

It represents what RoBoT believes to be true.

Examples:

concepts
skills
procedures
definitions
relationships
Knowledge Cards

RoBoT uses an index-card style architecture.

Each knowledge item is a card.

Example:

Knowledge Card

ID:
rust_async_basics

Type:
Skill

Content:
Rust async uses futures and executors.

Prerequisites:
rust_ownership

Confidence:
0.87

Last Used:
2026-07-28
Knowledge Table
CREATE TABLE knowledge_cards (

    id TEXT PRIMARY KEY,

    type TEXT,

    title TEXT,

    content TEXT,

    confidence REAL,

    created INTEGER,

    updated INTEGER,

    usage_count INTEGER
);
22.7 Skill Database

Skills are stored separately from general knowledge.

A skill represents capability.

Examples:

Skill:

Build Rust MCP Server


Prerequisites:

Rust ownership
Tokio async
SQLite


Confidence:

0.81
Skill Confidence

RoBoT does not use a simple:

I know this

model.

Instead:

Skill Confidence

+
Experience Count

+
Success Rate

+
Recency

+
Difficulty

Example:

Python scripting

Knowledge:
95%

Practical ability:
82%

Recent reliability:
88%
22.8 Relationship Database

Relationships are first-class objects.

A connection between two concepts is also knowledge.

Example:

Rust

    |
    |
requires

    |
    v

Ownership System

The relationship has:

Source

Relationship Type

Target

Confidence

Evidence Count

Database:

CREATE TABLE relationships (

id TEXT PRIMARY KEY,

source_id TEXT,

relation TEXT,

target_id TEXT,

confidence REAL,

evidence_count INTEGER

);
22.9 Memory Database

Memory stores remembered information.

It is different from knowledge.

Memory:

"The user prefers Rust implementations."

Knowledge:

"Rust provides memory safety."

Memory Categories
Memory

├── Episodic Memory
│
├── Semantic Memory
│
├── Preference Memory
│
└── Context Memory
Memory Card

Example:

Memory Card

User prefers:
Rust over Python

Source:
Conversation

Confidence:
0.91

Importance:
High

Last confirmed:
2026-07-20
22.10 Vector Storage Integration

RoBoT uses hybrid retrieval.

Not everything belongs in SQL lookup.

The retrieval architecture:

Query

 |
 |
 +----------------+
 |                |
SQL Search     Vector Search
 |                |
 |                |
 +-------+--------+

         |
         v

Knowledge Fusion

         |
         v

Context Builder

Vector storage provides:

semantic similarity
fuzzy recall
contextual matching

SQL provides:

precision
relationships
metadata
confidence filtering
22.11 Graph Storage

The Knowledge Graph is not separate from the database.

It is a specialized view of stored relationships.

Example:

        Rust

         |
      requires

         |

      Ownership

         |

      Enables

         |

     Memory Safety

The graph allows:

reasoning chains
dependency discovery
skill planning
explanation generation
22.12 System Database

The System Database stores RoBoT operational information.

Examples:

Configuration

Model information

Plugin registry

MCP tools

System state

Migration versions

Example:

CREATE TABLE system_state (

key TEXT PRIMARY KEY,

value TEXT,

updated INTEGER

);
22.13 Database Implementation

Current implementation:

Rust

    |
    |
rusqlite

    |
    |
SQLite

    |
    |
robot_brain.db

Database location:

data/
 |
 └── robot_brain.db

The database layer provides:

SqliteDatabase

    |
    |
Repositories

    |
    |
Cognitive Systems
22.14 Repository Pattern

RoBoT avoids direct database access from cognitive systems.

Incorrect:

Memory Engine
       |
       |
    SQLite

Correct:

Memory Engine

       |

Memory Repository

       |

Database Layer

       |

SQLite

Benefits:

easier migrations
testing
storage replacement
cleaner architecture
22.15 Database Migration Strategy

The database must evolve without losing knowledge.

Migration system:

Version 001

Initial schema


Version 002

Experience improvements


Version 003

Knowledge graph expansion

Every schema change receives:

version number
migration script
rollback strategy
22.16 Future Storage Evolution

SQLite is the foundation, not the final destination.

Future possibilities:

SQLite

  |

Hybrid Database

  |

Distributed Cognitive Storage

Potential future systems:

embedded vector databases
graph databases
distributed memory stores
cloud synchronization
multi-agent shared knowledge

The architecture remains unchanged because storage is abstracted.

22.17 Database Security and Integrity

RoBoT memory is valuable.

Protection includes:

transaction safety
backups
integrity checks
corruption recovery
audit history

Critical cognitive information should never silently disappear.

22.18 Database Flow

Complete storage lifecycle:

Input

 |

Conversation Engine

 |

Working Memory

 |

Experience Evaluation

 |

Learning System

 |

Confidence System

 |

Storage Decision

 |

----------------------------

Memory Database

Knowledge Database

Experience Database

System Database

----------------------------

 |

Future Retrieval

 |

Improved Reasoning
22.19 Design Goals

The RoBoT database system must provide:

Persistence

Knowledge survives restarts.

Organization

Information is separated by purpose.

Learning

Experience improves future behavior.

Confidence

Every belief has measurable reliability.

Explainability

RoBoT can explain why it knows something.

Evolution

The architecture can grow without rebuilding the brain.

22.20 Final Architecture Principle

A database is not merely where RoBoT stores information.

It is where experience becomes memory, memory becomes knowledge, and knowledge becomes capability.

The database provides the roots.

The cognitive engines provide the growth.

Together they create a system that does not simply retrieve information, but develops understanding over time.

End of Chapter 22 - Database Design

|==========|==========|==========|==========|      Chapter 23 - Background Workers       |==========|==========|==========|==========|

