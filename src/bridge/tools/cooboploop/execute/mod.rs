// src/bridge/tools/cooboploop/execute/mod.rs
//! CoObOpLoop tool execution functions

pub mod autonomy;
pub mod evaluation;
pub mod goals;
pub mod hardware_inspection;
pub mod idle_research;
pub mod loop_control;

// Re-export all public functions for backward compatibility
pub use autonomy::*;
pub use evaluation::*;
pub use goals::*;
pub use hardware_inspection::*;
pub use idle_research::*;
pub use loop_control::*;

/// Register all cooboploop tools
pub fn register_tools(registry: &mut crate::bridge::tools::ToolRegistry) {
    let cooboploop_tools = super::definitions::all();
    registry.tools.extend(cooboploop_tools);
}

pub fn register_cooboploop_tools(registry: &mut crate::bridge::tools::ToolRegistry) {
    register_tools(registry);
}
