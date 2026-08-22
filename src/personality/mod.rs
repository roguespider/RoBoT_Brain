// src/personality/mod.rs

//! Personality System
//!
//! Per Architecture: Defines the behavioral characteristics of the AI system,
//! influencing decision-making, communication style, and learning preferences.
//!
//! # Usage
//!
//! ```rust
//! use robot_brain::personality::{Personality, CommunicationStyle};
//!
//! let mut personality = Personality::new();
//! // Apply preset with proper error handling
//! let applied = personality.apply_preset("analytical");
//! if !applied {
//!     eprintln!("Warning: preset 'analytical' not found");
//! }
//!
//! // Use personality traits for decisions
//! if personality.should_explore(0.5) {
//!     // Explore new approaches
//! }
//!
//! // Get communication style
//! let style = personality.get_communication_style();
//! match style {
//!     CommunicationStyle::Concise => println!("Keep it brief"),
//!     CommunicationStyle::Balanced => println!("Medium detail"),
//!     CommunicationStyle::Detailed => println!("Full explanation"),
//! }
//! ```

pub mod adaptation;
pub mod communication;
pub mod core;
pub mod decision;
pub mod decision_making;
pub mod emotional;
pub mod presets;
pub mod traits;

pub use communication::CommunicationStyle;
pub use core::Personality;
pub use decision::{Decision, DecisionApproach, DecisionContext};
pub use traits::PersonalityTraits;
