// /src/CoObOpLoop/strategic.rs
// Strategic objectives for the CoObOpLoop system.

use rusqlite::OptionalExtension;

/// Strategic objective category (§18 / T13.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StrategicObjective {
    MaintainSystemReliability,
    ImproveMemoryRetrieval,
    IncreaseInferenceEfficiency,
    ExpandToolCapability,
    ImproveHardwareUtilization,
    ReduceRepeatedFailures,
    DevelopResearchCapability,
    ImprovePlanningReliability,
}

impl StrategicObjective {
    fn storage_name(&self) -> &'static str {
        match self {
            Self::MaintainSystemReliability => "MaintainSystemReliability",
            Self::ImproveMemoryRetrieval => "ImproveMemoryRetrieval",
            Self::IncreaseInferenceEfficiency => "IncreaseInferenceEfficiency",
            Self::ExpandToolCapability => "ExpandToolCapability",
            Self::ImproveHardwareUtilization => "ImproveHardwareUtilization",
            Self::ReduceRepeatedFailures => "ReduceRepeatedFailures",
            Self::DevelopResearchCapability => "DevelopResearchCapability",
            Self::ImprovePlanningReliability => "ImprovePlanningReliability",
        }
    }

    fn from_storage_name(name: &str) -> Result<Self, String> {
        match name {
            "MaintainSystemReliability" => Ok(Self::MaintainSystemReliability),
            "ImproveMemoryRetrieval" => Ok(Self::ImproveMemoryRetrieval),
            "IncreaseInferenceEfficiency" => Ok(Self::IncreaseInferenceEfficiency),
            "ExpandToolCapability" => Ok(Self::ExpandToolCapability),
            "ImproveHardwareUtilization" => Ok(Self::ImproveHardwareUtilization),
            "ReduceRepeatedFailures" => Ok(Self::ReduceRepeatedFailures),
            "DevelopResearchCapability" => Ok(Self::DevelopResearchCapability),
            "ImprovePlanningReliability" => Ok(Self::ImprovePlanningReliability),
            invalid_name => Err(format!(
                "invalid strategic objective category '{invalid_name}' in persistent storage"
            )),
        }
    }
}

/// Persisted strategic objective record (§18 / T13.2).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StrategicObjectiveRecord {
    pub id: String,
    pub name: String,
    pub category: StrategicObjective,
    pub status: String,
    pub priority: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Registry of strategic objectives (§18 / T13.2).
pub struct StrategicObjectiveRegistry {
    objectives: Vec<StrategicObjectiveRecord>,
    db: Option<rusqlite::Connection>,
}

impl StrategicObjectiveRegistry {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            db: None,
        }
    }

    /// Open SQLite-backed registry (§T13.3 — creates strategic_objectives table).
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = rusqlite::Connection::open(path).map_err(|e| format!("open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS strategic_objectives (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT,
                status TEXT DEFAULT 'active',
                priority REAL DEFAULT 0.0,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );",
        )
        .map_err(|error| format!("create strategic_objectives table: {error}"))?;

        let objectives = {
            let mut statement = conn
                .prepare(
                    "SELECT id, name, category, status, priority, created_at, updated_at
                     FROM strategic_objectives ORDER BY created_at, id",
                )
                .map_err(|error| format!("prepare strategic objective list: {error}"))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                })
                .map_err(|error| format!("query strategic objectives: {error}"))?;
            let mut objectives = Vec::new();
            for row in rows {
                let (id, name, category, status, priority, created_at, updated_at) =
                    row.map_err(|error| format!("read strategic objective row: {error}"))?;
                objectives.push(StrategicObjectiveRecord {
                    id,
                    name,
                    category: StrategicObjective::from_storage_name(&category)?,
                    status,
                    priority,
                    created_at: parse_stored_timestamp(&created_at)?,
                    updated_at: parse_stored_timestamp(&updated_at)?,
                });
            }
            objectives
        };

        Ok(Self {
            objectives,
            db: Some(conn),
        })
    }

    pub fn get(&self, id: &str) -> Result<Option<StrategicObjectiveRecord>, String> {
        if let Some(objective) = self.objectives.iter().find(|objective| objective.id == id) {
            return Ok(Some(objective.clone()));
        }

        let Some(connection) = self.db.as_ref() else {
            return Ok(None);
        };

        let stored = connection
            .query_row(
                "SELECT id, name, category, status, priority, created_at, updated_at
                 FROM strategic_objectives WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("get strategic objective '{id}': {error}"))?;

        let Some((id, name, category, status, priority, created_at, updated_at)) = stored else {
            return Ok(None);
        };

        Ok(Some(StrategicObjectiveRecord {
            id,
            name,
            category: StrategicObjective::from_storage_name(&category)?,
            status,
            priority,
            created_at: parse_stored_timestamp(&created_at)?,
            updated_at: parse_stored_timestamp(&updated_at)?,
        }))
    }

    pub fn add(&mut self, objective: StrategicObjectiveRecord) -> Result<(), String> {
        if let Some(connection) = self.db.as_ref() {
            connection
                .execute(
                    "INSERT OR REPLACE INTO strategic_objectives
                     (id, name, category, status, priority, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        &objective.id,
                        &objective.name,
                        objective.category.storage_name(),
                        &objective.status,
                        objective.priority,
                        objective.created_at.to_rfc3339(),
                        objective.updated_at.to_rfc3339(),
                    ],
                )
                .map_err(|error| {
                    format!("persist strategic objective '{}': {error}", objective.id)
                })?;
        }

        if let Some(index) = self
            .objectives
            .iter()
            .position(|stored| stored.id == objective.id)
        {
            self.objectives[index] = objective;
        } else {
            self.objectives.push(objective);
        }
        Ok(())
    }

    pub fn list(&self) -> &[StrategicObjectiveRecord] {
        &self.objectives
    }

    pub fn remove(&mut self, id: &str) -> Result<bool, String> {
        let Some(mut objective) = self.get(id)? else {
            return Ok(false);
        };

        objective.status = "removed".to_string();
        objective.updated_at = chrono::Utc::now();
        self.add(objective)?;
        Ok(true)
    }
}

fn parse_stored_timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(value) {
        return Ok(timestamp.with_timezone(&chrono::Utc));
    }

    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .map(|timestamp| timestamp.and_utc())
        .map_err(|error| format!("invalid strategic objective timestamp '{value}': {error}"))
}

impl Default for StrategicObjectiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Level in the persistent long-term objective hierarchy (§19 / T13.11).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HierarchyLevel {
    Mission,
    StrategicObjective,
    Capability,
    Project,
    Task,
    Action,
}

/// Hierarchy node for objective hierarchy (per §A.7 / T13.13).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct HierarchyNode {
    pub level: HierarchyLevel,
    pub name: String,
    pub children: Vec<HierarchyNode>,
}

/// Objective hierarchy manager.
pub struct ObjectiveHierarchy {
    nodes: Vec<HierarchyNode>,
}

impl ObjectiveHierarchy {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn build(mission: &str, objectives: &[StrategicObjectiveRecord]) -> Self {
        let strategic_nodes = objectives
            .iter()
            .filter(|objective| objective.status != "removed")
            .map(|objective| HierarchyNode {
                level: HierarchyLevel::StrategicObjective,
                name: objective.name.clone(),
                children: Vec::new(),
            })
            .collect();
        let root = HierarchyNode {
            level: HierarchyLevel::Mission,
            name: mission.to_string(),
            children: strategic_nodes,
        };
        let mut hierarchy = Self::new();
        hierarchy.add_node(root);
        hierarchy
    }

    pub fn add_node(&mut self, node: HierarchyNode) {
        self.nodes.push(node);
    }

    pub fn nodes(&self) -> &[HierarchyNode] {
        &self.nodes
    }
}

impl Default for ObjectiveHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for StrategicObjective {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_storage_name(value)
    }
}

impl std::fmt::Debug for StrategicObjective {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.storage_name())
    }
}
