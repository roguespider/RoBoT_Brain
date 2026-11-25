//! Communication style preferences and formatting.

use serde::{Deserialize, Serialize};

/// Communication style preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CommunicationStyle {
    /// Minimal output, just essential information
    Concise,
    /// Balanced between brief and detailed
    #[default]
    Balanced,
    /// Full explanations with context
    Detailed,
}

impl CommunicationStyle {
    /// Get format string for response based on style
    pub fn format_response(&self, content: &str) -> String {
        match self {
            CommunicationStyle::Concise => {
                // Strip extra whitespace, take first paragraph
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            CommunicationStyle::Balanced => {
                // Take first few paragraphs
                content
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .take(5)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            CommunicationStyle::Detailed => content.to_string(),
        }
    }
}
