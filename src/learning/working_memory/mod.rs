
// src/learning/working_memory.rs

//! Learning system's Working Memory with state machine for active context
//!
//! This is a DIFFERENT concept from `src/memory/working.rs`:
//! - Memory Working Memory: Stores MemoryItem objects for retrieval (per §6.3)
//! - Learning Working Memory: Tracks active context with state machine transitions
//!
//! The Learning Working Memory is used for:
//! - Active context tracking during task execution
//! - State machine transitions (Active → Evaluated → Promoted → Archived)
//! - Promotion policies for knowledge extraction
//! - Lineage tracking for memory provenance

mod store;

