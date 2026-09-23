pub mod full;

/// Security error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    AuditFailed(String),
    PermissionDenied,
    InvalidRecord,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::AuditFailed(msg) => write!(f, "audit failed: {}", msg),
            SecurityError::PermissionDenied => write!(f, "permission denied"),
            SecurityError::InvalidRecord => write!(f, "invalid audit record"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Audit record for security tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditRecord {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub confidence_change: f32,
    pub reason: String,
}

/// Log an audit record.
pub fn log_audit(record: AuditRecord) -> Result<(), SecurityError> {
    if record.actor.is_empty() || record.action.is_empty() {
        return Err(SecurityError::InvalidRecord);
    }
    tracing::info!(
        actor = %record.actor,
        action = %record.action,
        target = %record.target,
        confidence_change = record.confidence_change,
        reason = %record.reason,
        "Audit record logged"
    );
    Ok(())
}

/// Check if an actor has permission for an action on a target.
pub fn check_permission(actor: &str, action: &str, target: &str) -> bool {
    // Placeholder: in production this checks permission store
    tracing::debug!(actor, action, target, "Permission check");
    !actor.is_empty() && !action.is_empty() && !target.is_empty()
}
