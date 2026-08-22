// src/bridge/app/initialization/personality_diagnostics.rs
//! Personality subsystem self-check (explicit diagnostics, P2-001C).

use crate::bridge::app::state::App;

/// Exercise the personality decision surface and report results.
pub fn run_personality_self_check(app: &App) {
    use crate::bridge::app::{
        adapt_personality, apply_personality_preset, get_communication_style,
        get_personality_preset, get_personality_success_rate, get_personality_timeout,
        get_personality_traits, list_personality_presets, personality, set_personality_traits,
        should_explore, should_take_risk, should_use_creativity,
    };

    let preset = get_personality_preset(app);
    let traits = get_personality_traits(app);
    let success_rate = get_personality_success_rate(app);
    tracing::info!(
        "Personality subsystem online: preset='{}' curiosity={:.2} creativity={:.2} caution={:.2} success_rate={:.2}",
        preset,
        traits.curiosity,
        traits.creativity,
        traits.caution,
        success_rate
    );
    let presets = list_personality_presets(app);
    tracing::info!("Available personality presets: {:?}", presets);
    let comm_style = get_communication_style(app);
    tracing::info!("Communication style: {:?}", comm_style);

    // Exercise personality decision functions
    let explore = should_explore(app, 0.5);
    let risk = should_take_risk(app, 0.7, 0.3);
    let creativity = should_use_creativity(app, 0.5);
    let timeout = get_personality_timeout(app, 30);
    tracing::info!(
        "Personality decisions: explore={} risk={} creativity={} timeout={}s",
        explore,
        risk,
        creativity,
        timeout
    );

    // Re-apply current preset to verify the personality system is functional
    let personality_arc = personality(app);
    tracing::info!(
        "Personality system reference acquired: {} strong references",
        std::sync::Arc::strong_count(&personality_arc)
    );
    let preset_ok = apply_personality_preset(app, &preset);
    if preset_ok {
        tracing::info!("Personality preset '{}' re-applied successfully", preset);
    }
    let current_traits = get_personality_traits(app);
    set_personality_traits(app, current_traits.clone());
    adapt_personality(app, true, false);
    tracing::info!("Personality self-check complete: traits re-set and adaptation exercised");
}
