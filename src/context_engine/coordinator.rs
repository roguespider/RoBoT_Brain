//! Context engine coordinator — Per Architecture §07 (Context Lifecycle).
//! Coordinates context assembly, retrieval, and lifecycle management.

use crate::context_engine::assembler::ContextAssembler;
use crate::context_engine::lifecycle::ContextLifecycle;
use crate::context_engine::retrieval::ContextRetrieval;
use serde_json::Value;

/// Context engine — orchestrates context assembly, retrieval, and lifecycle.
/// Per Architecture §07: context engine assembles context for agent consumption.
pub struct ContextEngine {
    assembler: ContextAssembler,
    lifecycle: ContextLifecycle,
    retrieval: ContextRetrieval,
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEngine {
    /// Create a new context engine.
    pub fn new() -> Self {
        Self {
            assembler: ContextAssembler::new(),
            lifecycle: ContextLifecycle::new(),
            retrieval: ContextRetrieval::new(),
        }
    }

    /// Assemble context for a given request.
    /// Per Architecture §07: assemble_context produces a Context from inputs.
    pub fn assemble_context(&self, request: &ContextRequest) -> Context {
        let assembled = self.assembler.assemble(request);
        let retrieved = self.retrieval.retrieve(&assembled);
        self.lifecycle.finalize(retrieved)
    }
}

/// Context assembly request.
#[derive(Debug, Clone)]
pub struct ContextRequest {
    /// Context identifier.
    pub id: String,
    /// Input data for context assembly.
    pub input: Value,
    /// Context priority.
    pub priority: u8,
}

impl ContextRequest {
    /// Create a new context request.
    pub fn new(id: String, input: Value) -> Self {
        Self {
            id,
            input,
            priority: 5,
        }
    }
}

/// Assembled context result.
#[derive(Debug, Clone)]
pub struct Context {
    /// Context identifier.
    pub id: String,
    /// Context content.
    pub content: Value,
    /// Context metadata.
    pub metadata: Value,
    /// Whether context assembly is complete.
    pub complete: bool,
    /// Current lifecycle state.
    pub lifecycle_state: crate::context_engine::lifecycle::ContextLifecycleState,
}
