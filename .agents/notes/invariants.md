# Shared Invariants

Canonical invariants that every v0.0.2 subsystem must enforce.

## 1. Identity (Chapter 5)

All entities use UUID v4 identifiers. Rules:
- IDs are never reused — a deleted entity's ID is never assigned to a new entity
- IDs contain no PII — never embed names, emails, or other personal data
- IDs are opaque — consumers must not infer meaning from ID structure
- Every public type that represents a persistent entity MUST have an `id: String` field

## 2. Correlation (Chapter 16.1)

Every event and response carries correlation metadata:
- **`correlation_id`** — links all events in a single request/response cycle
- **`reply_to`** — on responses, references the original request's correlation_id
- Events emitted by a subsystem carry the `correlation_id` from the incoming event
- If no incoming event exists (e.g., scheduled task), generate a new `correlation_id`

This enables full request tracing across all subsystems.

## 3. Provenance (Chapter 5.2 + Chapter 19)

Every record tracks its origin:
- `source` — how this record was created (e.g., "user_input", "tool_execution", "research")
- `source_kind` — the type of source ("mcp_tool", "acp_message", "scheduled_task")
- `created_by` — the actor or system that created this record
- Every claim links to at least one Evidence record, or marks itself "ungrounded"

## 4. Uncertainty (Chapter 19.1)

Every numeric score has a confidence field:
- Confidence ranges from 0.0 to 1.0
- All confidence scores start at 0.5 (neutral) by default
- Promotions to higher-tier storage require confidence >= 0.7

## 5. Failure Visibility (Chapter 16.2)

No silent fallbacks — every error path emits an Error event:
- Failed retrievals emit `RetrievalFailed` events
- Timeout errors emit `Timeout` events
- Validation failures emit `ValidationError` events
- All error events include the original correlation_id for tracing

## 6. Versioned Evolution (Chapter 5.1)

Every data contract has a version field:
- `version: String` in SemVer format (e.g., "0.0.2")
- Backward-compatible changes increment the minor version
- Breaking changes increment the major version
- All new subsystems start at version "0.0.2"

## Checklist

All public types MUST have:
- [ ] An `id` field (UUID v4 format)
- [ ] A `correlation_id` field (for event tracking)
- [ ] A `version` field (SemVer format)
- [ ] A `created_at` timestamp
- [ ] Provenance metadata (`source`, `source_kind`)
