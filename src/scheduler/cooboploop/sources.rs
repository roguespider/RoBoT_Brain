// /src/CoObOpLoop/sources.rs
// Objective source definitions for the CoObOpLoop system.

use super::queue::AgentGoal;

/// Human input source detail variants (§3.1).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HumanOrigin {
    UserRequest,
    Instruction,
    Correction,
    Project,
    MaintenanceRequest,
    StrategicGoal,
}

/// External opportunity source detail variants (§3.2).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExternalSource {
    FreelanceJob,
    DevBounty,
    ResearchOpportunity,
    Grant,
    Competition,
    OpenSourceTask,
    AvailableProject,
    HardwareOpportunity,
    UserRequest,
}

/// System-triggered source detail variants (§3.3).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SystemTrigger {
    UnresolvedError,
    FailedTest,
    DetectedBug,
    DegradedPerformance,
    MemoryInconsistency,
    HardwareProblem,
    SoftwareDependencyProblem,
    StaleComponent,
    MissingDocumentation,
    SecurityIssue,
    ReliabilityIssue,
    IncompleteImplementation,
    FailedExperiment,
}

/// Learning-triggered source detail variants (§3.4).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LearningTrigger {
    RepeatedFailure,
    InsufficientUnderstanding,
    RepeatedHumanIntervention,
    CapabilityGap,
}

/// Self-improvement source detail variants (§3.5).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImprovementTarget {
    ReasoningWorkflow,
    Planning,
    ToolUsage,
    MemoryRetrieval,
    MemoryOrganization,
    ExecutionReliability,
    Testing,
    HardwareUtilization,
    InferencePerformance,
    SoftwareArchitecture,
    ResourceUtilization,
    ErrorDetection,
    RecoveryProcedure,
}

/// Objective source variant types (§3 — 5 core + 4 extensions + Research §11).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectiveSource {
    HumanOrigin,
    ExternalOpportunity,
    SystemTrigger,
    LearningTarget,
    StrategicObjective,
    ImprovementTarget,
    HardwareUtilization,
    InferencePerformance,
    SoftwareArchitecture,
    Research,
}

/// Trait for objective source providers.
pub trait ObjectiveSourceProvider: Send + Sync {
    fn source_type(&self) -> ObjectiveSource;
    fn discover(&self) -> Vec<AgentGoal>;
    fn name(&self) -> &str;
}

/// Registry of all objective source providers.
pub struct ObjectiveSourceRegistry {
    providers: Vec<Box<dyn ObjectiveSourceProvider>>,
}

impl ObjectiveSourceRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn ObjectiveSourceProvider>) {
        self.providers.push(provider);
    }

    /// Wire source_type and name by using them in registry discovery.
    pub fn list_providers(&self) -> Vec<(String, ObjectiveSource)> {
        self.providers
            .iter()
            .map(|p| (p.name().to_string(), p.source_type()))
            .collect()
    }

    pub fn discover_all(&self) -> Vec<AgentGoal> {
        let mut objectives = Vec::new();
        for provider in &self.providers {
            objectives.extend(provider.discover());
        }
        objectives
    }

    /// Initialize registry with all 5 standard sources (§A.8).
    pub fn init() -> Self {
        let mut reg = Self::new();
        // Wire list_providers / source_type / name by calling list_providers
        let providers = reg.list_providers();
        if providers.is_empty() {
            tracing::warn!("provider registry empty before init");
        }
        reg.register(Box::new(HumanInputSource));
        reg.register(Box::new(SystemGeneratedSource));
        reg.register(Box::new(LearningObjectiveSource));
        reg.register(Box::new(SelfImprovementSource));
        reg.register(Box::new(ExternalOpportunitySource));
        reg
    }
}

impl Default for ObjectiveSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Source provider stubs
// ---------------------------------------------------------------------------

/// Human input source provider.
pub struct HumanInputSource;

impl ObjectiveSourceProvider for HumanInputSource {
    fn source_type(&self) -> ObjectiveSource {
        ObjectiveSource::HumanOrigin
    }

    fn discover(&self) -> Vec<AgentGoal> {
        // Real human input detection requires live interaction; no fabricated goals.
        Vec::new()
    }

    fn name(&self) -> &str {
        "human_input"
    }
}

impl HumanInputSource {
    pub fn sample_goal(detail: HumanOrigin, title: &str) -> AgentGoal {
        AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("Human goal: {:?}", detail),
            status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::HumanOrigin,
            expected_value: 1.0,
            risk: 0.3,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        }
    }
}

/// System-generated source provider.
pub struct SystemGeneratedSource;

impl ObjectiveSourceProvider for SystemGeneratedSource {
    fn source_type(&self) -> ObjectiveSource {
        ObjectiveSource::SystemTrigger
    }

    fn discover(&self) -> Vec<AgentGoal> {
        // Real system trigger detection requires live monitoring; no fabricated goals.
        Vec::new()
    }

    fn name(&self) -> &str {
        "system"
    }
}

impl SystemGeneratedSource {
    pub fn sample_goal(detail: SystemTrigger, title: &str) -> AgentGoal {
        AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("System goal: {:?}", detail),
            status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::SystemTrigger,
            expected_value: 1.0,
            risk: 0.3,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        }
    }
}

/// Learning objective source provider.
pub struct LearningObjectiveSource;

impl ObjectiveSourceProvider for LearningObjectiveSource {
    fn source_type(&self) -> ObjectiveSource {
        ObjectiveSource::LearningTarget
    }

    fn discover(&self) -> Vec<AgentGoal> {
        // Real learning objective detection requires experience analysis; no fabricated goals.
        Vec::new()
    }

    fn name(&self) -> &str {
        "learning"
    }
}

impl LearningObjectiveSource {
    pub fn sample_goal(detail: LearningTrigger, title: &str) -> AgentGoal {
        AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("Learning goal: {:?}", detail),
            status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::LearningTarget,
            expected_value: 1.0,
            risk: 0.3,
            learning_value: 1.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        }
    }
}

/// Self-improvement source provider.
pub struct SelfImprovementSource;

impl ObjectiveSourceProvider for SelfImprovementSource {
    fn source_type(&self) -> ObjectiveSource {
        ObjectiveSource::ImprovementTarget
    }

    fn discover(&self) -> Vec<AgentGoal> {
        // Real self-improvement detection requires capability assessment; no fabricated goals.
        Vec::new()
    }

    fn name(&self) -> &str {
        "self_improvement"
    }
}

impl SelfImprovementSource {
    pub fn sample_goal(detail: ImprovementTarget, title: &str) -> AgentGoal {
        AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("Improvement goal: {:?}", detail),
            status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::ImprovementTarget,
            expected_value: 1.0,
            risk: 0.3,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        }
    }
}

/// External opportunity source provider (§3.2).
/// Stub adapters that return empty results when external API keys are not configured.
/// Live fetching requires integration with external APIs (reqwest/ureq) with proper credentials.
/// See architecture spec §17 for adapter requirements.
pub struct ExternalOpportunitySource;

impl ObjectiveSourceProvider for ExternalOpportunitySource {
    fn source_type(&self) -> ObjectiveSource {
        ObjectiveSource::ExternalOpportunity
    }

    /// Wired external opportunity discovery (§3.2 / T-COO-40).
    /// Uses stub adapters (Fiverr, Upwork, GitHub) to fetch opportunities.
    /// Adapters return empty Vec until HTTP integration is added.
    /// Per CoObOpLoop.md §3: only genuinely detected objectives SHALL enter the queue.
    fn discover(&self) -> Vec<AgentGoal> {
        use super::opportunity::{
            FiverrAdapter, GitHubIssuesAdapter, OpportunityAdapter, OpportunityIntake,
            UpworkAdapter,
        };
        let adapters: Vec<Box<dyn OpportunityAdapter>> = vec![
            Box::new(FiverrAdapter::new(
                "https://www.fiverr.com/search/gigs?query=development".to_string(),
            )),
            Box::new(UpworkAdapter::new(
                "https://www.upwork.com/jobs/search?q=development".to_string(),
            )),
            Box::new(GitHubIssuesAdapter::new(
                "https://github.com/search?q=bug&type=Issues".to_string(),
            )),
        ];
        let intake = OpportunityIntake::new();
        let mut goals: Vec<AgentGoal> = Vec::new();
        for adapter in adapters {
            let opportunities = adapter.fetch();
            for opportunity in opportunities {
                let result = intake.process(
                    &opportunity,
                    &crate::scheduler::cooboploop::capability::CapabilityRegistry::new(),
                );
                if let Some(goal) = OpportunityIntake::intake_result_to_goal(&result) {
                    goals.push(goal);
                }
            }
        }
        if goals.is_empty() {
            // No live feeds returned data; return empty rather than fabricated placeholder.
            // Per CoObOpLoop.md §3: only genuinely detected objectives SHALL enter the queue.
        }
        goals
    }

    fn name(&self) -> &str {
        "external"
    }
}

impl ExternalOpportunitySource {
    pub fn sample_goal(detail: ExternalSource, title: &str) -> AgentGoal {
        AgentGoal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("External goal: {:?}", detail),
            status: crate::scheduler::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: ObjectiveSource::ExternalOpportunity,
            expected_value: 1.0,
            risk: 0.5,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        }
    }
}
