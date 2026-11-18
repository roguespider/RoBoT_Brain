//! Confidence action types — action categories, decisions, and decision context.

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
