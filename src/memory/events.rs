// src/memory/events.rs
//! Memory Event Bus - Per Architecture §4.04, §5.2
//!
//! Provides event-driven integration for memory system.
//! Events flow into memory per the data flow architecture.



use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Memory event types - Per Architecture §5.2
#[derive(Debug, Clone)]
pub enum MemoryEvent {
    /// A new memory was stored
    MemoryStored {
        id: Uuid,
        memory_type: String,
        layer: String,
    },
    /// A memory was retrieved
    MemoryRetrieved {
        id: Uuid,
        access_count: u32,
    },
    /// A memory was updated
    MemoryUpdated {
        id: Uuid,
        old_confidence: f32,
        new_confidence: f32,
    },
    /// A memory was archived
    MemoryArchived {
        id: Uuid,
        reason: String,
    },
    /// A memory was deleted
    MemoryDeleted {
        id: Uuid,
    },
    /// A relationship was added
    RelationshipAdded {
        from_id: Uuid,
        to_id: Uuid,
        relationship_type: String,
    },
    /// A pattern was detected
    PatternDetected {
        pattern_type: String,
        related_memory_ids: Vec<Uuid>,
    },
    /// An experience was recorded
    ExperienceRecorded {
        experience_id: Uuid,
        outcome: String,
    },
}

/// Memory event handler trait (scaffolding for future use)

pub trait MemoryEventHandler: Send + Sync {
    /// Handle a memory event
    fn handle(&self, event: &MemoryEvent);
}

/// Memory event bus - Per Architecture §4.04
///
/// Provides publish-subscribe for memory events.
/// Allows other systems to react to memory changes.
pub struct MemoryEventBus {
    sender: broadcast::Sender<MemoryEvent>,
}

impl MemoryEventBus {
    /// Create a new memory event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Subscribe to memory events
    pub fn subscribe(&self) -> broadcast::Receiver<MemoryEvent> {
        self.sender.subscribe()
    }

    /// Publish a memory event
    pub fn publish(&self, event: MemoryEvent) {
        let _ = self.sender.send(event);
    }

    /// Emit memory stored event
    pub fn emit_stored(&self, id: Uuid, memory_type: &str, layer: &str) {
        self.publish(MemoryEvent::MemoryStored {
            id,
            memory_type: memory_type.to_string(),
            layer: layer.to_string(),
        });
    }

    /// Emit memory retrieved event
    pub fn emit_retrieved(&self, id: Uuid, access_count: u32) {
        self.publish(MemoryEvent::MemoryRetrieved {
            id,
            access_count,
        });
    }

    /// Emit memory updated event
    pub fn emit_updated(&self, id: Uuid, old_confidence: f32, new_confidence: f32) {
        self.publish(MemoryEvent::MemoryUpdated {
            id,
            old_confidence,
            new_confidence,
        });
    }

    /// Emit memory archived event
    pub fn emit_archived(&self, id: Uuid, reason: &str) {
        self.publish(MemoryEvent::MemoryArchived {
            id,
            reason: reason.to_string(),
        });
    }

    /// Emit memory deleted event
    pub fn emit_deleted(&self, id: Uuid) {
        self.publish(MemoryEvent::MemoryDeleted { id });
    }

    /// Emit relationship added event
    pub fn emit_relationship_added(&self, from_id: Uuid, to_id: Uuid, rel_type: &str) {
        self.publish(MemoryEvent::RelationshipAdded {
            from_id,
            to_id,
            relationship_type: rel_type.to_string(),
        });
    }

    /// Emit pattern detected event
    pub fn emit_pattern_detected(&self, pattern_type: &str, related_ids: Vec<Uuid>) {
        self.publish(MemoryEvent::PatternDetected {
            pattern_type: pattern_type.to_string(),
            related_memory_ids: related_ids,
        });
    }

    /// Emit experience recorded event
    pub fn emit_experience_recorded(&self, experience_id: Uuid, outcome: &str) {
        self.publish(MemoryEvent::ExperienceRecorded {
            experience_id,
            outcome: outcome.to_string(),
        });
    }
}

impl Default for MemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event-driven memory that wraps a repository and emits events
pub struct EventDrivenMemory<R: MemoryRepository> {
    repository: Arc<R>,
    event_bus: Arc<MemoryEventBus>,
}

impl<R: MemoryRepository> EventDrivenMemory<R> {
    /// Create a new event-driven memory
    pub fn new(repository: Arc<R>, event_bus: Arc<MemoryEventBus>) -> Self {
        Self {
            repository,
            event_bus,
        }
    }

    /// Store a memory item and emit event
    pub fn store(&self, item: &crate::memory::types::MemoryItem) -> anyhow::Result<()> {
        self.repository.store(item)?;
        self.event_bus.emit_stored(
            item.id,
            &item.memory_type.to_string(),
            &item.layer.to_string(),
        );
        Ok(())
    }

    /// Retrieve a memory item and emit event
    pub fn retrieve(&self, id: &Uuid) -> anyhow::Result<Option<crate::memory::types::MemoryItem>> {
        let result = self.repository.retrieve(id)?;
        if let Some(ref item) = result {
            self.event_bus.emit_retrieved(item.id, item.access_count);
        }
        Ok(result)
    }

    /// Update a memory item and emit event
    pub fn update(&self, item: &crate::memory::types::MemoryItem) -> anyhow::Result<()> {
        if let Ok(Some(old)) = self.repository.retrieve(&item.id) {
            self.repository.update(item)?;
            self.event_bus.emit_updated(item.id, old.confidence, item.confidence);
        }
        Ok(())
    }

    /// Delete a memory item and emit event
    pub fn delete(&self, id: &Uuid) -> anyhow::Result<()> {
        self.repository.delete(id)?;
        self.event_bus.emit_deleted(*id);
        Ok(())
    }
}

use crate::memory::repository::MemoryRepository;
