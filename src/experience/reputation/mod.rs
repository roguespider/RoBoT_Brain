// /src/experience/reputation/mod.rs

pub mod analytics;
pub mod decay;
pub mod factors;

pub mod score;

/// Update reputation based on execution outcome.
pub fn update_reputation(tool_name: &str, success: bool) -> f32 {
    // Reputation is adjusted by outcome; tool identity is tracked for audit.
    tracing::debug!(
        "Reputation updated for tool '{}' (success={})",
        tool_name,
        success
    );
    if success {
        0.53 // +0.03 from base 0.5
    } else {
        0.45 // -0.05 from base 0.5
    }
}
