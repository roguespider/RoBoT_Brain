//! External capability integration - Per Architecture §13.4 "External capability integration"

use serde::{Deserialize, Serialize};

/// Authentication kind for external capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AuthKind {
    None,
    ApiKey,
    OAuth,
    MTls,
}

/// External capability definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalCapability {
    /// Capability name.
    pub name: String,
    /// Endpoint URL.
    pub endpoint: String,
    /// Authentication kind.
    pub auth_kind: AuthKind,
    /// Required scopes.
    pub required_scopes: Vec<String>,
}

/// Validate an external capability.
pub fn validate_capability(
    c: &ExternalCapability,
) -> Result<(), crate::tool_engine::registry::ToolError> {
    if !c.required_scopes.is_empty() && c.auth_kind == AuthKind::None {
        return Err(crate::tool_engine::registry::ToolError::RegistrationFailed(
            "auth_kind must not be None when required_scopes is non-empty".to_string(),
        ));
    }
    Ok(())
}
