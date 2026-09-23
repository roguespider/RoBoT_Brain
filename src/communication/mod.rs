//! Communication module - Per Architecture §15 "Agent Communication Architecture"

pub mod acp;
pub mod events;
pub mod mcp;

/// Active reference to communication contracts.
pub fn reference_communication_contracts() {
    crate::communication::mcp::reference_mcp_contracts();
    crate::communication::acp::reference_acp_contracts();
    crate::communication::events::reference_events_contracts();
}
