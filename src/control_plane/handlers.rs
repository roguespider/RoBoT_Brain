//! Control plane handlers per Architecture Chapter 28.
//! Handles developer interface requests and routes to appropriate subsystems.

use crate::control_plane::{permissions::PermissionLevel, state::ControlPlaneState};

/// A control plane request.
#[derive(Debug, Clone)]
pub struct ControlPlaneRequest {
    /// The operation to perform.
    pub operation: String,
    /// The caller's permission level.
    pub permission_level: PermissionLevel,
    /// Optional payload data.
    pub payload: Option<String>,
}

/// A control plane response.
#[derive(Debug, Clone)]
pub struct ControlPlaneResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Response data.
    pub data: Option<String>,
    /// Error message, if any.
    pub error: Option<String>,
}

/// Handles control plane requests.
pub struct ControlPlaneHandler {
    state: ControlPlaneState,
}

impl ControlPlaneHandler {
    /// Create a new handler with the given state.
    pub fn new(state: ControlPlaneState) -> Self {
        Self { state }
    }

    /// Handle a control plane request.
    pub fn handle(&mut self, request: ControlPlaneRequest) -> ControlPlaneResponse {
        if request.permission_level == PermissionLevel::ReadOnly {
            match request.operation.as_str() {
                "get_state" => ControlPlaneResponse {
                    success: true,
                    data: Some(format!("{:?}", self.state.system)),
                    error: None,
                },
                "get_permissions" => ControlPlaneResponse {
                    success: true,
                    data: Some(format!("{:?}", request.permission_level)),
                    error: None,
                },
                _ => ControlPlaneResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Unknown operation: {}", request.operation)),
                },
            }
        } else {
            ControlPlaneResponse {
                success: false,
                data: None,
                error: Some("Insufficient permissions".to_string()),
            }
        }
    }
}

impl Default for ControlPlaneHandler {
    fn default() -> Self {
        Self::new(ControlPlaneState::new())
    }
}
