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
    /// Create a new queue.
    pub fn new() -> Self {
        Self {
            goals: std::collections::HashMap::new(),
            db: None,
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

        // Insert into SQLite
        if let Some(ref conn) = self.db {
            conn.execute(
                "INSERT OR REPLACE INTO objectives (
                    id, title, description, priority, source, status,
                    deadline, expected_value, risk, learning_value,
                    required_capabilities, dependencies, execution_history,
                    completion_state
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                "SELECT id, title, description, priority, source, status, deadline, expected_value, risk, learning_value, required_capabilities, dependencies, execution_history, completion_state FROM objectives WHERE id = ?1",
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

        if let Some(ref conn) = self.db {
            conn.execute(
                "UPDATE objectives SET title=?2, description=?3, priority=?4, source=?5, status=?6, deadline=?7, expected_value=?8, risk=?9, learning_value=?10, required_capabilities=?11, dependencies=?12, execution_history=?13, completion_state=?14 WHERE id=?1",
                rusqlite::params![
                    id, goal.title, goal.description, goal.priority, source_str,
                    format!("{:?}", goal.status), deadline, goal.expected_value, goal.risk,
                    goal.learning_value, required_caps, deps, hist, completion,
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
                updated_at TEXT DEFAULT (datetime('now'))
            );",
        )
        .map_err(|e| format!("create objectives table: {}", e))?;
        Ok(Self {
            goals: std::collections::HashMap::new(),
            db: Some(conn),
        })
    }
}
