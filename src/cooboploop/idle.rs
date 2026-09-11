// /src/CoObOpLoop/idle.rs
// Idle state management for the CoObOpLoop system.
// Will be populated incrementally per §9-10.

/// Phase of idle state.
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IdlePhase {
    /// Not idle - actively processing.
    Active,

    /// Waiting for new objectives.
    Waiting,

    /// Re-evaluating priorities.
    Reprioritizing,

    /// Performing maintenance.
    Maintenance,
}

impl std::fmt::Debug for IdlePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => f.write_str("Active"),
            Self::Waiting => f.write_str("Waiting"),
            Self::Reprioritizing => f.write_str("Reprioritizing"),
            Self::Maintenance => f.write_str("Maintenance"),
        }
    }
}

/// Event recorded when the system chooses deliberate inactivity (§7.6).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeliberateInactivity {
    /// Timestamp of the event.
    pub timestamp: std::time::SystemTime,
    /// Phase at time of inactivity.
    pub phase: IdlePhase,
    /// Reason for waiting.
    pub reason: String,
}

impl DeliberateInactivity {
    /// Create a new deliberate inactivity event.
    pub fn new(phase: IdlePhase, reason: String) -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            phase,
            reason,
        }
    }
}

/// Activity categories for idle work (§7.3).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivityCategory {
    PendingObjectives,
    SystemMaintenance,
    BugInvestigation,
    Research,
    KnowledgeConsolidation,
    MemoryMaintenance,
    HardwareEvaluation,
    PerformanceOptimization,
    CapabilityDevelopment,
    SelfImprovement,
    EnvironmentalObservation,
    LongTermPlanning,
}

impl std::fmt::Debug for ActivityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingObjectives => f.write_str("PendingObjectives"),
            Self::SystemMaintenance => f.write_str("SystemMaintenance"),
            Self::BugInvestigation => f.write_str("BugInvestigation"),
            Self::Research => f.write_str("Research"),
            Self::KnowledgeConsolidation => f.write_str("KnowledgeConsolidation"),
            Self::MemoryMaintenance => f.write_str("MemoryMaintenance"),
            Self::HardwareEvaluation => f.write_str("HardwareEvaluation"),
            Self::PerformanceOptimization => f.write_str("PerformanceOptimization"),
            Self::CapabilityDevelopment => f.write_str("CapabilityDevelopment"),
            Self::SelfImprovement => f.write_str("SelfImprovement"),
            Self::EnvironmentalObservation => f.write_str("EnvironmentalObservation"),
            Self::LongTermPlanning => f.write_str("LongTermPlanning"),
        }
    }
}

/// Current idle state of the system.
#[derive(Clone)]
pub struct IdleState {
    /// Current phase.
    pub phase: IdlePhase,

    /// Seconds since last activity.
    pub seconds_since_activity: u64,

    /// Number of objectives processed in current session.
    pub objectives_processed: u32,

    /// Reevaluation interval in seconds.
    pub reevaluation_interval_secs: u64,

    /// Whether the queue is empty (§T7.2).
    pub queue_empty: bool,

    /// Number of problems detected (§T7.2).
    pub problems_count: u32,

    /// Number of knowledge gaps (§T7.2).
    pub knowledge_gaps_count: u32,
}

impl IdleState {
    /// Create a new idle state.
    pub fn new(reevaluation_interval_secs: u64) -> Self {
        Self {
            phase: IdlePhase::Active,
            seconds_since_activity: 0,
            objectives_processed: 0,
            reevaluation_interval_secs,
            queue_empty: true,
            problems_count: 0,
            knowledge_gaps_count: 0,
        }
    }

    /// Determine if the system should wait (be idle).
    pub fn should_wait(&self) -> bool {
        matches!(self.phase, IdlePhase::Waiting | IdlePhase::Maintenance)
    }

    /// Set the reevaluation interval.
    pub fn set_interval(&mut self, seconds: u64) {
        self.reevaluation_interval_secs = seconds;
    }

    /// Get the reevaluation interval.
    pub fn get_interval(&self) -> u64 {
        self.reevaluation_interval_secs
    }

    /// Update interval and return the new value (wired usage).
    pub fn update_interval(&mut self, seconds: u64) -> u64 {
        self.set_interval(seconds);
        self.get_interval()
    }

    /// Evaluate categories with useful work available (§7.4).
    pub fn evaluate_useful_work(
        &mut self,
        strategic_objectives: &[crate::cooboploop::strategic::StrategicObjectiveRecord],
    ) -> (Vec<ActivityCategory>, Option<DeliberateInactivity>) {
        let interval = self.get_interval();
        let updated = self.update_interval(60);
        let interval_display = interval.to_string();
        let updated_display = updated.to_string();
        tracing::debug!("Idle interval: {interval_display} -> {updated_display}");
        let mut work = Vec::new();
        let mut inactivity = None;
        if self.phase != IdlePhase::Active {
            work.push(ActivityCategory::SystemMaintenance);
            work.push(ActivityCategory::MemoryMaintenance);
            work.push(ActivityCategory::PerformanceOptimization);
            work.push(ActivityCategory::EnvironmentalObservation);
            // Wire DeliberateInactivity when not active
            inactivity = Some(DeliberateInactivity::new(
                self.phase.clone(),
                "Idle phase detected".to_string(),
            ));
        }
        if strategic_objectives
            .iter()
            .any(|objective| objective.status == "active")
        {
            work.push(ActivityCategory::PendingObjectives);
            work.push(ActivityCategory::LongTermPlanning);
        }
        (work, inactivity)
    }
}

impl std::fmt::Debug for IdleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdleState")
            .field("phase", &self.phase)
            .field("seconds_since_activity", &self.seconds_since_activity)
            .field("objectives_processed", &self.objectives_processed)
            .field(
                "reevaluation_interval_secs",
                &self.reevaluation_interval_secs,
            )
            .field("queue_empty", &self.queue_empty)
            .field("problems_count", &self.problems_count)
            .field("knowledge_gaps_count", &self.knowledge_gaps_count)
            .finish()
    }
}
