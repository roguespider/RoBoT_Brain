//! Control plane permissions per Architecture Chapter 28.
//! Manages access control for developer interface operations.

/// Permission level for control plane operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    /// Read-only access to system state.
    ReadOnly,
    /// Read and limited write access.
    Operator,
    /// Full administrative access.
    Admin,
}

/// A permission check result.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionCheck {
    /// Whether the operation is allowed.
    pub allowed: bool,
    /// Required permission level.
    pub required: PermissionLevel,
    /// Reason for denial, if any.
    pub reason: Option<String>,
}

impl PermissionCheck {
    /// Create an allowed check.
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            required: PermissionLevel::ReadOnly,
            reason: None,
        }
    }

    /// Create a denied check.
    pub fn denied(required: PermissionLevel, reason: &str) -> Self {
        Self {
            allowed: false,
            required,
            reason: Some(reason.to_string()),
        }
    }
}

/// Permission checker for control plane operations.
pub struct PermissionChecker {
    /// Required level for state inspection.
    pub inspect_level: PermissionLevel,
    /// Required level for state modification.
    pub modify_level: PermissionLevel,
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self {
            inspect_level: PermissionLevel::ReadOnly,
            modify_level: PermissionLevel::Operator,
        }
    }
}

impl PermissionChecker {
    /// Check if the given level can inspect state.
    pub fn can_inspect(&self, level: PermissionLevel) -> PermissionCheck {
        if level as u8 >= self.inspect_level as u8 {
            PermissionCheck::allowed()
        } else {
            PermissionCheck::denied(
                self.inspect_level,
                "Insufficient permission to inspect state",
            )
        }
    }

    /// Check if the given level can modify state.
    pub fn can_modify(&self, level: PermissionLevel) -> PermissionCheck {
        if level as u8 >= self.modify_level as u8 {
            PermissionCheck::allowed()
        } else {
            PermissionCheck::denied(self.modify_level, "Insufficient permission to modify state")
        }
    }
}
