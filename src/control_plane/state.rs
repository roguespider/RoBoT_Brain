//! Control plane state per Architecture Chapter 28.
//! Manages the observable state of the RoBoT system for developer inspection.

/// Current system state visible through the control plane.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemState {
    /// Whether the system is running.
    pub running: bool,
    /// Current cycle count.
    pub cycle_count: u64,
    /// Active objectives count.
    pub active_objectives: usize,
    /// Memory usage estimate in MB.
    pub memory_mb: usize,
    /// Uptime in seconds.
    pub uptime_secs: u64,
}

impl SystemState {
    /// Create a new default system state.
    pub fn new() -> Self {
        Self {
            running: false,
            cycle_count: 0,
            active_objectives: 0,
            memory_mb: 0,
            uptime_secs: 0,
        }
    }

    /// Update the running flag.
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Increment the cycle count.
    pub fn increment_cycle(&mut self) {
        self.cycle_count = self.cycle_count.saturating_add(1);
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutable control plane state shared across handlers.
#[derive(Debug, Clone)]
pub struct ControlPlaneState {
    /// Current system state.
    pub system: SystemState,
}

impl ControlPlaneState {
    /// Create a new control plane state.
    pub fn new() -> Self {
        Self {
            system: SystemState::new(),
        }
    }
}

impl Default for ControlPlaneState {
    fn default() -> Self {
        Self::new()
    }
}
