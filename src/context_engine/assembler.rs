//! Context assembler — Per Architecture §07 (Context Lifecycle).
//! Assembles context from raw inputs into structured form.

/// Context assembler.
#[derive(Default)]
pub struct ContextAssembler;

impl ContextAssembler {
    /// Create a new context assembler.
    pub fn new() -> Self {
        Self
    }

    /// Assemble a context from a request.
    pub fn assemble(
        &self,
        request: &crate::context_engine::coordinator::ContextRequest,
    ) -> crate::context_engine::coordinator::Context {
        crate::context_engine::coordinator::Context {
            id: request.id.clone(),
            content: request.input.clone(),
            metadata: serde_json::json!({
                "priority": request.priority,
                "assembled_at": chrono::Utc::now().timestamp(),
            }),
            complete: false,
            lifecycle_state: crate::context_engine::lifecycle::ContextLifecycleState::Assembling,
        }
    }
}
