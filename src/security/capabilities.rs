//! Security Capabilities — Per Architecture Chapter 25.7 (Capability-Based Security).
//!
//! RoBoT uses capability-based access: specific capability → specific action →
//! specific resource. Each capability defines what actions are allowed on what
//! resources.

/// A capability defining allowed actions on resources.
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique capability identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what the capability allows.
    pub description: String,
    /// Resources this capability applies to.
    pub resources: Vec<String>,
    /// Actions allowed on those resources.
    pub actions: Vec<String>,
}

/// Capability registry.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// All registered capabilities.
    capabilities: Vec<Capability>,
}

impl CapabilityRegistry {
    /// Create a new empty capability registry.
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }

    /// Register a new capability.
    pub fn register(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }

    /// Get a capability by ID.
    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }

    /// List all capability IDs.
    pub fn list_ids(&self) -> Vec<String> {
        self.capabilities.iter().map(|c| c.id.clone()).collect()
    }

    /// Count total capabilities.
    pub fn count(&self) -> usize {
        self.capabilities.len()
    }
}
