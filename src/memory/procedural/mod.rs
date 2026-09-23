//! Procedural Memory — Skill and procedure storage (Architecture Chapter 8 — Memory Engine upgrade).
#![allow(unused)]
//!
//! Per Architecture §8.5 (Procedural Memory): reusable skills, workflows,
//! and procedures. Separate from semantic (concepts) and episodic (experience) memory.

/// A procedural memory record representing a skill or procedure.
#[derive(Debug, Clone, PartialEq)]
pub struct ProceduralRecord {
    /// Skill/procedure identifier.
    pub id: String,
    /// Skill/procedure name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Steps or actions.
    pub steps: Vec<String>,
    /// Required capabilities.
    pub required_capabilities: Vec<String>,
    /// Success rate (0.0 to 1.0).
    pub success_rate: f32,
    /// Execution count.
    pub execution_count: u32,
    /// Created timestamp.
    pub created_at: i64,
    /// Last updated.
    pub updated_at: i64,
}

impl ProceduralRecord {
    /// Create a new procedural record.
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            required_capabilities: Vec::new(),
            success_rate: 0.5,
            execution_count: 0,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Add a step.
    pub fn add_step(&mut self, step: &str) {
        self.steps.push(step.to_string());
    }

    /// Add required capability.
    pub fn add_capability(&mut self, cap: &str) {
        if !self.required_capabilities.contains(&cap.to_string()) {
            self.required_capabilities.push(cap.to_string());
        }
    }

    /// Record execution.
    pub fn record_execution(&mut self, success: bool) {
        self.execution_count += 1;
        let current_rate = self.success_rate;
        let new_rate = if success {
            (current_rate * (self.execution_count - 1) as f32 + 1.0) / self.execution_count as f32
        } else {
            (current_rate * (self.execution_count - 1) as f32) / self.execution_count as f32
        };
        self.success_rate = new_rate.clamp(0.0, 1.0);
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Procedural memory store.
#[derive(Debug, Clone, Default)]
pub struct ProceduralMemoryStore {
    /// Records by ID.
    records: std::collections::HashMap<String, ProceduralRecord>,
}

impl ProceduralMemoryStore {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            records: std::collections::HashMap::new(),
        }
    }

    /// Add a record.
    pub fn add(&mut self, record: ProceduralRecord) {
        self.records.insert(record.id.clone(), record);
    }

    /// Get a record by ID.
    pub fn get(&self, id: &str) -> Option<&ProceduralRecord> {
        self.records.get(id)
    }

    /// Get mutable record.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ProceduralRecord> {
        self.records.get_mut(id)
    }

    /// Search by name.
    pub fn search_by_name(&self, query: &str) -> Vec<&ProceduralRecord> {
        let lower_query = query.to_lowercase();
        self.records
            .values()
            .filter(|r| {
                r.name.to_lowercase().contains(&lower_query)
                    || r.description.to_lowercase().contains(&lower_query)
            })
            .collect()
    }

    /// Get all records.
    pub fn all(&self) -> Vec<&ProceduralRecord> {
        self.records.values().collect()
    }
}

/// Forgetting policy: decide when to demote or archive memory.
/// Per Architecture §8.5 (Forgetting) and §25.18 (Trust Decay).
#[derive(Debug, Clone, PartialEq)]
pub struct ForgettingPolicy {
    /// Minimum access count before retention.
    pub min_access_count: u32,
    /// Minimum confidence for retention.
    pub min_confidence: f32,
    /// Maximum age (days) before demotion.
    pub max_age_days: u32,
    /// Demotion enabled.
    pub demotion_enabled: bool,
    /// Archive enabled.
    pub archive_enabled: bool,
}

impl Default for ForgettingPolicy {
    fn default() -> Self {
        Self {
            min_access_count: 3,
            min_confidence: 0.3,
            max_age_days: 90,
            demotion_enabled: true,
            archive_enabled: true,
        }
    }
}

impl ForgettingPolicy {
    /// Check if a record should be retained.
    pub fn should_retain(&self, access_count: u32, confidence: f32, age_days: u32) -> bool {
        access_count >= self.min_access_count
            && confidence >= self.min_confidence
            && age_days <= self.max_age_days
    }

    /// Check if a record should be demoted.
    pub fn should_demote(&self, access_count: u32, confidence: f32, age_days: u32) -> bool {
        self.demotion_enabled
            && (access_count < self.min_access_count
                || confidence < self.min_confidence
                || age_days > self.max_age_days)
    }

    /// Check if a record should be archived.
    pub fn should_archive(&self, access_count: u32, confidence: f32, age_days: u32) -> bool {
        self.archive_enabled
            && (access_count == 0 || confidence < 0.1 || age_days > self.max_age_days * 2)
    }
}

/// Active reference to procedural memory contracts.
pub fn reference_procedural_memory() {
    let mut store = ProceduralMemoryStore::new();
    let record = ProceduralRecord::new(
        "skill-1",
        "Retrieve Memory",
        "Retrieve relevant memories for a query",
    );
    store.add(record);
    tracing::debug!(
        record_count = store.all().len(),
        "Procedural memory referenced"
    );

    let policy = ForgettingPolicy::default();
    tracing::debug!(
        min_access = policy.min_access_count,
        min_confidence = policy.min_confidence,
        max_age = policy.max_age_days,
        "Forgetting policy referenced"
    );
}
