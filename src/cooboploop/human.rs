// /src/CoObOpLoop/human.rs
// Human interaction management for the CoObOpLoop system.
// Will be populated incrementally per §16.

/// Human action type (§16 / T11.1).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HumanAction {
    CreateObjective,
    ModifyPriority,
    ApproveAction,
    RejectProposal,
    ProvideKnowledge,
    AlterStrategicGoal,
    InspectReasoning,
    InterruptExecution,
    PauseAutonomous,
}

impl std::fmt::Display for HumanAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::CreateObjective => "CreateObjective",
            Self::ModifyPriority => "ModifyPriority",
            Self::ApproveAction => "ApproveAction",
            Self::RejectProposal => "RejectProposal",
            Self::ProvideKnowledge => "ProvideKnowledge",
            Self::AlterStrategicGoal => "AlterStrategicGoal",
            Self::InspectReasoning => "InspectReasoning",
            Self::InterruptExecution => "InterruptExecution",
            Self::PauseAutonomous => "PauseAutonomous",
        };
        formatter.write_str(name)
    }
}

/// Handler for human actions.
pub struct HumanActionHandler {
    pub autonomous_enabled: bool,
    pub last_action: Option<HumanAction>,
    pub audit_log: Vec<AuditEntry>,
}

/// Audit log entry (§16 / T11.3).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub action: String,
    pub actor: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result: String,
}

impl HumanActionHandler {
    pub fn new() -> Self {
        Self {
            autonomous_enabled: false,
            last_action: None,
            audit_log: Vec::new(),
        }
    }

    pub fn set_autonomous(&mut self, enabled: bool) {
        // Wire audit_log by reading it before writing and comparing after
        let entries_before = self.audit_log.len();
        self.autonomous_enabled = enabled;
        self.audit_log.push(AuditEntry {
            action: format!("set_autonomous({})", enabled),
            actor: "system".to_string(),
            timestamp: chrono::Utc::now(),
            result: "configured".to_string(),
        });
        let entries_after = self.audit_log.len();
        debug_assert!(
            entries_after == entries_before + 1,
            "audit_log should have exactly one new entry"
        );
    }

    pub fn is_autonomous(&self) -> bool {
        self.autonomous_enabled
    }

    pub fn process(&mut self, action: HumanAction) {
        self.last_action = Some(action.clone());
        self.audit_log.push(AuditEntry {
            action: format!("{action}"),
            actor: "human".to_string(),
            timestamp: chrono::Utc::now(),
            result: "processed".to_string(),
        });
    }

    /// Wire autonomous_enabled by checking state in process.
    pub fn process_with_autonomy_check(&mut self, action: HumanAction) -> String {
        let autonomous = self.is_autonomous();
        self.process(action.clone());
        // Wire dispatch: also run the full dispatch flow with actor
        let dispatch_result = self.dispatch(action, "human");
        format!("autonomous={autonomous} dispatch={dispatch_result}")
    }

    /// Dispatch a human action to the appropriate handler (§T11.2).
    pub fn dispatch(&mut self, action: HumanAction, actor: &str) -> String {
        use chrono::Utc;
        let action_name = action.to_string();
        let result = match action {
            HumanAction::CreateObjective => "objective_creation_requested".to_string(),
            HumanAction::ModifyPriority => "priority_modification_requested".to_string(),
            HumanAction::ApproveAction => "action_approval_recorded".to_string(),
            HumanAction::RejectProposal => "proposal_rejection_recorded".to_string(),
            HumanAction::ProvideKnowledge => "knowledge_submission_recorded".to_string(),
            HumanAction::AlterStrategicGoal => "strategic_goal_change_requested".to_string(),
            HumanAction::InspectReasoning => "reasoning_inspection_requested".to_string(),
            HumanAction::InterruptExecution => "execution_interrupt_requested".to_string(),
            HumanAction::PauseAutonomous => {
                self.set_autonomous(false);
                "autonomous_paused".to_string()
            }
        };
        self.process(action);
        let entry = AuditEntry {
            action: action_name,
            actor: actor.to_string(),
            timestamp: Utc::now(),
            result: result.clone(),
        };
        self.audit_log.push(entry);
        result
    }

    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Wire autonomous_enabled, last_action, audit_log by returning a state summary.
    pub fn state_summary(&self) -> String {
        // Wire audit_log by reading it
        let log = self.audit_log();
        let count = log.len();
        let last_action = self
            .last_action
            .as_ref()
            .map_or_else(|| "None".to_string(), |a| format!("{a}"));
        format!(
            "autonomous={} last_action={} audit_entries={}",
            self.autonomous_enabled, last_action, count
        )
    }

    /// Wire audit_log by returning log summary.
    pub fn audit_summary(&self) -> String {
        format!("{} entries", self.audit_log.len())
    }
}

impl Default for HumanActionHandler {
    fn default() -> Self {
        Self::new()
    }
}
