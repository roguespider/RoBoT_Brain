================================================================================
# RoBoT_Brain

A Rust MCP (Model Context Protocol) server for Zed Editor — an AI agent with persistent memory, experience-based learning, and structured knowledge storage.

> **Status:** v0.7 complete — Memory System implemented per Architecture §4.08, §6.3 with Working Memory, Permanent Memory, and Memory Retrieval. Full event catalog per Architecture §4.04. Learning Pipeline per Architecture §9. Database layer with 8 migrations.

---

## Objective

**Problem:** Previous Python MCP memory suffered from storage bloat and slow ingestion due to embedding everything and eager graph extraction.

**Goal:** Redesign with selective storage, deferred processing, strict module boundaries, and a phased build that prioritizes reliability over premature intelligence.

### Core Principles

| Principle                   | Implementation                                                                                                      |
|-----------------------------|---------------------------------------------------------------------------------------------------------------------|
| Selective Embedding         | Score content on ingestion. Only embed high-value architectural decisions, not logs/temp data/repeated discussions  |
| Confidence + Exploration    | Track confidence ± range and exploration_value. Balance proven reliability vs. controlled experimentation           |
| Council Architecture        | No system overrides another. Modules advise via strict interfaces. Disagreements logged for policy tuning           |
| Event-Based Decisions       | Only record decisions for meaningful events (new workflows, failures, explorations). Avoid noise from trivial calls |
| Reflexes Before Imagination | Build execution/recording loop first. Defer LLM, graph, and learning modules until core is stable                   |

---

## Architecture

```
                    +----------------+                     +----------------+
                    |   RoBoT Brain  >-------------------> |   Zed Editor   |
                    +--------+-------+                     +--------v-------+
                             |          +--------+-------+          |
                             |<---------<   MCP Server   <----------+
                             |          +--------+-------+
              +--------------+--------------+
              |                             |
      +-------v--------+          +---------v---------+
      | Memory Core    |          | Experience System |
      +-------+--------+          +---------+---------+
              |                             |
    +---------+----------+          +-------+-------+
    |                    |          |                 |
    |  Memories          |          |  Recorder       |
    |  (content, types)  |          |  Coordinator    |
    |                    |          |  Pipeline       |
    |  Decisions         |          |                 |
    |  (workflow choices)|          |  Observers:     |
    |                    |          |  - Hypothesis   |
    |  Memory Sources    |          |  - Exploration  |
    |  (origin tracking) |          |  - Reflection   |
    |                    |          |  - Evolution    |
    |  Relationships     |          |                 |
    |  (graph links)     |          |  Scorer         |
    |                    |          |  Reputation     |
    |  Events            |          |                 |
    |  (timeline)        |          +-----------------+
    |  Reputations       |          |
    +--------------------+          +--------+----------+
              |                             |
              +-------------+---------------+
                            |
                    +-------v--------+
                    |    SQLite      |
                    |  Single Source |
                    |     of Truth   |
                    +----------------+
```


### Memory Layers (Per Architecture §6.3)

| Layer | Purpose | Size | Status |
|-------|---------|------|--------|
| **Working Memory** | Active context with LRU eviction, TTL, promotion policies | In-memory | ✅ Implemented |
| **Permanent Memory** | Indexed, connected, confidence weighted storage | In-memory + SQLite | ✅ Implemented |
| **Memory Retrieval** | Unified retrieval across memory layers with relevance scoring | Unified API | ✅ Implemented |
| **Index Card** (Short-term) | Lightweight metadata: ID, Title, Summary, Keywords, Pointer | ~200-500 bytes/card | ✅ Implemented (in-memory) |
| **Flat Memory** (Raw Chunks) | Original document chunks in SQLite. Only high-scoring chunks receive embeddings | Variable | ⏳ Deferred |
| **Graph Memory** | Stores relationships/facts only, never prose. Extracted async in background | Variable | ✅ Implemented (schema + tables) |
| **Long-term Memory** | Promoted memories with full lineage tracking | Persistent | ✅ Implemented (lineage) |

### Experience Compression

The Experience Compression system reduces memory overhead by detecting patterns across similar experiences and compressing them into efficient representations.

```
┌─────────────────────────────────────────────────────────────┐
│                    Experience Compression                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│   │ Experience 1│    │ Experience 2│    │ Experience 3│   │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘   │
│          │                   │                   │          │
│          └───────────────────┼───────────────────┘          │
│                              │                              │
│                    ┌─────────▼─────────┐                    │
│                    │ Pattern Detector  │                    │
│                    │  - Common tags   │                    │
│                    │  - Keywords      │                    │
│                    │  - Success rate  │                    │
│                    └─────────┬─────────┘                    │
│                              │                              │
│          ┌───────────────────┼───────────────────┐         │
│          │                   │                   │         │
│   ┌──────▼──────┐    ┌───────▼───────┐   ┌──────▼──────┐  │
│   │   Pattern   │    │  Compressed   │   │  Exception  │  │
│   │ (common     │    │  Experience   │   │  Tracker    │  │
│   │  elements)  │    │ (aggregated   │   │  (deviations│  │
│   └─────────────┘    │  confidence)  │   └─────────────┘  │
│                      └───────────────┘                     │
└─────────────────────────────────────────────────────────────┘
```

#### Components

| Component | File | Description |
|-----------|------|-------------|
| `ExperienceCompressor` | `compression/compressor.rs` | Main compressor for reducing similar experiences |
| `PatternDetector` | `compression/pattern.rs` | Finds common elements across experiences |
| `ExceptionTracker` | `compression/exceptions.rs` | Tracks deviations from patterns |

#### Compression Algorithm

1. **Collection**: Gather 3+ similar experiences
2. **Pattern Detection**: Extract common tags, keywords, and actions
3. **Confidence Calculation**: Aggregate confidence statistics (mean ± std)
4. **Exception Detection**: Identify experiences that deviate from the pattern
5. **Result**: Return `CompressionResult` with pattern, aggregated stats, and exceptions

#### Usage Example

```rust
use crate::experience::compression::{ExperienceCompressor, PatternDetector};

// Create compressor with custom settings
let compressor = ExperienceCompressor::with_config(
    min_experiences: 3,
    similarity_threshold: 0.7
);

// Compress multiple experiences
if let Some(result) = compressor.compress(&experiences) {
    println!("Compressed {} experiences into pattern: {}", 
             result.experience_count, 
             result.pattern.action);
    println!("Aggregated confidence: {:.2} ± {:.2}", 
             result.confidence, 
             result.confidence_range);
}
```

#### Pattern Detection

```rust
let detector = PatternDetector::new();
if let Some(pattern) = detector.detect_pattern(&experiences) {
    // Access common elements
    println!("Action: {}", pattern.action);
    println!("Tags: {:?}", pattern.common_tags);
    println!("Keywords: {:?}", pattern.keywords);
    println!("Success rate: {:.1}%", pattern.success_rate * 100.0);
}
```

#### Exception Tracking

```rust
let mut tracker = ExceptionTracker::new();

// Add exceptions when experiences deviate from patterns
let exception = Exception::new(
    experience_id,
    pattern_id,
    0.5, // deviation score
    "Unexpected outcome".to_string()
);
tracker.add_exception(exception);

// Query exceptions
let significant = tracker.get_significant(0.3);
let by_type = tracker.get_by_type(DeviationType::DifferentOutcome);
```

### Data Flow

1. **MCP Tools** receive requests from Zed Editor
2. **Experience System** records every action through the learning pipeline
3. **Memory Core** persists structured knowledge in SQLite
4. **Migration System** manages schema evolution automatically

---

## Database Schema

The database (`robot_brain.db`) is created automatically on first run via `SqliteStore::open()` using OS data directory resolution (`dirs` crate).

### Implemented Tables

| Table | Purpose | Created By |
|-------|---------|------------|
| `memories` | Core memory storage (content, type, confidence, importance) | Migration 0→1 |
| `decisions` | Records why workflows were chosen, alternatives considered, outcomes | Migration 1→2 |
| `memory_sources` | Tracks where each memory came from (chat, file import, user input, etc.) | Migration 2→3 |
| `relationships` | Graph connections between memories (source, target, type, strength) | `sqlite::initialize()` directly |
| `events` | Event timeline (what happened, when, what it relates to) | Migration 3→4 |
| `reputations` | Long-term reputation tracking per target | Migration 4→5 |
| `scheduled_tasks` | Persistent background task scheduling | Migration 5→6 |
| `memory_lineage` | Full history and evolution tracking for memories | Migration 6→7 |
| `lineage_evidence` | Supporting evidence references for memories | Migration 6→7 |
| `lineage_observations` | Observation records related to memories | Migration 6→7 |
| `lineage_refinements` | Content change history for memories | Migration 6→7 |
| `lineage_contradictions` | Contradiction challenges to memories | Migration 6→7 |
| `lineage_confirmations` | External confirmations for memories | Migration 6→7 |

> **Note:** The `relationships` table is created directly in `sqlite::initialize()` via raw SQL and has no corresponding migration. If the DB is re-created from scratch it works, but on upgrade from an old database that skipped init, it won't exist until a migration path handles it.

| Model | Maps To |
|-------|---------|
| `MemoryCard` | `memories` table |
| `MemorySource` | `memory_sources` table |
| `Relationship` | `relationships` table |
| `DecisionRecord` | `decisions` table |
| `MemoryEvent` | `events` table |
| `ReputationRecord` | `reputations` table |

### Query Functions (src/database/queries.rs)

| Function | Operation |
|----------|-----------|
| `insert_memory()` | INSERT OR REPLACE into memories |
| `get_memory()` | SELECT by ID, returns Option<MemoryCard> |
| `search_memory()` | LIKE search across content, limit 100 |
| `insert_decision()` | INSERT into decisions (alternatives serialized as JSON) |
| `insert_source()` | INSERT into memory_sources |
| `insert_relationship()` | INSERT into relationships |
| `record_event()` | INSERT into events |

### Migration History

| Version | Changes |
|---------|---------|
| 0 → 1 | Core memory (`memories` table) |
| 1 → 2 | Decision memory (`decisions` table) |
| 2 → 3 | Source tracking (`memory_sources` table) |
| 3 → 4 | Event history (`events` table) |
| 4 → 5 | Reputation tracking (`reputations` table) |
| 5 → 6 | Scheduled tasks persistence (`scheduled_tasks` table) |
| 6 → 7 | Memory lineage tracking (lineage tables) |

### Policy Engine Config (planned)

Behavior tuning is intended to be externalized via TOML config — no implementation yet:

```toml
[policy]
experience_first = true
minimum_confidence = 30
exploration_rate = 25
avoid_high_cost_failures = true
```

### Memory Types

| Type | Description |
|------|-------------|
| `note` | General notes and observations |
| `fact` | Discrete facts (user preferences, settings) |
| `task` | Task records and their outcomes |
| `file` | File-related memories (imports, changes) |
| `conversation` | Dialogue snippets |
| `code` | Code snippets and patterns |
| `decision` | Decision records |
| `event` | System events |
| `encounter` | Recorded encounters from interactions |
| `experience` | Full experience records with context |

---

## Experience System

The experience system tracks every action the agent takes, enabling learning over time. Modules communicate via typed structs passed through method calls (not yet event-driven — that's planned).

### Current Components

| File | Component | Status |
|------|-----------|--------|
| `experience/types.rs` | `Experience`, `ExperienceType`, `ExperienceScore`, `ReputationRecord`, `OutcomeKind`, etc. | ✅ Implemented |
| `experience/events.rs` | `ExperienceEvent` enum + `EventPayload` enum | ✅ Implemented |
| `experience/observer.rs` | `ExperienceObserver` trait (name, accepts, observe, priority) | ✅ Implemented |
| `experience/recorder.rs` | `ExperienceRecorder::record()` — inserts into DB via `ExperienceQueries` | ⚠️ Partial (see below) |
| `experience/bus.rs` | Publish/subscribe routing for events | ❌ Stub (`bus.publish(experience_id)` only) |
| `experience/queue.rs` | In-memory job queue with HashMap-backed push/pop/complete/fail | ✅ Implemented |
| `experience/worker.rs` | Spawns async worker per observer, processes jobs from channel receiver | ✅ Implemented |
| `experience/coordinator.rs` | Orchestrates full pipeline: recorder → scorer → reputation → hypothesis/exploration/reflection/evolution | ⚠️ Partial (imports resolved, but reflection/evolution stubbed) |

### Pipeline Design

```
Experience Recorded
        |
        v
    Recorder (insert_experience)
        |
        v
    Bus → Job Queue
        |
        v
    Notify Observers:
    ├── Hypothesis Engine  ✅
    ├── Exploration Engine  ✅
    ├── Reflection Engine   ⚠️ Stubbed
    └── Evolution Engine    ⚠️ Stubbed
```

### Key Types

| Component | Location | Description |
|-----------|----------|-------------|
| `Experience` | types.rs | A recorded action with context, outcome, and score |
| `ExperienceType` | types.rs | ToolExecution, MemoryLookup, Workflow, Planning, Exploration, etc. (15 variants) |
| `ExperienceScore` | types.rs | Multi-dimensional: importance, confidence, novelty, reliability |
| `ReputationRecord` | types.rs | Long-term reliability tracking per target (score, successes, failures) |
| `ExperienceObserver` | observer.rs | Trait for learning subsystems to react to events |
| `EventPayload` | events.rs | Recorded, ScoreCalculated, ReputationUpdated, ReflectionCompleted, HypothesisGenerated, ExplorationCompleted |
| `Exploration` | exploration/exploration.rs | Core exploration entity tracking state and results |
| `ExplorationStatus` | exploration/exploration.rs | Enum: pending, running, completed, failed |
| `Hypothesis` | exploration/hypothesis.rs | Struct representing a testable hypothesis |
| `HypothesisResult` | exploration/hypothesis.rs | Enum: supported, refuted, inconclusive |
| `ExplorationAttempt` | exploration/attempt.rs | Struct tracking individual experiment attempts |
| `ExplorationFinding` | exploration/finding.rs | Struct capturing results and insights from an exploration |

### Implemented Sub-Modules

All previously-planned sub-modules now exist as files:

| Module | Location | Purpose |
|--------|----------|---------|
| `scorer` | `experience/scorer.rs` | Score experiences on importance/confidence/novelty/reliability |
| `reputation` | `experience/reputation/` | Update long-term reputation for tools/workflows/models |
| `hypothesis` | `experience/hypothesis/` | Generate and track hypotheses from observations |
| `exploration` | `experience/exploration/` | Test new candidates via controlled experimentation |
| `reflection` | ⚠️ Stubbed | Analyze past experiences for patterns and improvements |
| `evolution` | ⚠️ Stubbed | Adapt behavior based on accumulated experience |

### Key Interfaces

#### Experience Observer Trait

```rust
pub trait ExperienceObserver: Send + Sync {
    fn name(&self) -> &'static str;       // Human-readable identifier
    fn start(&self) -> Result<()>;         // Initialization hook
    fn shutdown(&self) -> Result<()>;      // Cleanup hook
    fn accepts(&self, event: &ExperienceEvent) -> bool;  // Default: accept all
    fn priority(&self) -> u8;              // Lower = runs first (default: 100)
    fn observe(&self, event: &ExperienceEvent) -> Result<()>;  // Core logic
}
```

---

## Project Structure

```
robot/
src/
├── main.rs                     ✅
├── database\                   ✅
│   ├── sqlite.rs               ✅← connection + initialization
│   ├── models.rs               ✅← database structs
│   ├── migrations/             ✅← schema migrations module
│   │   └── mod.rs              ✅← migration functions
│   └── queries.rs              ✅← CRUD operations
├── experience\                 ⚠️
│   ├── mod.rs                  ✅←                                    ├─ xp backbone
│   ├── types.rs                ✅← → experience data structures       ├─ xp backbone
│   ├── observer.rs             ✅← → observer contract                ├─ xp backbone
│   └── events.rs               ✅← → ExperienceEvent + EventPayload   ├─ xp backbone
│   ├── events\                 ✅← →                                  ├─ xp backbone
│   │   ├── mod.rs              ✅← →                                  ├─ xp backbone
│   │   ├── event.rs            ✅← → ExperienceEvent                  ├─ xp backbone
│   │   └── payload.rs          ✅← →EventPayload enum                 ├─ xp backbone
│   ├── bus.rs                  ✅← → publish/subscribe routing        ├─ xp backbone
│   ├── queue.rs                ✅← → queued work + retry/recovery     ├─ xp backbone
│   ├── worker.rs               ✅← → executes queued observer work    ├─ xp backbone
│   ├── coordinator.rs          ✅← → owns the whole lifecycle         ├─ xp backbone
│   ├── recorder.rs             ✅← entry point for writes experiences
│   ├── scorer.rs               ✅←
│   └── reputation.rs           ✅←
│   ├── reputation/             ✅←
│   │   ├── mod.rs	            ✅← Exposes the reputation subsystem
│   │   ├── reputation.rs       ✅← Core reputation state and updates
│   │   ├── factors.rs	        ✅← Different trust dimensions
│   │   ├── decay.rs	          ✅← Time-based reputation aging
│   │   ├── analytics.rs        ✅← Reports, trends, statistics
│   │   └── repository.rs       ✅← Save/load reputation data
│   ├── working_memory/         ✅ Working memory with state machine
│   │   ├── mod.rs              ✅ Module entry point
│   │   ├── working_memory.rs   ✅ Working memory implementation
│   │   ├── memory_state.rs     ✅ State machine definitions
│   │   └── promotion.rs        ✅ Promotion policy engine
│   ├── lineage.rs              ✅ Memory lineage tracking
│   ├── candidates.rs           ✅ Candidate memory generation
│   ├── exploration/            ✅
│   │   ├── mod.rs              ✅
│   │   ├── exploration.rs      ✅
│   │   ├── hypothesis.rs       ✅
│   │   ├── attempt.rs          ✅
│   │   ├── finding.rs          ✅
│   │   └── store.rs            ✅
│   ├── hypothesis/             ✅
│   │    ├── mod.rs             ✅ Hypothesis engine entry point (moved from hypothesis.rs)
│   │    ├── core/              ✅
│   │    │   ├── mod.rs         ✅ Define what hypothesis is
│   │    │   ├── hypothesis.rs  ✅ Core data structures (Hypothesis + HypothesisId)
│   │    │   ├── evidence.rs    ✅ Evidence models
│   │    │   ├── evaluator.rs   ✅ Confidence updates and evaluation logic
│   │    │   └── lifecycle.rs   ✅ State transitions
│   │    ├── services/          ✅
│   │    │   ├── mod.rs         ✅
│   │    │   ├── repository.rs  ✅ Storage interface similar to Experience/Reputation
│   │    │   ├── analytics.rs   ✅ Statistics and trend reporting
│   │    │   ├── generator.rs   ✅ Basic pattern detection and generation
│   │    │   ├── matcher.rs     ✅ Bridge between experiences and beliefs
│   │    │   └── validator.rs   ✅ Contradiction checks and validation
│   │    └── support/           ✅
│   │         ├── mod.rs        ✅ Support module root
│   │         ├── statistics.rs ✅ Mostly counters and summaries
│   │         ├── graph.rs      ✅ Full hypothesis graph with cycle detection, path finding, SCC
│   │         ├── simulation.rs ✅ What-if reasoning system with outcome simulation
│   │         └── planner.rs    ✅ Decision-support layer converting hypotheses to actions
│   ├── reflection/             ✅
│   │   ├── mod.rs              ✅ Reflection module root
│   │   ├── reflection.rs       ✅ Core Reflection struct and methods
│   │   ├── insight.rs          ✅ Insight types for reusable knowledge
│   │   ├── pattern.rs          ✅ Pattern detection and management
│   │   ├── review.rs           ✅ Reflection review types
│   │   └── services/           ✅
│   │       ├── mod.rs          ✅ Services module
│   │       ├── analyzer.rs     ✅ ReflectionAnalyzer for analyzing experiences
│   │       ├── generator.rs    ✅ ReflectionGenerator for creating reflections
│   │       ├── repository.rs   ✅ Thread-safe in-memory reflection repository
│   │       └── validator.rs    ✅ ReflectionValidator for quality checks
│   ├── evolution/              ✅
│   │   ├── mod.rs              ✅ Evolution module root
│   │   ├── behavior.rs         ✅ Behavior struct and lifecycle management
│   │   ├── evidence.rs         ✅ Evolution evidence types
│   │   └── engine.rs           ✅ Evolution engine for behavior management
│   ├── compression/            ✅ Experience pattern compression
│   │   ├── mod.rs              ✅ Compression module root
│   │   ├── compressor.rs       ✅ Core compression algorithm
│   │   ├── pattern.rs         ✅ Pattern detection
│   │   └── exceptions.rs       ✅ Exception tracking
│   ├── metrics.rs              ✅ Metrics collection with counters, gauges, aggregation
│   ├── scheduler.rs            ✅ Background task scheduler with interval/daily/weekly schedules
├── planner/                    ✅
│   ├── planner.rs              ✅ Core planning engine for task decomposition
│   └── policy.rs               ✅ Policy engine for decision-making rules
├── skills/                     ✅
│   └── registry.rs             ✅ Skill registry with discovery and execution
├── workflows/                  ✅
│   └── engine.rs               ✅ Workflow execution engine
├── tools/                      ✅
│   ├── mod.rs                  ✅ Tools module root
│   ├── memory.rs               ✅ Memory tools (store, search, get, list)
│   ├── experience.rs           ✅ Experience tools
│   ├── reflection.rs           ✅ Reflection tools
│   ├── search.rs               ✅ Search tools
│   └── ingestor.rs             ✅ File ingestion tools (import, delete with confirmation)
├── learning/                   ✅
│   ├── working_memory.rs       ✅ Short-term memory management
│   ├── hypothesis.rs           ✅ Hypothesis tracking and evaluation
│   └── candidates.rs           ✅ Learning candidate generation
└── cli/                        ✅
    ├── mod.rs                  ✅ CLI module root
    ├── commands/               ✅ CLI commands
    │   ├── server.rs           ✅ Start MCP server
    │   ├── init.rs             ✅ Initialize database
    │   ├── status.rs           ✅ Check system status
    │   ├── memory.rs           ✅ Memory management
    │   ├── experience.rs       ✅ Experience statistics
    │   ├── config.rs           ✅ Show configuration
    │   └── migrate.rs          ✅ Run migrations
    └── output.rs               ✅ Formatted output utilities
```

**Legend:** ✅ Implemented | ⚠️ Stubbed/partial | ❌ Placeholder code only | 🟡 Partially done | 📋 Planned but not started

--- 
Upgrades to add

3. Confidence Graph

One thing we've discussed but haven't fully designed:

Don't score only nodes.

Score relationships.

Rust
 95%

SQLite
 90%

Rust ───── SQLite
        42%

The relationship confidence becomes its own entity.

That allows planner reasoning like

"I know Rust."

"I know SQLite."

"But I have little experience combining them."

4. Event Sourcing

Instead of modifying structures directly...

everything becomes an event.

MemoryCreated

MemoryUpdated

ExperienceRecorded

ExperienceMerged

HypothesisCreated

KnowledgeValidated

SkillImproved

Current state becomes

fold(events)

Advantages:

complete history
debugging
replay
rollback
explainability

It also fits the architecture you've been building around the Experience Engine.

5. Capability System

Instead of tools...

think capabilities.

Observe

Recall

Compare

Predict

Infer

Plan

Execute

Reflect

Teach

Planner requests capabilities.

Capabilities use tools.

Much cleaner dependency direction.

6. Skill Evolution

Instead of

Skill

store

Skill
├── prerequisites
├── confidence
├── decay
├── reinforcement
├── evidence
└── last successful use

Now skills become alive instead of static.

7. Experience Compression

This is one of my favorite additions.

Instead of keeping

100 nearly identical experiences

compress them into

Pattern

Confidence

Exceptions

Exactly what humans do.

8. Hypothesis Engine ✅ **IMPLEMENTED**

The Hypothesis Engine makes RoBoT capable of learning rather than merely remembering.

```
Observation → Hypothesis → Test (Evidence) → Evaluation → Knowledge
                    ↓
              Supported | Refuted | Inconclusive | Superseded
```

**Learning Flow:**
1. **Observation** - Record successes, failures, patterns, anomalies
2. **Hypothesis** - Form testable statements from observations
3. **Test** - Add supporting or contradicting evidence
4. **Evidence** - Accumulate proof for or against hypothesis
5. **Evaluation** - Calculate status based on evidence ratio
6. **Knowledge** - Extract validated hypotheses into reusable knowledge

**MCP Tools (9):**
| Tool | Description |
|------|-------------|
| `record_observation` | Record successes, failures, patterns, anomalies |
| `list_observations` | View recorded observations |
| `create_hypothesis` | Form testable hypothesis from observations |
| `get_hypothesis` | View hypothesis with all evidence |
| `list_hypotheses` | List hypotheses (filter by domain/status) |
| `add_evidence` | Add supporting or contradicting evidence |
| `evaluate_hypothesis` | Evaluate based on evidence, update status |
| `get_knowledge` | Get extracted learned knowledge |
| `extract_knowledge` | Convert supported hypothesis → reusable knowledge |

**Database Tables (Migration 008):**
- `hypotheses` - Testable hypotheses with status and confidence
- `observations` - Raw observations that trigger learning
- `evidence` - Supporting/contradicting evidence for hypotheses
- `learned_knowledge` - Extracted knowledge from validated hypotheses

**Status Evaluation Rules:**
- 3+ evidence required to evaluate
- Supported: supporting > contradicting × 2
- Refuted: contradicting > supporting × 2
- Inconclusive: otherwise
- Knowledge extraction only from Supported hypotheses

9. Planner Feedback Loop

Instead of

Plan

Execute

Done

make it

Goal
↓
Planner
↓
Action
↓
Outcome
↓
Experience
↓
Knowledge
↓
Improved Planner

Now every task makes the planner smarter.

10. Reflection Engine

Probably the biggest architectural upgrade.

Every N experiences
Reflect
↓
Find patterns
↓
Merge memories
↓
Retire obsolete facts
↓
Create new hypotheses
↓
Adjust confidence

This is remarkably similar to sleep consolidation in biological memory.

One thing I'd change from our earlier discussions

Originally we leaned toward:

Memory

Experience

Learning

After thinking through your architecture more, I'd separate them further:

Observation Layer
↓
Working Memory
↓
Experience Engine
↓
Reflection Engine
↓
Knowledge Graph
↓
Planning
↓
Execution

That keeps every subsystem responsible for exactly one transformation. It also makes testing easier because each layer has a single job.

What I think is the single biggest missing piece

If I could add one subsystem to RoBoT_mcp, it would be the Reflection Engine.

Most AI memory systems stop at:

"Store memory. Retrieve memory."

Your architecture is already aiming higher. A Reflection Engine turns accumulated experiences into refined
knowledge, updates confidence, discovers patterns, and retires stale information. That closes the learning 
loop and makes the system improve over time rather than simply grow larger.
---
speech engines upgrade
F5-TTS and whisper-rs (quantized to 4-bit) for STT

 code architecture needed to load a local .wav file, convert it to raw PCM data, and pass it directly to
 an F5-TTS ONNX model instance within your Rust application:
 1. Configure the Cargo.toml
 You need a WAV decoder (hound) and the ONNX model pipeline (ort with an ndarray mathematical backend):
 toml[dependencies]
 ort = { version = "2.0", features = ["load-dynamic"] }
 ndarray = "0.15"
 hound = "3.5"
 ---
 Core Rust Processing Scriptrustuse ort::{Session, SessionParameters, Value};
 use ndarray::{Array1, Array2};
 use std::path::Path;
 
 pub struct F5VoiceCloner {
     onnx_session: Session,
 }
 
 impl F5VoiceCloner {
     pub fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
         // Initialize the ONNX session optimized to strictly use CPU cores
         let session = Session::builder()?
             .commit_from_file(model_path)?;
         Ok(Self { onnx_session: session })
     }
 
     pub fn clone_voice_from_wav(
         &self, 
         wav_path: &str, 
         ref_text_tokens: Vec<i64>,  // Int tokens matching what is said in the WAV
         target_text_tokens: Vec<i64> // Int tokens for the new phrase
     ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
         
         // 1. Open the custom WAV file and decode its audio samples
         let mut reader = hound::WavReader::open(wav_path)?;
         let spec = reader.spec();
 
         // F5-TTS natively expects 24,000Hz mono audio data
         if spec.sample_rate != 24000 || spec.channels != 1 {
             return Err("Reference WAV must be exactly 24kHz Mono!".into());
         }
 
         // Convert the raw 16-bit sound waves into a normalized f32 vector array
         let raw_samples: Vec<f32> = reader
             .into_samples::<i16>()
             .map(|s| s.unwrap() as f32 / 32768.0) 
             .collect();
 
         // 2. Shape the reference audio into a 2D matrix shape for ONNX (1, sample_count)
         let sample_count = raw_samples.len();
         let audio_matrix = Array2::from_shape_vec((1, sample_count), raw_samples)?;
 
         // 3. Shape the text arrays into standard 2D token matrices
         let ref_text_matrix = Array2::from_shape_vec((1, ref_text_tokens.len()), ref_text_tokens)?;
         let target_text_matrix = Array2::from_shape_vec((1, target_text_tokens.len()), target_text_tokens)?;
 
         // 4. Pass all data directly into the F5-TTS model session inputs
         let inputs = ort::inputs![
             "ref_audio" => audio_matrix,
             "ref_text" => ref_text_matrix,
             "target_text" => target_text_matrix,
         ()?;
 
         // 5. Execute the generation process natively on the CPU
         let outputs = self.onnx_session.run(inputs)?;
         
         // Extract the newly generated audio array
         let output_tensor = outputs["output_audio"].try_extract_tensor::<f32>()?;
         let generated_speech_raw = output_tensor.view().to_owned().into_raw_vec();
 
         Ok(generated_speech_raw)
     }
 }
 ---

tools\
     ├──interaction\
     |  ├──audio <-- pass wav file directly into your execution pipeline for tts
     |  ├──chat --> output tts and print text to desktop ui
     |  ├──clipboard
     |  ├──documents
     |  ├──dragdrop
     |  ├──notifications
     |  ├──shortcuts
        
┌─────────────────────────────┐
│      Desktop UI (Rust)      │
│                             │
│ 🎤 Start Listening          │
│ 📄 Drop Files Here          │
│ 💬 Conversation             │
│ 🧠 Agent Thoughts           │
└──────────────┬──────────────┘
               │
               ▼
        RoBoT MCP Core
               │
               RoBoT Desktop (Rust)
                       │
                       ▼
               Interaction Layer
                       │
                ┌──────┴─────────┐
                ▼                ▼
               whisper-rs      F5-TTS
               (STT)            (TTS)
                       │
                       ▼
               Experience Engine
                       │
                       ▼
               Planner
                       │
                       ▼
               Memory System
               
When idle, it collapse's into a tiny floating microphone button. speak, and watch the transcript appear.
separate what the user says 'text in blue' from the agent's internal reasoning 'text in white' and what agent says 'text in lime green'.

Drop anything onto the window:
PDF
TXT
Markdown
DOCX
Images
Audio
Video
ZIP
Rust source
Entire folders
sent to ingestor which adds it to short term memory for agent usage. simply hands them to the ingestion pipeline, which routes each file to the appropriate processor.

 F5-TTS and whisper-rs (quantized to 4-bit) for STT




an Interaction Layer as a peer to your Experience and Memory systems:
Interaction
├── Voice
├── Chat
├── Documents
├── Clipboard
├── Screen (future)
├── Notifications
└── Commands

--------------------------------------------------------------------------------

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| MCP Protocol | `rmcp` v2 | Model Context Protocol server (stdio transport) |
| Runtime | `tokio` v1 | Async runtime (`#[tokio::main]`) |
| Database | `rusqlite` v0.32 | Embedded SQLite with `load_extension` feature |
| Serialization | `serde` + `serde_json` | Data serialization (Experience, EventPayload, etc.) |
| Identity | `uuid` v1 | Unique IDs (v4) for memories and experiences |
| Time | `chrono` v0.4 | Timestamps (RFC3339) |
| File walking | `walkdir` v2 | Directory traversal for file ingestion |
| Compression | `zip` v2, `tar` v0.4, `flate2` v1 | Archive handling (zip, tar, gz) |
| Hashing | `sha2` v0.10 | File content hashing |
| Paths | `dirs` v5 | OS data directory resolution |
| Error handling | `anyhow` v1 | Result propagation throughout |

---

## Getting Started

### Prerequisites

- Rust 2024 edition (per `Cargo.toml`)
- SQLite3 development libraries (for `rusqlite`)

started ### CLI Usage

```bash
# Start the MCP server (default)
cargo run

# Run CLI commands
cargo run -- init           # Initialize database
cargo run -- status         # Check system status
cargo run -- memory list    # List memories
cargo run -- memory search <query>  # Search memories
cargo run -- memory add <content>   # Add a memory
cargo run -- memory stats    # Show memory statistics
cargo run -- experience      # Show experience statistics
cargo run -- config          # Show configuration
cargo run -- migrate         # Run database migrations
```

---

## File Ingestion (Ingestor Tools)

The ingestor tools allow you to import files from a `files_to_import/` folder into short-term memory. Files are automatically chunked and stored as memory cards.

### Supported File Formats

| Format | Extensions | Processing |
|--------|------------|------------|
| Archives | `.zip`, `.tar`, `.tar.gz`, `.tgz`, `.gz` | Extracted recursively |
| Text | `.txt`, `.md`, `.rst`, `.csv`, `.log`, `.xml`, `.html` | Direct ingestion |
| JSON | `.json`, `.jsonl` | Pretty-printed for search |
| PDF | `.pdf` | Basic text extraction |
| Audio | `.mp3`, `.wav`, `.m4a`, `.flac`, `.ogg`, `.aac` | Placeholder for transcription |

### MCP Tools

#### `ingest_files`
Import files from `files_to_import/` folder into short-term memory.

```json
{
  "folder": "files_to_import",
  "chunk_size": 1000,
  "memory_type": "file"
}
```

**Response includes:**
- `summary`: Ingestion statistics (total, successful, failed, chunks)
- `successfully_ingested`: Array of file paths that were imported
- `user_action_required`: Prompt to confirm deletion

#### `list_importable`
List files ready for import in the folder.

```json
{
  "folder": "files_to_import"
}
```

#### `list_ingested_files`
List files that have been successfully ingested and can be deleted.

```json
{
  "folder": "files_to_import",
  "limit": 100
}
```

#### `delete_ingested_files`
**Requires confirmation** - Delete files after successful ingestion.

```json
{
  "files": ["path/to/file1.txt", "path/to/file2.pdf"],
  "confirmation": "yes"
}
```

**Safety:** Without `confirmation: "yes"`, the tool runs in simulation mode showing what would be deleted.

### Workflow

```
1. Place files in ./files_to_import/

2. Call ingest_files → Files are chunked and stored in memory
   └─ Response: List of successfully imported file paths

3. Review the imported files

4. Call delete_ingested_files with confirmation to remove originals
   └─ confirmation: "yes" → Actually deletes
   └─ confirmation: anything else → Shows simulation only
```

### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `chunk_size` | 1000 | Characters per chunk |
| `chunk_overlap` | 100 | Overlap between chunks |
| `memory_type` | "file" | Type for ingested memories |

### Build

```bash
# Development build
cargo build

# Release build (recommended for production)
cargo build --release
```

> **Note:** The project uses the system SQLite3 library. The database (`robot_brain.db`) is created automatically on first run.

---

## Current Status & Gaps

| Area | Status | Details |
|------|--------|---------|
| Database layer | ✅ Functional | Schema + 8 migrations (v0→v8 via `migrations/` module), CRUD queries all implemented |
| Memory System | ✅ Complete | Working Memory, Permanent Memory, Memory Retrieval per Architecture §6.3 |
| Event System | ✅ Complete | Full event catalog per Architecture §4.04 (30+ event types) |
| Learning Pipeline | ✅ Implemented | Input→Observation→Memory→Experience→Knowledge→Planning→Decision→Action→Reflection |
| Experience types/events | ✅ Complete | Full type system for experiences, scores, reputation, event payloads |
| Observer pattern | ✅ Implemented | Trait defined with priority and filter hooks |
| Job queue + worker | ✅ Implemented | In-memory queue with async worker (mpsc channel) |
| Event bus | ✅ Implemented | Full pub/sub with broadcast channel, subscriber tracking |
| Experience coordinator | ✅ Implemented | Pipeline logic with all sub-modules wired up |
| Experience recorder | ✅ Implemented | Record/success/failure methods working with database |
| Experience repository | ✅ Implemented | Full CRUD for encounters and experiences |
| Reflection system | ✅ Complete | Core types, services (analyzer, generator, repository, validator), patterns |
| Hypothesis Engine | ✅ Implemented | Observation → Hypothesis → Test → Evidence → Knowledge pipeline with 9 MCP tools and full database support |
| Exploration system | ✅ Implemented | Exploration tracking with repository |
| Reputation system | ✅ Implemented | Full reputation tracking with decay and analytics |
| Evolution system | ✅ Implemented | Behavior creation from insights, tracking, promotion/deprecation |
| Metrics collection | ✅ Implemented | Counters, gauges, time series with aggregation |
| Scheduler | ✅ Implemented | Background task scheduling with SQLite persistence |
| MCP bridge | ✅ Implemented | RMCP, MCP, and ACP protocol implementations in `bridge/` folder |
| MCP tools | ✅ Implemented | Memory, experience, reflection, search, and ingestor tools defined |
| Planner module | ✅ Implemented | Planning engine and policy engine for task decomposition |
| Skills module | ✅ Implemented | Skill registry for managing available skills |
| Workflows module | ✅ Implemented | Workflow execution engine for multi-step tasks |
| Learning module | ✅ Implemented | Working memory, hypothesis tracking, candidate generation, lineage tracking |
| Experience Compression | ✅ Implemented | Pattern detection, exception tracking, and compression algorithms |
| CLI interface | ✅ Implemented | Command-line interface with server, memory, experience commands |
| App entry point | ✅ Implemented | App struct with coordinator and stdio server |
| Main entry point | ✅ Implemented | init_logging() and App::new().run() working |

---

## Immediate Next Steps

1. **Wire MCP tools to handlers** — Connect tool definitions to actual functionality
2. **Implement tool execution** — Make tools actually perform their operations
3. **Implement knowledge graph** — Broader knowledge representation system
4. **Add LLM integration** — Enable actual reflection generation

---

## Known Issues

- **Knowledge graph is placeholder** — Broader knowledge representation needed

## ⚖️ License & Fair-Pay Rule

This project is open-source, but it is also built on fairness. We believe that if the community helps improve this software, the community should share in its financial success.

### 1. For Open-Source Use (AGPL-3.0)
This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. 
* You are completely free to use, modify, and share this code for personal or open-source projects.
* If you modify this code and run it as a cloud service or distribute it, **you must open-source your modifications** under the same AGPL-3.0 license.

### 2. For Commercial Use (Paid License)
Because many companies cannot or will not open-source their proprietary software, we offer a **Commercial License**. If a company wants to use this MCP server internally or in a closed-source product, they must purchase a commercial license from us.

### 3. The Fair-Pay Rule for Contributors
If you contribute code improvements to this project, you are an essential part of it. We do not believe in taking your work to enrich ourselves.
* **Revenue Sharing**: 100% of the net revenue generated from commercial licensing fees will be pooled and split among contributors.
* **How Payouts Work**: Payouts are distributed based on accepted code contributions (Pull Requests) and resolved GitHub issue bounties. 
* **Copyright**: By submitting a Pull Request, you maintain copyright over your code but grant us the right to include it in both the open-source AGPL-3.0 version and the paid commercial version, so we can legally sell it and pay you your share.
