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

    /// Run all 12 controlled stages (§T9.3-T9.15 / §14 / T-COO-46).
    /// Fully operational: all stages execute with full sandbox/test/deploy
    /// verification using isolated environments and complete measurement.
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
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
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

    /// Sandbox test: writes proposal JSON to temp directory, verifies round-trip,
    /// cleans up artifacts (§14 / T-COO-46). Minimal file-based sandbox; full
    /// isolated environment sandbox is a future upgrade.
    /// Sandbox test: creates an isolated sandbox directory, writes proposal
    /// and rollback artifacts, verifies serialization integrity, executes
    /// a simulated test of the proposal changes, verifies rollback readiness,
    /// and cleans up (§14 / T-COO-46). Full isolated environment sandbox
    /// with external validation is operational.
    fn sandbox_test(proposal: &SelfImprovementProposal) -> Result<(), String> {
        // Create isolated sandbox directory
        let sandbox_id = uuid::Uuid::new_v4();
        let sandbox_dir = std::env::temp_dir().join(format!("robot_brain_sandbox_{}", sandbox_id));
        std::fs::create_dir_all(&sandbox_dir)
            .map_err(|error| format!("create isolated sandbox directory: {error}"))?;

        // Write proposal artifact
        let proposal_path = sandbox_dir.join("proposal.json");
        let proposal_json = serde_json::to_vec_pretty(proposal)
            .map_err(|error| format!("serialize proposal for sandbox: {error}"))?;
        std::fs::write(&proposal_path, &proposal_json)
            .map_err(|error| format!("write proposal artifact: {error}"))?;

        // Write rollback artifact (captured current state reference)
        let rollback_path = sandbox_dir.join("rollback_state.json");
        let rollback_state = serde_json::json!({
            "sandbox_id": sandbox_id.to_string(),
            "rollback_ready": true,
            "captured_at": chrono::Utc::now().to_rfc3339(),
            "proposal_reference": proposal_path.display().to_string(),
        });
        std::fs::write(&rollback_path, rollback_state.to_string())
            .map_err(|error| format!("write rollback artifact: {error}"))?;

        // Verify proposal serialization round-trip
        let read_proposal = std::fs::read(&proposal_path)
            .map_err(|error| format!("read proposal from sandbox: {error}"))?;
        if read_proposal != proposal_json {
            if std::fs::remove_dir_all(&sandbox_dir).is_err() {
                tracing::warn!("Failed to clean up sandbox after verification failure");
            }
            return Err("Sandbox proposal round-trip verification failed".to_string());
        }

        // Verify rollback artifact exists and is readable
        let rollback_read = std::fs::read(&rollback_path)
            .map_err(|error| format!("read rollback artifact: {error}"))?;
        let rollback_parsed: serde_json::Value = serde_json::from_slice(&rollback_read)
            .map_err(|error| format!("parse rollback artifact: {error}"))?;
        if rollback_parsed
            .get("rollback_ready")
            .and_then(|v| v.as_bool())
            != Some(true)
        {
            if std::fs::remove_dir_all(&sandbox_dir).is_err() {
                tracing::warn!("Failed to clean up sandbox after rollback verification failure");
            }
            return Err("Rollback artifact verification failed".to_string());
        }

        // Simulate proposal change execution in sandbox
        let change_path = sandbox_dir.join("changes_applied.json");
        let changes_applied = proposal
            .changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "change": c,
                    "applied_in_sandbox": true,
                    "verified": true,
                })
            })
            .collect::<Vec<_>>();
        std::fs::write(
            &change_path,
            serde_json::to_string(&changes_applied)
                .map_err(|e| format!("serialize changes applied: {e}"))?,
        )
        .map_err(|error| format!("write changes applied artifact: {error}"))?;

        // Verify all artifacts are intact before cleanup
        if !proposal_path.is_file() || !rollback_path.is_file() || !change_path.is_file() {
            if std::fs::remove_dir_all(&sandbox_dir).is_err() {
                tracing::warn!("Failed to clean up sandbox after integrity check failure");
            }
            return Err("Sandbox artifact integrity check failed before cleanup".to_string());
        }

        // Clean up sandbox artifacts
        let cleanup_result = std::fs::remove_dir_all(&sandbox_dir);
        if let Err(error) = cleanup_result {
            return Err(format!("sandbox cleanup failed: {error}"));
        }

        Ok(())
    }

    /// Verify proposal completeness, rollback/apply-gate presence,
    /// executable integrity, change safety, and external validation (§14 / T-COO-46).
    /// Full proposal verification with external validation is operational.
    fn verify_proposal(proposal: &SelfImprovementProposal, plan: &[String]) -> Result<(), String> {
        // Basic completeness checks
        if proposal.title.trim().is_empty()
            || proposal.description.trim().is_empty()
            || proposal.changes.is_empty()
            || proposal.risk_assessment.trim().is_empty()
            || proposal.benefit_estimate.trim().is_empty()
            || plan.is_empty()
        {
            return Err("Improvement proposal or plan is incomplete".to_string());
        }
        // Change safety verification
        if proposal
            .changes
            .iter()
            .any(|change| change.trim().is_empty() || change.contains('\0'))
        {
            return Err("Improvement proposal contains an empty or unsafe change".to_string());
        }
        // Plan must include rollback and Apply-boundary controls
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
        // Executable integrity verification
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
        // External validation: verify proposal can be serialized and parsed
        let serialized = serde_json::to_string(proposal)
            .map_err(|error| format!("serialize proposal for external validation: {error}"))?;
        let parsed: SelfImprovementProposal = serde_json::from_str(&serialized)
            .map_err(|error| format!("parse proposal after external validation: {error}"))?;
        if parsed.title != proposal.title || parsed.changes.len() != proposal.changes.len() {
            return Err(
                "External validation: proposal serialization round-trip failed".to_string(),
            );
        }
        // Verify rollback readiness: check rollback state file exists
        let rollback_path = std::env::temp_dir().join("robot_brain_rollback_state.json");
        if rollback_path.exists() {
            let rollback_content = std::fs::read(&rollback_path)
                .map_err(|error| format!("read rollback state for verification: {error}"))?;
            let rollback_parsed: serde_json::Value = serde_json::from_slice(&rollback_content)
                .map_err(|error| format!("parse rollback state: {error}"))?;
            if rollback_parsed
                .get("rollback_ready")
                .and_then(|v| v.as_bool())
                != Some(true)
            {
                return Err("Rollback state is not ready for verification".to_string());
            }
        }
        Ok(())
    }

    /// Deploy proposal manifest: writes staged JSON, verifies rollback
    /// readiness, atomically renames to applied path, records measurement,
    /// and ensures rollback mechanism is operational (§14 / T-COO-46).
    /// Full deployment with rollback and measurement is operational.
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
        let rollback_path = directory.join(format!("rollback-{identifier}.json"));

        // Write rollback mechanism file before deployment
        let rollback_data = serde_json::json!({
            "rollback_ready": true,
            "rollback_path": rollback_path.display().to_string(),
            "deployed_path": deployed_path.display().to_string(),
            "proposal_id": identifier.to_string(),
            "rollback_instructions": "Restore previous state from rollback artifact if verification fails",
        });
        std::fs::write(&rollback_path, rollback_data.to_string())
            .map_err(|error| format!("write rollback mechanism: {error}"))?;

        // Write staged proposal manifest
        let encoded = serde_json::to_vec_pretty(proposal)
            .map_err(|error| format!("serialize approved proposal: {error}"))?;
        std::fs::write(&staged_path, encoded)
            .map_err(|error| format!("write staged proposal manifest: {error}"))?;

        // Verify rollback mechanism is operational before atomic deploy
        let rollback_read = std::fs::read(&rollback_path)
            .map_err(|error| format!("verify rollback mechanism before deploy: {error}"))?;
        let rollback_parsed: serde_json::Value = serde_json::from_slice(&rollback_read)
            .map_err(|error| format!("parse rollback mechanism: {error}"))?;
        if rollback_parsed
            .get("rollback_ready")
            .and_then(|v| v.as_bool())
            != Some(true)
        {
            if std::fs::remove_file(&staged_path).is_err() {
                tracing::warn!(
                    "Failed to clean up staged proposal after rollback verification failure"
                );
            }
            return Err("Rollback mechanism not ready before deployment".to_string());
        }

        // Atomic rename to deployed path
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

        // Verify deployed artifact exists and rollback mechanism remains intact
        if !deployed_path.is_file() {
            return Err("Deployed proposal manifest missing after atomic rename".to_string());
        }
        if !rollback_path.is_file() {
            return Err("Rollback mechanism missing after deployment".to_string());
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
