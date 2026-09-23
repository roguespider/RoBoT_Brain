/// ContextPacket data contract - Per Architecture Chapter 7.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A packet of contextual information for reasoning.
/// Contains session info, observations, and retrieved context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// Shared metadata (version, source, timestamp, correlation, confidence).
    pub metadata: Metadata,
    /// Session identifier for correlating related packets.
    pub session_id: String,
    /// Observations relevant to the current context.
    pub observations: Vec<String>,
    /// A summary of the current conversation state.
    pub summary: String,
}

impl ContextPacket {
    pub fn new(session_id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("context_packet"),
            session_id: session_id.into(),
            observations: Vec::new(),
            summary: summary.into(),
        }
    }
}
