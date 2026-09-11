// /src/CoObOpLoop/capability.rs
// Capability tracking for the CoObOpLoop system.

/// Autonomy level for a capability (§22 / T15.13).
#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AutonomyLevel {
    #[default]
    Manual,
    Assisted,
    SemiAutonomous,
    Autonomous,
}

impl AutonomyLevel {
    pub const ALL: [Self; 4] = [
        Self::Manual,
        Self::Assisted,
        Self::SemiAutonomous,
        Self::Autonomous,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::Assisted => "Assisted",
            Self::SemiAutonomous => "SemiAutonomous",
            Self::Autonomous => "Autonomous",
        }
    }
}

impl From<u8> for AutonomyLevel {
    fn from(level: u8) -> Self {
        match level {
            0 => Self::Manual,
            1 => Self::Assisted,
            2 => Self::SemiAutonomous,
            _ => Self::Autonomous,
        }
    }
}

impl std::fmt::Debug for AutonomyLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => formatter.write_str("Manual"),
            Self::Assisted => formatter.write_str("Assisted"),
            Self::SemiAutonomous => formatter.write_str("SemiAutonomous"),
            Self::Autonomous => formatter.write_str("Autonomous"),
        }
    }
}

/// Unique identifier for a capability (§6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CapabilityId {
    Rust,
    Mcp,
    Http,
    SQLite,
    Testing,
    Custom(String),
}

/// Assessment of a capability.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityAssessment {
    pub id: CapabilityId,
    pub name: String,
    pub level: f32,
    pub last_assessed: Option<chrono::DateTime<chrono::Utc>>,
    pub success_rate: f32,
}

/// Registry of all capabilities.
pub struct CapabilityRegistry {
    capabilities: std::collections::HashMap<String, CapabilityAssessment>,
    autonomy: std::collections::HashMap<CapabilityId, AutonomyLevel>,
    db: Option<rusqlite::Connection>,
}

impl CapabilityId {
    pub fn key(&self) -> String {
        match self {
            Self::Rust => "rust".to_string(),
            Self::Mcp => "mcp".to_string(),
            Self::Http => "http".to_string(),
            Self::SQLite => "sqlite".to_string(),
            Self::Testing => "testing".to_string(),
            Self::Custom(name) => format!("custom:{name}"),
        }
    }

    /// Parse a string identifier into a `CapabilityId`.
    /// Accepts "rust", "mcp", "http", "sqlite", "testing" (case-insensitive)
    /// and anything else as `CapabilityId::Custom`.
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rust" => CapabilityId::Rust,
            "mcp" => CapabilityId::Mcp,
            "http" => CapabilityId::Http,
            "sqlite" => CapabilityId::SQLite,
            "testing" => CapabilityId::Testing,
            other => CapabilityId::Custom(other.to_string()),
        }
    }
}

impl CapabilityRegistry {
    fn id_to_key(id: &CapabilityId) -> String {
        id.key()
    }

    pub fn new() -> Self {
        Self {
            capabilities: std::collections::HashMap::new(),
            autonomy: std::collections::HashMap::new(),
            db: None,
        }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let conn = rusqlite::Connection::open(path).map_err(|e| format!("open db: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS capabilities (id TEXT PRIMARY KEY, name TEXT, level REAL DEFAULT 0.5, last_assessed TEXT, success_rate REAL DEFAULT 0.5);",
        ).map_err(|e| format!("create table: {e}"))?;
        Ok(Self {
            capabilities: std::collections::HashMap::new(),
            autonomy: std::collections::HashMap::new(),
            db: Some(conn),
        })
    }

    pub fn register(&mut self, assessment: CapabilityAssessment) {
        let key = Self::id_to_key(&assessment.id);
        self.autonomy.entry(assessment.id.clone()).or_default();
        self.capabilities.insert(key.clone(), assessment.clone());
        if let Some(ref conn) = self.db {
            let last_str = assessment
                .last_assessed
                .map(|d| d.to_rfc3339())
                .unwrap_or_default();
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO capabilities (id, name, level, last_assessed, success_rate) VALUES (?1, ?2, ?3, ?4, ?5)",
                [&key, &assessment.name, &assessment.level.to_string(), &last_str, &assessment.success_rate.to_string()],
            ) {
                tracing::debug!("Failed to persist capability registration: {e}");
            }
        }
    }

    pub fn get(&self, id: &CapabilityId) -> Option<CapabilityAssessment> {
        let key = Self::id_to_key(id);
        if let Some(a) = self.capabilities.get(&key) {
            return Some(a.clone());
        }
        if let Some(ref conn) = self.db {
            let result: Option<(String, f32, Option<String>, f32)> = conn.query_row(
                "SELECT name, level, last_assessed, success_rate FROM capabilities WHERE id = ?1",
                [&key],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            ).ok();
            if let Some((name, level, last_assessed, success_rate)) = result {
                let last = last_assessed.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&chrono::Utc))
                });
                return Some(CapabilityAssessment {
                    id: id.clone(),
                    name,
                    level,
                    last_assessed: last,
                    success_rate,
                });
            }
        }
        None
    }

    /// Update assessment — updates level and success_rate in DB (§T4.6).
    pub fn update(&mut self, assessment: CapabilityAssessment) {
        let key = Self::id_to_key(&assessment.id);
        self.capabilities.insert(key.clone(), assessment.clone());
        if let Some(ref conn) = self.db {
            let last_str = assessment
                .last_assessed
                .map(|d| d.to_rfc3339())
                .unwrap_or_default();
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO capabilities (id, name, level, last_assessed, success_rate) VALUES (?1, ?2, ?3, ?4, ?5)",
                [&key, &assessment.name, &assessment.level.to_string(), &last_str, &assessment.success_rate.to_string()],
            ) {
                tracing::debug!("Failed to persist capability update: {e}");
            }
        }
    }

    pub fn list(&self) -> Vec<&CapabilityAssessment> {
        self.capabilities.values().collect()
    }

    /// Compare required capabilities list against current assessments.
    /// Returns a `CapabilityComparison` (§T4.7, §T4.8, §T4.9).
    pub fn compare_capabilities(&self, required: &[CapabilityId]) -> CapabilityComparison {
        let mut sufficient = Vec::new();
        let mut uncertain = Vec::new();
        let mut insufficient = Vec::new();
        let mut unavailable = Vec::new();

        for id in required {
            let label = Self::id_to_key(id);
            match self.get(id) {
                None => unavailable.push(label),
                Some(a) => {
                    // §A.3 threshold: < 0.7 = insufficient; < 0.85 = uncertain; >= 0.85 = sufficient
                    if a.level < 0.7 {
                        insufficient.push(label);
                    } else if a.level < 0.85 {
                        uncertain.push(label);
                    } else {
                        sufficient.push(label);
                    }
                }
            }
        }

        let mut comparison = CapabilityComparison {
            sufficient,
            uncertain,
            insufficient,
            unavailable,
            overall_outcome: String::new(),
        };
        comparison.overall_outcome = comparison.overall_outcome();
        comparison
    }

    /// Return the tracked level for a capability, defaulting new IDs to Manual.
    pub fn get_autonomy(&self, id: &CapabilityId) -> AutonomyLevel {
        self.autonomy.get(id).copied().unwrap_or_default()
    }

    /// Track an explicit autonomy level for a capability.
    pub fn set_autonomy(&mut self, id: CapabilityId, level: AutonomyLevel) {
        self.autonomy.insert(id, level);
    }

    /// Advance one capability by exactly one autonomy level.
    pub fn promote_autonomy(&mut self, id: CapabilityId) -> (AutonomyLevel, AutonomyLevel) {
        let previous_level = self.get_autonomy(&id);
        let new_level = match previous_level {
            AutonomyLevel::Manual => AutonomyLevel::Assisted,
            AutonomyLevel::Assisted => AutonomyLevel::SemiAutonomous,
            AutonomyLevel::SemiAutonomous | AutonomyLevel::Autonomous => AutonomyLevel::Autonomous,
        };
        self.set_autonomy(id, new_level);
        (previous_level, new_level)
    }

    /// Return a stable snapshot of all explicitly tracked capability levels.
    pub fn autonomy_levels(&self) -> Vec<(CapabilityId, AutonomyLevel)> {
        let mut levels: Vec<_> = self
            .autonomy
            .iter()
            .map(|(id, level)| (id.clone(), *level))
            .collect();
        levels.sort_by_key(|(id, level)| (Self::id_to_key(id), *level as u8));
        levels
    }

    /// Record a successful outcome for a capability (§T4.10).
    /// Increments experience count and updates success_rate using an exponential
    /// moving average: new_rate = (old_rate * (n-1) + 1.0) / n, where n is the
    /// updated experience count stored in last_assessed's nanoseconds as a proxy
    /// when no other counter exists. Here we approximate n from success_rate
    /// samples: if n is unknown we start at 1, otherwise increment.
    pub fn record_success(&mut self, id: &CapabilityId) -> Result<(), String> {
        let now = chrono::Utc::now();
        let mut assessment = self.get(id).unwrap_or_else(|| CapabilityAssessment {
            id: id.clone(),
            name: match id {
                CapabilityId::Rust => "Rust".to_string(),
                CapabilityId::Mcp => "MCP".to_string(),
                CapabilityId::Http => "HTTP".to_string(),
                CapabilityId::SQLite => "SQLite".to_string(),
                CapabilityId::Testing => "Testing".to_string(),
                CapabilityId::Custom(s) => format!("Custom:{}", s),
            },
            level: 0.5,
            last_assessed: None,
            success_rate: 0.5,
        });
        // EMA blend: blend old rate with a new success (1.0) using alpha=0.3
        assessment.success_rate = assessment.success_rate * 0.7 + 1.0 * 0.3;
        // Slight level increase on success (capped at 1.0)
        assessment.level = (assessment.level + 0.02).min(1.0);
        assessment.last_assessed = Some(now);
        self.update(assessment);
        Ok(())
    }

    /// Record a failed outcome for a capability (§T4.11).
    pub fn record_failure(&mut self, id: &CapabilityId) -> Result<(), String> {
        let now = chrono::Utc::now();
        let mut assessment = self.get(id).unwrap_or_else(|| CapabilityAssessment {
            id: id.clone(),
            name: match id {
                CapabilityId::Rust => "Rust".to_string(),
                CapabilityId::Mcp => "MCP".to_string(),
                CapabilityId::Http => "HTTP".to_string(),
                CapabilityId::SQLite => "SQLite".to_string(),
                CapabilityId::Testing => "Testing".to_string(),
                CapabilityId::Custom(s) => format!("Custom:{}", s),
            },
            level: 0.5,
            last_assessed: None,
            success_rate: 0.5,
        });
        // EMA blend toward 0.0 (failure)
        assessment.success_rate = assessment.success_rate * 0.7 + 0.0 * 0.3;
        // Slight level decrease on failure (floor at 0.0)
        assessment.level = (assessment.level - 0.02).max(0.0);
        assessment.last_assessed = Some(now);
        self.update(assessment);
        Ok(())
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Seed default capabilities at init (§T4.16): Rust, MCP, HTTP, SQLite, Testing at 0.5.
pub fn seed_default_capabilities(registry: &mut CapabilityRegistry) {
    let defaults = [
        (CapabilityId::Rust, "Rust", 0.5),
        (CapabilityId::Mcp, "MCP", 0.5),
        (CapabilityId::Http, "HTTP", 0.5),
        (CapabilityId::SQLite, "SQLite", 0.5),
        (CapabilityId::Testing, "Testing", 0.5),
    ];
    for (id, name, level) in defaults {
        registry.register(CapabilityAssessment {
            id,
            name: name.to_string(),
            level,
            last_assessed: None,
            success_rate: 0.5,
        });
    }
}

/// Result of comparing required capabilities against current assessments (§T4.8, §T4.9).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityComparison {
    /// Capabilities meeting threshold (level >= 0.85).
    pub sufficient: Vec<String>,
    /// Capabilities in the uncertain range (0.7 <= level < 0.85).
    pub uncertain: Vec<String>,
    /// Capabilities below threshold (level < 0.7).
    pub insufficient: Vec<String>,
    /// Capabilities with no recorded assessment.
    pub unavailable: Vec<String>,
    /// Overall outcome: "sufficient", "uncertain", or "insufficient".
    pub overall_outcome: String,
}

impl CapabilityComparison {
    /// Compute the overall outcome based on the per-capability classification.
    /// §T4.9: insufficient if any capability is in `insufficient` or `unavailable`,
    /// uncertain if any capability is in `uncertain`, otherwise sufficient.
    pub fn compute_outcome(&self) -> String {
        if !self.insufficient.is_empty() || !self.unavailable.is_empty() {
            "insufficient".to_string()
        } else if !self.uncertain.is_empty() {
            "uncertain".to_string()
        } else {
            "sufficient".to_string()
        }
    }

    /// Public alias matching the PLAN naming (§T4.9).
    pub fn overall_outcome(&self) -> String {
        self.compute_outcome()
    }
}

impl std::fmt::Debug for CapabilityComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityComparison")
            .field("sufficient", &self.sufficient)
            .field("uncertain", &self.uncertain)
            .field("insufficient", &self.insufficient)
            .field("unavailable", &self.unavailable)
            .field("overall_outcome", &self.overall_outcome)
            .finish()
    }
}

impl std::fmt::Debug for CapabilityAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityAssessment")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("level", &self.level)
            .field("last_assessed", &self.last_assessed)
            .field("success_rate", &self.success_rate)
            .finish()
    }
}
