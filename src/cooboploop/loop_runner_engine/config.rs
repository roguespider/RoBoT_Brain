//! Configuration for the LoopRunner.

/// Configuration parameters for the LoopRunner.
pub struct LoopRunnerConfig {
    pub heartbeat_secs: u64,
    pub reevaluation_interval_secs: u64,
    pub max_cycles: Option<u32>,
    pub autonomous_operation_enabled: bool,
    pub phase: String,
    pub task_completion_count: u32,
    pub cognitive_completion_count: u32,
    pub mission: String,
}

impl Default for LoopRunnerConfig {
    fn default() -> Self {
        Self {
            heartbeat_secs: 1,
            reevaluation_interval_secs: 60,
            max_cycles: None,
            autonomous_operation_enabled: false,
            phase: String::new(),
            task_completion_count: 0,
            cognitive_completion_count: 0,
            mission: String::new(),
        }
    }
}

impl LoopRunnerConfig {
    /// Apply configuration to a LoopRunner (§23 / architecture compliance).
    pub fn apply_to(&self, runner: &mut crate::cooboploop::loop_runner_engine::loop_runner::LoopRunner) {
        runner.set_autonomous(self.autonomous_operation_enabled);
        runner.set_mission(self.mission.clone());
        runner.heartbeat_secs = self.heartbeat_secs;
        runner.reevaluation_interval_secs = self.reevaluation_interval_secs;
        if let Some(max) = self.max_cycles {
            runner.set_max_cycles(max);
        }
        tracing::debug!(
            "Config applied: phase={}, task_completion={}, cognitive_completion={}",
            self.phase,
            self.task_completion_count,
            self.cognitive_completion_count
        );
    }
}
