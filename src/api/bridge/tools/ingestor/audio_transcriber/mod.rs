// src/api/bridge/tools/ingestor/audio_transcriber/mod.rs
//! Audio transcription using Candle Whisper
//!
//! ## Architecture (Architecture §17 - MCP)
//!
//! The transcribe_audio tool follows the standard MCP tool pattern:
//! 1. Load audio file (WAV format at 16kHz mono)
//! 2. Convert to mel spectrogram features
//! 3. Process through Whisper transformer model
//! 4. Return structured transcription result

use std::path::Path;

use anyhow::Result;

pub mod audio;
pub mod core;
pub mod decode;
pub mod format;
pub mod global_state;
pub mod memory;
pub mod types;
pub mod wav;

pub use core::WhisperTranscriber;
pub use format::{format_transcription_as_memory, generate_audio_analysis_text};
pub use memory::store_transcription_as_memory;
pub use types::{
    AudioAnalysis, TranscriptionResult, TranscriptionSegment, get_supported_extensions,
    is_audio_file,
};

/// Transcribe an audio file to text using Whisper
pub fn transcribe_audio(path: &Path) -> Result<TranscriptionResult> {
    let samples = audio::load_audio_file(path)?;
    let duration_seconds = samples.len() as f32 / 16000.0;

    tracing::info!(
        "Transcribing audio file: {} (duration: {:.1}s)",
        path.display(),
        duration_seconds
    );

    // Generate audio analysis for context
    let analysis = AudioAnalysis::from_samples(&samples, 16000, duration_seconds);
    let analysis_text = format::generate_audio_analysis_text(&analysis, path);

    // Try to get transcriber and run Whisper transcription
    // If model isn't loaded, gracefully fall back to audio analysis
    let transcription_result = match global_state::get_transcriber() {
        Ok(mut transcriber) => {
            // Check if model is actually loaded before trying to transcribe
            if transcriber.model.is_some() {
                match transcriber.transcribe(&samples) {
                    Ok(result) => {
                        // Successfully transcribed - use the result
                        Some(result)
                    }
                    Err(e) => {
                        tracing::warn!("Whisper transcription failed: {}, using audio analysis", e);
                        None
                    }
                }
            } else {
                tracing::info!("Whisper model not loaded, using audio analysis");
                None
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to get Whisper transcriber: {}, using audio analysis",
                e
            );
            None
        }
    };

    // Return result - either transcription or audio analysis
    match transcription_result {
        Some(result) => Ok(TranscriptionResult {
            text: format!(
                "{}\n\n[Transcribed using Candle Whisper - {:.1}s audio]",
                if result.text.is_empty() {
                    analysis_text
                } else {
                    result.text
                },
                duration_seconds
            ),
            language: result.language,
            duration_seconds: result.duration_seconds,
            segments: result.segments,
        }),
        None => {
            // Whisper model not available - return audio analysis instead
            Ok(TranscriptionResult {
                text: format!(
                    "{}\n\n[Audio analysis only - Whisper model not available - {:.1}s audio]",
                    analysis_text, duration_seconds
                ),
                language: None,
                duration_seconds,
                segments: vec![],
            })
        }
    }
}
