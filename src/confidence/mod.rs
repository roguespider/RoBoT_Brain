//! Confidence System — Confidence scoring, propagation, thresholds, audit (Chapter 19).
//!
//! Per Architecture §19.1-19.7:
//! - Confidence scoring for facts, relationships, experiences, skills, workflows, tools, strategies
//! - Confidence propagation through the cognitive pipeline
//! - Confidence thresholds for decision making
//! - Confidence audit trail
//! - Wiring: confidence/ -> memory/ (retention) -> knowledge/ (promotion) ->
//!   retrieval_pipeline/ (ranking) -> execution/ (budget/approval) -> database/ (confidence table)

/// Confidence domain categories.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfidenceDomain {
    /// Fact confidence.
    Fact,
    /// Relationship confidence.
    Relationship,
    /// Experience confidence.
    Experience,
    /// Skill confidence.
    Skill,
    /// Workflow confidence.
    Workflow,
    /// Tool confidence.
    Tool,
    /// Strategy confidence.
    Strategy,
    /// Prompt quality confidence.
    Prompt,
}

impl ConfidenceDomain {
    /// Return domain label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fact => "Fact",
            Self::Relationship => "Relationship",
            Self::Experience => "Experience",
            Self::Skill => "Skill",
            Self::Workflow => "Workflow",
            Self::Tool => "Tool",
            Self::Strategy => "Strategy",
            Self::Prompt => "Prompt",
        }
    }
}

/// A confidence score with provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceScore {
    /// The domain this score applies to.
    pub domain: ConfidenceDomain,
    /// The score value (0.0 to 1.0).
    pub value: f32,
    /// Source of the confidence assessment.
    pub source: String,
    /// Timestamp of assessment.
    pub timestamp: i64,
    /// Evidence supporting this score.
    pub evidence: Vec<String>,
}

impl ConfidenceScore {
    /// Create a new confidence score.
    pub fn new(domain: ConfidenceDomain, value: f32, source: &str) -> Self {
        Self {
            domain,
            value: value.clamp(0.0, 1.0),
            source: source.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            evidence: Vec::new(),
        }
    }

    /// Add evidence.
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence.push(evidence.to_string());
        self
    }

    /// Check if score exceeds a threshold.
    pub fn exceeds(&self, threshold: f32) -> bool {
        self.value >= threshold
    }

    /// Check if score exceeds the high threshold.
    pub fn exceeds_high(&self, thresholds: &ConfidenceThresholds) -> bool {
        self.exceeds(thresholds.high)
    }
}

/// Confidence thresholds for different decision contexts.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceThresholds {
    /// High confidence threshold (e.g., 0.85).
    pub high: f32,
    /// Medium confidence threshold (e.g., 0.60).
    pub medium: f32,
    /// Low confidence threshold (e.g., 0.30).
    pub low: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high: 0.85,
            medium: 0.60,
            low: 0.30,
        }
    }
}

/// A recorded confidence change for audit/history purposes.
#[derive(Debug, Clone)]
pub struct ConfidenceHistoryEntry {
    /// The identifier of the scored item.
    pub id: String,
    /// Previous confidence value.
    pub old_value: f32,
    /// New confidence value.
    pub new_value: f32,
    /// Reason for the change.
    pub reason: String,
    /// Timestamp of the change.
    pub timestamp: i64,
}

/// Evidence-based confidence update direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDirection {
    /// Supporting evidence (increases confidence).
    Support,
    /// Contradicting evidence (decreases confidence).
    Contradict,
}

impl EvidenceDirection {
    /// Return the label for this direction.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Support => "support",
            Self::Contradict => "contradict",
        }
    }
}

/// The confidence system manages scoring, propagation, thresholds, decay, and history.
#[derive(Debug, Clone, Default)]
pub struct ConfidenceSystem {
    /// Active scores by domain and identifier.
    scores: std::collections::HashMap<String, ConfidenceScore>,
    /// Default thresholds.
    thresholds: ConfidenceThresholds,
    /// Confidence history for audit trail (Architecture §19).
    history: Vec<ConfidenceHistoryEntry>,
    /// Maximum history entries to retain (prevents unbounded growth).
    max_history: usize,
}

impl ConfidenceSystem {
    /// Create a new confidence system.
    pub fn new() -> Self {
        Self {
            scores: std::collections::HashMap::new(),
            thresholds: ConfidenceThresholds::default(),
            history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Create a new confidence system with a custom max history size.
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            scores: std::collections::HashMap::new(),
            thresholds: ConfidenceThresholds::default(),
            history: Vec::new(),
            max_history,
        }
    }

    /// Initialize with a custom history size.
    pub fn init_with_custom_history(max_history: usize) -> Self {
        Self::with_max_history(max_history)
    }

    /// Get the current history of confidence changes.
    pub fn get_history(&self) -> &[ConfidenceHistoryEntry] {
        &self.history
    }

    /// Get all score IDs currently tracked.
    pub fn score_ids(&self) -> Vec<String> {
        self.scores.keys().cloned().collect()
    }

    /// Get the count of active scores.
    pub fn score_count(&self) -> usize {
        self.scores.len()
    }

    /// Check if a score exists.
    pub fn has_score(&self, id: &str) -> bool {
        self.scores.contains_key(id)
    }

    /// Record a confidence score.
    pub fn record(&mut self, id: &str, score: ConfidenceScore) {
        let domain_label = match score.domain {
            ConfidenceDomain::Fact => "fact",
            ConfidenceDomain::Relationship => "relationship",
            ConfidenceDomain::Experience => "experience",
            ConfidenceDomain::Skill => "skill",
            ConfidenceDomain::Workflow => "workflow",
            ConfidenceDomain::Tool => "tool",
            ConfidenceDomain::Strategy => "strategy",
            ConfidenceDomain::Prompt => "prompt",
        };
        tracing::debug!(
            domain = domain_label,
            score_id = id,
            "Confidence score recorded"
        );
        self.scores.insert(id.to_string(), score);
    }

    /// Get a confidence score by ID.
    pub fn get(&self, id: &str) -> Option<&ConfidenceScore> {
        self.scores.get(id)
    }

    /// Check if a score exceeds the high threshold.
    pub fn is_high_confidence(&self, id: &str) -> bool {
        self.get(id)
            .map(|s| s.exceeds(self.thresholds.high))
            .unwrap_or(false)
    }

    /// Check if a score exceeds the medium threshold.
    pub fn is_medium_confidence(&self, id: &str) -> bool {
        self.get(id)
            .map(|s| s.exceeds(self.thresholds.medium))
            .unwrap_or(false)
    }

    /// Check if a score exceeds the low threshold.
    pub fn is_low_confidence(&self, id: &str) -> bool {
        self.get(id)
            .map(|s| s.exceeds(self.thresholds.low))
            .unwrap_or(false)
    }

    /// Update thresholds.
    pub fn set_thresholds(&mut self, thresholds: ConfidenceThresholds) {
        self.thresholds = thresholds;
    }

    // ----------------------------------------------------------------
    // Confidence Decay (Architecture §19 "Confidence Decay")
    // ----------------------------------------------------------------

    /// Apply confidence decay to all tracked scores.
    ///
    /// Per Architecture §19: "Knowledge becomes less reliable over time.
    /// Confidence naturally decreases unless refreshed."
    ///
    /// Decay is applied proportionally to how old each score is.
    /// Scores older than `max_age_seconds` get the full decay_rate applied.
    /// Scores within `max_age_seconds` get partial decay proportional to age.
    ///
    /// # Arguments
    /// * `decay_rate` - How much to decay per second (e.g., 0.001 = 0.1% per second)
    /// * `max_age_seconds` - Maximum age before full decay applies (e.g., 3600 = 1 hour)
    ///
    /// Returns the number of scores that were decayed.
    pub fn confidence_decay(&mut self, decay_rate: f32, max_age_seconds: i64) -> usize {
        if decay_rate <= 0.0 {
            return 0;
        }

        let now = chrono::Utc::now().timestamp();
        let mut decayed_count = 0;

        // Collect decay records first to avoid double mutable borrow
        let mut decay_records: Vec<(String, f32, f32, String)> = Vec::new();

        for (id, score) in self.scores.iter_mut() {
            let age = now - score.timestamp;
            if age <= 0 {
                continue;
            }

            let old_value = score.value;

            // Calculate decay factor: 0.0 (no decay) to 1.0 (full decay)
            let decay_factor = if age >= max_age_seconds {
                1.0
            } else {
                (age as f32 / max_age_seconds as f32).clamp(0.0, 1.0)
            };

            // Apply decay: new_value = old_value * (1 - decay_rate * decay_factor)
            let new_value = old_value * (1.0 - decay_rate * decay_factor);
            let new_value = new_value.clamp(0.0, 1.0);

            if new_value != old_value {
                decay_records.push((
                    id.clone(),
                    old_value,
                    new_value,
                    format!("decay: age={age}s, rate={decay_rate}, factor={decay_factor:.3}"),
                ));

                score.value = new_value;
                score.timestamp = now;
                decayed_count += 1;
            }
        }

        // Record history after mutation is complete
        for (id, old_val, new_val, reason) in decay_records {
            self.record_history_entry(id, old_val, new_val, reason);
        }

        if decayed_count > 0 {
            tracing::debug!(
                decayed_scores = decayed_count,
                decay_rate = decay_rate,
                max_age_seconds = max_age_seconds,
                "Confidence decay applied"
            );
        }

        decayed_count
    }

    /// Apply decay to scores of a specific domain.
    pub fn decay_by_domain(
        &mut self,
        domain: &ConfidenceDomain,
        decay_rate: f32,
        max_age_seconds: i64,
    ) -> usize {
        let ids_to_decay: Vec<String> = self
            .scores
            .iter()
            .filter(|(_, s)| s.domain == *domain)
            .map(|(id, _)| id.clone())
            .collect();

        let mut count = 0;
        for id in ids_to_decay {
            let domain_match = self.scores.get(&id).is_some_and(|s| s.domain == *domain);
            if domain_match {
                count += self.confidence_decay_single(&id, decay_rate, max_age_seconds);
            }
        }
        count
    }

    /// Apply decay to a single score (internal helper).
    fn confidence_decay_single(
        &mut self,
        id: &str,
        decay_rate: f32,
        max_age_seconds: i64,
    ) -> usize {
        let now = chrono::Utc::now().timestamp();

        // Collect decay record first to avoid double mutable borrow
        let record = if let Some(score) = self.scores.get_mut(id) {
            let age = now - score.timestamp;
            if age <= 0 {
                return 0;
            }

            let old_value = score.value;
            let decay_factor = if age >= max_age_seconds {
                1.0
            } else {
                (age as f32 / max_age_seconds as f32).clamp(0.0, 1.0)
            };

            let new_value = old_value * (1.0 - decay_rate * decay_factor);
            let new_value = new_value.clamp(0.0, 1.0);

            if new_value != old_value {
                Some((
                    id.to_string(),
                    old_value,
                    new_value,
                    format!("decay: age={age}s, rate={decay_rate}, factor={decay_factor:.3}"),
                ))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((id, old_val, new_val, reason)) = record {
            self.record_history_entry(id, old_val, new_val, reason);
            return 1;
        }
        0
    }

    // ----------------------------------------------------------------
    // Evidence-Based Confidence Updates (Architecture §19 "Confidence Updating")
    // ----------------------------------------------------------------

    /// Update confidence based on evidence.
    ///
    /// Per Architecture §19: "Updates are gradual. A single event should not
    /// radically rewrite understanding."
    ///
    /// # Arguments
    /// * `id` - The identifier of the scored item
    /// * `direction` - Whether evidence supports or contradicts
    /// * `strength` - Strength of evidence (0.0-1.0), affects the update magnitude
    /// * `source` - Source of the evidence
    /// * `evidence_text` - Description of the evidence
    ///
    /// Returns the new confidence value, or None if no prior score exists.
    pub fn update_with_evidence(
        &mut self,
        id: &str,
        direction: EvidenceDirection,
        strength: f32,
        source: &str,
        evidence_text: &str,
    ) -> Option<f32> {
        let strength = strength.clamp(0.0, 1.0);

        let (old_value, new_value) = match self.scores.get_mut(id) {
            Some(score) => {
                let old_value = score.value;
                let new_value = match direction {
                    EvidenceDirection::Support => {
                        // Gradual increase: smaller steps as confidence approaches 1.0
                        let margin = 1.0 - old_value;
                        let increase = margin * strength * 0.1; // 10% of remaining margin
                        (old_value + increase).clamp(0.0, 1.0)
                    }
                    EvidenceDirection::Contradict => {
                        // Gradual decrease: smaller steps as confidence approaches 0.0
                        let decrease = old_value * strength * 0.2; // 20% of current value
                        (old_value - decrease).clamp(0.0, 1.0)
                    }
                };

                // Record evidence
                score.evidence.push(evidence_text.to_string());
                score.source = source.to_string();
                score.timestamp = chrono::Utc::now().timestamp();

                (old_value, new_value)
            }
            None => {
                // No prior score - create initial score based on evidence
                let initial_value = match direction {
                    EvidenceDirection::Support => 0.5 + strength * 0.2,
                    EvidenceDirection::Contradict => 0.5 - strength * 0.2,
                };
                let initial_value = initial_value.clamp(0.0, 1.0);

                let domain = ConfidenceDomain::Fact;
                let domain_label = domain.label();
                let score = ConfidenceScore {
                    domain,
                    value: initial_value,
                    source: source.to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    evidence: vec![evidence_text.to_string()],
                };
                tracing::debug!(
                    domain = domain_label,
                    score_id = id,
                    "Confidence score recorded"
                );
                self.scores.insert(id.to_string(), score);
                return Some(initial_value);
            }
        };

        // Record in history
        self.record_history_entry(
            id.to_string(),
            old_value,
            new_value,
            format!(
                "{}: strength={:.3}, source={}",
                direction.label(),
                strength,
                source
            ),
        );

        tracing::debug!(
            id = id,
            old_value = old_value,
            new_value = new_value,
            direction = direction.label(),
            "Confidence updated with evidence"
        );

        Some(new_value)
    }

    /// Update confidence based on success/failure of an action.
    ///
    /// This is the primary interface for updating confidence based on outcomes.
    /// Success increases confidence, failure decreases it.
    ///
    /// # Arguments
    /// * `id` - The identifier of the scored item
    /// * `success` - Whether the action succeeded
    /// * `tool_or_action` - The tool or action name
    /// * `details` - Additional context
    pub fn record_outcome(
        &mut self,
        id: &str,
        success: bool,
        tool_or_action: &str,
        details: &str,
    ) -> Option<f32> {
        let direction = if success {
            EvidenceDirection::Support
        } else {
            EvidenceDirection::Contradict
        };

        let strength = if success { 0.7 } else { 0.8 }; // Failure is slightly stronger signal

        self.update_with_evidence(id, direction, strength, tool_or_action, details)
    }

    /// Update confidence for a completed goal.
    ///
    /// Uses the goal's expected value and actual outcome to adjust confidence.
    pub fn update_goal_confidence(
        &mut self,
        goal_id: &str,
        expected_value: f32,
        actual_success: bool,
    ) -> Option<f32> {
        if actual_success {
            // Successful goal increases confidence in similar future goals
            let factor = expected_value.clamp(0.0, 1.0);
            self.update_with_evidence(
                goal_id,
                EvidenceDirection::Support,
                factor,
                "goal_completion",
                &format!("Goal completed with expected_value={expected_value:.2}"),
            )
        } else {
            // Failed goal decreases confidence
            let factor = (1.0 - expected_value).clamp(0.0, 1.0);
            self.update_with_evidence(
                goal_id,
                EvidenceDirection::Contradict,
                factor,
                "goal_failure",
                &format!("Goal failed, expected_value={expected_value:.2}"),
            )
        }
    }

    // ----------------------------------------------------------------
    // Decision Calibration (Architecture §19 "Confidence and Decision Making")
    // ----------------------------------------------------------------

    /// Evaluate confidence for a given action type.
    ///
    /// Per Architecture §19: Different actions require different confidence levels.
    ///
    /// # Returns
    /// A decision with context based on current confidence.
    pub fn evaluate_for_action(
        &self,
        id: &str,
        action_type: &ActionType,
    ) -> ActionDecisionWithContext {
        let confidence = match self.scores.get(id) {
            Some(score) => score.value,
            None => 0.0, // No prior knowledge = low confidence
        };

        let required = action_type.required_confidence();
        let recommended = action_type.recommended_confidence();

        let decision = if confidence >= required {
            ActionDecision::Proceed
        } else if confidence >= recommended {
            ActionDecision::ProceedWithCaution
        } else {
            ActionDecision::RequestVerification
        };

        ActionDecisionWithContext {
            confidence,
            required,
            recommended,
            decision,
            explanation: self.explain_confidence(id),
        }
    }

    /// Get a detailed confidence explanation for an ID.
    pub fn explain_confidence(&self, id: &str) -> String {
        match self.scores.get(id) {
            Some(score) => {
                format!(
                    "Confidence: {:.2} ({}) - Based on {} evidence items",
                    score.value,
                    score.domain.label(),
                    score.evidence.len()
                )
            }
            None => "No confidence data available".to_string(),
        }
    }

    /// Get the action decision with its confidence requirements.
    pub fn get_action_decision(
        &self,
        id: &str,
        action_type: &ActionType,
    ) -> ActionDecisionWithContext {
        self.evaluate_for_action(id, action_type)
    }

    // ----------------------------------------------------------------
    // Internal: History Recording
    // ----------------------------------------------------------------

    /// Record a confidence history entry.
    fn record_history_entry(&mut self, id: String, old_value: f32, new_value: f32, reason: String) {
        let entry = ConfidenceHistoryEntry {
            id,
            old_value,
            new_value,
            reason,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.history.push(entry);

        // Trim history to max size (keep most recent entries)
        if self.history.len() > self.max_history {
            self.history.drain(0..self.history.len() - self.max_history);
        }
    }

    /// Get recent history entries (most recent last).
    pub fn get_recent_history(&self, limit: usize) -> &[ConfidenceHistoryEntry] {
        let start = if self.history.len() > limit {
            self.history.len() - limit
        } else {
            0
        };
        &self.history[start..]
    }

    /// Get summary statistics of all tracked scores.
    pub fn summary(&self) -> ConfidenceSummary {
        let values: Vec<f32> = self.scores.values().map(|s| s.value).collect();
        let count = values.len();

        if count == 0 {
            return ConfidenceSummary {
                total_scores: 0,
                avg_confidence: 0.0,
                min_confidence: 0.0,
                max_confidence: 0.0,
                high_confidence_count: 0,
                medium_confidence_count: 0,
                low_confidence_count: 0,
            };
        }

        let avg = values.iter().sum::<f32>() / count as f32;
        let min = values.iter().cloned().fold(f32::MAX, f32::min);
        let max = values.iter().cloned().fold(f32::MIN, f32::max);

        let high = values
            .iter()
            .filter(|v| **v >= self.thresholds.high)
            .count();
        let medium = values
            .iter()
            .filter(|v| **v >= self.thresholds.medium && **v < self.thresholds.high)
            .count();
        let low = values
            .iter()
            .filter(|v| **v < self.thresholds.medium)
            .count();

        ConfidenceSummary {
            total_scores: count,
            avg_confidence: avg,
            min_confidence: min,
            max_confidence: max,
            high_confidence_count: high,
            medium_confidence_count: medium,
            low_confidence_count: low,
        }
    }

    /// Return summary statistics of all tracked scores.
    pub fn get_summary_stats(&self) -> (usize, f32, f32, f32, usize, usize, usize) {
        let s = self.summary();
        (
            s.total_scores,
            s.avg_confidence,
            s.min_confidence,
            s.max_confidence,
            s.high_confidence_count,
            s.medium_confidence_count,
            s.low_confidence_count,
        )
    }

    /// Create a domain-specific confidence score.
    ///
    /// Per Architecture §19: each domain (fact, relationship, experience, skill,
    /// workflow, tool, strategy, prompt) may need its own confidence tracking.
    /// This method creates an initial score for the given domain.
    pub fn init_domain_score(
        &mut self,
        domain: ConfidenceDomain,
        id: &str,
        initial_value: f32,
        source: &str,
    ) {
        let domain_label = domain.label();
        let score = ConfidenceScore {
            domain,
            value: initial_value.clamp(0.0, 1.0),
            source: source.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            evidence: Vec::new(),
        };
        tracing::debug!(
            domain = domain_label,
            score_id = id,
            initial_value = initial_value,
            "Initializing domain confidence score"
        );
        self.scores.insert(id.to_string(), score);
        self.record_history_entry(
            id.to_string(),
            0.0,
            initial_value.clamp(0.0, 1.0),
            format!("init_domain: domain={}", domain_label),
        );
    }

    /// Initialize domain scores for all domains (wires ConfidenceDomain variants).
    pub fn init_all_domains(&mut self) {
        let domains = [
            (ConfidenceDomain::Fact, "system/fact"),
            (ConfidenceDomain::Relationship, "system/relationship"),
            (ConfidenceDomain::Experience, "system/experience"),
            (ConfidenceDomain::Skill, "system/skill"),
            (ConfidenceDomain::Workflow, "system/workflow"),
            (ConfidenceDomain::Tool, "system/tool"),
            (ConfidenceDomain::Strategy, "system/strategy"),
            (ConfidenceDomain::Prompt, "system/prompt"),
        ];
        for (domain, id) in &domains {
            self.init_domain_score(domain.clone(), id, 0.5, "bootstrap");
        }
    }
}

/// The type of action being evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    /// Read-only operation (low confidence required).
    Read,
    /// Configuration change (medium confidence required).
    ModifyConfig,
    /// Data modification (high confidence required).
    ModifyData,
    /// Data deletion (very high confidence required).
    Delete,
    /// External API call (medium confidence required).
    ExternalCall,
    /// Skill execution (medium-high confidence required).
    SkillExecution,
    /// Workflow execution (high confidence required).
    WorkflowExecution,
}

impl ActionType {
    /// Get the required confidence threshold for this action type.
    ///
    /// Per Architecture §19: "Risk determines required certainty."
    pub fn required_confidence(&self) -> f32 {
        match self {
            Self::Read => 0.30,
            Self::ModifyConfig => 0.60,
            Self::ModifyData => 0.75,
            Self::Delete => 0.90,
            Self::ExternalCall => 0.60,
            Self::SkillExecution => 0.70,
            Self::WorkflowExecution => 0.80,
        }
    }

    /// Get the recommended confidence threshold for this action type.
    ///
    /// Recommended is slightly lower than required - you can proceed with caution.
    pub fn recommended_confidence(&self) -> f32 {
        match self {
            Self::Read => 0.20,
            Self::ModifyConfig => 0.50,
            Self::ModifyData => 0.65,
            Self::Delete => 0.80,
            Self::ExternalCall => 0.50,
            Self::SkillExecution => 0.60,
            Self::WorkflowExecution => 0.70,
        }
    }

    /// Get a human-readable label for this action type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::ModifyConfig => "ModifyConfig",
            Self::ModifyData => "ModifyData",
            Self::Delete => "Delete",
            Self::ExternalCall => "ExternalCall",
            Self::SkillExecution => "SkillExecution",
            Self::WorkflowExecution => "WorkflowExecution",
        }
    }

    pub fn label_for_decision(&self) -> &'static str {
        self.label()
    }
}

/// The decision returned by confidence evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionDecision {
    /// Confident enough to proceed without additional verification.
    Proceed,
    /// Proceed with caution - some verification recommended.
    ProceedWithCaution,
    /// Request human verification before proceeding.
    RequestVerification,
}

impl ActionDecision {
    /// Return the decision as a string label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::ProceedWithCaution => "proceed_with_caution",
            Self::RequestVerification => "request_verification",
        }
    }

    pub fn decision_label(&self) -> &'static str {
        self.label()
    }
}

/// Context for an action decision.
#[derive(Debug, Clone)]
pub struct ActionDecisionWithContext {
    /// Current confidence value.
    pub confidence: f32,
    /// Required confidence threshold.
    pub required: f32,
    /// Recommended confidence threshold.
    pub recommended: f32,
    /// The decision.
    pub decision: ActionDecision,
    /// Human-readable explanation.
    pub explanation: String,
}

/// Summary statistics of all tracked confidence scores.
#[derive(Debug, Clone)]
pub struct ConfidenceSummary {
    /// Total number of tracked scores.
    pub total_scores: usize,
    /// Average confidence across all scores.
    pub avg_confidence: f32,
    /// Minimum confidence.
    pub min_confidence: f32,
    /// Maximum confidence.
    pub max_confidence: f32,
    /// Count of high-confidence scores.
    pub high_confidence_count: usize,
    /// Count of medium-confidence scores.
    pub medium_confidence_count: usize,
    /// Count of low-confidence scores.
    pub low_confidence_count: usize,
}
