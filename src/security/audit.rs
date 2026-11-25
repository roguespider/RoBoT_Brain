//! Security Audit — Per Architecture Chapter 25.14 (Audit System).
//!
//! Every significant action creates an audit event providing debugging,
//! accountability, learning history, and rollback capability.

/// An audit event record.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub id: String,
    /// Actor performing the action.
    pub actor: String,
    /// Action performed.
    pub action: String,
    /// Target of the action.
    pub target: String,
    /// Confidence change resulting from the action.
    pub confidence_change: String,
    /// Reason for the action.
    pub reason: String,
}

/// Audit log storing all security events.
#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    /// All audit events.
    events: Vec<AuditEvent>,
}

impl AuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Add an audit event.
    pub fn add_event(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// Get all audit events.
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Get events for a specific actor.
    pub fn events_for_actor(&self, actor: &str) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.actor == actor).collect()
    }

    /// Count total events.
    pub fn count(&self) -> usize {
        self.events.len()
    }
}
