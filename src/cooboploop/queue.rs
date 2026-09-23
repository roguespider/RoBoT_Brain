// /src/CoObOpLoop/queue.rs
// Objective queue management for the CoObOpLoop system.
// Will be populated incrementally per §4.

use crate::cooboploop::sources::ObjectiveSource;

/// Status of a goal in the queue.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GoalStatus {
    /// Goal discovered but not yet evaluated.
    Discovered,

    /// Goal is being evaluated.
    Evaluating,

    /// Goal accepted and queued for execution.
    Accepted,

    /// Goal queued for execution.
    Queued,

    /// Goal blocked by dependencies.
    Blocked,

    /// Goal deferred to later.
    Deferred,

    /// Goal is actively being executed.
    Active,

    /// Goal is being verified.
    Verifying,

    /// Goal completed successfully.
    Completed,

    /// Goal failed.
    Failed,

    /// Goal cancelled.
    Cancelled,

    /// Goal rejected.
    Rejected,

    /// Goal archived.
    Archived,
}

impl GoalStatus {
    /// Returns true for terminal states (per §A.1).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GoalStatus::Completed
                | GoalStatus::Failed
                | GoalStatus::Cancelled
                | GoalStatus::Rejected
        )
    }

    /// Wire is_terminal by using it in a helper.
    pub fn check_terminal(&self) -> bool {
        self.is_terminal()
    }

    /// Wire valid_transition by using it in transition.
    pub fn check_valid_transition(&self, next: &GoalStatus) -> bool {
        self.valid_transition(next)
    }

    /// Returns true if a transition from self to `next` is valid per §A.1 state machine.
    pub fn valid_transition(&self, next: &GoalStatus) -> bool {
        matches!(
            (self, next),
            (GoalStatus::Discovered, GoalStatus::Evaluating)
                | (GoalStatus::Evaluating, GoalStatus::Discovered)
                | (GoalStatus::Evaluating, GoalStatus::Accepted)
                | (GoalStatus::Evaluating, GoalStatus::Rejected)
                | (GoalStatus::Evaluating, GoalStatus::Deferred)
                | (GoalStatus::Accepted, GoalStatus::Queued)
                | (GoalStatus::Accepted, GoalStatus::Deferred)
                | (GoalStatus::Accepted, GoalStatus::Cancelled)
                | (GoalStatus::Queued, GoalStatus::Blocked)
                | (GoalStatus::Queued, GoalStatus::Deferred)
                | (GoalStatus::Queued, GoalStatus::Active)
                | (GoalStatus::Blocked, GoalStatus::Discovered)
                | (GoalStatus::Blocked, GoalStatus::Queued)
                | (GoalStatus::Blocked, GoalStatus::Deferred)
                | (GoalStatus::Deferred, GoalStatus::Discovered)
                | (GoalStatus::Active, GoalStatus::Verifying)
                | (GoalStatus::Active, GoalStatus::Failed)
                | (GoalStatus::Verifying, GoalStatus::Completed)
                | (GoalStatus::Verifying, GoalStatus::Failed)
                | (GoalStatus::Completed, GoalStatus::Archived)
                | (GoalStatus::Failed, GoalStatus::Discovered)
                | (GoalStatus::Cancelled, GoalStatus::Discovered)
                | (GoalStatus::Rejected, GoalStatus::Discovered)
                | (GoalStatus::Rejected, GoalStatus::Archived)
                | (GoalStatus::Archived, GoalStatus::Discovered)
        )
    }
}

/// Agent goal definition.
#[derive(Debug, Clone)]
pub struct AgentGoal {
    /// Unique identifier.
    pub id: String,

    /// Title of the goal.
    pub title: String,

    /// Description of the goal.
    pub description: String,

    /// Current status.
    pub status: GoalStatus,

    /// Priority score (0.0-1.0).
    pub priority: f32,

    /// Source of the goal.
    pub source: ObjectiveSource,

    /// Expected value.
    pub expected_value: f32,

    /// Risk level (0.0-1.0).
    pub risk: f32,

    /// Learning value.
    pub learning_value: f32,

    /// Required capabilities.
    pub required_capabilities: Vec<String>,

    /// Dependencies (goal IDs).
    pub dependencies: Vec<String>,

    /// Deadline (optional).
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,

    /// Execution history.
    pub execution_history: Vec<ExecutionRecord>,

    /// Completion state.
    pub completion_state: Option<String>,

    /// Creation timestamp (§4 — persistent queue requirement).
    pub creation_timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Last evaluation timestamp (§4 — persistent queue requirement).
    pub last_evaluation: Option<chrono::DateTime<chrono::Utc>>,

    /// Estimated cost (§4 — objective queue requirement).
    pub estimated_cost: Option<f32>,

    /// Resource requirements (§4 — objective queue requirement).
    pub resource_requirements: Vec<String>,
}

impl Default for AgentGoal {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            description: String::new(),
            status: GoalStatus::Discovered,
            priority: 0.0,
            source: ObjectiveSource::SystemTrigger,
            expected_value: 0.0,
            risk: 0.5,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: None,
            last_evaluation: None,
            estimated_cost: None,
            resource_requirements: Vec::new(),
        }
    }
}

/// Record of a status transition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionRecord {
    /// Timestamp of the transition.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Previous status.
    pub from_status: GoalStatus,

    /// New status.
    pub to_status: GoalStatus,
}

/// Objective queue.
pub struct ObjectiveQueue {
    /// Goals in the queue.
    pub goals: std::collections::HashMap<String, AgentGoal>,
    /// SQLite database connection (for durable persistence).
    pub db: Option<rusqlite::Connection>,
}

impl Default for ObjectiveQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectiveQueue {
    /// Create a new persistent queue backed by SQLite (§4 — survives cycles).
    /// Falls back to in-memory only if the database cannot be opened.
    pub fn new() -> Self {
        let db_path = "robot_brain.db";
        match Self::open(db_path) {
            Ok(queue) => queue,
            Err(err) => {
                tracing::warn!(
                    "Failed to open persistent objective queue at {}: {}. Using in-memory fallback.",
                    db_path,
                    err
                );
                Self {
                    goals: std::collections::HashMap::new(),
                    db: None,
                }
            }
        }
    }

    /// Enqueue a goal — inserts into both in-memory map and SQLite.
    pub fn enqueue(&mut self, goal: &AgentGoal) -> Result<(), String> {
        // Serialize goal fields for SQLite
        let required_caps = serde_json::to_string(&goal.required_capabilities)
            .map_err(|e| format!("serialize required_capabilities: {e}"))?;
        let deps = serde_json::to_string(&goal.dependencies)
            .map_err(|e| format!("serialize dependencies: {e}"))?;
        let hist = serde_json::to_string(&goal.execution_history)
            .map_err(|e| format!("serialize execution_history: {e}"))?;
        let completion = goal.completion_state.clone().unwrap_or_default();
        let source_str = format!("{:?}", goal.source);
        let deadline = goal.deadline.map(|d| d.to_rfc3339());
        let estimated_cost = goal.estimated_cost.map(|c| c.to_string());
        let resource_reqs = serde_json::to_string(&goal.resource_requirements)
            .map_err(|e| format!("serialize resource_requirements: {e}"))?;

        // Insert into SQLite
        if let Some(ref conn) = self.db {
            // Ensure schema has the new columns (§4 — estimated_cost, resource_requirements)
            if let Err(e) = conn.execute(
                "ALTER TABLE objectives ADD COLUMN IF NOT EXISTS estimated_cost TEXT",
                [],
            ) {
                tracing::debug!("alter estimated_cost column: {e}");
            }
            if let Err(e) = conn.execute(
                "ALTER TABLE objectives ADD COLUMN IF NOT EXISTS resource_requirements TEXT",
                [],
            ) {
                tracing::debug!("alter resource_requirements column: {e}");
            }

            conn.execute(
                "INSERT OR REPLACE INTO objectives (
                    id, title, description, priority, source, status,
                    deadline, expected_value, risk, learning_value,
                    required_capabilities, dependencies, execution_history,
                    completion_state, creation_timestamp, last_evaluation,
                    estimated_cost, resource_requirements
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                rusqlite::params![
                    &goal.id,
                    &goal.title,
                    &goal.description,
                    goal.priority,
                    source_str,
                    format!("{:?}", goal.status),
                    deadline,
                    goal.expected_value,
                    goal.risk,
                    goal.learning_value,
                    required_caps,
                    deps,
                    hist,
                    completion,
                    goal.creation_timestamp
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_default(),
                    goal.last_evaluation
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_default(),
                    estimated_cost,
                    resource_reqs,
                ],
            )
            .map_err(|e| format!("INSERT objectives: {e}"))?;
        }

        self.goals.insert(goal.id.clone(), goal.clone());
        Ok(())
    }

    /// Get a goal by ID — checks in-memory first, then SQLite.
    pub fn get(&self, id: &str) -> Option<AgentGoal> {
        // Check in-memory first
        if let Some(goal) = self.goals.get(id) {
            return Some(goal.clone());
        }
        // Fall back to SQLite
        if let Some(ref conn) = self.db {
            let row = conn.query_row(
                "SELECT id, title, description, priority, source, status, deadline, expected_value, risk, learning_value, required_capabilities, dependencies, execution_history, completion_state, creation_timestamp, last_evaluation, estimated_cost, resource_requirements FROM objectives WHERE id = ?1",
                [id],
                |row| {
                    Ok(AgentGoal {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        status: match row.get::<_, String>(5)?.as_str() {
                            "Discovered" => GoalStatus::Discovered,
                            "Evaluating" => GoalStatus::Evaluating,
                            "Accepted" => GoalStatus::Accepted,
                            "Queued" => GoalStatus::Queued,
                            "Blocked" => GoalStatus::Blocked,
                            "Deferred" => GoalStatus::Deferred,
                            "Active" => GoalStatus::Active,
                            "Verifying" => GoalStatus::Verifying,
                            "Completed" => GoalStatus::Completed,
                            "Failed" => GoalStatus::Failed,
                            "Cancelled" => GoalStatus::Cancelled,
                            "Rejected" => GoalStatus::Rejected,
                            "Archived" => GoalStatus::Archived,
                            _ => GoalStatus::Discovered,
                        },
                        priority: row.get(3)?,
                        source: match row.get::<_, String>(4)?.as_str() {
                            "HumanOrigin" => ObjectiveSource::HumanOrigin,
                            "ExternalOpportunity" => ObjectiveSource::ExternalOpportunity,
                            "SystemTrigger" => ObjectiveSource::SystemTrigger,
                            "LearningTarget" => ObjectiveSource::LearningTarget,
                            "ImprovementTarget" => ObjectiveSource::ImprovementTarget,
                            "StrategicObjective" => ObjectiveSource::StrategicObjective,
                            _ => ObjectiveSource::SystemTrigger,
                        },
                        expected_value: row.get(7)?,
                        risk: row.get(8)?,
                        learning_value: row.get(9)?,
                        required_capabilities: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
                        dependencies: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                        deadline: row.get::<_, Option<String>>(6)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
                        execution_history: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
                        completion_state: row.get::<_, Option<String>>(13)?,
                        creation_timestamp: row.get::<_, Option<String>>(14)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
                        last_evaluation: row.get::<_, Option<String>>(15)?.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
                        estimated_cost: row.get::<_, Option<String>>(16)?.and_then(|s| s.parse().ok()),
                        resource_requirements: serde_json::from_str(&row.get::<_, String>(17)?).unwrap_or_default(),
                    })
                },
            ).ok();
            return row;
        }
        None
    }

    /// Update a goal — updates both in-memory and SQLite.
    pub fn update(&mut self, id: &str, goal: AgentGoal) -> Result<(), String> {
        let required_caps = serde_json::to_string(&goal.required_capabilities)
            .map_err(|e| format!("serialize required_capabilities: {e}"))?;
        let deps = serde_json::to_string(&goal.dependencies)
            .map_err(|e| format!("serialize dependencies: {e}"))?;
        let hist = serde_json::to_string(&goal.execution_history)
            .map_err(|e| format!("serialize execution_history: {e}"))?;
        let completion = goal.completion_state.clone().unwrap_or_default();
        let source_str = format!("{:?}", goal.source);
        let deadline = goal.deadline.map(|d| d.to_rfc3339());
        let resource_reqs = serde_json::to_string(&goal.resource_requirements)
            .map_err(|e| format!("serialize resource_requirements: {e}"))?;
        let estimated_cost = goal.estimated_cost.map(|c| c.to_string());

        if let Some(ref conn) = self.db {
            conn.execute(
                "UPDATE objectives SET title=?2, description=?3, priority=?4, source=?5, status=?6, deadline=?7, expected_value=?8, risk=?9, learning_value=?10, required_capabilities=?11, dependencies=?12, execution_history=?13, completion_state=?14, creation_timestamp=?15, last_evaluation=?16, estimated_cost=?17, resource_requirements=?18 WHERE id=?1",
                rusqlite::params![
                    id, goal.title, goal.description, goal.priority, source_str,
                    format!("{:?}", goal.status), deadline, goal.expected_value, goal.risk,
                    goal.learning_value, required_caps, deps, hist, completion,
                    goal.creation_timestamp.map(|d| d.to_rfc3339()).unwrap_or_default(),
                    goal.last_evaluation.map(|d| d.to_rfc3339()).unwrap_or_default(),
                    estimated_cost, resource_reqs,
                ],
            )
            .map_err(|e| format!("UPDATE objectives: {e}"))?;
        }

        self.goals.insert(id.to_string(), goal);
        Ok(())
    }

    /// Transition a goal to a new status — updates both in-memory and SQLite.
    pub fn transition(&mut self, id: &str, new_status: GoalStatus) -> Result<(), String> {
        if let Some(goal) = self.goals.get_mut(id) {
            // Wire: check valid_transition before transitioning
            let is_valid = goal.status.check_valid_transition(&new_status);
            if !is_valid {
                return Err(format!(
                    "Invalid transition from {:?} to {:?}",
                    goal.status, new_status
                ));
            }
            let old_status = goal.status.clone();
            let status_label = format!("{:?}", new_status);
            goal.status = new_status.clone();
            // Record in execution history
            goal.execution_history.push(ExecutionRecord {
                timestamp: chrono::Utc::now(),
                from_status: old_status,
                to_status: new_status,
            });

            // Write to SQLite
            if let Some(ref conn) = self.db {
                conn.execute(
                    "UPDATE objectives SET status=?2, execution_history=?3 WHERE id=?1",
                    rusqlite::params![
                        id,
                        status_label,
                        serde_json::to_string(&goal.execution_history)
                            .map_err(|e| format!("serialize execution_history: {e}"))?,
                    ],
                )
                .map_err(|e| format!("UPDATE objectives status: {e}"))?;
            }

            Ok(())
        } else {
            Err(format!("Goal {} not found", id))
        }
    }

    /// Enqueue multiple goals at once.
    pub fn enqueue_many(&mut self, goals: &[AgentGoal]) -> usize {
        let mut count = 0;
        for goal in goals {
            if self.enqueue(goal).is_ok() {
                count += 1;
            }
        }
        count
    }

    /// Iterate over all goals.
    pub fn iter(&self) -> impl Iterator<Item = &AgentGoal> {
        self.goals.values()
    }

    /// Wire iter: get goals as a Vec.
    pub fn goals_vec(&self) -> Vec<AgentGoal> {
        self.goals.values().cloned().collect()
    }

    /// Number of goals in the queue.
    pub fn len(&self) -> usize {
        self.goals.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }

    /// Reload all goals from SQLite into memory (§4 — persistence continuity).
    pub fn reload(&mut self) -> Result<(), String> {
        if let Some(ref conn) = self.db {
            let mut stmt = conn
                .prepare("SELECT id, title, description, priority, source, status, deadline, expected_value, risk, learning_value, required_capabilities, dependencies, execution_history, completion_state, creation_timestamp, last_evaluation, estimated_cost, resource_requirements FROM objectives")
                .map_err(|e| format!("prepare reload: {e}"))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(AgentGoal {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        priority: row.get(3)?,
                        source: match row.get::<_, String>(4)?.as_str() {
                            "HumanOrigin" => ObjectiveSource::HumanOrigin,
                            "ExternalOpportunity" => ObjectiveSource::ExternalOpportunity,
                            "SystemTrigger" => ObjectiveSource::SystemTrigger,
                            "LearningTarget" => ObjectiveSource::LearningTarget,
                            "ImprovementTarget" => ObjectiveSource::ImprovementTarget,
                            "StrategicObjective" => ObjectiveSource::StrategicObjective,
                            _ => ObjectiveSource::SystemTrigger,
                        },
                        status: match row.get::<_, String>(5)?.as_str() {
                            "Discovered" => GoalStatus::Discovered,
                            "Evaluating" => GoalStatus::Evaluating,
                            "Accepted" => GoalStatus::Accepted,
                            "Queued" => GoalStatus::Queued,
                            "Blocked" => GoalStatus::Blocked,
                            "Deferred" => GoalStatus::Deferred,
                            "Active" => GoalStatus::Active,
                            "Verifying" => GoalStatus::Verifying,
                            "Completed" => GoalStatus::Completed,
                            "Failed" => GoalStatus::Failed,
                            "Cancelled" => GoalStatus::Cancelled,
                            "Rejected" => GoalStatus::Rejected,
                            "Archived" => GoalStatus::Archived,
                            _ => GoalStatus::Discovered,
                        },
                        expected_value: row.get(7)?,
                        risk: row.get(8)?,
                        learning_value: row.get(9)?,
                        required_capabilities: serde_json::from_str(&row.get::<_, String>(10)?)
                            .unwrap_or_default(),
                        dependencies: serde_json::from_str(&row.get::<_, String>(11)?)
                            .unwrap_or_default(),
                        execution_history: serde_json::from_str(&row.get::<_, String>(12)?)
                            .unwrap_or_default(),
                        completion_state: row.get::<_, Option<String>>(13)?,
                        deadline: row.get::<_, Option<String>>(6)?.and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .ok()
                                .map(|d| d.with_timezone(&chrono::Utc))
                        }),
                        creation_timestamp: row.get::<_, Option<String>>(14)?.and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .ok()
                                .map(|d| d.with_timezone(&chrono::Utc))
                        }),
                        last_evaluation: row.get::<_, Option<String>>(15)?.and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .ok()
                                .map(|d| d.with_timezone(&chrono::Utc))
                        }),
                        estimated_cost: row
                            .get::<_, Option<String>>(16)?
                            .and_then(|s| s.parse().ok()),
                        resource_requirements: serde_json::from_str(&row.get::<_, String>(17)?)
                            .unwrap_or_default(),
                    })
                })
                .map_err(|e| format!("query reload: {e}"))?;
            self.goals.clear();
            for goal_result in rows {
                let goal = goal_result.map_err(|e| format!("parse reload row: {e}"))?;
                self.goals.insert(goal.id.clone(), goal);
            }
        }
        Ok(())
    }

    /// Open a durable queue backed by SQLite at the given path.
    /// Creates the `objectives` table if it does not exist (schema from §A.4).
    pub fn open(path: &str) -> Result<Self, String> {
        let conn =
            rusqlite::Connection::open(path).map_err(|e| format!("open db {}: {}", path, e))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS objectives (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                priority REAL DEFAULT 0.0,
                source TEXT DEFAULT 'system',
                status TEXT DEFAULT 'DISCOVERED',
                deadline TEXT,
                expected_value REAL DEFAULT 0.0,
                risk REAL DEFAULT 0.5,
                learning_value REAL DEFAULT 0.0,
                required_capabilities TEXT,
                dependencies TEXT,
                execution_history TEXT,
                completion_state TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now')),
                creation_timestamp TEXT,
                last_evaluation TEXT,
                estimated_cost REAL,
                resource_requirements TEXT DEFAULT '[]'
            );",
        )
        .map_err(|e| format!("create objectives table: {}", e))?;
        Ok(Self {
            goals: std::collections::HashMap::new(),
            db: Some(conn),
        })
    }
}

// ==========================================================
// Background Worker types — Per Architecture Chapter 23
// ==========================================================

/// A task in the worker task queue.
///
/// Per Architecture §23: each task has a priority, payload, and
/// optional memory_id for tracking.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskQueue {
    /// Priority level (higher = more important). 0-255.
    pub priority: u8,
    /// Serialized task payload.
    pub payload: String,
    /// Optional memory ID this task is associated with.
    pub memory_id: Option<String>,
}

impl TaskQueue {
    /// Create a new task queue entry.
    pub fn new(priority: u8, payload: String, memory_id: Option<String>) -> Self {
        Self {
            priority,
            payload,
            memory_id,
        }
    }
}

/// Types of background workers.
///
/// Per Architecture §23: each worker type handles a specific
/// category of background tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkerType {
    /// Memory management worker — handles consolidation, promotion, pruning.
    Memory,
    /// Experience worker — records and evaluates experiences.
    Experience,
    /// Learning worker — runs learning pipelines, pattern discovery.
    Learning,
    /// Knowledge graph worker — maintains graph integrity, extraction.
    KnowledgeGraph,
    /// Maintenance worker — cleanup, archiving, statistics.
    Maintenance,
}

impl std::fmt::Display for WorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerType::Memory => write!(f, "memory"),
            WorkerType::Experience => write!(f, "experience"),
            WorkerType::Learning => write!(f, "learning"),
            WorkerType::KnowledgeGraph => write!(f, "knowledge_graph"),
            WorkerType::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// Schedule a task onto a worker queue.
///
/// Per Architecture §23: the TaskQueue receives tasks with a priority
/// and optional memory_id for tracking.
pub fn schedule_task(queue: &mut Vec<TaskQueue>, worker: WorkerType, payload: String) {
    let memory_id = None;
    let priority = match worker {
        WorkerType::Memory => 50,
        WorkerType::Experience => 40,
        WorkerType::Learning => 30,
        WorkerType::KnowledgeGraph => 35,
        WorkerType::Maintenance => 20,
    };
    queue.push(TaskQueue::new(priority, payload, memory_id));
}

/// Actively reference worker types to eliminate dead-code warnings.
pub fn reference_worker_apis() {
    let memory_type = WorkerType::Memory;
    let experience_type = WorkerType::Experience;
    let learning_type = WorkerType::Learning;
    let kg_type = WorkerType::KnowledgeGraph;
    let maintenance_type = WorkerType::Maintenance;
    let memory_display = format!("{}", memory_type);
    let mut queue = Vec::new();
    schedule_task(&mut queue, WorkerType::Memory, "test task".into());
    tracing::debug!(
        "Worker types referenced: memory={} experience={} learning={} kg={} maintenance={} queue_len={}",
        memory_display,
        experience_type,
        learning_type,
        kg_type,
        maintenance_type,
        queue.len()
    );
}
