// src/tools/memory/mod.rs
// Memory-related MCP tools
// Per Architecture §07: Every experience originates from observations
//
// This module is split into focused submodules:
//   - `types`:       input structs for every memory/embedding tool
//   - `definitions`: tool-name constants + `tools/list` JSON schemas
//   - `helpers`:     internal type-conversion helpers
//   - `handlers`:    memory CRUD tool handlers (store/search/get/list/...)
//   - `embedding`:   vector-index tool handlers
//
// Public items are re-exported here so external callers can keep using
// `memory::StoreMemoryInput`, `memory::execute_store_memory`,
// `memory::definitions::all()`, etc.

pub mod definitions;
mod embedding;
mod handlers;
mod helpers;
mod types;

pub use embedding::*;
pub use handlers::*;
pub use types::*;
