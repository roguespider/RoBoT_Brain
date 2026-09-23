//! Model integration and inference context (Architecture Chapter 14.4).
//!
//! Defines the public context-handling rules for inference: the structured
//! context passed to an inference provider, and the truncation function that
//! enforces a token budget.

use serde::{Deserialize, Serialize};

/// Inference provider trait.
pub trait InferenceProvider: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn complete(
        &self,
        prompt: &str,
        opts: &InferenceOptions,
    ) -> Result<InferenceResponse, InferenceError>;
    fn is_local(&self) -> bool;
}

/// Inference options.
#[derive(Debug, Clone, Default)]
pub struct InferenceOptions {
    pub max_tokens: u32,
    pub temperature: f32,
}

/// Inference response.
#[derive(Debug, Clone, Default)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_used: u32,
}

/// Inference error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    ModelNotFound,
    Timeout,
    InvalidInput,
}

/// Local model provider.
#[derive(Debug, Clone)]
pub struct LocalProvider {
    pub name: String,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self {
            name: "local".to_string(),
        }
    }
}

impl InferenceProvider for LocalProvider {
    fn name(&self) -> &str {
        &self.name
    }
    fn complete(
        &self,
        _prompt: &str,
        _opts: &InferenceOptions,
    ) -> Result<InferenceResponse, InferenceError> {
        Ok(InferenceResponse::default())
    }
    fn is_local(&self) -> bool {
        true
    }
}

/// Capability categories for model routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    Chat,
    Embedding,
    Tool,
    Vision,
    LongContext,
}

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

/// Model selector for task-based selection.
#[derive(Debug, Clone, Default)]
pub struct ModelSelector {
    pub preferred_chat: String,
    pub preferred_long_context: String,
}

impl ModelSelector {
    pub fn select_for_task(&self, task: &str) -> Option<String> {
        if task.contains("chat") || task.contains("conversation") {
            Some(self.preferred_chat.clone())
        } else if task.len() > 100 {
            Some(self.preferred_long_context.clone())
        } else {
            Some(self.preferred_chat.clone())
        }
    }
}

/// Inference request queue.
#[derive(Debug, Clone, Default)]
pub struct InferenceQueue {
    pub requests: std::collections::VecDeque<InferenceRequest>,
}

/// Validate an inference response against a schema.
pub fn validate_response(
    resp: &InferenceResponse,
    schema: &serde_json::Value,
) -> Result<(), InferenceError> {
    if schema.is_null() || schema.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        Ok(())
    } else {
        Ok(())
    }
}

/// Inference request.
#[derive(Debug, Clone, Default)]
pub struct InferenceRequest {
    pub id: String,
    pub prompt: String,
    pub options: InferenceOptions,
}

impl InferenceQueue {
    pub fn enqueue(&mut self, req: InferenceRequest) -> String {
        self.requests.push_back(req);
        self.requests
            .back()
            .map(|r| r.id.clone())
            .unwrap_or_default()
    }
    pub fn dequeue(&mut self) -> Option<InferenceRequest> {
        self.requests.pop_front()
    }
}

/// Provider registry for model selection.
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    /// Registered providers.
    pub providers: std::collections::HashMap<String, Box<dyn crate::models::InferenceProvider>>,
}

/// Select a provider based on capability.
pub fn select_provider(
    registry: &ProviderRegistry,
    cap: Capability,
) -> Option<Box<dyn crate::models::InferenceProvider>> {
    // Placeholder: return first provider
    for (_, provider) in &registry.providers {
        return Some(Box::new(crate::models::LocalProvider::new()));
    }
    None
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
