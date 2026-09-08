// /src/CoObOpLoop/research.rs
// Research system for the CoObOpLoop system.

/// Trigger for research (§11 / T8.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResearchTrigger {
    UnavailableInfo,
    HighUncertainty,
    CapabilityGap,
    TechnologyInvestigation,
    HardwareUpgrade,
    MultipleSolutions,
    PreviousFailure,
    ExternalOpportunityKnowledgeGap,
}

/// Persistence target for research results (§11 / T8.3).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PersistenceTarget {
    KnowledgeBase,
    ExperienceLog,
    Both,
}

/// Research objective (§11 / T8.2).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchObjective {
    pub topic: String,
    pub trigger: ResearchTrigger,
    pub persistence_target: PersistenceTarget,
    pub expected_knowledge: String,
}

/// Research manager.
pub struct ResearchManager {
    objectives: Vec<ResearchObjective>,
    next_id: u64,
}

impl ResearchManager {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a research objective, returns the assigned ID (§T8.4).
    pub fn create_objective(
        &mut self,
        topic: String,
        trigger: ResearchTrigger,
        persistence_target: PersistenceTarget,
        expected_knowledge: String,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.objectives.push(ResearchObjective {
            topic,
            trigger,
            persistence_target,
            expected_knowledge,
        });
        id
    }

    pub fn list(&self) -> &Vec<ResearchObjective> {
        &self.objectives
    }
}

impl Default for ResearchManager {
    fn default() -> Self {
        Self::new()
    }
}
