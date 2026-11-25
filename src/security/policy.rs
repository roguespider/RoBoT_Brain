//! Security Policy — Per Architecture Chapter 25 (Security Layers).
//!
//! Security policies define rules for what actions are permitted under
//! what conditions. Policies are enforced by the permission system.

/// A security policy rule.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Unique policy identifier.
    pub id: String,
    /// Policy name.
    pub name: String,
    /// Policy description.
    pub description: String,
    /// Whether the policy is active.
    pub active: bool,
}

/// Security policy manager.
#[derive(Debug, Clone, Default)]
pub struct PolicyManager {
    /// All policies.
    policies: Vec<Policy>,
}

impl PolicyManager {
    /// Create a new empty policy manager.
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add a policy.
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Get a policy by ID.
    pub fn get(&self, id: &str) -> Option<&Policy> {
        self.policies.iter().find(|p| p.id == id)
    }

    /// List all active policies.
    pub fn active_policies(&self) -> Vec<&Policy> {
        self.policies.iter().filter(|p| p.active).collect()
    }

    /// Count total policies.
    pub fn count(&self) -> usize {
        self.policies.len()
    }
}
