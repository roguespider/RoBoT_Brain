//! Retrieval feedback loop per Architecture Chapter 16.
//! Records retrieval outcomes to improve future retrieval quality.

/// Feedback record for a retrieval operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalFeedback {
    /// The query that was retrieved.
    pub query: String,
    /// Whether the results were useful.
    pub useful: bool,
    /// User-provided rating (0.0 - 1.0).
    pub rating: f32,
    /// Number of candidates returned.
    pub candidate_count: usize,
    /// Timestamp of the feedback.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl RetrievalFeedback {
    /// Create new feedback for a retrieval operation.
    pub fn new(query: &str, useful: bool, rating: f32, candidate_count: usize) -> Self {
        Self {
            query: query.to_string(),
            useful,
            rating,
            candidate_count,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if the feedback indicates good results.
    pub fn is_positive(&self) -> bool {
        self.useful && self.rating >= 0.7
    }
}

/// Collects and aggregates retrieval feedback.
pub struct FeedbackCollector {
    feedback: Vec<RetrievalFeedback>,
}

impl FeedbackCollector {
    /// Create a new empty collector.
    pub fn new() -> Self {
        Self { feedback: Vec::new() }
    }

    /// Record feedback for a retrieval operation.
    pub fn record(&mut self, feedback: RetrievalFeedback) {
        self.feedback.push(feedback);
    }

    /// Get the average rating across all feedback.
    pub fn average_rating(&self) -> f32 {
        if self.feedback.is_empty() {
            return 0.0;
        }
        let total: f32 = self.feedback.iter().map(|f| f.rating).sum();
        total / self.feedback.len() as f32
    }

    /// Get the proportion of positive feedback.
    pub fn positive_ratio(&self) -> f32 {
        if self.feedback.is_empty() {
            return 0.0;
        }
        let positive = self
            .feedback
            .iter()
            .filter(|f| f.is_positive())
            .count();
        positive as f32 / self.feedback.len() as f32
    }

    /// Get the total number of feedback records.
    pub fn count(&self) -> usize {
        self.feedback.len()
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}
