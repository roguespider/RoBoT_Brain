//! Control plane commands per Architecture Chapter 28.
//! Defines commands that can be executed through the developer interface.

use crate::control_plane::permissions::PermissionLevel;

/// A control plane command.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPlaneCommand {
    /// Command name.
    pub name: String,
    /// Command description.
    pub description: String,
    /// Required permission level.
    pub required_level: PermissionLevel,
}

impl ControlPlaneCommand {
    /// Create a new command.
    pub fn new(name: &str, description: &str, required_level: PermissionLevel) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required_level,
        }
    }
}

/// Built-in control plane commands.
pub struct BuiltInCommands;

impl BuiltInCommands {
    /// Get the list of built-in commands.
    pub fn list() -> Vec<ControlPlaneCommand> {
        vec![
            ControlPlaneCommand::new(
                "get_state",
                "Retrieve current system state",
                PermissionLevel::ReadOnly,
            ),
            ControlPlaneCommand::new(
                "get_metrics",
                "Retrieve system metrics",
                PermissionLevel::ReadOnly,
            ),
            ControlPlaneCommand::new(
                "health_check",
                "Run system health check",
                PermissionLevel::Operator,
            ),
            ControlPlaneCommand::new(
                "reload_config",
                "Reload system configuration",
                PermissionLevel::Admin,
            ),
        ]
    }
}
