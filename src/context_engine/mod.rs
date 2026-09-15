//! Context Engine — Temporary cognitive environment assembly (Architecture Chapter 7).
//!
//! Context is derived, disposable, and assembled on demand.
//! It does not own persistent memory or knowledge.
use std::collections::HashMap;

/// The 9 stages of the context assembly pipeline (Chapter 7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssemblyStage {
    /// 1. Conversation Analysis.
    ConversationAnalysis,
    /// 2. Planner Requirements.
    PlannerRequirements,
    /// 3. Memory Retrieval.
    MemoryRetrieval,
    /// 4. Experience Retrieval.
    ExperienceRetrieval,
    /// 5. Knowledge Retrieval.
    KnowledgeRetrieval,
    /// 6. Context Ranking.
    ContextRanking,
    /// 7. Deduplication.
    Deduplication,
    /// 8. Compression.
    Compression,
    /// 9. Token Budget Allocation.
    TokenBudgetAllocation,
}

/// A context layer representing temporary cognitive state.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextLayer {
    /// Working memory layer.
    Working,
    /// Short-term context layer.
    ShortTerm,
    /// Session context layer.
    Session,
    /// Project context layer.
    Project,
    /// Historical context layer.
    Historical,
}

/// Context assembly tracking references and state.
#[derive(Debug, Clone)]
pub struct ContextAssembly {
    /// Active context references.
    pub references: Vec<String>,
    /// Active goals.
    pub goals: Vec<String>,
    /// Context layers included.
    pub layers: Vec<ContextLayer>,
    /// Constraints applied.
    pub constraints: Vec<String>,
    /// Timestamp of assembly.
    pub assembled_at: i64,
}

impl ContextAssembly {
    /// Create a new context assembly.
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            goals: Vec::new(),
            layers: Vec::new(),
            constraints: Vec::new(),
            assembled_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Add a reference.
    pub fn add_reference(&mut self, reference: &str) {
        self.references.push(reference.to_string());
    }

    /// Add a goal.
    pub fn add_goal(&mut self, goal: &str) {
        if !self.goals.contains(&goal.to_string()) {
            self.goals.push(goal.to_string());
        }
    }

    /// Add a layer.
    pub fn add_layer(&mut self, layer: ContextLayer) {
        if !self.layers.contains(&layer) {
            self.layers.push(layer);
        }
    }

    /// Add a constraint.
    pub fn add_constraint(&mut self, constraint: &str) {
        self.constraints.push(constraint.to_string());
    }
}

impl Default for ContextAssembly {
    fn default() -> Self {
        Self::new()
    }
}

/// The Context Engine assembles temporary cognitive environments.
///
/// Per Architecture Chapter 7: context is requested, not accumulated.
/// It is disposable and does not own persistent state.
#[derive(Debug, Clone, Default)]
pub struct ContextEngine {
    /// Active assemblies by correlation ID.
    assemblies: HashMap<String, ContextAssembly>,
}

impl ContextEngine {
    /// Create a new context engine.
    pub fn new() -> Self {
        Self {
            assemblies: HashMap::new(),
        }
    }

    /// Assemble context for a correlation.
    /// Returns a clone of the ContextAssembly.
    pub fn assemble_context(&mut self, correlation_id: &str) -> ContextAssembly {
        if !self.assemblies.contains_key(correlation_id) {
            self.assemblies
                .insert(correlation_id.to_string(), ContextAssembly::new());
        }
        self.assemblies
            .get(correlation_id)
            .cloned()
            .unwrap_or_else(ContextAssembly::new)
    }

    /// Get a mutable assembly.
    pub fn get_assembly_mut(&mut self, correlation_id: &str) -> Option<&mut ContextAssembly> {
        self.assemblies.get_mut(correlation_id)
    }

    /// Dispose of a context assembly.
    pub fn dispose_context(&mut self, correlation_id: &str) {
        self.assemblies.remove(correlation_id);
    }
}

/// Run the context assembly pipeline through stages.
pub fn run_assembly(stages: &[AssemblyStage], correlation_id: &str) -> ContextAssembly {
    let mut assembly = ContextAssembly::new();
    assembly.add_reference(correlation_id);
    for _stage in stages {
        assembly.add_layer(ContextLayer::Working);
    }
    assembly
}
