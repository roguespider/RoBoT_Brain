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
) -> Result<(), crate::tools::registry::ToolError> {
    if !c.required_scopes.is_empty() && c.auth_kind == AuthKind::None {
        return Err(crate::tools::registry::ToolError::RegistrationFailed(
            "auth_kind must not be None when required_scopes is non-empty".to_string(),
        ));
    }
    Ok(())
}

/// Active reference to external capability contracts.
pub fn reference_external_capability_contracts() {
    let cap = ExternalCapability {
        name: "test-capability".to_string(),
        endpoint: "https://example.com".to_string(),
        auth_kind: AuthKind::ApiKey,
        required_scopes: vec!["read".to_string()],
    };
    let validation = validate_capability(&cap);
    tracing::info!(
        validation_ok = validation.is_ok(),
        "validate_capability actively referenced"
    );
}
