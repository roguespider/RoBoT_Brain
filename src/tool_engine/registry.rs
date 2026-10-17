//! Tool registry - Per Architecture §13.1 "Tool registration"
//!
//! The Tool Registry acts as the directory of every available capability.
//! Each tool registers itself during startup.
//! The registry allows discovery without hardcoding.

use serde::{Deserialize, Serialize};

/// Capability levels for a tool per Architecture §13.10 (Capability Negotiation).
/// Tools may expose different levels of functionality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CapabilityLevel {
    /// Basic functionality only.
    Basic,
    /// Extended functionality (e.g., read + read_range).
    Extended,
    /// Full functionality (e.g., read + read_range + stream + metadata).
    Full,
}

impl std::fmt::Display for CapabilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityLevel::Basic => write!(f, "basic"),
            CapabilityLevel::Extended => write!(f, "extended"),
            CapabilityLevel::Full => write!(f, "full"),
        }
    }
}

/// A registered tool contract with full metadata.
/// Per Architecture §13.4 (Tool Metadata):
/// id, name, description, version, category, permissions,
/// parameters, return_schema, timeout, retry_policy, estimated_cost,
/// supports_streaming, supports_parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContract {
    /// Tool name / identifier.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Input schema (JSON Schema).
    pub input_schema: serde_json::Value,
    /// Output schema (JSON Schema).
    pub output_schema: serde_json::Value,
    /// Tool version (semantic versioning).
    pub version: String,
    /// Tool category (build, filesystem, memory, network, etc.).
    pub category: String,
    /// Required permissions.
    pub permissions: Vec<String>,
    /// Maximum execution time in milliseconds.
    pub timeout_ms: u64,
    /// Whether the tool supports parallel execution.
    pub supports_parallel: bool,
    /// Whether the tool supports streaming output.
    pub supports_streaming: bool,
    /// Capability level exposed by this tool implementation.
    /// Per Architecture §13.10: tools may expose different capability levels.
    pub capability_level: CapabilityLevel,
    /// Retry policy for this tool.
    /// Per Architecture §13.23: tools declare retry characteristics.
    pub retry_policy: Option<crate::tool_engine::retry_policy::RetryPolicy>,
}

impl ToolContract {
    /// Create a basic tool contract with defaults.
    pub fn basic(name: &str, description: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
            output_schema: serde_json::json!({"type": "object"}),
            version: "1.0.0".to_string(),
            category: category.to_string(),
            permissions: Vec::new(),
            timeout_ms: 30_000,
            supports_parallel: false,
            supports_streaming: false,
            capability_level: CapabilityLevel::Basic,
            retry_policy: None,
        }
    }

    /// Parse a semantic version string into major, minor, patch components.
    /// Returns None if the version string is not valid semver.
    pub fn parse_version(&self) -> Option<(u64, u64, u64)> {
        let v = &self.version;
        let parts: Vec<&str> = v.trim_start_matches('v').split('.').collect();
        if parts.len() < 3 {
            return None;
        }
        let major = parts[0].parse::<u64>().ok()?;
        let minor = parts[1].parse::<u64>().ok()?;
        // Ignore pre-release suffix (e.g., "1.0.0-alpha" -> 0)
        let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
        let patch = patch_str.parse::<u64>().ok()?;
        Some((major, minor, patch))
    }

    /// Check if this tool version is compatible with a required version.
    /// Compatible if major version matches (semver-compatible).
    pub fn is_compatible_with(&self, required_version: &str) -> bool {
        let Some((req_major, _, _)) = Self::parse_version_req(required_version) else {
            return true; // No version specified or parseable, assume compatible
        };
        let Some((this_major, _, _)) = self.parse_version() else {
            return true; // No version parseable, assume compatible
        };
        this_major == req_major
    }

    /// Parse a required version string.
    fn parse_version_req(version: &str) -> Option<(u64, u64, u64)> {
        let v = version.trim_start_matches(['=', '~', '^', 'v']);
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() < 3 {
            return None;
        }
        let major = parts[0].parse::<u64>().ok()?;
        let minor = parts[1].parse::<u64>().ok()?;
        let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
        let patch = patch_str.parse::<u64>().ok()?;
        Some((major, minor, patch))
    }

    /// Negotiate capability level between what the tool provides and what is requested.
    /// Per Architecture §13.10: the Tool Engine negotiates the appropriate capability level.
    pub fn negotiate_capability(&self, requested: CapabilityLevel) -> Option<CapabilityLevel> {
        // The negotiated level is the minimum of what's requested and what's available
        match (requested, &self.capability_level) {
            (CapabilityLevel::Basic, _) => Some(CapabilityLevel::Basic),
            (CapabilityLevel::Extended, CapabilityLevel::Basic) => None,
            (CapabilityLevel::Extended, _) => Some(CapabilityLevel::Extended),
            (CapabilityLevel::Full, CapabilityLevel::Basic) => None,
            (CapabilityLevel::Full, CapabilityLevel::Extended) => None,
            (CapabilityLevel::Full, CapabilityLevel::Full) => Some(CapabilityLevel::Full),
        }
    }

    /// Set the capability level for this tool.
    pub fn with_capability_level(self, level: CapabilityLevel) -> Self {
        Self {
            capability_level: level,
            ..self
        }
    }

    /// Set the retry policy for this tool.
    pub fn with_retry_policy(self, policy: crate::tool_engine::retry_policy::RetryPolicy) -> Self {
        Self {
            retry_policy: Some(policy),
            ..self
        }
    }
}

/// Tool identifier.
pub type ToolId = String;

/// Tool error types.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolError {
    RegistrationFailed(String),
    NotFound,
    InvalidSchema(String),
    Duplicate(String),
    VersionIncompatible {
        tool: String,
        found: String,
        required: String,
    },
    CapabilityNotAvailable {
        tool: String,
        level: String,
    },
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::RegistrationFailed(msg) => write!(f, "registration failed: {}", msg),
            ToolError::NotFound => write!(f, "tool not found"),
            ToolError::InvalidSchema(msg) => write!(f, "invalid schema: {}", msg),
            ToolError::Duplicate(name) => write!(f, "tool already registered: {}", name),
            ToolError::VersionIncompatible {
                tool,
                found,
                required,
            } => write!(
                f,
                "tool '{}' version {} incompatible with required {}",
                tool, found, required
            ),
            ToolError::CapabilityNotAvailable { tool, level } => write!(
                f,
                "tool '{}' does not support capability level {}",
                tool, level
            ),
        }
    }
}

impl std::error::Error for ToolError {}

/// The Tool Registry — a directory of every available capability.
/// Per Architecture §13.1: each tool registers itself during startup.
/// Per Architecture §13.16: includes lifecycle tracking for health monitoring.
/// Per Architecture §25.6-25.7: includes permission management for tool access.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    /// Registered tools indexed by name.
    tools: std::collections::HashMap<String, ToolContract>,
    /// Lifecycle manager for tool health monitoring.
    /// Per Architecture §13.16: tracks tool health events and lifecycle states.
    pub(crate) lifecycle: crate::tool_engine::lifecycle::ToolLifecycle,
    /// Permission checker for tool access control.
    /// Per Architecture §25.6-25.7: validates caller permissions before execution.
    permissions: crate::tool_engine::permissions::PermissionChecker,
}

impl ToolRegistry {
    /// Register a new tool in the registry.
    /// Per Architecture §13.1: registers the tool contract.
    /// Per Architecture §13.16: also registers the tool in the lifecycle manager.
    pub fn register(&mut self, contract: ToolContract) -> Result<ToolId, ToolError> {
        if self.tools.contains_key(&contract.name) {
            return Err(ToolError::Duplicate(contract.name.clone()));
        }
        tracing::debug!(tool_name = %contract.name, category = %contract.category, "Registering tool in registry");
        let id = format!("tool-id-{}", uuid::Uuid::new_v4());
        self.tools.insert(contract.name.clone(), contract);
        // Wire into lifecycle manager per Architecture §13.16
        self.lifecycle.register_tool(&id);
        Ok(id)
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&ToolContract> {
        self.tools.get(name)
    }

    /// List all registered tool names.
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Find tools by category.
    pub fn find_by_category(&self, category: &str) -> Vec<&ToolContract> {
        let category = category.to_lowercase();
        self.tools
            .values()
            .filter(|t| t.category.to_lowercase() == category)
            .collect()
    }

    /// Find tools that match a capability.
    pub fn find_by_capability(&self, capability: &str) -> Vec<&ToolContract> {
        let capability = capability.to_lowercase();
        self.tools
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&capability)
                    || t.description.to_lowercase().contains(&capability)
            })
            .collect()
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Get all tools as a vector (for tool selection).
    pub fn all_tools(&self) -> Vec<ToolContract> {
        self.tools.values().cloned().collect()
    }

    /// Resolve a tool by name and required version.
    /// Per Architecture §13.26: version metadata allows selecting compatible implementations.
    pub fn resolve(
        &self,
        tool_name: &str,
        required_version: Option<&str>,
    ) -> Result<&ToolContract, ToolError> {
        let contract = self.tools.get(tool_name).ok_or(ToolError::NotFound)?;
        let req_ver = match required_version {
            Some(v) => v,
            None => return Ok(contract),
        };
        if !contract.is_compatible_with(req_ver) {
            return Err(ToolError::VersionIncompatible {
                tool: tool_name.to_string(),
                found: contract.version.clone(),
                required: req_ver.to_string(),
            });
        }
        Ok(contract)
    }

    /// Negotiate capability level for a tool.
    /// Per Architecture §13.10: negotiate appropriate capability level.
    pub fn negotiate_capability(
        &self,
        tool_name: &str,
        requested: CapabilityLevel,
    ) -> Option<CapabilityLevel> {
        self.tools
            .get(tool_name)
            .and_then(|c| c.negotiate_capability(requested))
    }

    /// Get the retry policy for a tool.
    pub fn get_retry_policy(
        &self,
        tool_name: &str,
    ) -> Option<&crate::tool_engine::retry_policy::RetryPolicy> {
        self.tools
            .get(tool_name)
            .and_then(|c| c.retry_policy.as_ref())
    }

    /// Register a tool with a specific capability level.
    pub fn register_with_capability(
        &mut self,
        contract: ToolContract,
        capability_level: CapabilityLevel,
    ) -> Result<ToolId, ToolError> {
        let contract = contract.with_capability_level(capability_level);
        self.register(contract)
    }

    /// Check if a tool is available (registered and compatible).
    pub fn is_available(&self, tool_name: &str, required_version: Option<&str>) -> bool {
        self.resolve(tool_name, required_version).is_ok()
    }

    /// Get the lifecycle manager for health monitoring.
    /// Per Architecture §13.16: health monitoring loop accesses tool health via registry.
    pub fn get_lifecycle(&self) -> &crate::tool_engine::lifecycle::ToolLifecycle {
        &self.lifecycle
    }

    /// Get mutable access to the lifecycle manager for health updates.
    /// Per Architecture §13.16: status updates propagate via registry.
    pub fn get_lifecycle_mut(&mut self) -> &mut crate::tool_engine::lifecycle::ToolLifecycle {
        &mut self.lifecycle
    }

    /// Grant a permission for a tool caller.
    /// Per Architecture §25.6: permission granting for access control.
    pub fn grant_permission(&mut self, tool: &str, caller: &str, max_invocations_per_minute: u32) {
        use crate::tool_engine::permissions::ToolPermission;
        let perm = ToolPermission {
            tool_name: tool.to_string(),
            allowed_callers: vec![caller.to_string()],
            max_invocations_per_minute,
        };
        self.permissions.grant_permission(perm);
        tracing::debug!(tool = %tool, caller = %caller, "Permission granted");
    }

    /// Revoke a permission for a tool caller.
    /// Per Architecture §25.6: permission revocation for access control.
    pub fn revoke_permission(&mut self, tool: &str, caller: &str) {
        self.permissions.revoke_permission(tool, caller);
        tracing::debug!(tool = %tool, caller = %caller, "Permission revoked");
    }

    /// Get all permissions for a tool.
    /// Per Architecture §25.6: used for permission management and auditing.
    pub fn get_permissions(
        &self,
        tool: &str,
    ) -> Vec<crate::tool_engine::permissions::ToolPermission> {
        self.permissions.get_permissions(tool)
    }

    /// Update permissions for a tool.
    /// Per Architecture §25.6: used for permission management.
    pub fn update_permissions(
        &mut self,
        tool: &str,
        permissions: Vec<crate::tool_engine::permissions::ToolPermission>,
    ) {
        self.permissions.update_permissions(tool, permissions);
        tracing::debug!(tool = %tool, "Permissions updated");
    }

    /// Get the audit log for permission checks.
    /// Per Architecture §25.7: audit logging for security compliance.
    pub fn get_audit_log(&self) -> Vec<crate::tool_engine::permissions::PermissionAuditEntry> {
        self.permissions.get_audit_log()
    }

    /// Get the count of audit log entries.
    pub fn audit_log_count(&self) -> usize {
        self.permissions.audit_log_count()
    }

    /// List all tools that have permissions defined.
    /// Per Architecture §25.6: used for permission inventory.
    pub fn list_permitted_tools(&self) -> Vec<String> {
        self.permissions.list_permitted_tools()
    }

    /// Check if a tool has any permissions defined.
    /// Per Architecture §25.6: used for permission checks.
    pub fn has_permissions(&self, tool: &str) -> bool {
        self.permissions.has_permissions(tool)
    }
}

/// Global tool registry instance.
/// Tools are registered at startup and available throughout the application.
static TOOL_REGISTRY: std::sync::OnceLock<std::sync::RwLock<ToolRegistry>> =
    std::sync::OnceLock::new();

/// Initialize the global tool registry.
pub fn init_registry() {
    let prev_state = TOOL_REGISTRY.get().is_some();
    let registry_initialized = !prev_state;
    let registry_tool_count = TOOL_REGISTRY
        .get_or_init(|| std::sync::RwLock::new(ToolRegistry::default()))
        .read()
        .map_or(0, |r| r.len());
    tracing::debug!(
        registry_initialized_new = registry_initialized,
        registry_tool_count = registry_tool_count,
        "Tool registry initialized"
    );
}

/// Get a read guard to the global tool registry.
pub fn get_registry() -> Result<std::sync::RwLockReadGuard<'static, ToolRegistry>, &'static str> {
    let lock = TOOL_REGISTRY.get().ok_or("registry not initialized")?;
    lock.read().map_err(|poison_err| {
        tracing::warn!("Registry lock poisoned: {:?}", poison_err);
        "registry lock poisoned"
    })
}

/// Get a write guard to the global tool registry.
pub fn get_registry_mut() -> Result<std::sync::RwLockWriteGuard<'static, ToolRegistry>, &'static str>
{
    let lock = TOOL_REGISTRY.get().ok_or("registry not initialized")?;
    lock.write().map_err(|poison_err| {
        tracing::warn!("Registry lock poisoned: {:?}", poison_err);
        "registry lock poisoned"
    })
}

/// Register a tool in the global registry.
pub fn register_tool(contract: ToolContract) -> Result<ToolId, ToolError> {
    init_registry();
    let mut registry =
        get_registry_mut().map_err(|e| ToolError::RegistrationFailed(e.to_string()))?;
    let result = registry.register(contract);
    tracing::debug!(tool_result = ?result.is_ok(), "Tool registration result");
    result
}

/// Run an execution function in isolation.
pub fn run_isolated<
    F: FnOnce() -> Result<
        crate::skills::registry::result::ExecutionResult,
        crate::execution::ExecutionError,
    >,
>(
    ctx: &crate::execution::IsolationContext,
    f: F,
) -> Result<crate::skills::registry::result::ExecutionResult, crate::execution::ExecutionError> {
    tracing::debug!(
        timeout_ms = ctx.timeout_ms,
        "Invoking tool with authorization"
    );
    f()
}

/// Invoke a tool through the execution pipeline with authentication.
/// Per Architecture §13.8: uses the global ToolRegistry (not a fresh one) so
/// registered tools are actually available for dispatch.
pub fn invoke_tool_with_auth(tool: &str, caller: &str) -> Result<serde_json::Value, ToolError> {
    // Check authorization first
    if !crate::tool_engine::permissions::is_authorized(tool, caller) {
        return Err(ToolError::NotFound);
    }
    // Use the GLOBAL registry so registered tools are actually available.
    // This is the fix for ARCH-V02-13: previously created ToolRegistry::default()
    // which had zero registered tools, causing every dispatch to fail with NotFound.
    let registry = get_registry().map_err(|_| ToolError::NotFound)?;
    let permission_checker = crate::tool_engine::permissions::PermissionChecker::new();
    // Pass a reference to the registry (dispatcher takes &'a ToolRegistry now)
    let dispatcher =
        crate::tool_engine::dispatcher::ToolDispatcher::new(&registry, permission_checker);
    // Execute via async dispatcher using tokio runtime
    let params = serde_json::json!({});
    let result =
        tokio::runtime::Handle::current().block_on(dispatcher.dispatch(tool, caller, params));
    // Return the result from dispatcher
    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(ToolError::RegistrationFailed(e.to_string())),
    }
}
