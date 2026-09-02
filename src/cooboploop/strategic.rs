// /src/CoObOpLoop/strategic.rs
// Strategic objectives for the CoObOpLoop system.

/// Strategic objective category (§18 / T13.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StrategicObjectiveCategory {
    MaintainSystemReliability,
    ImproveMemoryRetrieval,
    IncreaseInferenceEfficiency,
    ExpandToolCapability,
    ImproveHardwareUtilization,
    ReduceRepeatedFailures,
    DevelopResearchCapability,
    ImprovePlanningReliability,
}

/// Strategic objective record (§18 / T13.1).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StrategicObjective {
    pub id: String,
    pub name: String,
    pub category: StrategicObjectiveCategory,
    pub status: String,
    pub priority: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Registry of strategic objectives (§18 / T13.2).
pub struct StrategicObjectiveRegistry {
    objectives: Vec<StrategicObjective>,
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
            "CREATE TABLE IF NOT EXISTS strategic_objectives (id TEXT PRIMARY KEY, name TEXT, category TEXT, status TEXT, priority REAL, created_at TEXT, updated_at TEXT);",
        ).map_err(|e| format!("create table: {e}"))?;
        Ok(Self {
            objectives: Vec::new(),
            db: Some(conn),
        })
    }

    pub fn add(&mut self, objective: StrategicObjective) {
        self.objectives.push(objective.clone());
        if let Some(ref conn) = self.db {
            let cat_str = format!("{:?}", objective.category);
            let _ = conn.execute(
                "INSERT OR REPLACE INTO strategic_objectives (id, name, category, status, priority, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                [&objective.id, &objective.name, &cat_str, &objective.status, &objective.priority.to_string(), &objective.created_at.to_rfc3339(), &objective.updated_at.to_rfc3339()],
            );
        }
    }

    pub fn list(&self) -> &[StrategicObjective] {
        &self.objectives
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.objectives.len();
        self.objectives.retain(|o| o.id != id);
        if self.objectives.len() < initial_len {
            if let Some(ref conn) = self.db {
                let _ = conn.execute("DELETE FROM strategic_objectives WHERE id = ?1", [id]);
            }
            true
        } else {
            false
        }
    }
}

impl Default for StrategicObjectiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Hierarchy node for objective hierarchy (per §A.7 / T13.13).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct HierarchyNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
}

/// Objective hierarchy manager.
pub struct ObjectiveHierarchy {
    nodes: Vec<HierarchyNode>,
}

impl ObjectiveHierarchy {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
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

impl std::fmt::Debug for StrategicObjectiveCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaintainSystemReliability => write!(f, "MaintainSystemReliability"),
            Self::ImproveMemoryRetrieval => write!(f, "ImproveMemoryRetrieval"),
            Self::IncreaseInferenceEfficiency => write!(f, "IncreaseInferenceEfficiency"),
            Self::ExpandToolCapability => write!(f, "ExpandToolCapability"),
            Self::ImproveHardwareUtilization => write!(f, "ImproveHardwareUtilization"),
            Self::ReduceRepeatedFailures => write!(f, "ReduceRepeatedFailures"),
            Self::DevelopResearchCapability => write!(f, "DevelopResearchCapability"),
            Self::ImprovePlanningReliability => write!(f, "ImprovePlanningReliability"),
        }
    }
}
