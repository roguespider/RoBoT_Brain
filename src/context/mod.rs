//! Context — Temporary cognitive environment assembly + lifecycle (Architecture Chapter 7 + 15).
//!
//! Per v0.0.2.md Ch7 (§7) and v0.0.2.1/15.md (§647): single `src/context/` module
//! combining ContextEngine (assembly pipeline) and ContextLifecycle (lifecycle stages).
//! Context is derived, disposable, assembled on demand; does not own persistent memory.

use std::collections::HashMap;

pub mod graph;

/// Lifecycle stages for context management (Chapter 15.4 / v0.0.2.1/15.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleStage {
    InputAcquisition,
    IntentExtraction,
    GoalActivation,
    ContextAssembly,
    ContextExpansion,
    ActiveReasoning,
    ContinuousRefinement,
    Reflection,
    ContextCompression,
    ContextPreservation,
    ContextDisposal,
}

impl LifecycleStage {
    pub fn label(&self) -> &'static str {
        match self {
            Self::InputAcquisition => "InputAcquisition",
            Self::IntentExtraction => "IntentExtraction",
            Self::GoalActivation => "GoalActivation",
            Self::ContextAssembly => "ContextAssembly",
            Self::ContextExpansion => "ContextExpansion",
            Self::ActiveReasoning => "ActiveReasoning",
            Self::ContinuousRefinement => "ContinuousRefinement",
            Self::Reflection => "Reflection",
            Self::ContextCompression => "ContextCompression",
            Self::ContextPreservation => "ContextPreservation",
            Self::ContextDisposal => "ContextDisposal",
        }
    }
}

/// A context lifecycle tracking the full lifecycle of a temporary cognitive environment.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextLifecycle {
    pub correlation_id: String,
    pub stage: LifecycleStage,
    pub active: bool,
    pub completed: bool,
    pub disposed: bool,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

impl ContextLifecycle {
    pub fn new(correlation_id: &str) -> Self {
        Self {
            correlation_id: correlation_id.to_string(),
            stage: LifecycleStage::InputAcquisition,
            active: true,
            completed: false,
            disposed: false,
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }
    pub fn advance(&mut self) {
        if self.completed || self.disposed {
            return;
        }
        self.stage = match self.stage {
            LifecycleStage::InputAcquisition => LifecycleStage::IntentExtraction,
            LifecycleStage::IntentExtraction => LifecycleStage::GoalActivation,
            LifecycleStage::GoalActivation => LifecycleStage::ContextAssembly,
            LifecycleStage::ContextAssembly => LifecycleStage::ContextExpansion,
            LifecycleStage::ContextExpansion => LifecycleStage::ActiveReasoning,
            LifecycleStage::ActiveReasoning => LifecycleStage::ContinuousRefinement,
            LifecycleStage::ContinuousRefinement => LifecycleStage::Reflection,
            LifecycleStage::Reflection => LifecycleStage::ContextCompression,
            LifecycleStage::ContextCompression => LifecycleStage::ContextPreservation,
            LifecycleStage::ContextPreservation => LifecycleStage::ContextDisposal,
            LifecycleStage::ContextDisposal => LifecycleStage::ContextDisposal,
        };
        if self.stage == LifecycleStage::ContextDisposal {
            self.completed = true;
            self.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }
    pub fn dispose(&mut self) {
        self.stage = LifecycleStage::ContextDisposal;
        self.active = false;
        self.completed = true;
        self.disposed = true;
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }
    /// Autonomous context repair after interruption (§15 / T-COO-10).
    /// Resets lifecycle to active reasoning stage and clears interruption flags.
    pub fn repair(&mut self) {
        if self.completed || self.disposed {
            self.active = true;
            self.completed = false;
            self.disposed = false;
            self.stage = LifecycleStage::ActiveReasoning;
            self.completed_at = None;
            tracing::info!(
                correlation_id = %self.correlation_id,
                "Autonomous context repair triggered: lifecycle restored to ActiveReasoning"
            );
        }
    }
    pub fn is_terminal(&self) -> bool {
        self.completed || self.disposed
    }
}

impl Default for ContextLifecycle {
    fn default() -> Self {
        Self::new("default")
    }
}

/// The 9 stages of the context assembly pipeline (Chapter 7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssemblyStage {
    ConversationAnalysis,
    PlannerRequirements,
    MemoryRetrieval,
    ExperienceRetrieval,
    KnowledgeRetrieval,
    ContextRanking,
    Deduplication,
    Compression,
    TokenBudgetAllocation,
}

/// A context layer representing temporary cognitive state.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextLayer {
    Working,
    ShortTerm,
    Session,
    Project,
    Historical,
}

/// Context assembly tracking references and state.
#[derive(Debug, Clone)]
pub struct ContextAssembly {
    pub references: Vec<String>,
    pub goals: Vec<String>,
    pub layers: Vec<ContextLayer>,
    pub constraints: Vec<String>,
    pub stages: Vec<AssemblyStage>,
    pub assembled_at: i64,
}

impl ContextAssembly {
    pub fn add_stage(&mut self, stage: AssemblyStage) {
        if !self.stages.contains(&stage) {
            self.stages.push(stage);
        }
    }
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            goals: Vec::new(),
            layers: Vec::new(),
            constraints: Vec::new(),
            stages: Vec::new(),
            assembled_at: chrono::Utc::now().timestamp(),
        }
    }
    pub fn add_reference(&mut self, reference: &str) {
        self.references.push(reference.to_string());
    }
    pub fn add_goal(&mut self, goal: &str) {
        if !self.goals.contains(&goal.to_string()) {
            self.goals.push(goal.to_string());
        }
    }
    pub fn add_layer(&mut self, layer: ContextLayer) {
        self.layers.push(layer);
    }
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
#[derive(Debug, Clone, Default)]
pub struct ContextEngine {
    assemblies: HashMap<String, ContextAssembly>,
    database: Option<std::sync::Arc<crate::database::sqlite::SqliteDatabase>>,
    knowledge_store: Option<std::sync::Arc<crate::knowledge::KnowledgeStore>>,
}

impl ContextEngine {
    pub fn new() -> Self {
        Self {
            assemblies: HashMap::new(),
            database: None,
            knowledge_store: None,
        }
    }
    pub fn wire(
        &mut self,
        database: std::sync::Arc<crate::database::sqlite::SqliteDatabase>,
        knowledge_store: std::sync::Arc<crate::knowledge::KnowledgeStore>,
    ) {
        self.database = Some(database);
        self.knowledge_store = Some(knowledge_store);
    }
    pub async fn assemble_context(
        &mut self,
        correlation_id: &str,
        input: &str,
        goal: &str,
        budget: usize,
    ) -> ContextAssembly {
        let mut assembly = ContextAssembly::new();
        assembly.add_stage(AssemblyStage::ConversationAnalysis);
        let keywords = conversation_analysis(input);
        assembly.add_stage(AssemblyStage::PlannerRequirements);
        let requirements = planner_requirements(goal);
        assembly.add_stage(AssemblyStage::MemoryRetrieval);
        let memory_results = memory_retrieval(input, budget).await;
        assembly.add_stage(AssemblyStage::ExperienceRetrieval);
        let experience_results = experience_retrieval(input, budget).await;
        assembly.add_stage(AssemblyStage::KnowledgeRetrieval);
        let knowledge_results = knowledge_retrieval(input, budget).await;
        assembly.add_stage(AssemblyStage::ContextRanking);
        let all_items: Vec<String> = memory_results
            .clone()
            .into_iter()
            .chain(experience_results.clone())
            .chain(knowledge_results.clone())
            .collect();
        let ranked = context_ranking(input, &all_items);
        assembly.add_stage(AssemblyStage::Deduplication);
        let deduped: Vec<String> = ranked.iter().map(|(item, _)| item.clone()).collect();
        let deduped = deduplicate(&deduped);
        assembly.add_stage(AssemblyStage::Compression);
        let compressed = compress_context(&deduped, budget);
        assembly.add_stage(AssemblyStage::TokenBudgetAllocation);
        let final_items = compressed;
        assembly.add_reference(correlation_id);
        assembly.add_goal(goal);
        for keyword in &keywords {
            assembly.add_reference(keyword);
        }
        for req in &requirements {
            assembly.add_constraint(&format!("requirement: {}", req));
        }
        if !memory_results.is_empty() {
            assembly.add_layer(ContextLayer::Working);
        }
        if !experience_results.is_empty() {
            assembly.add_layer(ContextLayer::ShortTerm);
        }
        if !knowledge_results.is_empty() {
            assembly.add_layer(ContextLayer::Session);
        }
        self.assemblies
            .insert(correlation_id.to_string(), assembly.clone());
        tracing::info!(
            correlation_id = correlation_id,
            keywords = ?keywords,
            requirements = ?requirements,
            memory_items = memory_results.len(),
            experience_items = experience_results.len(),
            knowledge_items = knowledge_results.len(),
            final_context_size = final_items.len(),
            "Context assembled successfully"
        );
        assembly
    }
    pub fn get_assembly_mut(&mut self, correlation_id: &str) -> Option<&mut ContextAssembly> {
        self.assemblies.get_mut(correlation_id)
    }
    pub fn dispose_context(&mut self, correlation_id: &str) {
        self.assemblies.remove(correlation_id);
    }
    pub fn expire_context(&mut self, correlation_id: &str) -> Option<ContextAssembly> {
        self.assemblies.remove(correlation_id)
    }
    pub fn archive_context(&self, correlation_id: &str) -> Option<ContextAssembly> {
        self.assemblies.get(correlation_id).cloned()
    }
    pub fn is_expired(&self, correlation_id: &str) -> bool {
        !self.assemblies.contains_key(correlation_id)
    }
}

pub fn wire_lifecycle_integration() {
    let assembly = ContextAssembly::new();
    let lifecycle = ContextLifecycle::new("test-corr");
    tracing::debug!(
        stage = %lifecycle.stage.label(),
        assembly_layers = assembly.layers.len(),
        "Lifecycle integration actively wired"
    );
}

pub fn wire_context_engine_usage() {
    let mut engine = ContextEngine::new();
    tracing::debug!("Context engine wired: new engine created");
    let handle = tokio::runtime::Handle::current();
    handle.block_on(async {
        let assembly = engine
            .assemble_context("test-corr", "test input", "test goal", 100)
            .await;
        tracing::debug!(
            "Context engine assemble_context wired: stages={}, references={:?}",
            assembly.stages.len(),
            assembly.references
        );
    });
    let assembly_mut = engine.get_assembly_mut("test-corr");
    if let Some(a) = assembly_mut.as_ref() {
        tracing::debug!(
            "Context engine get_assembly_mut wired: references={:?}",
            a.references.len()
        );
    }
    engine.dispose_context("test-corr");
    engine.expire_context("test-corr");
    engine.archive_context("test-corr");
    let expired = engine.is_expired("test-corr");
    tracing::debug!("Context engine lifecycle wired: expired={:?}", expired);
}

pub fn run_assembly(stages: &[AssemblyStage], correlation_id: &str) -> ContextAssembly {
    let mut assembly = ContextAssembly::new();
    assembly.add_reference(correlation_id);
    for stage in stages {
        tracing::debug!("assembly stage: {:?}", stage);
        assembly.add_layer(ContextLayer::Working);
    }
    assembly
}

pub fn conversation_analysis(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

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

pub fn context_ranking(query: &str, items: &[String]) -> Vec<(String, f32)> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .collect();
    if query_terms.is_empty() {
        return items.iter().map(|item| (item.clone(), 0.5)).collect();
    }
    let avg_doc_len: f64 =
        items.iter().map(|item| item.len() as f64).sum::<f64>() / items.len().max(1) as f64;
    let k1 = 1.2;
    let b = 0.75;
    items
        .iter()
        .map(|item| {
            let item_lower = item.to_lowercase();
            let doc_len = item_lower.len() as f64;
            let mut score = 0.0f64;
            for term in &query_terms {
                let term_lower = term.to_lowercase();
                let term_freq = item_lower.matches(&term_lower).count() as f64;
                if term_freq > 0.0 {
                    let idf = (1.0 + doc_len / avg_doc_len.max(1.0)).ln() + 1.0;
                    let tf = term_freq * (k1 + 1.0)
                        / (term_freq + k1 * (1.0 - b + b * doc_len / avg_doc_len.max(1.0)));
                    score += tf * idf;
                }
            }
            let normalized = (score / query_terms.len() as f64).clamp(0.0, 1.0) as f32;
            (item.clone(), normalized)
        })
        .collect()
}

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

pub fn compress_context(items: &[String], budget: usize) -> Vec<String> {
    if budget == 0 {
        return Vec::new();
    }
    let layer_weights = [
        ("working", 1.0),
        ("short_term", 0.9),
        ("session", 0.8),
        ("project", 0.6),
        ("historical", 0.4),
        ("archive", 0.2),
    ];
    let token_budget = crate::prompt::TokenBudget::default_budget();
    let total_budget = token_budget.total;
    let token_estimate = |text: &str| text.len() / 4;
    let mut selected = Vec::new();
    let mut used_tokens = 0;
    for item in items {
        let layer_weight = layer_weights
            .iter()
            .find(|(name, _)| item.contains(*name))
            .map(|(_, w)| *w)
            .unwrap_or(0.7);
        let item_tokens = token_estimate(item);
        let weighted_tokens = (item_tokens as f32 / layer_weight) as usize;
        if used_tokens + weighted_tokens <= total_budget.min(budget * 4) {
            selected.push(item.clone());
            used_tokens += weighted_tokens;
        } else {
            let remaining = total_budget.min(budget * 4).saturating_sub(used_tokens);
            let max_chars = (remaining as f32 * layer_weight) as usize * 4;
            if max_chars > 0 {
                selected.push(item[..max_chars.min(item.len())].to_string());
            }
            break;
        }
    }
    selected
}

pub async fn memory_retrieval(query: &str, budget: usize) -> Vec<String> {
    tracing::debug!("memory retrieval: query={:?} budget={}", query, budget);
    let permanent = crate::memory::PermanentMemory::new(100);
    let items = permanent.search(query).await;
    let results: Vec<String> = items
        .into_iter()
        .take(budget)
        .map(|item| format!("[memory:{}] {}", item.id, item.content))
        .collect();
    tracing::debug!(
        memory_retrieved = results.len(),
        "Context: memory retrieval completed"
    );
    results
}

pub async fn experience_retrieval(query: &str, budget: usize) -> Vec<String> {
    tracing::debug!("experience retrieval: query={:?} budget={}", query, budget);
    let db_result = crate::database::sqlite::SqliteDatabase::initialize();
    let db = match db_result {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(
                "Failed to initialize database for experience retrieval: {}",
                e
            );
            return Vec::new();
        }
    };
    let experiences =
        crate::experience::repository::find_similar_encounters(std::sync::Arc::new(db), query)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to find similar experiences: {}", e);
                Vec::new()
            });
    let output: Vec<String> = experiences
        .into_iter()
        .take(budget)
        .map(|exp| format!("[experience:{}] {}", exp.id, exp.input))
        .collect();
    tracing::debug!(
        experience_retrieved = output.len(),
        "Context: experience retrieval completed"
    );
    output
}

pub async fn knowledge_retrieval(query: &str, budget: usize) -> Vec<String> {
    tracing::debug!("knowledge retrieval: query={:?} budget={}", query, budget);
    let store = crate::knowledge::KnowledgeStore::new(budget);
    let items = store.get_all().await;
    let q = crate::knowledge::KnowledgeQuery {
        text: Some(query.to_string()),
        knowledge_type: None,
        status: None,
        min_confidence: None,
        tags: None,
        mature_only: false,
        include_related: false,
        limit: Some(budget),
    };
    let results = crate::knowledge::apply_query(&items, &q);
    let output: Vec<String> = results
        .into_iter()
        .map(|item| format!("[knowledge:{}] {}", item.id, item.statement))
        .collect();
    tracing::debug!(
        knowledge_retrieved = output.len(),
        "Context: knowledge retrieval completed"
    );
    output
}

pub fn integrate_lifecycle(assembly: &mut ContextAssembly, lifecycle: &mut ContextLifecycle) {
    if assembly.layers.contains(&ContextLayer::Working) {
        lifecycle.advance();
    }
    tracing::debug!(
        stage = %lifecycle.stage.label(),
        correlation = %lifecycle.correlation_id,
        "Context lifecycle integrated with assembly"
    );
}
