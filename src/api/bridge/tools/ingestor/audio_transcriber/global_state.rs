// src/api/bridge/tools/ingestor/audio_transcriber/global_state.rs
//! Global transcriber singleton and model caching

use anyhow::Result;

use super::core::WhisperTranscriber;

/// Global transcriber instance (using Mutex for interior mutability)
static TRANSCRIBER: std::sync::OnceLock<std::sync::Mutex<WhisperTranscriber>> =
    std::sync::OnceLock::new();

/// Check if model is cached and ready for use (non-blocking)
pub fn is_model_cached() -> bool {
    // Check standard HuggingFace cache locations
    let possible_paths = [
        std::env::var("HF_HUB_CACHE").ok(),
        Some(format!(
            "{}/.cache/huggingface/hub",
            std::env::var("HOME").unwrap_or_default()
        )),
    ];

    for cache_dir in possible_paths.into_iter().flatten() {
        let model_path = std::path::Path::new(&cache_dir).join("models--openai--whisper-base");
        if model_path.join("models/model.safetensors").exists()
            && model_path.join("config.json").exists()
        {
            return true;
        }
    }
    false
}

pub fn get_transcriber() -> Result<std::sync::MutexGuard<'static, WhisperTranscriber>> {
    // Initialize the transcriber if not already done
    if TRANSCRIBER.get().is_none() {
        let transcriber = if is_model_cached() {
            // Model is cached, try to load it
            match WhisperTranscriber::new() {
                Ok(t) => {
                    tracing::info!("Whisper model loaded successfully");
                    std::sync::Mutex::new(t)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load cached Whisper model: {}, using audio analysis only",
                        e
                    );
                    std::sync::Mutex::new(WhisperTranscriber::default())
                }
            }
        } else {
            // Model not cached, skip download (would block on network)
            tracing::info!(
                "Whisper model not cached - audio analysis mode only (set HF_HUB_CACHE or download model manually for full transcription)"
            );
            std::sync::Mutex::new(WhisperTranscriber::default())
        };

        if TRANSCRIBER.set(transcriber).is_err() {
            tracing::debug!("Transcriber was initialized by another thread");
        }
    }

    // Now safely get the transcriber (it must exist after initialization)
    match TRANSCRIBER.get() {
        Some(transcriber) => transcriber
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e)),
        None => {
            // This shouldn't happen since we initialized above, but handle gracefully
            Err(anyhow::anyhow!("Transcriber not initialized"))
        }
    }
}
