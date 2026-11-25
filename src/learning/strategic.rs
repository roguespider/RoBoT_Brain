//! Strategic Learning — Strategic objectives, strategic updates, strategic evaluation,
//! strategic feedback (Architecture Chapter 18).
//!
//! Per Architecture §18.1-18.5:
//! - Strategic objectives: long-term improvement goals (§18.2)
//! - Strategic updates: changes to capabilities, skills, workflows (§18.3)
//! - Strategic evaluation: assessment of strategic progress (§18.4)
//! - Strategic feedback: feedback loop to execution/planning (§18.5)
//! - Wiring: learning/strategic/ -> cooboploop/ (existing loop) -> learning/ (ch 10) ->
//!   execution/ (ch 12, feedback loop 12.34) -> database/ (strategic updates persistence)

/// Strategic objective categories per Architecture §18.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum StrategicCategory {
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
pub(crate) struct StrategicObjective {
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
pub(crate) struct StrategicUpdate {
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
pub(crate) struct StrategicEvaluation {
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
pub(crate) struct StrategicFeedback {
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
pub(crate) struct StrategicLearningEngine {
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

    /// Add a strategic update (Architecture §18.3).
    pub fn add_update(&mut self, update: StrategicUpdate) {
        self.updates.insert(update.id.clone(), update);
    }

    /// Add success indicator to strategic evaluation (Architecture §18.4).
    pub fn add_evaluation_success(&mut self, objective_id: &str, indicator: &str) {
        if let Some(eval) = self.evaluations.get_mut(objective_id) {
            eval.add_success(indicator);
        }
    }

    /// Add failure indicator to strategic evaluation (Architecture §18.4).
    pub fn add_evaluation_failure(&mut self, objective_id: &str, indicator: &str) {
        if let Some(eval) = self.evaluations.get_mut(objective_id) {
            eval.add_failure(indicator);
        }
    }

    /// Add recommendation to strategic evaluation (Architecture §18.4).
    pub fn add_evaluation_recommendation(&mut self, objective_id: &str, recommendation: &str) {
        if let Some(eval) = self.evaluations.get_mut(objective_id) {
            eval.add_recommendation(recommendation);
        }
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

    /// Update a strategic objective by ID (modifies in place).
    pub fn update_objective<F>(&mut self, id: &str, f: F)
    where
        F: Fn(&mut StrategicObjective),
    {
        if let Some(obj) = self.objectives.get_mut(id) {
            f(obj);
        }
    }

    /// Process strategic feedback: update objectives and refine strategies.
    /// Per Architecture §18.5: feedback SHALL form a continuous improvement loop
    /// by updating strategic objectives and generating strategic updates.
    pub fn process_feedback(&mut self) -> Vec<String> {
        let mut processed = Vec::new();
        // Process each feedback record to update objectives or create updates.
        for feedback in self.feedback.iter() {
            // Positive feedback: add success indicator to related objective evaluation.
            if let Some(obj) = self.objectives.get_mut(&feedback.objective_id)
                && feedback.feedback_type == "positive"
            {
                obj.status = "active".to_string();
                obj.updated_at = chrono::Utc::now().timestamp();
                processed.push(format!(
                    "positive_feedback_applied: {} -> {}",
                    feedback.objective_id, feedback.content
                ));
            }
            // Negative feedback: add failure indicator and create strategic update.
            if let Some(obj) = self.objectives.get_mut(&feedback.objective_id)
                && feedback.feedback_type == "negative"
            {
                obj.status = "active".to_string();
                obj.updated_at = chrono::Utc::now().timestamp();
                // Create a strategic update to address the negative feedback.
                let update_id = format!("feedback_update_{}", feedback.timestamp);
                let update = StrategicUpdate::new(
                    &update_id,
                    &feedback.objective_id,
                    "review",
                    &feedback.content,
                    &format!("Address negative feedback: {}", feedback.content),
                );
                self.updates.insert(update.id.clone(), update);
                processed.push(format!(
                    "negative_feedback_update_created: {} -> {}",
                    feedback.objective_id, update_id
                ));
            }
            // Neutral feedback: add recommendation to evaluation.
            if let Some(eval) = self.evaluations.get_mut(&feedback.objective_id)
                && feedback.feedback_type == "neutral"
            {
                eval.add_recommendation(&feedback.content);
                processed.push(format!(
                    "neutral_feedback_recommendation: {} -> {}",
                    feedback.objective_id, feedback.content
                ));
            }
        }
        // Clear processed feedback after applying updates.
        self.feedback.clear();
        processed
    }

    /// Update objective priorities based on feedback impact scores.
    /// Per Architecture §18.5: feedback SHALL update strategic objectives.
    /// Positive feedback increases priority; negative feedback decreases it.
    pub fn update_objectives_from_feedback(&mut self) {
        for feedback in self.feedback.iter() {
            if let Some(obj) = self.objectives.get_mut(&feedback.objective_id) {
                // Adjust priority based on feedback impact score.
                // Positive impact (0.0-1.0): increase priority
                // Negative impact (-1.0-0.0): decrease priority
                let impact = feedback.impact_score;
                let priority_delta = if impact > 0.0 {
                    (impact * 10.0) as u32
                } else {
                    ((-impact) * 10.0) as u32
                };
                obj.priority = if impact > 0.0 {
                    obj.priority.saturating_add(priority_delta)
                } else {
                    obj.priority.saturating_sub(priority_delta)
                };
                obj.updated_at = chrono::Utc::now().timestamp();
            }
        }
    }

    /// Refine strategies based on accumulated feedback patterns.
    /// Per Architecture §18.4-18.5: evaluation results SHALL refine future strategies.
    pub fn refine_strategies(&mut self) -> Vec<String> {
        let mut refined = Vec::new();
        // Analyze feedback patterns to identify strategy improvements.
        let positive_count = self
            .feedback
            .iter()
            .filter(|f| f.feedback_type == "positive")
            .count();
        let negative_count = self
            .feedback
            .iter()
            .filter(|f| f.feedback_type == "negative")
            .count();
        if positive_count > negative_count * 2 && !self.feedback.is_empty() {
            // Strong positive signal: reinforce current strategy.
            for obj in self.objectives.values_mut() {
                if obj.status == "active" {
                    obj.priority = obj.priority.saturating_add(1);
                    refined.push(format!(
                        "strategy_reinforced: {} (positive_ratio={:.2})",
                        obj.id,
                        positive_count as f32 / (positive_count + negative_count) as f32
                    ));
                }
            }
        } else if negative_count > positive_count && !self.feedback.is_empty() {
            // Negative signal: flag objectives for review.
            for obj in self.objectives.values_mut() {
                if obj.status == "active" {
                    obj.updated_at = chrono::Utc::now().timestamp();
                    refined.push(format!(
                        "strategy_flagged_for_review: {} (negative_ratio={:.2})",
                        obj.id,
                        negative_count as f32 / (positive_count + negative_count) as f32
                    ));
                }
            }
        }
        refined
    }

    /// Generate new objectives from feedback patterns.
    /// Per Architecture §18.5: feedback SHALL generate new learning objectives.
    pub fn generate_objectives_from_feedback(&self) -> Vec<StrategicObjective> {
        let mut objectives = Vec::new();
        // Find objectives with high negative feedback rates.
        for feedback in self.feedback.iter() {
            if feedback.feedback_type == "negative"
                && let Some(obj) = self.objectives.get(&feedback.objective_id)
            {
                let new_id = format!("improvement_{}_{}", obj.id, uuid::Uuid::new_v4());
                let improvement_obj = StrategicObjective::new(
                    &new_id,
                    &format!("Improve: {}", obj.name),
                    obj.category.clone(),
                    &format!(
                        "Address repeated failures: {} (impact: {})",
                        feedback.content, feedback.impact_score
                    ),
                    obj.priority.saturating_add(2),
                );
                objectives.push(improvement_obj);
            }
        }
        objectives
    }

    /// Update a strategic update by ID (modifies in place).
    pub fn update_strategic_update<F>(&mut self, id: &str, f: F)
    where
        F: Fn(&mut StrategicUpdate),
    {
        if let Some(update) = self.updates.get_mut(id) {
            f(update);
        }
    }
}
