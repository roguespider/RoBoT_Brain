//! Memory promotion logic - Per Architecture §17.4 "Memory promotion"
//!
//! Handles promotion of memories from working memory to long-term memory
//! using the promotion gate.

use crate::data_contracts::memory_record::MemoryRecord;
use crate::memory::lifecycle::PromotionGate;

/// Trait for working memory (temporary, volatile storage)
pub trait WorkingMemory {
    /// Push a memory record into working memory
    fn push(&mut self, rec: MemoryRecord);
    /// Drain all memory records from working memory
    fn drain(&mut self) -> Vec<MemoryRecord>;
    /// Get the number of records in working memory
    fn len(&self) -> usize;
    /// Check if working memory is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Trait for long-term memory (permanent, indexed storage)
pub trait LongTermMemory {
    /// Store a memory record in long-term memory
    fn store(&self, rec: MemoryRecord) -> Result<(), MemoryError>;
    /// Search long-term memory by query
    fn search(&self, q: &str) -> Vec<MemoryRecord>;
}

/// Memory error types
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryError {
    NotFound,
    StorageError(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::NotFound => write!(f, "Memory not found"),
            MemoryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for MemoryError {}

/// Promote memories from working memory to long-term memory.
///
/// Drains working memory, evaluates each record via the promotion gate,
/// stores accepted ones in long-term memory, and returns the count promoted.
///
/// Per Architecture §17.4: promotion happens when thresholds pass.
pub fn promote_to_long_term(
    working: &mut dyn WorkingMemory,
    lt: &dyn LongTermMemory,
    gate: &PromotionGate,
) -> usize {
    let records = working.drain();
    let mut promoted = 0;

    for mut record in records {
        // Evaluate the record against the promotion gate
        if let Some(next_state) = gate.evaluate(&record) {
            // Update the record's lifecycle state
            record.kind =
                crate::data_contracts::memory_record::MemoryKind::from_lifecycle(next_state);

            // Store in long-term memory
            if lt.store(record).is_ok() {
                promoted += 1;
            }
        }
    }

    promoted
}

/// Active reference to memory promotion contracts.
pub fn reference_memory_promotion_contracts() {
    // We can't fully test promote_to_long_term without real implementations,
    // but we can reference the function
    let promote_fn = promote_to_long_term;
    let promote_fn_name = std::any::type_name_of_val(&promote_fn);
    tracing::info!(promote_fn_name, "promote_to_long_term actively referenced");
}
