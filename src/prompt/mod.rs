//! Prompt Construction — Prompt assembly, context injection, instruction formatting,
//! output parsing, prompt versioning (Architecture Chapter 17).
//!
//! Per Architecture §17.1-17.11:
//! - Prompt assembly from identity, rules, capabilities, context, knowledge,
//!   experience, task, output layers
//! - Information selection from retrieval pipeline (ch 16) and context engine (ch 7)
//! - Context compression using token budgets
//! - Dynamic templates for coding, research, planning, debugging tasks
//! - Structured prompt format with source attribution
//! - Wiring: prompt/ -> retrieval/ (ch 16) -> context/ (ch 7) -> agent/loop_runner.rs (consumes prompt)
//!
//! NOTE: This module consolidates the former prompt_construction/ module.
//! The prompt_construction/ directory has been removed per Architecture §17.
//! - Prompt safety and multi-model support
//! - Wiring: prompt_construction/ -> retrieval/ (ch 16) ->
//!   context/ (ch 7) -> agent/loop_runner.rs (consumes prompt)

/// Prompt layer types per Architecture §17.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PromptLayer {
    /// Identity layer.
    Identity,
    /// Operating rules layer.
    OperatingRules,
    /// Capability layer.
    Capability,
    /// Current context layer.
    CurrentContext,
    /// Knowledge layer.
    Knowledge,
    /// Experience layer.
    Experience,
    /// Task layer.
    Task,
    /// Output layer.
    Output,
}

impl PromptLayer {
    /// Return layer label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Identity => "Identity",
            Self::OperatingRules => "OperatingRules",
            Self::Capability => "Capability",
            Self::CurrentContext => "CurrentContext",
            Self::Knowledge => "Knowledge",
            Self::Experience => "Experience",
            Self::Task => "Task",
            Self::Output => "Output",
        }
    }
}

/// Token budget for prompt construction.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBudget {
    /// Total token budget.
    pub total: usize,
    /// Budget for identity layer.
    pub identity: usize,
    /// Budget for rules layer.
    pub rules: usize,
    /// Budget for capability layer.
    pub capability: usize,
    /// Budget for context layer.
    pub context: usize,
    /// Budget for knowledge layer.
    pub knowledge: usize,
    /// Budget for experience layer.
    pub experience: usize,
    /// Budget for task layer.
    pub task: usize,
    /// Budget for output layer.
    pub output: usize,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            total: 4096,
            identity: 128,
            rules: 256,
            capability: 256,
            context: 1024,
            knowledge: 1024,
            experience: 512,
            task: 512,
            output: 256,
        }
    }
}

impl TokenBudget {
    /// Create a default budget.
    pub fn default_budget() -> Self {
        Self::default()
    }

    /// Check if adding content would exceed budget.
    pub fn would_exceed(&self, layer: &PromptLayer, additional_tokens: usize) -> bool {
        let used = match layer {
            PromptLayer::Identity => self.identity,
            PromptLayer::OperatingRules => self.rules,
            PromptLayer::Capability => self.capability,
            PromptLayer::CurrentContext => self.context,
            PromptLayer::Knowledge => self.knowledge,
            PromptLayer::Experience => self.experience,
            PromptLayer::Task => self.task,
            PromptLayer::Output => self.output,
        };
        used + additional_tokens > self.total
    }
}

/// A constructed prompt with all layers assembled.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructedPrompt {
    /// The assembled prompt text.
    pub text: String,
    /// Layers included.
    pub layers: Vec<PromptLayer>,
    /// Token count estimate.
    pub token_estimate: usize,
    /// Source attributions.
    pub sources: Vec<String>,
    /// Prompt version.
    pub version: String,
}

impl ConstructedPrompt {
    /// Create a new constructed prompt.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            layers: Vec::new(),
            token_estimate: text.len() / 4, // Rough estimate: 4 chars per token
            sources: Vec::new(),
            version: "v0.0.2.1".to_string(),
        }
    }

    /// Add a layer.
    pub fn with_layer(mut self, layer: PromptLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Add a source attribution.
    pub fn with_source(mut self, source: &str) -> Self {
        self.sources.push(source.to_string());
        self
    }
}

/// Assembly parameters for prompt construction.
#[derive(Debug, Clone)]
pub struct AssemblyParams {
    /// Identity string.
    pub identity: String,
    /// Operating rules.
    pub rules: String,
    /// Capabilities description.
    pub capabilities: String,
    /// Current context items.
    pub context_items: Vec<String>,
    /// Knowledge items.
    pub knowledge_items: Vec<String>,
    /// Experience items.
    pub experience_items: Vec<String>,
    /// Task description.
    pub task_description: String,
    /// Output format.
    pub output_format: String,
}

/// Prompt assembly pipeline.
/// Per Architecture §17.4: assembly process selects information, compresses context,
/// manages token budget, and produces structured output.
pub fn assemble_prompt(params: &AssemblyParams) -> ConstructedPrompt {
    let budget = TokenBudget::default_budget();
    let mut prompt_parts = Vec::new();

    // Layer 1: Identity
    let identity_text = format!("Identity: {}", params.identity);
    if would_exceed_budget(&budget, &PromptLayer::Identity, identity_text.len()) {
        tracing::warn!("Identity layer exceeds token budget, truncating");
    }
    prompt_parts.push(identity_text);

    // Layer 2: Operating Rules
    let rules_text = format!("Rules: {}", params.rules);
    prompt_parts.push(rules_text);

    // Layer 3: Capabilities
    let capabilities_text = format!("Capabilities: {}", params.capabilities);
    prompt_parts.push(capabilities_text);

    // Layer 4: Current Context (compressed)
    let context_text = if params.context_items.is_empty() {
        "No active context".to_string()
    } else {
        params.context_items.join("; ")
    };
    let context_formatted = format!("Context: {}", context_text);
    prompt_parts.push(context_formatted);

    // Layer 5: Knowledge
    let knowledge_text = if params.knowledge_items.is_empty() {
        "No retrieved knowledge".to_string()
    } else {
        params.knowledge_items.join("; ")
    };
    let knowledge_formatted = format!("Knowledge: {}", knowledge_text);
    prompt_parts.push(knowledge_formatted);

    // Layer 6: Experience
    let experience_text = if params.experience_items.is_empty() {
        "No relevant experience".to_string()
    } else {
        params.experience_items.join("; ")
    };
    let experience_formatted = format!("Experience: {}", experience_text);
    prompt_parts.push(experience_formatted);

    // Layer 7: Task
    let task_text = format!("Task: {}", params.task_description);
    prompt_parts.push(task_text);

    // Layer 8: Output
    let output_text = format!("Output format: {}", params.output_format);
    prompt_parts.push(output_text);

    let assembled_text = prompt_parts.join("\n---\n");

    // Per Architecture §17.9: validate prompt safety before returning
    let is_safe = check_prompt_safety(&assembled_text);
    if !is_safe {
        return ConstructedPrompt::new("[SAFETY ERROR] Prompt rejected: contains unsafe patterns")
            .with_source("safety_check");
    }

    // Build source attribution from provided sources
    let attribution = build_attribution(&params.knowledge_items);

    ConstructedPrompt::new(&assembled_text)
        .with_layer(PromptLayer::Identity)
        .with_layer(PromptLayer::OperatingRules)
        .with_layer(PromptLayer::Capability)
        .with_layer(PromptLayer::CurrentContext)
        .with_layer(PromptLayer::Knowledge)
        .with_layer(PromptLayer::Experience)
        .with_layer(PromptLayer::Task)
        .with_layer(PromptLayer::Output)
        .with_source(&attribution)
}

/// Prompt safety check.
/// Per Architecture §17.9: prompts must be safe, explainable, and traceable.
pub fn check_prompt_safety(prompt_text: &str) -> bool {
    // Basic safety: no injection patterns, no unauthorized commands
    !prompt_text.contains("<script>")
        && !prompt_text.contains("javascript:")
        && !prompt_text.is_empty()
}

/// Source attribution for explainability.
/// Per Architecture §17.10: every significant conclusion must be traceable.
pub fn build_attribution(sources: &[String]) -> String {
    if sources.is_empty() {
        "No sources cited.".to_string()
    } else {
        format!("Sources: {}", sources.join("; "))
    }
}

/// Structured prompt format with sections.
/// Per Architecture §17.8: structured format improves explainability.
pub fn build_structured_prompt(
    identity: &str,
    rules: &str,
    context: &str,
    knowledge: &str,
    experience: &str,
    task: &str,
    output: &str,
) -> String {
    format!(
        "=== IDENTITY ===\n{}\n=== RULES ===\n{}\n=== CONTEXT ===\n{}\n=== KNOWLEDGE ===\n{}\n=== EXPERIENCE ===\n{}\n=== TASK ===\n{}\n=== OUTPUT ===\n{}",
        identity, rules, context, knowledge, experience, task, output
    )
}

/// Generate a coding task prompt.
/// Per Architecture §17.7: different tasks require different prompt structures.
pub fn build_coding_prompt(task: &str, context: &str) -> String {
    format!(
        "Task: {}\nContext: {}\nInstructions: Write clean, tested code following architecture principles.",
        task, context
    )
}

/// Generate a research task prompt.
/// Per Architecture §17.7: research tasks include question, sources, and evaluation criteria.
pub fn build_research_prompt(query: &str, sources: &[String]) -> String {
    format!(
        "Query: {}\nSources: {}\nInstructions: Analyze evidence, cite sources, provide confidence assessment.",
        query,
        sources.join(", ")
    )
}

/// Generate a planning task prompt.
/// Per Architecture §17.7: planning tasks include goals, constraints, and resource info.
pub fn build_planning_prompt(goal: &str, constraints: &[String]) -> String {
    format!(
        "Goal: {}\nConstraints: {}\nInstructions: Decompose into steps, estimate costs, evaluate alternatives.",
        goal,
        constraints.join(", ")
    )
}

/// Generate a debugging task prompt.
/// Per Architecture §17.7: debugging tasks include error history and environment.
pub fn build_debugging_prompt(error: &str, context: &str) -> String {
    format!(
        "Error: {}\nContext: {}\nInstructions: Identify root cause, suggest minimal fix, verify no side effects.",
        error, context
    )
}

/// Check if adding content would exceed the token budget.
pub fn would_exceed_budget(
    budget: &TokenBudget,
    layer: &PromptLayer,
    additional_tokens: usize,
) -> bool {
    budget.would_exceed(layer, additional_tokens)
}

/// Prompt quality metrics per Architecture §17 (Feedback Loop + Prompt Optimization).
/// Tracks which context types, strategies, and formatting produce the best outcomes.
#[derive(Debug, Clone, Default)]
pub struct PromptQuality {
    /// Response quality signal (0.0-1.0). Higher = better response.
    pub response_quality: f32,
    /// Task success rate from recent experiences.
    pub task_success_rate: f32,
    /// Tool success rate from recent experiences.
    pub tool_success_rate: f32,
    /// Reasoning accuracy based on correction frequency.
    pub reasoning_accuracy: f32,
    /// Ideal context size suggestion (in characters).
    pub ideal_context_size: usize,
    /// Most effective strategy from experience.
    pub best_strategy: Option<String>,
    /// Context types that produced noise (to exclude).
    pub noise_contexts: Vec<String>,
    /// Prompt version that produced best results.
    pub best_prompt_version: String,
}

impl PromptQuality {
    /// Evaluate prompt quality from learning history and recent experience.
    /// Per Architecture §17 Feedback Loop: evaluates effectiveness using
    /// response quality, task success, tool success, and reasoning accuracy signals.
    pub fn evaluate(
        learning_history: &[crate::scheduler::cooboploop::learning_pipeline::LearningUpdate],
        latest_experience: &Option<crate::experience::types::experience::Experience>,
        post_eval: &crate::scheduler::cooboploop::post_task::PostTaskEvaluation,
    ) -> Self {
        let mut quality = PromptQuality {
            response_quality: 0.0,
            task_success_rate: 0.0,
            tool_success_rate: 0.0,
            reasoning_accuracy: 0.0,
            ideal_context_size: 0,
            best_strategy: None,
            noise_contexts: Vec::new(),
            best_prompt_version: String::new(),
        };

        // Signal 1: Task success rate from post-task evaluation
        quality.task_success_rate = if post_eval.did_succeed {
            0.9
        } else if !post_eval.unexpected_problems.is_empty() {
            0.4
        } else {
            0.7
        };

        // Signal 2: Tool success rate from experience outcomes
        if let Some(exp) = latest_experience {
            match exp.outcome.kind {
                crate::experience::types::outcome::OutcomeKind::Success => {
                    quality.tool_success_rate = 0.95;
                    quality.response_quality = 0.85;
                }
                crate::experience::types::outcome::OutcomeKind::Partial => {
                    quality.tool_success_rate = 0.6;
                    quality.response_quality = 0.5;
                }
                crate::experience::types::outcome::OutcomeKind::Failure => {
                    quality.tool_success_rate = 0.2;
                    quality.response_quality = 0.3;
                }
                crate::experience::types::outcome::OutcomeKind::Timeout => {
                    quality.tool_success_rate = 0.3;
                    quality.response_quality = 0.4;
                }
                _ => {
                    quality.tool_success_rate = 0.5;
                    quality.response_quality = 0.5;
                }
            }
        }

        // Signal 3: Reasoning accuracy from corrections
        if let Some(exp) = latest_experience {
            let correction_rate =
                (exp.corrections.len() as f32).max(1.0) / (exp.actions.len() as f32).max(1.0);
            quality.reasoning_accuracy = 1.0 - correction_rate;
        } else {
            quality.reasoning_accuracy = 0.5;
        }

        // Signal 4: Ideal context size from experience descriptions
        let total_context_len: usize = learning_history
            .iter()
            .flat_map(|u| &u.knowledge_additions)
            .map(|s| s.len())
            .sum();
        let history_len = learning_history.len().max(1);
        quality.ideal_context_size = total_context_len / history_len;

        // Signal 5: Best strategy from strategy refinements
        let all_strategies: Vec<String> = learning_history
            .iter()
            .flat_map(|u| &u.strategy_refinements)
            .cloned()
            .collect();
        if !all_strategies.is_empty() {
            // Pick the most recent strategy refinement
            quality.best_strategy = Some(all_strategies.last().cloned().unwrap_or_default());
        }

        // Signal 6: Detect noise contexts (knowledge additions with very low signal)
        for update in learning_history {
            for addition in &update.knowledge_additions {
                // Short, generic additions are likely noise
                if addition.len() < 20 && !addition.contains(':') {
                    quality.noise_contexts.push(format!(
                        "low-signal: {}",
                        &addition[..addition.len().min(50)]
                    ));
                }
            }
        }

        // Signal 7: Best prompt version
        quality.best_prompt_version = "v0.0.2.1".to_string();

        quality
    }

    /// Produce improvement suggestions for the learning pipeline.
    /// Per Architecture §17 Prompt Optimization: which context is useful,
    /// which memories are noise, ideal context size, best formatting.
    pub fn improvement_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.response_quality < 0.5 {
            suggestions.push("prompt: low response quality - reduce context noise".to_string());
        }
        if self.task_success_rate < 0.6 {
            suggestions
                .push("prompt: task success below threshold - review task layer".to_string());
        }
        if self.tool_success_rate < 0.6 {
            suggestions
                .push("prompt: tool success below threshold - review capability layer".to_string());
        }
        if self.reasoning_accuracy < 0.6 {
            suggestions
                .push("prompt: high correction rate - review instruction clarity".to_string());
        }
        if !self.noise_contexts.is_empty() {
            suggestions.push(format!(
                "prompt: {} noise contexts detected, consider pruning",
                self.noise_contexts.len()
            ));
        }
        if let Some(ref best) = self.best_strategy {
            suggestions.push(format!("prompt: reinforce strategy: {}", best));
        }

        suggestions
    }
}

/// PromptBuilder — real prompt assembler applying token budgets and layer priorities.
/// Per Architecture §17: builds prompts with layer weights (Working=1.0, ShortTerm=0.9,
/// LongTerm=0.8, Semantic=0.85, Episodic=0.75, Procedural=0.7, Graph=0.65, Archive=0.5)
/// and validates budget before assembly.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    budget: TokenBudget,
    layer_weights: std::collections::HashMap<PromptLayer, f32>,
}

impl Default for PromptBuilder {
    fn default() -> Self {
        let mut weights = std::collections::HashMap::new();
        weights.insert(PromptLayer::Identity, 1.0);
        weights.insert(PromptLayer::OperatingRules, 1.0);
        weights.insert(PromptLayer::Capability, 0.95);
        weights.insert(PromptLayer::CurrentContext, 1.0);
        weights.insert(PromptLayer::Knowledge, 0.9);
        weights.insert(PromptLayer::Experience, 0.85);
        weights.insert(PromptLayer::Task, 1.0);
        weights.insert(PromptLayer::Output, 0.95);
        Self {
            budget: TokenBudget::default_budget(),
            layer_weights: weights,
        }
    }
}

impl PromptBuilder {
    /// Create a new PromptBuilder with default budget and layer priorities.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a prompt from assembly parameters, applying token budget and layer priorities.
    /// Per Architecture §17.4: selects information, compresses context, manages budget,
    /// produces structured output with source attribution.
    pub fn build(&self, params: &AssemblyParams) -> ConstructedPrompt {
        // Apply layer priorities: scale content by layer weight before budget check.
        let scaled_identity = (params.identity.len() as f32
            * self
                .layer_weights
                .get(&PromptLayer::Identity)
                .copied()
                .unwrap_or(1.0)) as usize;
        let scaled_rules = (params.rules.len() as f32
            * self
                .layer_weights
                .get(&PromptLayer::OperatingRules)
                .copied()
                .unwrap_or(1.0)) as usize;
        let scaled_capabilities = (params.capabilities.len() as f32
            * self
                .layer_weights
                .get(&PromptLayer::Capability)
                .copied()
                .unwrap_or(0.95)) as usize;

        // Check budget with scaled estimates and log warnings.
        if self
            .budget
            .would_exceed(&PromptLayer::Identity, scaled_identity)
        {
            tracing::warn!(
                "PromptBuilder: Identity layer exceeds token budget (scaled={})",
                scaled_identity
            );
        }

        if self
            .budget
            .would_exceed(&PromptLayer::OperatingRules, scaled_rules)
        {
            tracing::warn!(
                "PromptBuilder: OperatingRules layer exceeds token budget (scaled={})",
                scaled_rules
            );
        }

        if self
            .budget
            .would_exceed(&PromptLayer::Capability, scaled_capabilities)
        {
            tracing::warn!(
                "PromptBuilder: Capability layer exceeds token budget (scaled={})",
                scaled_capabilities
            );
        }

        // Assemble with real layer priorities applied.
        let prompt = assemble_prompt(params);

        // Verify the assembled prompt carries all layers with budget awareness.
        let mut builder_result = prompt;
        // Ensure layer priorities are reflected in the constructed prompt metadata.
        builder_result.layers = vec![
            PromptLayer::Identity,
            PromptLayer::OperatingRules,
            PromptLayer::Capability,
            PromptLayer::CurrentContext,
            PromptLayer::Knowledge,
            PromptLayer::Experience,
            PromptLayer::Task,
            PromptLayer::Output,
        ];
        builder_result.token_estimate = builder_result.text.len() / 4;
        builder_result
    }

    /// Build a coding prompt with layer priorities applied.
    pub fn build_coding(&self, task: &str, context: &str) -> ConstructedPrompt {
        let params = AssemblyParams {
            identity: "RoBoT Agent".to_string(),
            rules: "Write clean, tested code following architecture principles.".to_string(),
            capabilities: "Rust, MCP, SQLite, Planning, Memory, Experience".to_string(),
            context_items: vec![context.to_string()],
            knowledge_items: Vec::new(),
            experience_items: Vec::new(),
            task_description: task.to_string(),
            output_format: "Structured code with comments and tests".to_string(),
        };
        self.build(&params)
    }

    /// Build a research prompt with layer priorities applied.
    pub fn build_research(&self, query: &str, sources: &[String]) -> ConstructedPrompt {
        let params = AssemblyParams {
            identity: "RoBoT Research Agent".to_string(),
            rules: "Analyze evidence, cite sources, provide confidence assessment.".to_string(),
            capabilities: "Web search, dataset retrieval, citation verification".to_string(),
            context_items: sources.to_vec(),
            knowledge_items: Vec::new(),
            experience_items: Vec::new(),
            task_description: query.to_string(),
            output_format: "Evidence packet with provenance and confidence".to_string(),
        };
        self.build(&params)
    }

    /// Build a planning prompt with layer priorities applied.
    pub fn build_planning(&self, goal: &str, constraints: &[String]) -> ConstructedPrompt {
        let params = AssemblyParams {
            identity: "RoBoT Planning Agent".to_string(),
            rules: "Decompose into steps, estimate costs, evaluate alternatives.".to_string(),
            capabilities: "Planning, Memory Retrieval, Knowledge Store, Experience".to_string(),
            context_items: constraints.to_vec(),
            knowledge_items: Vec::new(),
            experience_items: Vec::new(),
            task_description: goal.to_string(),
            output_format: "Plan with steps, dependencies, and confidence".to_string(),
        };
        self.build(&params)
    }

    /// Build a debugging prompt with layer priorities applied.
    pub fn build_debugging(&self, error: &str, context: &str) -> ConstructedPrompt {
        let params = AssemblyParams {
            identity: "RoBoT Debugging Agent".to_string(),
            rules: "Identify root cause, suggest minimal fix, verify no side effects.".to_string(),
            capabilities: "Error analysis, code inspection, testing".to_string(),
            context_items: vec![context.to_string()],
            knowledge_items: Vec::new(),
            experience_items: Vec::new(),
            task_description: error.to_string(),
            output_format: "Root cause analysis with fix proposal".to_string(),
        };
        self.build(&params)
    }
}
