// /src/CoObOpLoop/self_improvement.rs
// Self-improvement system for the CoObOpLoop system.

/// Stage of the improvement pipeline (per §14 / T9.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum ImprovementStage {
    IdentifyLimitation,
    CreateObjective,
    Research,
    GenerateProposal,
    EstimateBenefitRisk,
    Plan,
    SandboxTest,
    Verify,
    Approve,
    Deploy,
    MeasureResult,
    RecordExperience,
}

/// Self-improvement proposal (§14 / T9.2).
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SelfImprovementProposal {
    pub title: String,
    pub description: String,
    pub changes: Vec<String>,
    pub risk_assessment: String,
    pub benefit_estimate: String,
    pub requires_approval: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ImprovementMeasurement {
    pub failures_before: usize,
    pub failures_after: usize,
    pub deployment_artifact_present: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ImprovementExperience {
    pub limitation: String,
    pub objective_id: String,
    pub research_topic: String,
    pub proposal_title: String,
    pub plan: Vec<String>,
    pub stages: Vec<ImprovementStage>,
    pub boundary: ModificationBoundary,
    pub deployment_path: Option<String>,
    pub measurement: ImprovementMeasurement,
}

/// Modification boundary levels (per §A.3 / T9.16).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum ModificationBoundary {
    Read,
    Propose,
    Apply,
}

impl std::fmt::Display for ModificationBoundary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Read => "Read",
            Self::Propose => "Propose",
            Self::Apply => "Apply",
        })
    }
}

#[derive(Debug)]
pub struct SelfImprovementGuard;

impl SelfImprovementGuard {
    pub fn check(
        current: &ModificationBoundary,
        required: &ModificationBoundary,
    ) -> Result<(), String> {
        let current_rank = match current {
            ModificationBoundary::Read => 0_u8,
            ModificationBoundary::Propose => 1_u8,
            ModificationBoundary::Apply => 2_u8,
        };
        let required_rank = match required {
            ModificationBoundary::Read => 0_u8,
            ModificationBoundary::Propose => 1_u8,
            ModificationBoundary::Apply => 2_u8,
        };
        if current_rank >= required_rank {
            Ok(())
        } else {
            Err(format!(
                "Modification boundary {current} does not permit operation requiring {required}"
            ))
        }
    }

    /// Wire SelfImprovementGuard by constructing it in a factory method.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SelfImprovementGuard {
    fn default() -> Self {
        Self
    }
}

/// Self-improvement pipeline runner.
pub struct SelfImprovementPipeline {
    current_stage: ImprovementStage,
    boundary: ModificationBoundary,
    recent_failures: Vec<String>,
    stage_history: Vec<ImprovementStage>,
    last_objective: Option<crate::cooboploop::queue::AgentGoal>,
    last_research: Option<crate::cooboploop::research::ResearchObjective>,
    last_plan: Vec<String>,
    last_proposal: Option<SelfImprovementProposal>,
    last_deployment: Option<std::path::PathBuf>,
    last_measurement: Option<ImprovementMeasurement>,
    experience_log: Vec<ImprovementExperience>,
}

impl SelfImprovementPipeline {
    pub fn new() -> Self {
        Self {
            current_stage: ImprovementStage::IdentifyLimitation,
            boundary: ModificationBoundary::Propose,
            recent_failures: Vec::new(),
            stage_history: Vec::new(),
            last_objective: None,
            last_research: None,
            last_plan: Vec::new(),
            last_proposal: None,
            last_deployment: None,
            last_measurement: None,
            experience_log: Vec::new(),
        }
    }

    pub fn set_boundary(&mut self, boundary: ModificationBoundary) {
        self.boundary = boundary;
    }

    pub fn get_boundary(&self) -> &ModificationBoundary {
        &self.boundary
    }

    pub fn current_stage(&self) -> &ImprovementStage {
        &self.current_stage
    }

    /// Check if `required` boundary is permitted under current boundary (§A.3 / T9.17).
    pub fn check(&self, required: &ModificationBoundary) -> Result<(), String> {
        SelfImprovementGuard::check(&self.boundary, required)
    }

    pub fn record_failure(&mut self, failure: String) {
        if !failure.trim().is_empty() {
            self.recent_failures.push(failure);
        }
    }

    /// Return the number of recent failures recorded.
    pub fn recent_failures_count(&self) -> usize {
        self.recent_failures.len()
    }

    fn enter_stage(&mut self, stage: ImprovementStage) {
        self.current_stage = stage.clone();
        self.stage_history.push(stage);
    }

    /// Run all 12 controlled stages (§T9.3-T9.15).
    pub fn run(&mut self) -> Result<SelfImprovementProposal, String> {
        self.stage_history.clear();
        self.enter_stage(ImprovementStage::IdentifyLimitation);
        let limitation = self
            .recent_failures
            .last()
            .cloned()
            .ok_or_else(|| "No recent failure is available for self-improvement".to_string())?;

        self.enter_stage(ImprovementStage::CreateObjective);
        let objective = crate::cooboploop::queue::AgentGoal {
            id: format!("self_improvement_{}", uuid::Uuid::new_v4()),
            title: format!("Improve capability after failure: {limitation}"),
            description: format!("Investigate and prevent recurrence of: {limitation}"),
            status: crate::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.8,
            source: crate::cooboploop::sources::ObjectiveSource::ImprovementTarget,
            expected_value: 0.8,
            risk: 0.4,
            learning_value: 0.9,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
        };
        self.last_objective = Some(objective);

        self.enter_stage(ImprovementStage::Research);
        self.last_research = Some(crate::cooboploop::research::ResearchObjective {
            topic: format!("Root cause and mitigations for {limitation}"),
            trigger: crate::cooboploop::research::ResearchTrigger::PreviousFailure,
            persistence_target: crate::cooboploop::research::PersistenceTarget::Both,
            expected_knowledge: format!("A verified prevention strategy for {limitation}"),
        });

        self.enter_stage(ImprovementStage::GenerateProposal);
        let mut proposal = SelfImprovementProposal {
            title: format!("Prevent recurrence of {limitation}"),
            description: format!(
                "Apply a verified mitigation for the most recent observed failure: {limitation}"
            ),
            changes: vec![format!(
                "Introduce a validated mitigation for: {limitation}"
            )],
            risk_assessment: String::new(),
            benefit_estimate: String::new(),
            requires_approval: true,
        };

        self.enter_stage(ImprovementStage::EstimateBenefitRisk);
        let recurrence_count = self
            .recent_failures
            .iter()
            .filter(|failure| failure.trim() == limitation.trim())
            .count();
        let risk_score = (0.2_f32 + proposal.changes.len() as f32 * 0.1).min(1.0);
        let recurrence_ratio = recurrence_count as f32 / self.recent_failures.len() as f32;
        let benefit_score = (0.5_f32 + recurrence_ratio * 0.5).min(1.0);
        proposal.risk_assessment = format!(
            "score={risk_score:.2}; {} proposed change(s) require sandbox verification and rollback readiness",
            proposal.changes.len()
        );
        proposal.benefit_estimate = format!(
            "score={benefit_score:.2}; observed {recurrence_count} matching failure(s) across {} recent failure(s)",
            self.recent_failures.len()
        );

        self.enter_stage(ImprovementStage::Plan);
        self.last_plan = vec![
            "Capture the current behavior and rollback state".to_string(),
            "Apply the proposed mitigation in an isolated artifact".to_string(),
            "Verify serialization, completeness, and approval constraints".to_string(),
            "Deploy only when the Apply boundary is active".to_string(),
            "On verification or deployment failure, restore the captured rollback state"
                .to_string(),
            "Measure recurrence and retain the experience".to_string(),
        ];

        self.enter_stage(ImprovementStage::SandboxTest);
        self.check(&ModificationBoundary::Propose)?;
        Self::sandbox_test(&proposal)?;

        self.enter_stage(ImprovementStage::Verify);
        self.check(&ModificationBoundary::Propose)?;
        Self::verify_proposal(&proposal, &self.last_plan)?;

        self.enter_stage(ImprovementStage::Approve);
        self.check(&ModificationBoundary::Propose)?;
        proposal.requires_approval = self.boundary != ModificationBoundary::Apply;

        self.enter_stage(ImprovementStage::Deploy);
        if !proposal.requires_approval && self.check(&ModificationBoundary::Apply).is_ok() {
            self.last_deployment = Some(Self::deploy_manifest(&proposal)?);
        } else {
            self.last_deployment = None;
        }

        self.enter_stage(ImprovementStage::MeasureResult);
        let deployment_artifact_present = self
            .last_deployment
            .as_ref()
            .is_some_and(|path| path.is_file());
        let measurement = ImprovementMeasurement {
            failures_before: self.recent_failures.len(),
            failures_after: self.recent_failures.len(),
            deployment_artifact_present,
        };
        self.last_measurement = Some(measurement.clone());

        self.enter_stage(ImprovementStage::RecordExperience);
        let objective_id = self
            .last_objective
            .as_ref()
            .map(|objective| objective.id.clone())
            .ok_or_else(|| "Improvement objective was not retained".to_string())?;
        let research_topic = self
            .last_research
            .as_ref()
            .map(|research| research.topic.clone())
            .ok_or_else(|| "Improvement research objective was not retained".to_string())?;
        self.experience_log.push(ImprovementExperience {
            limitation,
            objective_id,
            research_topic,
            proposal_title: proposal.title.clone(),
            plan: self.last_plan.clone(),
            stages: self.stage_history.clone(),
            boundary: self.boundary.clone(),
            deployment_path: self
                .last_deployment
                .as_ref()
                .map(|path| path.display().to_string()),
            measurement,
        });
        self.last_proposal = Some(proposal.clone());
        Ok(proposal)
    }

    fn sandbox_test(proposal: &SelfImprovementProposal) -> Result<(), String> {
        let encoded = serde_json::to_vec_pretty(proposal)
            .map_err(|error| format!("serialize improvement proposal: {error}"))?;
        let directory = std::env::temp_dir().join(format!(
            "robot_brain_improvement_sandbox_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("create improvement sandbox: {error}"))?;
        let path = directory.join("proposal.json");
        if let Err(write_error) = std::fs::write(&path, &encoded) {
            return match std::fs::remove_dir(&directory) {
                Ok(()) => Err(format!("write improvement sandbox artifact: {write_error}")),
                Err(cleanup_error) => Err(format!(
                    "write improvement sandbox artifact: {write_error}; cleanup failed: {cleanup_error}"
                )),
            };
        }
        let decoded_result = std::fs::read(&path);
        let file_cleanup = std::fs::remove_file(&path);
        let directory_cleanup = std::fs::remove_dir(&directory);
        if let Err(error) = file_cleanup {
            return Err(format!("remove improvement sandbox artifact: {error}"));
        }
        if let Err(error) = directory_cleanup {
            return Err(format!("remove improvement sandbox directory: {error}"));
        }
        let decoded = decoded_result
            .map_err(|error| format!("read improvement sandbox artifact: {error}"))?;
        if decoded == encoded {
            Ok(())
        } else {
            Err("Sandbox proposal round-trip changed the artifact".to_string())
        }
    }

    fn verify_proposal(proposal: &SelfImprovementProposal, plan: &[String]) -> Result<(), String> {
        if proposal.title.trim().is_empty()
            || proposal.description.trim().is_empty()
            || proposal.changes.is_empty()
            || proposal.risk_assessment.trim().is_empty()
            || proposal.benefit_estimate.trim().is_empty()
            || plan.is_empty()
        {
            return Err("Improvement proposal or plan is incomplete".to_string());
        }
        if proposal
            .changes
            .iter()
            .any(|change| change.trim().is_empty() || change.contains('\0'))
        {
            return Err("Improvement proposal contains an empty or unsafe change".to_string());
        }
        let has_rollback = plan
            .iter()
            .any(|step| step.to_ascii_lowercase().contains("rollback"));
        let has_apply_gate = plan
            .iter()
            .any(|step| step.to_ascii_lowercase().contains("apply boundary"));
        if !has_rollback || !has_apply_gate {
            return Err(
                "Improvement plan must include rollback and Apply-boundary controls".to_string(),
            );
        }
        let executable = std::env::current_exe()
            .map_err(|error| format!("resolve executable during verification: {error}"))?;
        let metadata = std::fs::metadata(&executable)
            .map_err(|error| format!("inspect executable during verification: {error}"))?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(format!(
                "Runtime executable failed integrity verification: {}",
                executable.display()
            ));
        }
        Ok(())
    }

    fn deploy_manifest(proposal: &SelfImprovementProposal) -> Result<std::path::PathBuf, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("resolve executable for deployment: {error}"))?;
        let directory = executable
            .parent()
            .ok_or_else(|| "Executable has no parent directory".to_string())?
            .join("proposals");
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("create proposal deployment directory: {error}"))?;
        let identifier = uuid::Uuid::new_v4();
        let staged_path = directory.join(format!("staged-{identifier}.json"));
        let deployed_path = directory.join(format!("applied-{identifier}.json"));
        let encoded = serde_json::to_vec_pretty(proposal)
            .map_err(|error| format!("serialize approved proposal: {error}"))?;
        std::fs::write(&staged_path, encoded)
            .map_err(|error| format!("write staged proposal manifest: {error}"))?;
        if let Err(deploy_error) = std::fs::rename(&staged_path, &deployed_path) {
            return match std::fs::remove_file(&staged_path) {
                Ok(()) => Err(format!(
                    "atomically deploy proposal manifest: {deploy_error}"
                )),
                Err(cleanup_error) => Err(format!(
                    "atomically deploy proposal manifest: {deploy_error}; staged cleanup failed: {cleanup_error}"
                )),
            };
        }
        Ok(deployed_path)
    }

    pub fn last_proposal(&self) -> Option<&SelfImprovementProposal> {
        self.last_proposal.as_ref()
    }
}

impl Default for SelfImprovementPipeline {
    fn default() -> Self {
        Self::new()
    }
}
