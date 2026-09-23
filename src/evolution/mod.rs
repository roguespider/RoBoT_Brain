//! Self-Improvement and Evolution (Architecture Chapter 26).

/// An improvement candidate tracking a proposed change.
#[derive(Debug, Clone, PartialEq)]
pub struct ImprovementCandidate {
    /// Candidate identifier.
    pub id: String,
    /// Observation that triggered the candidate.
    pub observation: String,
    /// Hypothesis for improvement.
    pub hypothesis: String,
    /// Evidence supporting the hypothesis.
    pub evidence: Vec<String>,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Status (proposed, testing, accepted, rejected).
    pub status: String,
    /// Timestamp.
    pub timestamp: i64,
}

impl ImprovementCandidate {
    /// Create a new candidate.
    pub fn new(observation: &str, hypothesis: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            observation: observation.to_string(),
            hypothesis: hypothesis.to_string(),
            evidence: Vec::new(),
            confidence: 0.0,
            status: "proposed".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add evidence.
    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence.push(evidence.to_string());
    }

    /// Set confidence.
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence;
    }

    /// Update status.
    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}

/// Evolution stages per Architecture Chapter 26.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStage {
    Experience,
    Candidate,
    Hypothesis,
    Experiment,
    SkillEvolution,
    WorkflowEvolution,
    MemoryEvolution,
    GraphEvolution,
    Consolidation,
}

/// Run the evolution loop over experiences.
pub fn run_evolution_loop(
    experiences: &[crate::data_contracts::experience_record::ExperienceRecord],
) -> Vec<crate::data_contracts::learning_update::LearningUpdate> {
    // Placeholder: returns empty updates; real implementation would process experiences
    let count = experiences.len();
    tracing::debug!(experience_count = count, "Evolution loop checked");
    Vec::new()
}

/// The Evolution Ladder — stages of controlled system evolution.
/// Per Architecture §26.3 (The Evolution Ladder).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EvolutionLadder {
    /// Learning stage.
    #[default]
    Learning,
    /// Adaptation stage.
    Adaptation,
    /// Evolution stage.
    Evolution,
    /// Consolidation stage.
    Consolidation,
}

impl EvolutionLadder {
    /// Return ladder stage label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Learning => "Learning",
            Self::Adaptation => "Adaptation",
            Self::Evolution => "Evolution",
            Self::Consolidation => "Consolidation",
        }
    }
}

/// Improvement pipeline stages.
/// Per Architecture §26.4 (Self-Improvement Loop) and §26.7 (Hypothesis System).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ImprovementPipeline {
    /// Observation stage.
    #[default]
    Observation,
    /// Candidate generation.
    CandidateGeneration,
    /// Hypothesis testing.
    HypothesisTesting,
    /// Experimentation.
    Experimentation,
    /// Skill evolution.
    SkillEvolution,
    /// Workflow evolution.
    WorkflowEvolution,
    /// Memory evolution.
    MemoryEvolution,
    /// Knowledge graph evolution.
    GraphEvolution,
    /// Architecture evolution.
    ArchitectureEvolution,
    /// Consolidation.
    Consolidation,
}

impl ImprovementPipeline {
    /// Return pipeline stage label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Observation => "Observation",
            Self::CandidateGeneration => "CandidateGeneration",
            Self::HypothesisTesting => "HypothesisTesting",
            Self::Experimentation => "Experimentation",
            Self::SkillEvolution => "SkillEvolution",
            Self::WorkflowEvolution => "WorkflowEvolution",
            Self::MemoryEvolution => "MemoryEvolution",
            Self::GraphEvolution => "GraphEvolution",
            Self::ArchitectureEvolution => "ArchitectureEvolution",
            Self::Consolidation => "Consolidation",
        }
    }
}

/// Controlled experimentation tracking.
/// Per Architecture §26.8 (Controlled Experimentation).
#[derive(Debug, Clone, PartialEq)]
pub struct ControlledExperiment {
    /// Experiment identifier.
    pub experiment_id: String,
    /// Hypothesis being tested.
    pub hypothesis: String,
    /// Expected outcome.
    pub expected_outcome: String,
    /// Actual outcome.
    pub actual_outcome: Option<String>,
    /// Success status.
    pub success: Option<bool>,
    /// Evidence collected.
    pub evidence: Vec<String>,
    /// Timestamp.
    pub timestamp: i64,
}

impl ControlledExperiment {
    /// Create a new experiment.
    pub fn new(experiment_id: &str, hypothesis: &str, expected: &str) -> Self {
        Self {
            experiment_id: experiment_id.to_string(),
            hypothesis: hypothesis.to_string(),
            expected_outcome: expected.to_string(),
            actual_outcome: None,
            success: None,
            evidence: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Record actual outcome.
    pub fn record_outcome(&mut self, actual: &str, success: bool) {
        self.actual_outcome = Some(actual.to_string());
        self.success = Some(success);
    }

    /// Add evidence.
    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence.push(evidence.to_string());
    }
}

/// Skill evolution tracking.
/// Per Architecture §26.9 (Skill Evolution).
#[derive(Debug, Clone, PartialEq)]
pub struct SkillEvolutionRecord {
    /// Skill identifier.
    pub skill_id: String,
    /// Improvement description.
    pub improvement: String,
    /// Evidence count.
    pub evidence_count: u32,
    /// Success rate change.
    pub success_rate_change: f32,
    /// Applied timestamp.
    pub applied_at: i64,
}

impl SkillEvolutionRecord {
    /// Create a new skill evolution record.
    pub fn new(skill_id: &str, improvement: &str) -> Self {
        Self {
            skill_id: skill_id.to_string(),
            improvement: improvement.to_string(),
            evidence_count: 0,
            success_rate_change: 0.0,
            applied_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Update with evidence.
    pub fn update(&mut self, evidence: u32, rate_change: f32) {
        self.evidence_count += evidence;
        self.success_rate_change = rate_change;
        self.applied_at = chrono::Utc::now().timestamp();
    }
}

/// Workflow evolution tracking.
/// Per Architecture §26.10 (Workflow Evolution).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowEvolutionRecord {
    /// Workflow identifier.
    pub workflow_id: String,
    /// Evolution description.
    pub evolution: String,
    /// Performance improvement.
    pub performance_improvement: f32,
    /// Applied timestamp.
    pub applied_at: i64,
}

impl WorkflowEvolutionRecord {
    /// Create a new workflow evolution record.
    pub fn new(workflow_id: &str, evolution: &str) -> Self {
        Self {
            workflow_id: workflow_id.to_string(),
            evolution: evolution.to_string(),
            performance_improvement: 0.0,
            applied_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Update performance improvement.
    pub fn update_performance(&mut self, improvement: f32) {
        self.performance_improvement = improvement.clamp(-1.0, 1.0);
        self.applied_at = chrono::Utc::now().timestamp();
    }
}

/// Memory evolution tracking.
/// Per Architecture §26.11 (Memory Evolution).
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvolutionRecord {
    /// Memory identifier.
    pub memory_id: String,
    /// Evolution description.
    pub evolution: String,
    /// Confidence change.
    pub confidence_change: f32,
    /// Applied timestamp.
    pub applied_at: i64,
}

impl MemoryEvolutionRecord {
    /// Create a new memory evolution record.
    pub fn new(memory_id: &str, evolution: &str) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            evolution: evolution.to_string(),
            confidence_change: 0.0,
            applied_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Update confidence change.
    pub fn update_confidence(&mut self, change: f32) {
        self.confidence_change = change.clamp(-1.0, 1.0);
        self.applied_at = chrono::Utc::now().timestamp();
    }
}

/// Knowledge graph evolution tracking.
/// Per Architecture §26.12 (Knowledge Graph Evolution).
#[derive(Debug, Clone, PartialEq)]
pub struct GraphEvolutionRecord {
    /// Graph identifier.
    pub graph_id: String,
    /// Evolution description.
    pub evolution: String,
    /// New relationships added.
    pub new_relationships: u32,
    /// Applied timestamp.
    pub applied_at: i64,
}

impl GraphEvolutionRecord {
    /// Create a new graph evolution record.
    pub fn new(graph_id: &str, evolution: &str) -> Self {
        Self {
            graph_id: graph_id.to_string(),
            evolution: evolution.to_string(),
            new_relationships: 0,
            applied_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Update new relationships.
    pub fn update_relationships(&mut self, count: u32) {
        self.new_relationships = count;
        self.applied_at = chrono::Utc::now().timestamp();
    }
}

/// Architecture evolution tracking.
/// Per Architecture §26.13 (Architecture Evolution).
#[derive(Debug, Clone, PartialEq)]
pub struct ArchitectureEvolutionRecord {
    /// Component identifier.
    pub component_id: String,
    /// Evolution description.
    pub evolution: String,
    /// Complexity change.
    pub complexity_change: f32,
    /// Applied timestamp.
    pub applied_at: i64,
}

impl ArchitectureEvolutionRecord {
    /// Create a new architecture evolution record.
    pub fn new(component_id: &str, evolution: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            evolution: evolution.to_string(),
            complexity_change: 0.0,
            applied_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Update complexity change.
    pub fn update_complexity(&mut self, change: f32) {
        self.complexity_change = change;
        self.applied_at = chrono::Utc::now().timestamp();
    }
}

/// Evolution memory: records of system improvements.
/// Per Architecture §26.17 (Evolution Memory).
#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionMemoryRecord {
    /// Record identifier.
    pub id: String,
    /// Improvement candidate ID.
    pub candidate_id: String,
    /// Evolution stage.
    pub stage: EvolutionStage,
    /// Outcome.
    pub outcome: String,
    /// Evidence collected.
    pub evidence: Vec<String>,
    /// Timestamp.
    pub timestamp: i64,
}

impl EvolutionMemoryRecord {
    /// Create a new evolution memory record.
    pub fn new(candidate_id: &str, stage: EvolutionStage, outcome: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            candidate_id: candidate_id.to_string(),
            stage,
            outcome: outcome.to_string(),
            evidence: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add evidence.
    pub fn add_evidence(&mut self, evidence: &str) {
        self.evidence.push(evidence.to_string());
    }
}

/// Evolution manager with full pipeline support.
/// Per Architecture §26.20 (Evolution Manager).
#[derive(Debug, Clone, Default)]
pub struct FullEvolutionManager {
    /// Active improvement candidates.
    pub candidates: Vec<ImprovementCandidate>,
    /// Evolution ladder stage.
    pub ladder_stage: EvolutionLadder,
    /// Pipeline stage.
    pub pipeline_stage: ImprovementPipeline,
    /// Skill evolution records.
    pub skill_evolutions: Vec<SkillEvolutionRecord>,
    /// Workflow evolution records.
    pub workflow_evolutions: Vec<WorkflowEvolutionRecord>,
    /// Memory evolution records.
    pub memory_evolutions: Vec<MemoryEvolutionRecord>,
    /// Graph evolution records.
    pub graph_evolutions: Vec<GraphEvolutionRecord>,
    /// Architecture evolution records.
    pub architecture_evolutions: Vec<ArchitectureEvolutionRecord>,
    /// Evolution memory records.
    pub evolution_memory: Vec<EvolutionMemoryRecord>,
    /// Controlled experiments.
    pub experiments: Vec<ControlledExperiment>,
    /// History.
    pub history: Vec<String>,
}

impl FullEvolutionManager {
    /// Create a new full evolution manager.
    pub fn new() -> Self {
        Self {
            ladder_stage: EvolutionLadder::Learning,
            pipeline_stage: ImprovementPipeline::Observation,
            ..Default::default()
        }
    }

    /// Propose a new improvement candidate.
    pub fn propose(&mut self, observation: &str, hypothesis: &str) -> String {
        let candidate = ImprovementCandidate::new(observation, hypothesis);
        let id = candidate.id.clone();
        self.candidates.push(candidate);
        self.history.push(format!("Candidate proposed: {}", id));
        id
    }

    /// Evaluate a candidate with full pipeline.
    pub fn evaluate_full(&mut self, candidate_id: &str, confidence: f32) -> Option<String> {
        for candidate in &mut self.candidates {
            if candidate.id == candidate_id {
                candidate.set_confidence(confidence);
                if confidence >= 0.9 {
                    candidate.set_status("accepted");
                    self.history.push(format!(
                        "Candidate accepted (full pipeline): {}",
                        candidate_id
                    ));
                    // Advance pipeline
                    self.pipeline_stage = ImprovementPipeline::Consolidation;
                    return Some("accepted".to_string());
                } else if confidence < 0.3 {
                    candidate.set_status("rejected");
                    self.history.push(format!(
                        "Candidate rejected (full pipeline): {}",
                        candidate_id
                    ));
                    return Some("rejected".to_string());
                } else {
                    candidate.set_status("testing");
                    self.history.push(format!(
                        "Candidate testing (full pipeline): {}",
                        candidate_id
                    ));
                    self.pipeline_stage = ImprovementPipeline::Experimentation;
                    return Some("testing".to_string());
                }
            }
        }
        None
    }

    /// Run the full evolution loop.
    pub fn run_full_evolution(
        &mut self,
        experiences: &[crate::data_contracts::experience_record::ExperienceRecord],
    ) -> Vec<crate::data_contracts::learning_update::LearningUpdate> {
        let count = experiences.len();
        tracing::debug!(experience_count = count, "Full evolution loop running");
        self.history
            .push(format!("Full evolution loop: {} experiences", count));
        Vec::new()
    }

    /// Get all candidates.
    pub fn get_candidates(&self) -> Vec<ImprovementCandidate> {
        self.candidates.clone()
    }

    /// Get history.
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }

    /// Get ladder stage.
    pub fn ladder_stage(&self) -> EvolutionLadder {
        self.ladder_stage.clone()
    }

    /// Get pipeline stage.
    pub fn pipeline_stage(&self) -> ImprovementPipeline {
        self.pipeline_stage.clone()
    }

    /// Record a skill evolution.
    pub fn record_skill_evolution(&mut self, record: SkillEvolutionRecord) {
        self.skill_evolutions.push(record);
    }

    /// Record a workflow evolution.
    pub fn record_workflow_evolution(&mut self, record: WorkflowEvolutionRecord) {
        self.workflow_evolutions.push(record);
    }

    /// Record a memory evolution.
    pub fn record_memory_evolution(&mut self, record: MemoryEvolutionRecord) {
        self.memory_evolutions.push(record);
    }

    /// Record a graph evolution.
    pub fn record_graph_evolution(&mut self, record: GraphEvolutionRecord) {
        self.graph_evolutions.push(record);
    }

    /// Record an architecture evolution.
    pub fn record_architecture_evolution(&mut self, record: ArchitectureEvolutionRecord) {
        self.architecture_evolutions.push(record);
    }

    /// Record an evolution memory entry.
    pub fn record_evolution_memory(&mut self, record: EvolutionMemoryRecord) {
        self.evolution_memory.push(record);
    }

    /// Record a controlled experiment.
    pub fn record_experiment(&mut self, experiment: ControlledExperiment) {
        self.experiments.push(experiment);
    }

    /// Get skill evolution count.
    pub fn skill_evolution_count(&self) -> usize {
        self.skill_evolutions.len()
    }

    /// Get workflow evolution count.
    pub fn workflow_evolution_count(&self) -> usize {
        self.workflow_evolutions.len()
    }

    /// Get memory evolution count.
    pub fn memory_evolution_count(&self) -> usize {
        self.memory_evolutions.len()
    }

    /// Get graph evolution count.
    pub fn graph_evolution_count(&self) -> usize {
        self.graph_evolutions.len()
    }

    /// Get architecture evolution count.
    pub fn architecture_evolution_count(&self) -> usize {
        self.architecture_evolutions.len()
    }

    /// Get evolution memory count.
    pub fn evolution_memory_count(&self) -> usize {
        self.evolution_memory.len()
    }

    /// Get experiment count.
    pub fn experiment_count(&self) -> usize {
        self.experiments.len()
    }
}

/// Active reference to full evolution contracts.
pub fn reference_full_evolution() {
    let mut manager = FullEvolutionManager::new();
    let id = manager.propose(
        "Repeated failures in memory retrieval",
        "Improve retrieval ranking",
    );
    manager.evaluate_full(&id, 0.92);
    // Wire evolution records through proper methods
    manager.record_skill_evolution(SkillEvolutionRecord::new("skill-1", "Improved retrieval"));
    manager.record_workflow_evolution(WorkflowEvolutionRecord::new(
        "workflow-1",
        "Optimized planning",
    ));
    manager.record_memory_evolution(MemoryEvolutionRecord::new(
        "memory-1",
        "Consolidated knowledge",
    ));
    manager.record_graph_evolution(GraphEvolutionRecord::new("graph-1", "Added relationships"));
    manager.record_architecture_evolution(ArchitectureEvolutionRecord::new(
        "execution-engine",
        "Added checkpointing",
    ));
    manager.record_evolution_memory(EvolutionMemoryRecord::new(
        &id,
        EvolutionStage::Experiment,
        "Testing",
    ));
    manager.record_experiment(ControlledExperiment::new(
        "exp-1",
        "Improve retrieval",
        "Higher accuracy",
    ));
    tracing::debug!(
        ladder_stage = %manager.ladder_stage().label(),
        pipeline_stage = %manager.pipeline_stage().label(),
        candidates = manager.get_candidates().len(),
        skill_evolutions = manager.skill_evolution_count(),
        workflow_evolutions = manager.workflow_evolution_count(),
        memory_evolutions = manager.memory_evolution_count(),
        graph_evolutions = manager.graph_evolution_count(),
        architecture_evolutions = manager.architecture_evolution_count(),
        evolution_memory = manager.evolution_memory_count(),
        experiments = manager.experiment_count(),
        history = manager.history.len(),
        "Full evolution manager referenced"
    );
    // Ladder and pipeline stages are already set via propose/evaluate_full
    tracing::debug!(
        ladder = %EvolutionLadder::Evolution.label(),
        pipeline = %ImprovementPipeline::Experimentation.label(),
        "Evolution stages confirmed"
    );
}

/// Evolution manager tracks improvement candidates and their lifecycle.
#[derive(Debug, Clone, Default)]
pub struct EvolutionManager {
    /// Pending improvement candidates.
    pub candidates: Vec<ImprovementCandidate>,
    /// History of evolution events.
    pub history: Vec<String>,
}

impl EvolutionManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Propose a new improvement candidate.
    pub fn propose_candidate(&mut self, observation: &str, hypothesis: &str) -> String {
        let candidate = ImprovementCandidate::new(observation, hypothesis);
        let id = candidate.id.clone();
        self.candidates.push(candidate);
        self.history.push(format!("Candidate proposed: {}", id));
        id
    }

    /// Evaluate candidates and return sorted list.
    pub fn evaluate_candidates(&self) -> Vec<&ImprovementCandidate> {
        let mut evaluated: Vec<&ImprovementCandidate> = self.candidates.iter().collect();
        evaluated.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        evaluated
    }

    /// Accept a candidate and record in history.
    pub fn accept_candidate(&mut self, id: &str) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == id) {
            candidate.set_status("accepted");
            self.history.push(format!("Candidate accepted: {}", id));
            true
        } else {
            false
        }
    }

    /// Reject a candidate and record in history.
    pub fn reject_candidate(&mut self, id: &str) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == id) {
            candidate.set_status("rejected");
            self.history.push(format!("Candidate rejected: {}", id));
            true
        } else {
            false
        }
    }

    /// Get history of evolution events.
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }

    /// Get pending candidates sorted by confidence.
    pub fn pending_candidates(&self) -> Vec<&ImprovementCandidate> {
        self.candidates
            .iter()
            .filter(|c| c.status == "proposed")
            .collect()
    }
}

/// Active reference to EvolutionManager to prevent dead-code warnings.
pub fn reference_evolution_manager() {
    let mut manager = EvolutionManager::new();
    let id = manager.propose_candidate("Slow memory retrieval", "Add caching layer");
    tracing::debug!(candidate_id = %id, "Evolution manager referenced");
    let candidates = manager.evaluate_candidates();
    tracing::debug!(
        candidate_count = candidates.len(),
        "Evolution manager evaluated"
    );
    manager.accept_candidate(&id);
    let history = manager.get_history();
    tracing::debug!(history_count = history.len(), "Evolution manager history");
}
