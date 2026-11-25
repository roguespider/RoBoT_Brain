//! History — Per Architecture Chapter 26.17 (Evolution Memory).
//!
//! RoBoT maintains a history of its own changes. Evolution records
//! include previous behavior, new behavior, reason for change, evidence,
//! confidence, and result.

/// A record of an evolutionary change.
#[derive(Debug, Clone)]
pub struct EvolutionRecord {
    /// Unique record identifier.
    pub id: String,
    /// Previous behavior before the change.
    pub previous_behavior: String,
    /// New behavior after the change.
    pub new_behavior: String,
    /// Reason for the change.
    pub reason: String,
    /// Evidence supporting the change.
    pub evidence: Vec<String>,
    /// Confidence in the change (0.0-1.0).
    pub confidence: f32,
    /// Result of applying the change.
    pub result: String,
}

/// History of evolutionary changes.
#[derive(Debug, Clone, Default)]
pub struct EvolutionHistory {
    /// All recorded evolution records.
    records: Vec<EvolutionRecord>,
}

impl EvolutionHistory {
    /// Create a new empty history.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Add a new evolution record.
    pub fn add_record(&mut self, record: EvolutionRecord) {
        self.records.push(record);
    }

    /// Get all records.
    pub fn records(&self) -> &[EvolutionRecord] {
        &self.records
    }

    /// Count total records.
    pub fn count(&self) -> usize {
        self.records.len()
    }
}
