//! Model integration and inference context (Architecture Chapter 14.4).
//!
//! Defines the public context-handling rules for inference: the structured
//! context passed to an inference provider, and the truncation function that
//! enforces a token budget.

use serde::{Deserialize, Serialize};

/// A single message in a conversation context.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// The role of the message sender (e.g. "system", "user", "assistant").
    pub role: String,
    /// The text content of the message.
    pub content: String,
}

impl ChatMessage {
    /// Create a new chat message.
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// Structured context for an inference request.
///
/// Per Architecture §14.4, this carries the system prompt, the conversation
/// messages, and the token budget for the request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceContext {
    /// System-level instructions for the inference.
    pub system: String,
    /// Ordered conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Maximum tokens allowed for the response.
    pub max_tokens: u32,
}

impl InferenceContext {
    /// Create a new inference context with an empty message list.
    pub fn new(system: impl Into<String>, max_tokens: u32) -> Self {
        Self {
            system: system.into(),
            messages: Vec::new(),
            max_tokens,
        }
    }

    /// Add a message to the context.
    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }
}

/// Truncate the conversation context to stay within a token budget.
///
/// This is a placeholder implementation: it keeps the system prompt intact,
/// preserves the most recent messages, and drops older messages until the
/// estimated token count falls within the budget.
///
/// Per Architecture §14.4, real implementations should use a tokenizer-based
/// budget calculation; this version uses a rough per-message estimate.
pub fn truncate_context(ctx: &InferenceContext, budget: u32) -> InferenceContext {
    // Rough estimate: 4 tokens per message + 10 tokens for system.
    let estimated_per_message = 4u32;
    let system_overhead = 10u32;
    let available_for_messages = budget.saturating_sub(system_overhead);
    let max_messages = (available_for_messages / estimated_per_message).max(0) as usize;

    let retained_messages = if ctx.messages.len() > max_messages {
        let drop_count = ctx.messages.len() - max_messages;
        ctx.messages.iter().skip(drop_count).cloned().collect()
    } else {
        ctx.messages.clone()
    };

    InferenceContext {
        system: ctx.system.clone(),
        messages: retained_messages,
        max_tokens: ctx.max_tokens,
    }
}
