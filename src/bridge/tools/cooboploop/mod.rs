// src/bridge/tools/cooboploop/mod.rs
//! CoObOpLoop MCP tools - continuous objective-observation-operation loop
//!
//! This module has been split into sub-modules for maintainability:
//! - `inputs` — tool input structs and their Debug impls
//! - `definitions` — tool name constants and the `all()` catalog
//! - `execute` — async handler functions for each tool

pub mod definitions;
pub mod execute;
pub mod inputs;

// Re-export all public functions and types for backward compatibility
pub use execute::*;
pub use inputs::*;
