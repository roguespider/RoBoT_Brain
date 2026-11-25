//! Security Permissions — Per Architecture Chapter 25.6 (Permission Architecture).
//!
//! Permissions determine what an actor can do. Each permission grants
//! access to a specific capability on a specific resource.

/// A permission granting access to a capability on a resource.
#[derive(Debug, Clone)]
pub struct Permission {
    /// Unique permission identifier.
    pub id: String,
    /// Actor identity that holds this permission.
    pub actor_id: String,
    /// Capability granted (e.g., "create_memory", "execute_tool").
    pub capability: String,
    /// Resource the capability applies to (e.g., "memory.store", "tool.cargo.build").
    pub resource: String,
    /// Whether the permission is active.
    pub active: bool,
}

/// Permission management system.
#[derive(Debug, Clone, Default)]
pub struct PermissionSystem {
    /// All permissions.
    permissions: Vec<Permission>,
}

impl PermissionSystem {
    /// Create a new empty permission system.
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
        }
    }

    /// Grant a permission to an actor.
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Check if an actor has a specific capability on a resource.
    pub fn check(&self, actor_id: &str, capability: &str, resource: &str) -> bool {
        self.permissions.iter().any(|p| {
            p.actor_id == actor_id
                && p.capability == capability
                && p.resource == resource
                && p.active
        })
    }

    /// Revoke a permission.
    pub fn revoke(&mut self, actor_id: &str, capability: &str, resource: &str) {
        self.permissions.retain(|p| {
            !(p.actor_id == actor_id && p.capability == capability && p.resource == resource)
        });
    }

    /// Count total permissions.
    pub fn count(&self) -> usize {
        self.permissions.len()
    }
}
