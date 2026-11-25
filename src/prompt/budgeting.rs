//! Prompt token budgeting per Architecture Chapter 17.
//! Tracks and manages token budgets across prompt layers.

/// Token budget allocation for a single layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerBudget {
    /// Layer name.
    pub name: String,
    /// Allocated tokens for this layer.
    pub allocated: usize,
    /// Tokens actually used.
    pub used: usize,
}

impl LayerBudget {
    /// Create a new layer budget.
    pub fn new(name: &str, allocated: usize) -> Self {
        Self {
            name: name.to_string(),
            allocated,
            used: 0,
        }
    }

    /// Record tokens used by this layer.
    pub fn record_usage(&mut self, used: usize) {
        self.used = used.min(self.allocated);
    }

    /// Get remaining tokens for this layer.
    pub fn remaining(&self) -> usize {
        self.allocated.saturating_sub(self.used)
    }

    /// Check if the layer is over budget.
    pub fn over_budget(&self) -> bool {
        self.used > self.allocated
    }
}

/// Manages token budgets across multiple prompt layers.
pub struct BudgetManager {
    /// Total token budget.
    total_budget: usize,
    /// Per-layer budgets.
    layers: Vec<LayerBudget>,
}

impl BudgetManager {
    /// Create a new budget manager with the given total budget.
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            layers: Vec::new(),
        }
    }

    /// Add a layer budget.
    pub fn add_layer(&mut self, name: &str, allocated: usize) {
        self.layers.push(LayerBudget::new(name, allocated));
    }

    /// Record token usage for a layer.
    pub fn record_usage(&mut self, layer_name: &str, used: usize) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == layer_name) {
            layer.record_usage(used);
        }
    }

    /// Get the total tokens used across all layers.
    pub fn total_used(&self) -> usize {
        self.layers.iter().map(|l| l.used).sum()
    }

    /// Get the remaining tokens in the total budget.
    pub fn remaining(&self) -> usize {
        self.total_budget.saturating_sub(self.total_used())
    }

    /// Check if any layer is over budget.
    pub fn any_over_budget(&self) -> bool {
        self.layers.iter().any(|l| l.over_budget())
    }

    /// Get the total budget.
    pub fn total_budget(&self) -> usize {
        self.total_budget
    }
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new(4000)
    }
}
