//! Developer Interface and Control Plane (Architecture Chapter 28).
//!
//! Provides visibility and control without direct manipulation of
//! internal subsystem state.
pub mod full;

use uuid::Uuid;

/// A control plane command.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlCommand {
    /// Command identifier.
    pub id: String,
    /// Command name.
    pub name: String,
    /// Parameters.
    pub parameters: std::collections::HashMap<String, String>,
    /// Timestamp.
    pub timestamp: i64,
}

impl ControlCommand {
    /// Create a new command.
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            parameters: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add a parameter.
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Control plane architecture.
#[derive(Debug, Clone, Default)]
pub struct ControlPlaneArchitecture {
    pub cli: String,
    pub dashboard: String,
    pub api: String,
}

/// Inspect memory by entity ID — queries the real memory store.
/// Per Architecture §28.7 (Memory Management Interface).
pub fn inspect_memory(entity_id: &str) -> Option<crate::memory::types::MemoryItem> {
    if entity_id.is_empty() {
        return None;
    }
    tracing::debug!(entity_id, "Inspecting memory by entity ID");
    // Query the memories table for a matching record.
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for memory inspection");
            return None;
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for memory inspection");
            return None;
        }
    };
    let item = conn.query_row(
        "SELECT id, content, memory_type, confidence, importance, created_at, source \
         FROM memories WHERE id = ?1 OR content LIKE ?2 \
         ORDER BY created_at DESC LIMIT 1",
        [entity_id, &format!("%{}%", entity_id)],
        |row| {
            let id_str = row.get::<_, String>(0).unwrap_or_default();
            let content = row.get::<_, String>(1).unwrap_or_default();
            let memory_type_str = row.get::<_, String>(2).unwrap_or_default();
            let confidence = row.get::<_, f32>(3).unwrap_or(0.5);
            let importance = row.get::<_, f32>(4).unwrap_or(0.5);
            let created_at_str = row.get::<_, String>(5).unwrap_or_default();
            let source = row.get::<_, String>(6).unwrap_or_default();

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let memory_type = match memory_type_str.as_str() {
                "experience" => crate::memory::types::MemoryType::Experience,
                "concept" => crate::memory::types::MemoryType::Concept,
                "knowledge" => crate::memory::types::MemoryType::Knowledge,
                "skill" => crate::memory::types::MemoryType::Skill,
                "workflow" => crate::memory::types::MemoryType::Workflow,
                "context" => crate::memory::types::MemoryType::Context,
                "observation" => crate::memory::types::MemoryType::Observation,
                _ => crate::memory::types::MemoryType::Knowledge,
            };

            Ok(crate::memory::types::MemoryItem {
                id,
                layer: crate::memory::types::MemoryLayer::LongTerm,
                memory_type,
                status: crate::memory::types::MemoryStatus::Active,
                content,
                title: None,
                confidence,
                importance,
                created_at,
                accessed_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                last_consolidated: None,
                access_count: 0,
                lifecycle: crate::memory::lifecycle::MemoryLifecycle::Working,
                tags: Vec::new(),
                source,
                related_ids: Vec::new(),
            })
        },
    );
    match item {
        Ok(mem) => {
            tracing::debug!(entity_id, "Memory inspection found record");
            Some(mem)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            tracing::debug!(entity_id, "No memory record found");
            None
        }
        Err(e) => {
            tracing::warn!(db_error = %e, "Memory inspection query failed");
            None
        }
    }
}

/// Replay trace by correlation ID — queries real cognitive_events table.
/// Per Architecture §27.10 (Trace Storage).
pub fn replay_trace(correlation_id: &str) -> Vec<String> {
    if correlation_id.is_empty() {
        return Vec::new();
    }
    tracing::debug!(correlation_id, "Replaying trace");
    // Query the cognitive_events table for matching events.
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for trace replay");
            return Vec::new();
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for trace replay");
            return Vec::new();
        }
    };
    let mut stmt = match conn.prepare(
        "SELECT id, event_type, description, timestamp FROM cognitive_events WHERE correlation_id = ? ORDER BY timestamp ASC"
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(db_error = %e, "Trace replay prepare failed");
            return Vec::new();
        }
    };
    let rows = match stmt.query_map([correlation_id], |row| {
        let id = row.get::<_, String>(0).unwrap_or_default();
        let description = row.get::<_, String>(2).unwrap_or_default();
        let timestamp = row.get::<_, i64>(3).unwrap_or(0);
        Ok(format!("[{}] {} (ts={})", id, description, timestamp))
    }) {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(e) => {
            tracing::warn!(db_error = %e, "Trace replay query failed");
            Vec::new()
        }
    };
    if !rows.is_empty() {
        tracing::debug!(
            correlation_id,
            found = rows.len(),
            "Trace replay found events"
        );
    }
    rows
}

/// The developer interface provides control plane access.
#[derive(Debug, Clone, Default)]
pub struct DeveloperInterface {
    /// Command history.
    commands: Vec<ControlCommand>,
}

impl DeveloperInterface {
    /// Create a new interface.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Execute a command.
    pub fn execute_command(&mut self, command: ControlCommand) {
        self.commands.push(command);
    }

    /// Get command history.
    pub fn get_history(&self) -> Vec<ControlCommand> {
        self.commands.clone()
    }
}
