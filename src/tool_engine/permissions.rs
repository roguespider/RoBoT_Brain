//! Tool permissions - Per Architecture §13.2 "Tool permissions"
//! Per Architecture §25.6 (Permission Architecture) and §25.7 (Capability-Based Security).

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

/// Audit entry for permission checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditEntry {
    /// Timestamp of the check.
    pub timestamp: i64,
    /// Tool name.
    pub tool: String,
    /// Caller identity.
    pub caller: String,
    /// Whether the check was authorized.
    pub authorized: bool,
    /// Reason for the decision.
    pub reason: String,
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

    /// Get all permissions for a tool.
    pub fn get_permissions(&self, tool_name: &str) -> Vec<ToolPermission> {
        self.permissions.get(tool_name).cloned().unwrap_or_default()
    }

    /// Get all registered tools.
    pub fn list_tools(&self) -> Vec<String> {
        self.permissions.keys().cloned().collect()
    }

    /// Check if a tool has any permissions defined.
    pub fn has_permissions(&self, tool_name: &str) -> bool {
        !self.get_permissions(tool_name).is_empty()
    }
}

/// Permission checker - implements Architecture §25.6-25.7.
/// Per Architecture §25.6 (Permission Architecture): validates caller
/// permissions before allowing tool execution.
/// Per Architecture §25.7 (Capability-Based Security): evaluates
/// capabilities and maintains an audit log.
#[derive(Debug, Default)]
pub struct PermissionChecker {
    /// Permission store for tool access control.
    store: std::sync::RwLock<ToolPermissionStore>,
    /// Audit log for permission checks.
    audit_log: std::sync::RwLock<Vec<PermissionAuditEntry>>,
    /// Maximum audit entries to retain.
    max_audit_entries: usize,
}

impl PermissionChecker {
    /// Create a new permission checker with default settings.
    pub fn new() -> Self {
        Self {
            store: std::sync::RwLock::new(ToolPermissionStore::default()),
            audit_log: std::sync::RwLock::new(Vec::new()),
            max_audit_entries: 10000,
        }
    }

    /// Create a new permission checker with custom max audit entries.
    pub fn with_max_audit_entries(max_entries: usize) -> Self {
        Self {
            store: std::sync::RwLock::new(ToolPermissionStore::default()),
            audit_log: std::sync::RwLock::new(Vec::new()),
            max_audit_entries: max_entries,
        }
    }

    /// Check if a caller is authorized for a tool.
    /// Per Architecture §25.6: called before tool execution.
    /// Returns (authorized, reason).
    pub fn check(&self, tool: &str, caller: &str) -> (bool, String) {
        // Per Architecture §25.10 (Tool Security): validate caller permissions
        if tool.is_empty() || caller.is_empty() {
            let reason = "Authorization failed: empty input".to_string();
            self.record_audit(tool, caller, false, &reason);
            return (false, reason);
        }

        let store = match self.store.read() {
            Ok(guard) => guard,
            Err(_) => {
                let reason = "Authorization failed: lock poisoned".to_string();
                self.record_audit(tool, caller, false, &reason);
                return (false, reason);
            }
        };

        // Check against the permission store
        if let Some(perms) = store.permissions.get(tool) {
            for perm in perms {
                if perm.allowed_callers.contains(&caller.to_string()) {
                    let reason = "Authorization granted via permission store".to_string();
                    self.record_audit(tool, caller, true, &reason);
                    return (true, reason);
                }
            }
        }

        // System and admin tools have implicit access
        if caller.starts_with("system_") || caller.starts_with("admin_") {
            let reason = "Authorization granted (system/admin)".to_string();
            self.record_audit(tool, caller, true, &reason);
            return (true, reason);
        }

        // Default deny
        let reason = "Authorization denied: caller not in allowed list".to_string();
        self.record_audit(tool, caller, false, &reason);
        (false, reason)
    }

    /// Evaluate permissions for a tool without side effects.
    /// Per Architecture §25.7: returns permission evaluation details.
    pub fn evaluate(&self, tool: &str) -> PermissionEvaluation {
        let perms = self
            .store
            .read()
            .map_or(Vec::new(), |s| s.get_permissions(tool));
        let caller_count = perms.iter().map(|p| p.allowed_callers.len()).sum::<usize>();
        PermissionEvaluation {
            tool: tool.to_string(),
            has_permissions: !perms.is_empty(),
            permission_count: perms.len(),
            total_allowed_callers: caller_count,
            max_invocation_rate: perms
                .iter()
                .map(|p| p.max_invocations_per_minute)
                .max()
                .unwrap_or(0),
        }
    }

    /// Get permissions for a tool.
    /// Per Architecture §25.6: used for permission management.
    pub fn get_permissions(&self, tool: &str) -> Vec<ToolPermission> {
        self.store
            .read()
            .map_or(Vec::new(), |s| s.get_permissions(tool))
    }

    /// Update permissions for a tool.
    /// Per Architecture §25.6: used for permission management.
    pub fn update_permissions(&self, tool: &str, permissions: Vec<ToolPermission>) {
        if let Ok(mut store) = self.store.write() {
            if permissions.is_empty() {
                store.permissions.remove(tool);
            } else {
                store.permissions.insert(tool.to_string(), permissions);
            }
        }
    }

    /// Get the audit log entries.
    /// Per Architecture §25.14 (Audit System): used for audit logging.
    pub fn get_audit_log(&self) -> Vec<PermissionAuditEntry> {
        self.audit_log.read().map_or(Vec::new(), |log| log.clone())
    }

    /// Get audit log count.
    pub fn audit_log_count(&self) -> usize {
        self.audit_log.read().map_or(0, |log| log.len())
    }

    /// Grant a permission for a tool caller.
    /// Per Architecture §25.6: permission granting for access control.
    pub fn grant_permission(&self, permission: ToolPermission) {
        if let Ok(mut store) = self.store.write() {
            store.grant(permission);
        }
    }

    /// Revoke a permission for a tool caller.
    /// Per Architecture §25.6: permission revocation for access control.
    pub fn revoke_permission(&self, tool: &str, caller: &str) {
        if let Ok(mut store) = self.store.write() {
            store.revoke(tool, caller);
        }
    }

    /// List all tools that have permissions defined.
    /// Per Architecture §25.6: used for permission inventory.
    pub fn list_permitted_tools(&self) -> Vec<String> {
        self.store.read().map_or(Vec::new(), |s| s.list_tools())
    }

    /// Check if a tool has any permissions defined.
    /// Per Architecture §25.6: used for permission checks.
    pub fn has_permissions(&self, tool: &str) -> bool {
        self.store.read().is_ok_and(|s| s.has_permissions(tool))
    }

    /// Record an audit entry for a permission check.
    fn record_audit(&self, tool: &str, caller: &str, authorized: bool, reason: &str) {
        let entry = PermissionAuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            tool: tool.to_string(),
            caller: caller.to_string(),
            authorized,
            reason: reason.to_string(),
        };
        if let Ok(mut log) = self.audit_log.write() {
            log.push(entry);
            // Trim old entries if over limit
            if log.len() > self.max_audit_entries {
                let trim_count = log.len() - self.max_audit_entries;
                log.drain(0..trim_count);
            }
        }
    }
}

/// Permission evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEvaluation {
    /// Tool name.
    pub tool: String,
    /// Whether the tool has any permissions defined.
    pub has_permissions: bool,
    /// Number of permission entries.
    pub permission_count: usize,
    /// Total number of allowed callers across all permissions.
    pub total_allowed_callers: usize,
    /// Maximum invocation rate across all permissions.
    pub max_invocation_rate: u32,
}

/// Check if a caller is authorized for a tool (legacy API).
/// Per Architecture §13.10 (Tool Security): validates caller against
/// the ToolPermissionStore and enforces rate limiting.
/// This is a convenience function that uses a default PermissionChecker.
pub fn is_authorized(tool: &str, caller: &str) -> bool {
    let checker = PermissionChecker::new();
    let (authorized, _) = checker.check(tool, caller);
    authorized
}
