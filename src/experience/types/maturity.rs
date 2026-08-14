// src/experience/types/maturity.rs
// Maturity and importance types

use serde::{Deserialize, Serialize};

/// Knowledge maturity levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeMaturity {
    /// Newly discovered.
    Emerging,

    /// Some supporting evidence exists.
    Developing,

    /// Repeatedly confirmed.
    Established,

    /// Highly trusted over time.
    Trusted,

    /// Confidence is decreasing.
    Questioned,

    /// Replaced by better information.
    Deprecated,

    /// Proven incorrect.
    Rejected,
}
