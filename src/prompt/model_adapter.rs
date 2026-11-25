//! Model-specific prompt adaptation per Architecture Chapter 17.
//! Adapts prompts for different language model types and versions.

/// Model type for prompt adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    /// General-purpose language model.
    General,
    /// Code-focused model.
    Code,
    /// Reasoning-focused model.
    Reasoning,
    /// Small/efficient model.
    Small,
}

/// Adapt a prompt for the given model type.
pub fn adapt_prompt(text: &str, model_type: ModelType) -> String {
    match model_type {
        ModelType::General => text.to_string(),
        ModelType::Code => {
            let mut adapted = String::from("You are an expert coding assistant.\n");
            adapted.push_str(text);
            adapted.push_str("\nProvide clean, well-documented code.");
            adapted
        }
        ModelType::Reasoning => {
            let mut adapted = String::from("Reason step by step.\n");
            adapted.push_str(text);
            adapted.push_str("\nProvide your reasoning before the final answer.");
            adapted
        }
        ModelType::Small => {
            // Truncate for small models with limited context
            let max_len = 2000;
            if text.len() > max_len {
                format!("{}\n... [truncated for small model]", &text[..max_len])
            } else {
                text.to_string()
            }
        }
    }
}

/// Get the recommended context budget for the model type.
pub fn recommended_budget(model_type: ModelType) -> usize {
    match model_type {
        ModelType::General => 4000,
        ModelType::Code => 6000,
        ModelType::Reasoning => 8000,
        ModelType::Small => 1000,
    }
}
