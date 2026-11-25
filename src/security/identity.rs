//! Security Identity — Per Architecture Chapter 25.5 (Identity System).
//!
//! Every actor inside RoBoT has an identity providing ownership,
//! accountability, permissions, and history.

/// A security identity for an actor in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Unique identity identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Type of actor (e.g., "human", "ai_contributor", "background_worker").
    pub actor_type: String,
}

/// Identities managed by the system.
#[derive(Debug, Clone, Default)]
pub struct IdentitySystem {
    /// All registered identities.
    identities: Vec<Identity>,
}

impl IdentitySystem {
    /// Create a new empty identity system.
    pub fn new() -> Self {
        Self {
            identities: Vec::new(),
        }
    }

    /// Register a new identity.
    pub fn register(&mut self, identity: Identity) {
        self.identities.push(identity);
    }

    /// Get an identity by ID.
    pub fn get(&self, id: &str) -> Option<&Identity> {
        self.identities.iter().find(|i| i.id == id)
    }

    /// List all identity IDs.
    pub fn list_ids(&self) -> Vec<String> {
        self.identities.iter().map(|i| i.id.clone()).collect()
    }

    /// Count total identities.
    pub fn count(&self) -> usize {
        self.identities.len()
    }
}
