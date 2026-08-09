// src/bridge/app/mod.rs
//! Application module - Root application container per Architecture §03

pub mod acp;
pub mod initialization;
pub mod personality;
pub mod scheduler;
pub mod state;

// Re-export the App struct
pub use state::App;

// Re-export personality methods
pub use personality::{
    adapt_personality, apply_personality_preset, get_communication_style, get_personality_preset,
    get_personality_success_rate, get_personality_timeout, get_personality_traits, list_personality_presets,
    personality, set_personality_traits, should_explore, should_take_risk, should_use_creativity,
};

// Re-export ACP methods
pub use acp::{
    acp_agent_count, acp_registry, acp_router, list_acp_agents, route_acp_message,
};
