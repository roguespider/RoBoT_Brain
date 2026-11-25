//! Prompt composition per Architecture Chapter 17.
//! Combines multiple prompt layers into a unified constructed prompt.

use crate::prompt::{ConstructedPrompt, PromptLayer, TokenBudget};

/// Composes prompt layers into a final prompt string.
pub struct PromptComposer {
    /// Template for the final prompt structure.
    template: String,
}

impl PromptComposer {
    /// Create a new composer with the given template.
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }

    /// Compose the given layers into a ConstructedPrompt within the budget.
    pub fn compose(
        &self,
        layers: &[(&PromptLayer, String)],
        budget: &TokenBudget,
    ) -> ConstructedPrompt {
        let mut text_parts = Vec::new();
        let mut layer_names = Vec::new();
        let mut token_estimate = 0usize;

        for (layer, content) in layers {
            let entry = format!("{}:\n{}\n", layer.label(), content);
            let entry_len = entry.len();
            if token_estimate + entry_len / 4 > budget.total {
                break;
            }
            text_parts.push(entry);
            layer_names.push((*layer).clone());
            token_estimate += entry_len / 4;
        }

        let mut text = text_parts.join("\n---\n");
        // Apply the template structure by wrapping composed content.
        if !text.is_empty() {
            text = format!("{}\n\n{}", self.template, text);
        }
        let sources: Vec<String> = Vec::new();

        ConstructedPrompt {
            text,
            layers: layer_names,
            token_estimate,
            sources,
            version: "1".to_string(),
        }
    }
}

impl Default for PromptComposer {
    fn default() -> Self {
        Self::new(
            "{identity}\n{rules}\n{capabilities}\n{context}\n{knowledge}\n{experience}\n{task}\n{output}",
        )
    }
}
