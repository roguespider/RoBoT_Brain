//! Prompt context compression per Architecture Chapter 17.
//! Compresses context to fit within token budgets.

/// Compression strategy for prompt context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionStrategy {
    /// Keep all content, no compression.
    None,
    /// Truncate to fit budget.
    Truncate,
    /// Summarize content to fit budget.
    Summarize,
}

/// Compress text to fit within a token budget.
pub fn compress_text(text: &str, budget: usize, strategy: CompressionStrategy) -> String {
    let char_budget = budget * 4; // Rough estimate: 4 chars per token

    if text.len() <= char_budget {
        return text.to_string();
    }

    match strategy {
        CompressionStrategy::None => text[..char_budget.min(text.len())].to_string(),
        CompressionStrategy::Truncate => {
            let truncated = &text[..char_budget];
            let last_newline = truncated.rfind('\n').unwrap_or(char_budget);
            format!("{}\n... [truncated]", &text[..last_newline])
        }
        CompressionStrategy::Summarize => {
            // Simple summarization: keep first and last portions
            let half = char_budget / 2;
            let start = &text[..half.min(text.len())];
            let end = if text.len() > half {
                &text[text.len().saturating_sub(half)..]
            } else {
                ""
            };
            format!("{start}\n... [compressed] ...\n{end}")
        }
    }
}

/// Compress multiple context items to fit within budget.
pub fn compress_contexts(
    items: &[String],
    total_budget: usize,
    strategy: CompressionStrategy,
) -> Vec<String> {
    let per_item = total_budget / items.len().max(1);
    items
        .iter()
        .map(|item| compress_text(item, per_item, strategy))
        .collect()
}
