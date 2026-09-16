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

    /// Add a layer. Each call represents a distinct pipeline stage.
    pub fn add_layer(&mut self, layer: ContextLayer) {
        self.layers.push(layer);
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

/// Stage 1: Analyze conversation input into topics/keywords.
pub fn conversation_analysis(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Stage 2: Extract planner requirements from goal description.
pub fn planner_requirements(goal: &str) -> Vec<String> {
    let mut reqs = Vec::new();
    let lower = goal.to_lowercase();
    if lower.contains("search") || lower.contains("find") || lower.contains("lookup") {
        reqs.push("knowledge".to_string());
    }
    if lower.contains("store") || lower.contains("save") {
        reqs.push("memory".to_string());
    }
    if lower.contains("plan") || lower.contains("create") {
        reqs.push("planning".to_string());
    }
    if reqs.is_empty() {
        reqs.push("knowledge".to_string());
    }
    reqs
}

/// Stage 6: Rank context items by relevance (score by length for placeholder).
pub fn context_ranking(items: &[String]) -> Vec<(String, f32)> {
    items
        .iter()
        .map(|item| {
            let score = item.len() as f32 / 10.0; // Simple placeholder scoring
            (item.clone(), score.clamp(0.0, 1.0))
        })
        .collect()
}

/// Stage 7: Deduplicate items using HashSet.
pub fn deduplicate(items: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in items {
        if seen.insert(item.to_lowercase()) {
            result.push(item.clone());
        }
    }
    result
}

/// Stage 8: Compress context to token budget.
pub fn compress_context(items: &[String], budget: usize) -> Vec<String> {
    if budget == 0 {
        return Vec::new();
    }
    items[..budget.min(items.len())].to_vec()
}

/// Stage 3-5: Placeholder retrieval functions.
pub fn memory_retrieval(_query: &str, _budget: usize) -> Vec<String> {
    Vec::new()
}

pub fn experience_retrieval(_query: &str, _budget: usize) -> Vec<String> {
    Vec::new()
}

pub fn knowledge_retrieval(_query: &str, _budget: usize) -> Vec<String> {
    Vec::new()
}
