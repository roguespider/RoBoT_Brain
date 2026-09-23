//! Strategic Learning — Strategic objectives, strategic updates, strategic evaluation,
//! strategic feedback (Architecture Chapter 18).
//!
//! Per Architecture §18.1-18.5:
//! - Strategic objectives: long-term improvement goals (§18.2)
//! - Strategic updates: changes to capabilities, skills, workflows (§18.3)
//! - Strategic evaluation: assessment of strategic progress (§18.4)
//! - Strategic feedback: feedback loop to execution/planning (§18.5)
//! - Wiring: strategic_learning/ -> cooboploop/ (existing loop) -> learning/ (ch 10) ->
//!   execution/ (ch 12, feedback loop 12.34) -> database/ (strategic updates persistence)

/// Strategic objective categories per Architecture §18.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StrategicCategory {
    /// System reliability improvement.
    SystemReliability,
    /// Memory retrieval improvement.
    MemoryRetrieval,
    /// Inference efficiency improvement.
    InferenceEfficiency,
    /// Tool capability expansion.
    ToolCapability,
    /// Hardware utilization improvement.
    HardwareUtilization,
    /// Repeated failure reduction.
    FailureReduction,
    /// Research capability development.
    ResearchCapability,
    /// Planning reliability improvement.
    PlanningReliability,
}

impl StrategicCategory {
    /// Return category label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::SystemReliability => "SystemReliability",
            Self::MemoryRetrieval => "MemoryRetrieval",
            Self::InferenceEfficiency => "InferenceEfficiency",
            Self::ToolCapability => "ToolCapability",
            Self::HardwareUtilization => "HardwareUtilization",
            Self::FailureReduction => "FailureReduction",
            Self::ResearchCapability => "ResearchCapability",
            Self::PlanningReliability => "PlanningReliability",
        }
    }
}

/// A strategic objective defining a long-term improvement goal.
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicObjective {
    /// Objective identifier.
    pub id: String,
    /// Objective name.
    pub name: String,
    /// Category.
    pub category: StrategicCategory,
    /// Description.
    pub description: String,
    /// Priority (higher = more important).
    pub priority: u32,
    /// Status (active, completed, suspended, failed).
    pub status: String,
    /// Created timestamp.
    pub created_at: i64,
    /// Updated timestamp.
    pub updated_at: i64,
    /// Related capabilities.
    pub related_capabilities: Vec<String>,
}

impl StrategicObjective {
    /// Create a new strategic objective.
    pub fn new(
        id: &str,
        name: &str,
        category: StrategicCategory,
        description: &str,
        priority: u32,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            description: description.to_string(),
            priority,
            status: "active".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            related_capabilities: Vec::new(),
        }
    }

    /// Mark as completed.
    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Mark as suspended.
    pub fn suspend(&mut self) {
        self.status = "suspended".to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Add related capability.
    pub fn add_capability(&mut self, capability: &str) {
        if !self.related_capabilities.contains(&capability.to_string()) {
            self.related_capabilities.push(capability.to_string());
        }
    }
}

/// A strategic update describing a change to apply.
/// Per Architecture §18.3 (Strategic Updates).
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicUpdate {
    /// Update identifier.
    pub id: String,
    /// Related objective ID.
    pub objective_id: String,
    /// Update action (create_memory, update_confidence, strengthen_relationship, etc.).
    pub action: String,
    /// Target resource.
    pub target: String,
    /// Update details.
    pub details: String,
    /// Status.
    pub status: String,
    /// Created timestamp.
    pub created_at: i64,
    /// Applied timestamp (if completed).
    pub applied_at: Option<i64>,
}

impl StrategicUpdate {
    /// Create a new strategic update.
    pub fn new(id: &str, objective_id: &str, action: &str, target: &str, details: &str) -> Self {
        Self {
            id: id.to_string(),
            objective_id: objective_id.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            details: details.to_string(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            applied_at: None,
        }
    }

    /// Apply the update.
    pub fn apply(&mut self) {
        self.status = "applied".to_string();
        self.applied_at = Some(chrono::Utc::now().timestamp());
    }

    /// Fail the update.
    pub fn fail(&mut self) {
        self.status = "failed".to_string();
    }
}

/// Strategic evaluation assessing progress toward objectives.
/// Per Architecture §18.4 (Strategic Evaluation).
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicEvaluation {
    /// Objective ID.
    pub objective_id: String,
    /// Progress score (0.0 to 1.0).
    pub progress_score: f32,
    /// Evidence count.
    pub evidence_count: u32,
    /// Success indicators.
    pub success_indicators: Vec<String>,
    /// Failure indicators.
    pub failure_indicators: Vec<String>,
    /// Recommendations.
    pub recommendations: Vec<String>,
    /// Evaluated timestamp.
    pub evaluated_at: i64,
}

impl StrategicEvaluation {
    /// Create a new evaluation.
    pub fn new(objective_id: &str, progress_score: f32) -> Self {
        Self {
            objective_id: objective_id.to_string(),
            progress_score: progress_score.clamp(0.0, 1.0),
            evidence_count: 0,
            success_indicators: Vec::new(),
            failure_indicators: Vec::new(),
            recommendations: Vec::new(),
            evaluated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Add success indicator.
    pub fn add_success(&mut self, indicator: &str) {
        self.success_indicators.push(indicator.to_string());
        self.evidence_count += 1;
    }

    /// Add failure indicator.
    pub fn add_failure(&mut self, indicator: &str) {
        self.failure_indicators.push(indicator.to_string());
        self.evidence_count += 1;
    }

    /// Add recommendation.
    pub fn add_recommendation(&mut self, recommendation: &str) {
        self.recommendations.push(recommendation.to_string());
    }
}

/// Strategic feedback connecting execution/planning to strategic objectives.
/// Per Architecture §18.5 (Strategic Feedback) and §12.34 (Execution/Planning Feedback).
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicFeedback {
    /// Source subsystem (execution, planning, learning, etc.).
    pub source: String,
    /// Related objective ID.
    pub objective_id: String,
    /// Feedback type (positive, negative, neutral).
    pub feedback_type: String,
    /// Feedback content.
    pub content: String,
    /// Impact score.
    pub impact_score: f32,
    /// Timestamp.
    pub timestamp: i64,
}

impl StrategicFeedback {
    /// Create new strategic feedback.
    pub fn new(
        source: &str,
        objective_id: &str,
        feedback_type: &str,
        content: &str,
        impact: f32,
    ) -> Self {
        Self {
            source: source.to_string(),
            objective_id: objective_id.to_string(),
            feedback_type: feedback_type.to_string(),
            content: content.to_string(),
            impact_score: impact.clamp(-1.0, 1.0),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// The strategic learning engine manages strategic objectives and updates.
/// Per Architecture §18.1 (Strategic Learning Overview).
#[derive(Debug, Clone, Default)]
pub struct StrategicLearningEngine {
    /// Active strategic objectives.
    objectives: std::collections::HashMap<String, StrategicObjective>,
    /// Pending strategic updates.
    updates: std::collections::HashMap<String, StrategicUpdate>,
    /// Completed evaluations.
    evaluations: std::collections::HashMap<String, StrategicEvaluation>,
    /// Feedback records.
    feedback: Vec<StrategicFeedback>,
}

impl StrategicLearningEngine {
    /// Create a new strategic learning engine.
    pub fn new() -> Self {
        Self {
            objectives: std::collections::HashMap::new(),
            updates: std::collections::HashMap::new(),
            evaluations: std::collections::HashMap::new(),
            feedback: Vec::new(),
        }
    }

    /// Add a strategic objective.
    pub fn add_objective(&mut self, objective: StrategicObjective) {
        self.objectives.insert(objective.id.clone(), objective);
    }

    /// Get an objective by ID.
    pub fn get_objective(&self, id: &str) -> Option<&StrategicObjective> {
        self.objectives.get(id)
    }

    /// Add a strategic update.
    pub fn add_update(&mut self, update: StrategicUpdate) {
        self.updates.insert(update.id.clone(), update);
    }

    /// Apply a strategic update.
    pub fn apply_update(&mut self, id: &str) -> bool {
        if let Some(update) = self.updates.get_mut(id) {
            update.apply();
            true
        } else {
            false
        }
    }

    /// Evaluate strategic progress.
    pub fn evaluate(&mut self, objective_id: &str, progress: f32) -> Option<&StrategicEvaluation> {
        let evaluation = StrategicEvaluation::new(objective_id, progress);
        self.evaluations
            .insert(objective_id.to_string(), evaluation);
        self.evaluations.get(objective_id)
    }

    /// Add strategic feedback.
    pub fn add_feedback(&mut self, feedback: StrategicFeedback) {
        self.feedback.push(feedback);
    }

    /// Get all active objectives.
    pub fn active_objectives(&self) -> Vec<&StrategicObjective> {
        self.objectives
            .values()
            .filter(|o| o.status == "active")
            .collect()
    }

    /// Get all pending updates.
    pub fn pending_updates(&self) -> Vec<&StrategicUpdate> {
        self.updates
            .values()
            .filter(|u| u.status == "pending")
            .collect()
    }
}

/// Active reference to strategic learning contracts.
pub fn reference_strategic_learning() {
    let mut engine = StrategicLearningEngine::new();
    let objective = StrategicObjective::new(
        "obj-1",
        "Improve Memory Retrieval",
        StrategicCategory::MemoryRetrieval,
        "Enhance retrieval accuracy",
        5,
    );
    engine.add_objective(objective);

    let update = StrategicUpdate::new(
        "upd-1",
        "obj-1",
        "update_confidence",
        "memory_record_1",
        "Increase confidence based on evidence",
    );
    engine.add_update(update);
    engine.apply_update("upd-1");

    let evaluation = engine.evaluate("obj-1", 0.75);
    tracing::debug!(
        evaluation_exists = evaluation.is_some(),
        "Strategic evaluation referenced"
    );

    let feedback = StrategicFeedback::new(
        "execution",
        "obj-1",
        "positive",
        "Execution completed successfully",
        0.8,
    );
    engine.add_feedback(feedback);
    tracing::debug!(
        feedback_count = engine.feedback.len(),
        "Strategic feedback referenced"
    );
}
