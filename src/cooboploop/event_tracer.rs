// /src/CoObOpLoop/event_tracer.rs
// Event tracer for the CoObOpLoop system (§15 / T10.19).

/// Logs transitions: Experience -> Learning -> Capability/Knowledge Update.
#[derive(Clone, Default)]
pub struct EventTracer {
    events: Vec<String>,
}

impl EventTracer {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Log a transition event (§T10.19).
    pub fn log(&mut self, event: &str) {
        self.events.push(event.to_string());
    }

    /// Get all logged events.
    pub fn events(&self) -> &[String] {
        &self.events
    }

    /// Log the full transition: Experience -> Learning -> Capability/Knowledge Update.
    pub fn log_learning_cycle(&mut self, experience_title: &str) {
        self.log(&format!("Experience -> Learning: {}", experience_title));
        self.log("Learning -> Capability/Knowledge Update");
    }
}
