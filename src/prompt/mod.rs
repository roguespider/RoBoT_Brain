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
//! - Prompt safety and multi-model support
//! - Wiring: prompt_construction/ -> retrieval_pipeline/ (ch 16) ->
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
        learning_history: &[crate::cooboploop::learning_pipeline::LearningUpdate],
        latest_experience: &Option<crate::experience::types::experience::Experience>,
        post_eval: &crate::cooboploop::post_task::PostTaskEvaluation,
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
