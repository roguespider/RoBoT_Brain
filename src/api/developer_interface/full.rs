//! Developer Interface — Cognitive explorer, memory management, worker controls,
//! debugging interfaces, learning/evolution interface (Architecture Chapter 28 — expanded).
//!
//! Per Architecture §28.6-28.10:
//! - Cognitive explorer: inspect memory, knowledge graph, experience, planning (§28.6)
//! - Memory management interface: memory inspection, promotion, archive (§28.7)
//! - Worker management interface: worker status, scheduling, recovery (§28.9)
//! - Learning and evolution interface: strategic objectives, hypothesis status (§28.10)
//! - Debugging tools: trace replay, state inspection, event search (§28.14)
//! - Wiring: developer_interface/ -> memory/ + knowledge/ + experience/ +
//!   planner/ + cooboploop/ + workers/ + tracing/

use rusqlite::params;

/// Cognitive explorer: inspect cognitive subsystems.
/// Per Architecture §28.6 (Cognitive Explorer).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExplorerTarget {
    /// Inspect memory subsystem.
    Memory,
    /// Inspect knowledge graph.
    KnowledgeGraph,
    /// Inspect experience engine.
    Experience,
    /// Inspect planning engine.
    Planning,
    /// Inspect execution engine.
    Execution,
    /// Inspect learning engine.
    Learning,
    /// Inspect strategic objectives.
    StrategicObjectives,
    /// Inspect background workers.
    BackgroundWorkers,
    /// Inspect confidence system.
    ConfidenceSystem,
}

impl ExplorerTarget {
    /// Return target label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Memory => "Memory",
            Self::KnowledgeGraph => "KnowledgeGraph",
            Self::Experience => "Experience",
            Self::Planning => "Planning",
            Self::Execution => "Execution",
            Self::Learning => "Learning",
            Self::StrategicObjectives => "StrategicObjectives",
            Self::BackgroundWorkers => "BackgroundWorkers",
            Self::ConfidenceSystem => "ConfidenceSystem",
        }
    }
}

/// Cognitive explorer result.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplorerResult {
    /// Target inspected.
    pub target: ExplorerTarget,
    /// Inspection summary.
    pub summary: String,
    /// Details found.
    pub details: Vec<String>,
    /// Timestamp.
    pub timestamp: i64,
}

impl ExplorerResult {
    /// Create a new explorer result.
    pub fn new(target: ExplorerTarget, summary: &str) -> Self {
        Self {
            target,
            summary: summary.to_string(),
            details: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add detail.
    pub fn add_detail(&mut self, detail: &str) {
        self.details.push(detail.to_string());
    }
}

/// Memory management interface: inspect and manage memory.
/// Per Architecture §28.7 (Memory Management Interface).
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManagementResult {
    /// Memory layer inspected.
    pub layer: String,
    /// Records found.
    pub record_count: usize,
    /// Promotion candidates.
    pub promotion_candidates: Vec<String>,
    /// Archive candidates.
    pub archive_candidates: Vec<String>,
    /// Timestamp.
    pub timestamp: i64,
}

impl MemoryManagementResult {
    /// Create a new memory management result.
    pub fn new(layer: &str, record_count: usize) -> Self {
        Self {
            layer: layer.to_string(),
            record_count,
            promotion_candidates: Vec::new(),
            archive_candidates: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add promotion candidate.
    pub fn add_promotion(&mut self, id: &str) {
        self.promotion_candidates.push(id.to_string());
    }

    /// Add archive candidate.
    pub fn add_archive(&mut self, id: &str) {
        self.archive_candidates.push(id.to_string());
    }
}

/// Inspect memory by layer — queries the real memories table.
/// Per Architecture §28.7 (Memory Management Interface).
pub fn inspect_memory_by_layer(layer: &str) -> MemoryManagementResult {
    tracing::debug!(layer, "Inspecting memory layer via database");
    let mut result = MemoryManagementResult::new(layer, 0);
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for memory inspection");
            return result;
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for memory inspection");
            return result;
        }
    };
    // Count records in the memories table.
    let count: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE memory_type = ?",
        params![layer],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(layer, query_error = %e, "Memory count query failed");
            return result;
        }
    };
    result.record_count = count as usize;
    // Find promotion candidates (high-confidence, low-access records).
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id FROM memories WHERE confidence > 0.8 AND access_count < 5 ORDER BY confidence DESC LIMIT 10"
    ) &&
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
    {
        for row in rows.filter_map(|r| r.ok()) {
            result.add_promotion(&row);
        }
    }
    // Find archive candidates (old, low-importance records).
    if let Ok(mut stmt) = conn
        .prepare("SELECT id FROM memories WHERE importance < 0.3 ORDER BY created_at ASC LIMIT 10")
        && let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
    {
        for row in rows.filter_map(|r| r.ok()) {
            result.add_archive(&row);
        }
    }
    tracing::debug!(
        layer,
        record_count = result.record_count,
        "Memory layer inspected"
    );
    tracing::debug!(
        layer,
        record_count = result.record_count,
        "Memory layer inspected"
    );
    result
}

/// Inspect strategic learning state.
/// Per Architecture §28.10 (Learning and Evolution Interface).
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicInspectionResult {
    /// Active objectives.
    pub active_objectives: Vec<String>,
    /// Completed objectives.
    pub completed_objectives: Vec<String>,
    /// Pending updates.
    pub pending_updates: Vec<String>,
    /// Feedback count.
    pub feedback_count: usize,
    /// Timestamp.
    pub timestamp: i64,
}

impl Default for StrategicInspectionResult {
    fn default() -> Self {
        Self {
            active_objectives: Vec::new(),
            completed_objectives: Vec::new(),
            pending_updates: Vec::new(),
            feedback_count: 0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl StrategicInspectionResult {
    /// Create a new strategic inspection result.
    pub fn new() -> Self {
        Self {
            active_objectives: Vec::new(),
            completed_objectives: Vec::new(),
            pending_updates: Vec::new(),
            feedback_count: 0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add active objective.
    pub fn add_active(&mut self, id: &str) {
        self.active_objectives.push(id.to_string());
    }

    /// Add completed objective.
    pub fn add_completed(&mut self, id: &str) {
        self.completed_objectives.push(id.to_string());
    }

    /// Add pending update.
    pub fn add_pending_update(&mut self, id: &str) {
        self.pending_updates.push(id.to_string());
    }

    /// Set feedback count.
    pub fn set_feedback_count(&mut self, count: usize) {
        self.feedback_count = count;
    }
}

/// Inspect strategic learning state — queries real data from knowledge and experience tables.
/// Per Architecture §28.10 (Learning and Evolution Interface).
pub fn inspect_strategic_learning() -> StrategicInspectionResult {
    tracing::debug!("Inspecting strategic learning state via database");
    let mut result = StrategicInspectionResult::new();
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for strategic inspection");
            return result;
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for strategic inspection");
            return result;
        }
    };
    // Count active knowledge nodes.
    if let Ok(active) = conn.query_row(
        "SELECT COUNT(*) FROM knowledge_graph WHERE status = 'active'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        for i in 0..(active.min(10) as usize) {
            if let Ok(id) = conn.query_row(
                "SELECT id FROM knowledge_graph WHERE status = 'active' LIMIT 1 OFFSET ?",
                params![i as i32],
                |row| row.get::<_, String>(0),
            ) {
                result.add_active(&id);
            }
        }
    }
    // Count completed experiences.
    if let Ok(completed) = conn.query_row(
        "SELECT COUNT(*) FROM experiences WHERE outcome = 'solved'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        result.set_feedback_count(completed as usize);
    }
    // Count pending learning updates.
    if let Ok(pending) = conn.query_row(
        "SELECT COUNT(*) FROM learning WHERE status = 'pending'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        for i in 0..(pending.min(5) as usize) {
            if let Ok(id) = conn.query_row(
                "SELECT id FROM learning WHERE status = 'pending' LIMIT 1 OFFSET ?",
                params![i as i32],
                |row| row.get::<_, String>(0),
            ) {
                result.add_pending_update(&id);
            }
        }
    }
    tracing::debug!(
        active = result.active_objectives.len(),
        feedback = result.feedback_count,
        "Strategic learning inspection completed"
    );
    result
}

/// Inspect background workers — queries real worker data from the database.
/// Per Architecture §28.9 (Worker Management Interface).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerInspectionResult {
    /// Running workers.
    pub running_workers: Vec<String>,
    /// Pending tasks.
    pub pending_tasks: Vec<String>,
    /// Completed tasks (history count).
    pub completed_tasks: usize,
    /// Capacity used (percentage).
    pub capacity_used_percent: f32,
    /// Timestamp.
    pub timestamp: i64,
}

impl Default for WorkerInspectionResult {
    fn default() -> Self {
        Self {
            running_workers: Vec::new(),
            pending_tasks: Vec::new(),
            completed_tasks: 0,
            capacity_used_percent: 0.0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl WorkerInspectionResult {
    /// Create a new worker inspection result.
    pub fn new() -> Self {
        Self {
            running_workers: Vec::new(),
            pending_tasks: Vec::new(),
            completed_tasks: 0,
            capacity_used_percent: 0.0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add running worker.
    pub fn add_running(&mut self, id: &str) {
        self.running_workers.push(id.to_string());
    }

    /// Add pending task.
    pub fn add_pending(&mut self, id: &str) {
        self.pending_tasks.push(id.to_string());
    }

    /// Set completed count.
    pub fn set_completed(&mut self, count: usize) {
        self.completed_tasks = count;
    }

    /// Set capacity used.
    pub fn set_capacity(&mut self, percent: f32) {
        self.capacity_used_percent = percent.clamp(0.0, 100.0);
    }
}

/// Inspect background workers — queries real worker data from the database.
/// Per Architecture §28.9 (Worker Management Interface).
pub fn inspect_background_workers() -> WorkerInspectionResult {
    tracing::debug!("Inspecting background workers via database");
    let mut result = WorkerInspectionResult::new();
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for worker inspection");
            return result;
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for worker inspection");
            return result;
        }
    };
    // List running workers from the scheduled_tasks table.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT task_id FROM scheduled_tasks WHERE status = 'running' ORDER BY created_at DESC LIMIT 20"
    ) &&
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
    {
        for row in rows.filter_map(|r| r.ok()) {
            result.add_running(&row);
        }
    }
    // List pending tasks.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT task_id FROM scheduled_tasks WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20"
    ) &&
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
    {
        for row in rows.filter_map(|r| r.ok()) {
            result.add_pending(&row);
        }
    }
    // Count completed tasks.
    if let Ok(completed) = conn.query_row(
        "SELECT COUNT(*) FROM scheduled_tasks WHERE status = 'completed'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        result.set_completed(completed as usize);
    }
    // Calculate capacity used (running / total slots).
    if let Ok(total) = conn.query_row("SELECT COUNT(*) FROM scheduled_tasks", [], |row| {
        row.get::<_, i64>(0)
    }) && total > 0
    {
        let running_count = result.running_workers.len() as i64;
        result.set_capacity((running_count as f64 / total as f64 * 100.0) as f32);
    }
    tracing::debug!(
        running = result.running_workers.len(),
        pending = result.pending_tasks.len(),
        completed = result.completed_tasks,
        capacity_used = result.capacity_used_percent,
        "Worker inspection completed"
    );
    result
}

/// Cognitive explorer: inspect all cognitive subsystems via real database queries.
/// Per Architecture §28.6 (Cognitive Explorer).
pub fn explore_cognitive_system(target: ExplorerTarget) -> ExplorerResult {
    let target_label = target.label();
    let summary = format!("Explored: {}", target_label);
    let mut result = ExplorerResult::new(target.clone(), &summary);
    result.add_detail(&format!("Subsystem: {}", target_label));
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for cognitive exploration");
            result.add_detail("Status: DB unavailable");
            return result;
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for cognitive exploration");
            result.add_detail("Status: DB connection failed");
            return result;
        }
    };
    // Provide real status info based on the target.
    match target {
        ExplorerTarget::Memory => {
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM memories", [], |row| {
                row.get::<_, i64>(0)
            }) {
                result.add_detail(&format!("Records: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::KnowledgeGraph => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM knowledge_graph WHERE status = 'active'",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Active nodes: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::Experience => {
            if let Ok(count) = conn.query_row("SELECT COUNT(*) FROM experiences", [], |row| {
                row.get::<_, i64>(0)
            }) {
                result.add_detail(&format!("Experiences: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::Planning => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM planning WHERE status = 'active'",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Active plans: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::Execution => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM execution WHERE status IN ('running', 'pending')",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Active tasks: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::Learning => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM learning WHERE status = 'pending'",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Pending updates: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::StrategicObjectives => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM strategic_objectives WHERE status = 'active'",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Active objectives: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::BackgroundWorkers => {
            if let Ok(count) = conn.query_row(
                "SELECT COUNT(*) FROM scheduled_tasks WHERE status = 'running'",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                result.add_detail(&format!("Running workers: {}", count));
            }
            result.add_detail("Status: connected");
        }
        ExplorerTarget::ConfidenceSystem => {
            if let Ok(count) =
                conn.query_row("SELECT COUNT(*) FROM confidence_history", [], |row| {
                    row.get::<_, i64>(0)
                })
            {
                result.add_detail(&format!("Confidence records: {}", count));
            }
            result.add_detail("Status: connected");
        }
    }
    tracing::debug!(target = %target_label, "Cognitive exploration completed");
    result
}

/// Memory management interface: inspect and manage memory layers.
/// Per Architecture §28.7 (Memory Management Interface).
/// Supports actions: inspect (default), promote, archive.
pub fn manage_memory_layer(layer: &str, action: &str) -> MemoryManagementResult {
    tracing::debug!(layer, action, "Memory management via database");
    match action {
        "promote" => {
            let mut result = MemoryManagementResult::new(layer, 0);
            let db = match crate::database::sqlite::SqliteDatabase::initialize() {
                Ok(db) => db,
                Err(e) => {
                    tracing::warn!(db_error = %e, "DB unavailable for memory promotion");
                    return result;
                }
            };
            if let Ok(conn) = db.connection() &&
                let Ok(mut stmt) = conn.prepare(
                    "SELECT id FROM memories WHERE layer = ? AND confidence > 0.7 ORDER BY confidence DESC LIMIT 5"
                ) &&
                let Ok(rows) = stmt.query_map(params![layer], |row| row.get::<_, String>(0))
            {
                for row in rows.filter_map(|r| r.ok()) {
                    result.add_promotion(&row);
                }
            }
            result
        }
        "archive" => {
            let mut result = MemoryManagementResult::new(layer, 0);
            let db = match crate::database::sqlite::SqliteDatabase::initialize() {
                Ok(db) => db,
                Err(e) => {
                    tracing::warn!(db_error = %e, "DB unavailable for memory archival");
                    return result;
                }
            };
            if let Ok(conn) = db.connection() &&
                let Ok(mut stmt) = conn.prepare(
                    "SELECT id FROM memories WHERE importance < 0.3 ORDER BY created_at ASC LIMIT 5"
                ) &&
                let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0))
            {
                for row in rows.filter_map(|r| r.ok()) {
                    result.add_archive(&row);
                }
            }
            result
        }
        _ => inspect_memory_by_layer(layer),
    }
}

/// Learning and evolution interface: inspect strategic state.
/// Per Architecture §28.10 (Learning and Evolution Interface).
pub fn inspect_learning_evolution() -> StrategicInspectionResult {
    // Delegate to inspect_strategic_learning for real data.
    inspect_strategic_learning()
}
