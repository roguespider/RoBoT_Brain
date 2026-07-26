// src/bridge/rmcp/mod.rs
// RMCP module - contains handler and tool definitions

pub mod types;
pub mod helpers;
pub mod handler;
pub mod generated;

pub use handler::run_stdio_server;
