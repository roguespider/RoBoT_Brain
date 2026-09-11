# RoBoT Brain v0.0.2 Architecture Foundation

This note captures the core architectural principles from the v0.0.2 architecture
that every subsystem upgrade must preserve.

## Memory-First Design

The cognitive architecture is memory-first: every operation begins with a retrieval
from the memory hierarchy before any reasoning or decision-making occurs. This
follows the architecture §2.3 principle that "memory retrieval precedes reasoning."
The system always queries working memory, permanent memory, and knowledge stores
as the first step in its cognitive pipeline, ensuring that new actions are grounded
in stored experience and validated knowledge.

## Experience-Based Learning

Every action, observation, and outcome is recorded as an experience. These experiences
form the learning substrate from which patterns, skills, and knowledge emerge. The
architecture §2.5 "Controlled evolution" principle states that the system learns
through experience-driven loops: Act → New Experience → Learn → Better Decisions.
The Learning Engine (§10) processes experiences through scoring, generalization,
and promotion pipelines to continuously refine the agent's capabilities.

## Persistence and Continuity

The memory hierarchy (Working → Permanent → Knowledge) ensures that information
persists across sessions and grows in quality over time. Per architecture §17
"Memory Architecture," information flows through promotion stages: working memory
items are consolidated into permanent storage, and high-confidence permanent items
are promoted to the knowledge engine. Each memory item carries a lifecycle state
and promotion history, enabling the system to reason about the freshness and
reliability of stored information.

## Controlled Evolution

The system evolves through controlled, evidence-based changes rather than arbitrary
modifications. Architecture §2.6 "Controlled evolution" mandates that all subsystem
changes follow a validation pipeline: propose → test → validate → deploy. The
Confidence System (§19) tracks confidence scores for all knowledge items, and
promotions require a confidence threshold (>= 0.7) before moving information to
higher-tier storage. This prevents low-quality information from polluting the
knowledge base.

## Modularity

Each subsystem encapsulates a single responsibility and communicates through
well-defined event contracts. Architecture §2.1 establishes that subsystems
MUST NOT directly import each other's internal types — instead, they communicate
via the event-driven coordination layer (§16). This loose coupling enables
independent testing, replacement, and evolution of each subsystem without
ripple effects across the architecture.

## Explainability

Every decision the agent makes must be traceable to its evidence sources. The
Confidence System (§19.1) requires that each decision carries a breakdown of
supporting evidence: which memory items, knowledge items, and experiences
contributed to the confidence score. This enables post-hoc auditing of the
agent's reasoning and provides a path for human review of critical decisions.

## Event-Driven Design

Subsystems communicate through an event bus rather than direct function calls.
Architecture §2.4 and Chapter 16 "Cognitive Coordination Layer" mandate that
each subsystem emits events (e.g., `KnowledgeUpdated`, `ExperienceRecorded`,
`DecisionMade`) and reacts to events from other subsystems. This event-driven
pattern ensures loose coupling and enables the system to maintain an audit
trail of all internal state changes.

## Confidence-Based Decisions

All subsystem decisions are gated by confidence thresholds derived from evidence.
Architecture §2.5 states that "confidence must be quantified and compared against
a threshold before acting." The Confidence System (§19) defines scoring rules
for each type of evidence (memory retrieval, knowledge match, experience similarity)
and combines them into a blended confidence score. Actions below the threshold
trigger the research cascade (S7) or abstain, preventing low-confidence decisions.

## Controlled Evolution (Cross-Link)

See the "Controlled Evolution" section above for the full principle. All subsystem
upgrades in Tier 2 follow this controlled evolution pattern: each change is
proposed, tested in isolation, validated against the architecture, and deployed
with rollback capability.
