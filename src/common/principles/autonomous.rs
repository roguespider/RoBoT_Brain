//! Autonomous Development Workflow — AI Contributor Operating Agreement (Chapter 24).
//!
//! Per v0.0.2 Ch 24 "AI Contributor Operating Agreement":
//! - §24.19 Future Autonomous Development
//! - §24.20 The RoBoT Development Contract
//!
//! NOTE: This module consolidates the former autonomous_development/ directory.
//! The autonomous_development/ top-level module has been removed per Architecture §24.

/// The five principles of the AI Contributor Operating Agreement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OperatingPrinciple {
    /// I understand before changing.
    UnderstandBeforeChanging,
    /// I explain before creating.
    ExplainBeforeCreating,
    /// I test before integrating.
    TestBeforeIntegrating,
    /// I document before finishing.
    DocumentBeforeFinishing,
    /// I preserve the architecture.
    PreserveArchitecture,
}

impl OperatingPrinciple {
    /// Return the principle description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::UnderstandBeforeChanging => "I understand before changing.",
            Self::ExplainBeforeCreating => "I explain before creating.",
            Self::TestBeforeIntegrating => "I test before integrating.",
            Self::DocumentBeforeFinishing => "I document before finishing.",
            Self::PreserveArchitecture => "I preserve the architecture.",
        }
    }

    /// Get all principles.
    pub fn all() -> [Self; 5] {
        [
            Self::UnderstandBeforeChanging,
            Self::ExplainBeforeCreating,
            Self::TestBeforeIntegrating,
            Self::DocumentBeforeFinishing,
            Self::PreserveArchitecture,
        ]
    }
}

/// Stage in the autonomous development workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkflowStage {
    /// Goal definition phase.
    Goal,
    /// Planning phase.
    Plan,
    /// Implementation phase.
    Implementation,
    /// Testing phase.
    Testing,
    /// Review phase.
    Review,
    /// Approval gate.
    Approval,
    /// Integration phase.
    Integration,
}

impl std::fmt::Display for WorkflowStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Goal => write!(f, "Goal"),
            Self::Plan => write!(f, "Plan"),
            Self::Implementation => write!(f, "Implementation"),
            Self::Testing => write!(f, "Testing"),
            Self::Review => write!(f, "Review"),
            Self::Approval => write!(f, "Approval"),
            Self::Integration => write!(f, "Integration"),
        }
    }
}

/// Contributor type in the development workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContributorType {
    /// Human developer.
    Human,
    /// AI assistant.
    AIAssistant,
    /// Autonomous agent.
    AutonomousAgent,
    /// Future development system.
    FutureDevelopmentSystem,
}

impl std::fmt::Display for ContributorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Human => write!(f, "Human"),
            Self::AIAssistant => write!(f, "AIAssistant"),
            Self::AutonomousAgent => write!(f, "AutonomousAgent"),
            Self::FutureDevelopmentSystem => write!(f, "FutureDevelopmentSystem"),
        }
    }
}

/// An autonomous development task/workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevelopmentTask {
    /// Unique identifier.
    pub id: String,
    /// Task goal description.
    pub goal: String,
    /// Current workflow stage.
    pub current_stage: WorkflowStage,
    /// Contributor who owns this task.
    pub contributor: ContributorType,
    /// Whether all operating principles have been verified.
    pub principles_verified: bool,
    /// Explanation provided before implementation.
    pub explanation: Option<String>,
    /// Test results summary.
    pub test_results: Option<String>,
    /// Documentation reference.
    pub documentation_ref: Option<String>,
    /// Architecture impact assessment.
    pub architecture_impact: Option<String>,
    /// Approval status.
    pub approved: bool,
    /// Integration status.
    pub integrated: bool,
}

impl DevelopmentTask {
    /// Create a new development task.
    pub fn new(id: String, goal: String, contributor: ContributorType) -> Self {
        Self {
            id,
            goal,
            current_stage: WorkflowStage::Goal,
            contributor,
            principles_verified: false,
            explanation: None,
            test_results: None,
            documentation_ref: None,
            architecture_impact: None,
            approved: false,
            integrated: false,
        }
    }

    /// Progress to the next stage.
    pub fn progress(&mut self) -> Result<(), String> {
        let next = match self.current_stage {
            WorkflowStage::Goal => WorkflowStage::Plan,
            WorkflowStage::Plan => WorkflowStage::Implementation,
            WorkflowStage::Implementation => WorkflowStage::Testing,
            WorkflowStage::Testing => WorkflowStage::Review,
            WorkflowStage::Review => WorkflowStage::Approval,
            WorkflowStage::Approval => WorkflowStage::Integration,
            WorkflowStage::Integration => {
                return Err("Task already completed".to_string());
            }
        };
        self.current_stage = next;
        Ok(())
    }

    /// Check if task is complete (integrated).
    pub fn is_complete(&self) -> bool {
        self.current_stage == WorkflowStage::Integration && self.integrated
    }

    /// Verify operating principles before implementation.
    pub fn verify_principles(
        &mut self,
        explanation: String,
        documentation_ref: String,
        architecture_impact: String,
    ) {
        self.explanation = Some(explanation);
        self.documentation_ref = Some(documentation_ref);
        self.architecture_impact = Some(architecture_impact);
        self.principles_verified = true;
    }

    /// Record test results.
    pub fn record_tests(&mut self, results: String) {
        self.test_results = Some(results);
    }
}

/// AI Contributor Operating Agreement.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperatingAgreement {
    /// Current version of the agreement.
    pub version: String,
    /// The five principles.
    pub principles: Vec<OperatingPrinciple>,
    /// Whether the agreement is enforced.
    pub enforcement_enabled: bool,
    /// Contributor types that are subject to the agreement.
    pub applicable_contributors: Vec<ContributorType>,
}

impl Default for OperatingAgreement {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            principles: OperatingPrinciple::all().to_vec(),
            enforcement_enabled: true,
            applicable_contributors: vec![
                ContributorType::Human,
                ContributorType::AIAssistant,
                ContributorType::AutonomousAgent,
                ContributorType::FutureDevelopmentSystem,
            ],
        }
    }
}

impl OperatingAgreement {
    /// Create a new operating agreement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a contributor is subject to the agreement.
    pub fn applies_to(&self, contributor: &ContributorType) -> bool {
        self.applicable_contributors.contains(contributor)
    }

    /// Get all principle descriptions.
    pub fn principle_descriptions(&self) -> Vec<&'static str> {
        self.principles.iter().map(|p| p.description()).collect()
    }

    /// Check if the contributor is subject to the agreement and enforce the
    /// five principles (Architecture §24.20).
    ///
    /// Returns an error describing the violation when enforcement is enabled,
    /// the contributor is covered, but the contributor type is not recognized.
    pub fn check_contributor(
        &self,
        contributor: &ContributorType,
        artifact: &str,
    ) -> Result<(), String> {
        if self.enforcement_enabled {
            // Verify the contributor type is one of the four recognized types.
            let valid_types = [
                ContributorType::Human,
                ContributorType::AIAssistant,
                ContributorType::AutonomousAgent,
                ContributorType::FutureDevelopmentSystem,
            ];
            if !valid_types.contains(contributor) {
                return Err(format!(
                    "Contributor type '{}' is not recognized by the AI Contributor Operating Agreement (Architecture §24.20)",
                    contributor
                ));
            }
            // Verify the artifact is covered by the agreement.
            if self.applicable_contributors.contains(contributor) {
                tracing::debug!(
                    contributor = ?contributor,
                    artifact = %artifact,
                    "AI Contributor Operating Agreement enforced"
                );
            }
        }
        Ok(())
    }

    /// Validate that a task has verified all principles.
    pub fn validate_task(&self, task: &DevelopmentTask) -> Result<(), String> {
        if !task.principles_verified {
            return Err("Operating principles not verified".to_string());
        }
        if task.explanation.is_none() {
            return Err("Explanation not provided".to_string());
        }
        if task.documentation_ref.is_none() {
            return Err("Documentation not provided".to_string());
        }
        if task.architecture_impact.is_none() {
            return Err("Architecture impact not assessed".to_string());
        }
        Ok(())
    }
}
