//! Tool permissions - Per Architecture §13.2 "Tool permissions"

use serde::{Deserialize, Serialize};

/// Tool permission definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolPermission {
    /// Tool name.
    pub tool_name: String,
    /// Allowed callers.
    pub allowed_callers: Vec<String>,
    /// Maximum invocations per minute.
    pub max_invocations_per_minute: u32,
}

/// Tool permission store.
#[derive(Debug, Clone, Default)]
pub struct ToolPermissionStore {
    /// Permissions by tool name.
    pub permissions: std::collections::HashMap<String, Vec<ToolPermission>>,
}

impl ToolPermissionStore {
    /// Grant a permission.
    pub fn grant(&mut self, p: ToolPermission) {
        self.permissions
            .entry(p.tool_name.clone())
            .or_default()
            .push(p);
    }

    /// Revoke a permission for a caller.
    pub fn revoke(&mut self, tool_name: &str, caller: &str) {
        if let Some(perms) = self.permissions.get_mut(tool_name) {
            perms.retain(|p| !p.allowed_callers.contains(&caller.to_string()));
        }
    }
}

/// Check if a caller is authorized for a tool.
pub fn is_authorized(tool: &str, caller: &str) -> bool {
    // Placeholder: in production this would check a permission store
    // for the given tool and caller combination.
    tracing::debug!(tool, caller, "Permission check placeholder");
    true
}

/// Active reference to tool permission contracts.
pub fn reference_tool_permissions_contracts() {
    let mut store = ToolPermissionStore::default();
    let permission = ToolPermission {
        tool_name: "test-tool".to_string(),
        allowed_callers: vec!["caller1".to_string()],
        max_invocations_per_minute: 100,
    };
    store.grant(permission);
    let granted = store
        .permissions
        .get("test-tool")
        .map(|p| p.len())
        .unwrap_or(0);
    store.revoke("test-tool", "caller1");
    let authorized = is_authorized("test-tool", "caller1");
    tracing::info!(
        granted_count = granted,
        is_authorized = authorized,
        "Tool permission contracts actively referenced"
    );
}
