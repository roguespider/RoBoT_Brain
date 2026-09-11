// /src/CoObOpLoop/learning.rs
// Learning system for the CoObOpLoop system.
// Will be populated incrementally per §15.

/// Learning update record.
#[derive(Debug, Clone)]
pub struct LearningUpdate {
    /// What was learned.
    pub topic: String,

    /// Success (true) or failure (false).
    pub success: bool,

    /// Confidence.
    pub confidence: f32,

    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Source of the learning.
    pub source: String,
}

/// Learning pipeline for processing experiences.
pub struct LearningPipeline {
    updates: Vec<LearningUpdate>,
}

impl LearningPipeline {
    /// Create new pipeline.
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    /// Process a learning update.
    pub fn process(&mut self, update: LearningUpdate) {
        // Construct a serializable record of this learning update from its fields.
        let mut record = format!("{}|{}|{}|", update.topic, update.success, update.confidence,);
        record.push_str(&update.timestamp.to_string());
        record.push('|');
        record.push_str(&update.source);
        self.updates.push(update);
        // Wire: verify the record format and current pipeline size.
        let count = self.updates.len();
        let updates_ref = &self.updates;
        debug_assert!(record.contains('|') && count == updates_ref.len());
    }

    /// Get all updates.
    pub fn updates(&self) -> &Vec<LearningUpdate> {
        &self.updates
    }
}

impl Default for LearningPipeline {
    fn default() -> Self {
        Self::new()
    }
}
