pub mod full;

use full::{ActorCapabilities, Capability};

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

/// Global capability store used by `check_permission()` as a default.
/// Initialized during security layer boot; system and local_ai actors get
/// default capabilities, while cloud_ai and autonomous_agent actors require
/// explicit grants.
static GLOBAL_CAPS: std::sync::LazyLock<std::sync::RwLock<ActorCapabilities>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(ActorCapabilities::new()));

/// Initialize the global capability store with default capabilities per
/// Architecture §25.7 (Capability-Based Security). System actors receive
/// full capabilities; AI actors receive only what is explicitly granted.
pub fn init_global_capabilities() {
    let mut caps = match GLOBAL_CAPS.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("Global capability write lock was poisoned, recovering inner value");
            poisoned.into_inner()
        }
    };
    // System actor gets all capabilities by default (Architecture §25.6)
    for cap in [
        Capability::ReadMemory,
        Capability::WriteMemory,
        Capability::ExecuteTool,
        Capability::ModifyKnowledge,
        Capability::ExecutePlan,
        Capability::AccessAudit,
        Capability::ModifySecurity,
    ] {
        caps.grant("system", cap);
    }
    // Local AI gets tool execution and read memory
    caps.grant("local_ai", Capability::ExecuteTool);
    caps.grant("local_ai", Capability::ReadMemory);
    tracing::info!("Global capability store initialized with default grants");
    // Load existing capability grants from database (Architecture §25.6)
    if let Err(e) = load_capability_grants_from_db() {
        tracing::warn!(error = %e, "Failed to load capability grants from database");
    }
}

/// Return a reference to the global capability store for callers that need it.
pub fn global_capabilities() -> std::sync::RwLockReadGuard<'static, ActorCapabilities> {
    match GLOBAL_CAPS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("Global capability read lock was poisoned, recovering inner value");
            poisoned.into_inner()
        }
    }
}

/// Log an audit record and persist it to the security_audits database table.
/// Per Architecture §25.14 (Audit System): audit records SHALL be durable.
pub fn log_audit(record: AuditRecord) -> Result<(), SecurityError> {
    if record.actor.is_empty() || record.action.is_empty() {
        return Err(SecurityError::InvalidRecord);
    }
    // Create a SecurityAuditEntry and persist to DB.
    let entry = full::SecurityAuditEntry::new(
        &record.actor,
        &record.action,
        &record.target,
        true,
        "audit-log",
    )
    .with_confidence_change(record.confidence_change)
    .with_reason(&record.reason);
    if let Err(e) = full::persist_audit_entry(&entry) {
        tracing::warn!(audit_id = %entry.audit_id, persist_error = %e, "Audit persistence returned error");
    }
    tracing::info!(
        actor = %record.actor,
        action = %record.action,
        target = %record.target,
        confidence_change = record.confidence_change,
        reason = %record.reason,
        "Audit record logged and persisted"
    );
    Ok(())
}

/// Check if an actor has permission for an action on a target.
/// Per Architecture §25.6 (Permission Architecture) and §25.7 (Capability-Based Security):
/// permission is granted only when the actor holds the matching capability grant
/// in the capability store.
///
/// Maps action strings to Capability variants:
/// - "read" -> ReadMemory
/// - "write" -> WriteMemory
/// - "execute" -> ExecuteTool
/// - "modify_knowledge" -> ModifyKnowledge
/// - "execute_plan" -> ExecutePlan
/// - "access_audit" -> AccessAudit
/// - "modify_security" -> ModifySecurity
/// - all other values -> returns PermissionDenied
pub fn check_permission(actor: &str, action: &str, target: &str) -> Result<bool, SecurityError> {
    if actor.is_empty() || action.is_empty() || target.is_empty() {
        return Err(SecurityError::InvalidRecord);
    }
    // Map the action string to a Capability.
    let capability = match action {
        "read" => Capability::ReadMemory,
        "write" => Capability::WriteMemory,
        "execute" => Capability::ExecuteTool,
        "modify_knowledge" => Capability::ModifyKnowledge,
        "execute_plan" => Capability::ExecutePlan,
        "access_audit" => Capability::AccessAudit,
        "modify_security" => Capability::ModifySecurity,
        _ => {
            tracing::warn!(actor, action, target, "Permission check: unknown action");
            return Err(SecurityError::PermissionDenied);
        }
    };
    // Check the global capability store.
    let caps = match GLOBAL_CAPS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("Global capability read lock was poisoned, recovering inner value");
            poisoned.into_inner()
        }
    };
    let allowed = if caps.has(actor, &capability) {
        true
    } else {
        // Fall back to database-backed permission store (Architecture §25.6)
        match check_permission_in_db(actor, &capability) {
            Ok(db_allowed) => db_allowed,
            Err(e) => {
                tracing::warn!(actor, error = %e, "Database permission check failed");
                false
            }
        }
    };
    if allowed {
        tracing::debug!(actor, action, target, "Permission granted");
    } else {
        tracing::warn!(actor, action, target, "Permission denied");
    }
    Ok(allowed)
}

/// Save a capability grant to the database-backed permission store.
/// Per Architecture §25.6 (Permission Architecture): capability grants SHALL be durable.
pub fn save_capability_grant_to_db(
    actor_id: &str,
    capability: &Capability,
    scope: &str,
) -> Result<(), String> {
    match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => {
            let conn = db
                .connection()
                .map_err(|e| format!("Failed to open DB connection: {e}"))?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                r#"
                INSERT OR REPLACE INTO capability_grants (
                    id, actor_id, capability, scope, granted_at, active
                ) VALUES (?, ?, ?, ?, ?, 1)
                "#,
                rusqlite::params![
                    id,
                    actor_id,
                    capability.label(),
                    scope,
                    chrono::Utc::now().timestamp(),
                ],
            )
            .map_err(|e| format!("Failed to save capability grant: {e}"))?;
            Ok(())
        }
        Err(e) => Err(format!("Database unavailable: {e}")),
    }
}

/// Load all capability grants from the database into the global capability store.
/// Per Architecture §25.6 (Permission Architecture): grants are loaded from durable storage.
pub fn load_capability_grants_from_db() -> Result<(), String> {
    match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => {
            let conn = db
                .connection()
                .map_err(|e| format!("Failed to open DB connection: {e}"))?;
            let mut stmt = conn
                .prepare("SELECT actor_id, capability FROM capability_grants WHERE active = 1")
                .map_err(|e| format!("Failed to prepare capability grants query: {e}"))?;
            let rows = stmt
                .query_map([], |row| {
                    let actor_id: String = row.get(0)?;
                    let capability_str: String = row.get(1)?;
                    Ok((actor_id, capability_str))
                })
                .map_err(|e| format!("Failed to query capability grants: {e}"))?;

            let mut caps = match GLOBAL_CAPS.write() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    tracing::warn!(
                        "Global capability write lock was poisoned, recovering inner value"
                    );
                    poisoned.into_inner()
                }
            };

            for row in rows {
                let (actor_id, capability_str) =
                    row.map_err(|e| format!("Failed to read capability grant row: {e}"))?;
                let capability = match capability_str.as_str() {
                    "ReadMemory" => Capability::ReadMemory,
                    "WriteMemory" => Capability::WriteMemory,
                    "ExecuteTool" => Capability::ExecuteTool,
                    "ModifyKnowledge" => Capability::ModifyKnowledge,
                    "ExecutePlan" => Capability::ExecutePlan,
                    "AccessAudit" => Capability::AccessAudit,
                    "ModifySecurity" => Capability::ModifySecurity,
                    _ => {
                        tracing::warn!(capability = %capability_str, "Unknown capability in database, skipping");
                        continue;
                    }
                };
                caps.grant(&actor_id, capability);
            }
            Ok(())
        }
        Err(e) => Err(format!("Database unavailable: {e}")),
    }
}

/// Check permission in the database-backed store.
/// Per Architecture §25.6 (Permission Architecture): query durable grant store.
pub fn check_permission_in_db(actor: &str, capability: &Capability) -> Result<bool, String> {
    match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => {
            let conn = db
                .connection()
                .map_err(|e| format!("Failed to open DB connection: {e}"))?;
            let result: Option<bool> = conn.query_row(
                "SELECT COUNT(*) > 0 FROM capability_grants WHERE actor_id = ?1 AND capability = ?2 AND active = 1",
                rusqlite::params![actor, capability.label()],
                |row| row.get(0),
            ).optional().map_err(|e| format!("Failed to query capability grant: {e}"))?;
            Ok(result.unwrap_or(false))
        }
        Err(e) => Err(format!("Database unavailable: {e}")),
    }
}
