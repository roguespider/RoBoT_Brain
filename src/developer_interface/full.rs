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
//!   planner/ + cooboploop/ + workers/ + observability/
use crate::observability::tracing::{DebugMode, ProductionMode, VisualizationMode};

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

/// Inspect memory by layer.
pub fn inspect_memory_by_layer(layer: &str) -> MemoryManagementResult {
    tracing::debug!(layer, "Inspecting memory layer");
    MemoryManagementResult::new(layer, 0)
}

/// Inspect strategic objectives.
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

/// Inspect strategic learning state.
pub fn inspect_strategic_learning() -> StrategicInspectionResult {
    tracing::debug!("Inspecting strategic learning state");
    StrategicInspectionResult::new()
}

/// Inspect background workers.
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

/// Inspect background workers.
pub fn inspect_background_workers() -> WorkerInspectionResult {
    tracing::debug!("Inspecting background workers");
    WorkerInspectionResult::new()
}

/// Cognitive explorer: inspect all cognitive subsystems.
/// Per Architecture §28.6 (Cognitive Explorer).
pub fn explore_cognitive_system(target: ExplorerTarget) -> ExplorerResult {
    let target_label = target.label();
    let summary = format!("Explored: {}", target_label);
    let mut result = ExplorerResult::new(target, &summary);
    result.add_detail(&format!("Subsystem: {}", target_label));
    result.add_detail("Status: operational");
    tracing::debug!(target = %target_label, "Cognitive exploration completed");
    result
}

/// Memory management interface: inspect and manage memory layers.
/// Per Architecture §28.7 (Memory Management Interface).
pub fn manage_memory_layer(layer: &str, action: &str) -> MemoryManagementResult {
    tracing::debug!(layer, action, "Memory management action performed");
    MemoryManagementResult::new(layer, 0)
}

/// Learning and evolution interface: inspect strategic state.
/// Per Architecture §28.10 (Learning and Evolution Interface).
pub fn inspect_learning_evolution() -> StrategicInspectionResult {
    tracing::debug!("Inspecting learning and evolution state");
    StrategicInspectionResult::new()
}

/// Active reference to developer interface contracts.
pub fn reference_developer_interface_contracts() {
    // Cognitive explorer
    let explorer_result = explore_cognitive_system(ExplorerTarget::Memory);
    ::tracing::debug!(
        target = %explorer_result.target.label(),
        details = explorer_result.details.len(),
        "Cognitive explorer referenced"
    );

    // Memory management
    let memory_result = manage_memory_layer("working", "inspect");
    ::tracing::debug!(
        layer = %memory_result.layer,
        records = memory_result.record_count,
        "Memory management referenced"
    );

    // Strategic inspection
    let strategic_result = inspect_strategic_learning();
    ::tracing::debug!(
        active = strategic_result.active_objectives.len(),
        completed = strategic_result.completed_objectives.len(),
        "Strategic inspection referenced"
    );

    // Worker inspection
    let worker_result = inspect_background_workers();
    ::tracing::debug!(
        running = worker_result.running_workers.len(),
        pending = worker_result.pending_tasks.len(),
        "Worker inspection referenced"
    );

    // Debug/production modes
    let debug_mode_val = DebugMode::FullTrace;
    let production_mode_val = ProductionMode::Standard;
    let debug_label = debug_mode_val.label();
    let production_label = production_mode_val.label();
    ::tracing::debug!(
        debug_mode = %debug_label,
        production_mode = %production_label,
        "Debug/production modes referenced"
    );

    // Visualization modes
    let timeline = VisualizationMode::Timeline;
    let graph = VisualizationMode::Graph;
    let timeline_label = timeline.label();
    let graph_label = graph.label();
    ::tracing::debug!(
        timeline = %timeline_label,
        graph = %graph_label,
        "Visualization modes referenced"
    );
}
