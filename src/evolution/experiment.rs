//! Experiment — Per Architecture Chapter 26.8 (Controlled Experimentation).
//!
//! Experiments test hypotheses before adoption. Each experiment records
//! its hypothesis reference, description, results, and success/failure.

/// An experiment testing a hypothesis.
#[derive(Debug, Clone)]
pub struct Experiment {
    /// Unique experiment identifier.
    pub id: String,
    /// Hypothesis being tested.
    pub hypothesis_id: String,
    /// Experiment description.
    pub description: String,
    /// Results of the experiment.
    pub results: Vec<String>,
    /// Whether the experiment succeeded.
    pub success: bool,
}

impl Experiment {
    /// Create a new experiment.
    pub fn new(id: String, hypothesis_id: String, description: String) -> Self {
        Self {
            id,
            hypothesis_id,
            description,
            results: Vec::new(),
            success: false,
        }
    }

    /// Add a result to the experiment.
    pub fn add_result(&mut self, result: String) {
        self.results.push(result);
    }

    /// Mark the experiment as succeeded or failed.
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}
