// src/bridge/app/personality.rs
//! Personality-related methods for the App

use std::sync::Arc;

use crate::personality::{Personality, PersonalityTraits};

use super::state::App;

// =========================================================================
// Personality Methods (designed for future use)
// =========================================================================
/// Get reference to personality system
pub fn personality(app: &App) -> Arc<std::sync::Mutex<Personality>> {
    app.personality.clone()
}

/// Get current personality traits
pub fn get_personality_traits(app: &App) -> PersonalityTraits {
    match app.personality.lock() {
        Ok(guard) => guard.get_traits().clone(),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, recovering");
            poisoned.into_inner().get_traits().clone()
        }
    }
}

/// Set personality traits
pub fn set_personality_traits(app: &App, traits: PersonalityTraits) {
    if let Err(poisoned) = app.personality.lock() {
        tracing::error!("Personality mutex poisoned during set_traits, recovering");
        poisoned.into_inner().set_traits(traits);
    } else {
        // Lock succeeded and will be released when scope ends
    }
}

/// Apply a personality preset (balanced, analytical, creative, cautious, bold)
pub fn apply_personality_preset(app: &App, preset: &str) -> bool {
    match app.personality.lock() {
        Ok(mut guard) => guard.apply_preset(preset),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned during apply_preset, recovering");
            poisoned.into_inner().apply_preset(preset)
        }
    }
}

/// Get available personality presets
pub fn list_personality_presets(app: &App) -> Vec<String> {
    match app.personality.lock() {
        Ok(guard) => guard.list_presets(),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned during list_presets, recovering");
            poisoned.into_inner().list_presets()
        }
    }
}

/// Get current personality preset name
pub fn get_personality_preset(app: &App) -> String {
    match app.personality.lock() {
        Ok(guard) => guard.get_current_preset().to_string(),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned during get_current_preset, recovering");
            poisoned.into_inner().get_current_preset().to_string()
        }
    }
}

/// Adapt personality based on experience outcome
pub fn adapt_personality(app: &App, success: bool, risk_taken: bool) {
    if let Err(poisoned) = app.personality.lock() {
        tracing::error!("Personality mutex poisoned during adapt, recovering");
        poisoned.into_inner().adapt_from_experience(success, risk_taken);
    }
}

/// Get communication style based on personality verbosity
pub fn get_communication_style(app: &App) -> crate::personality::CommunicationStyle {
    match app.personality.lock() {
        Ok(guard) => guard.get_communication_style(),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, returning default communication style");
            poisoned.into_inner().get_communication_style()
        }
    }
}

/// Decide if system should explore new approaches
pub fn should_explore(app: &App, confidence: f32) -> bool {
    match app.personality.lock() {
        Ok(guard) => guard.should_explore(confidence),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, defaulting to exploration");
            poisoned.into_inner().should_explore(confidence)
        }
    }
}

/// Decide if system should take a risk
pub fn should_take_risk(app: &App, potential_gain: f32, potential_loss: f32) -> bool {
    match app.personality.lock() {
        Ok(guard) => guard.should_take_risk(potential_gain, potential_loss),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, defaulting risk assessment");
            poisoned.into_inner().should_take_risk(potential_gain, potential_loss)
        }
    }
}

/// Decide if a creative approach should be used for planning.
/// Uses personality creativity trait combined with problem complexity
/// to determine whether to explore unconventional solutions.
pub fn should_use_creativity(app: &App, problem_complexity: f32) -> bool {
    match app.personality.lock() {
        Ok(guard) => guard.should_use_creativity(problem_complexity),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, defaulting creativity");
            poisoned.into_inner().should_use_creativity(problem_complexity)
        }
    }
}

/// Get patience-based timeout
pub fn get_personality_timeout(app: &App, base_timeout_secs: u64) -> u64 {
    match app.personality.lock() {
        Ok(guard) => guard.get_timeout(base_timeout_secs),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, returning base timeout");
            poisoned.into_inner().get_timeout(base_timeout_secs)
        }
    }
}

/// Get personality success rate
pub fn get_personality_success_rate(app: &App) -> f32 {
    match app.personality.lock() {
        Ok(guard) => guard.success_rate(),
        Err(poisoned) => {
            tracing::error!("Personality mutex poisoned, returning 0.0 success rate");
            poisoned.into_inner().success_rate()
        }
    }
}
