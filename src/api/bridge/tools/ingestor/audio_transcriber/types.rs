// src/api/bridge/tools/ingestor/audio_transcriber/types.rs
//! Audio transcription types and file detection utilities

use std::path::Path;

/// Get supported audio extensions
pub fn get_supported_extensions() -> &'static [&'static str] {
    &["mp3", "wav", "m4a", "flac", "ogg", "aac", "wma", "opus"]
}

/// Check if a file is a supported audio format
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| get_supported_extensions().contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Transcription result
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_seconds: f32,
    pub segments: Vec<TranscriptionSegment>,
}

/// A segment of the transcription
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: f32,
    pub end: f32,
}

/// Audio analysis metrics extracted from the file
#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub rms_db: f32,
    pub peak_db: f32,
    pub speech_estimate_percent: f32,
    pub silence_percent: f32,
    pub zero_crossings_per_second: f32,
}

impl AudioAnalysis {
    /// Create analysis from audio samples
    pub fn from_samples(samples: &[f32], sample_rate: u32, duration: f32) -> Self {
        let rms = samples.iter().map(|&s| s * s).sum::<f32>() / samples.len().max(1) as f32;
        let rms_db = 20.0 * (rms.sqrt() + 1e-10).log10();

        let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        let peak_db = 20.0 * (peak + 1e-10).log10();

        let frame_size = (sample_rate as f32 * 0.025) as usize;
        let speech_frames = samples
            .chunks(frame_size)
            .filter(|frame| {
                let frame_rms =
                    frame.iter().map(|&s| s * s).sum::<f32>() / frame.len().max(1) as f32;
                frame_rms.sqrt() > 0.02
            })
            .count();
        let speech_estimate_percent =
            (speech_frames as f32 / samples.chunks(frame_size).count().max(1) as f32) * 100.0;

        let silence_frames = samples
            .chunks(frame_size)
            .filter(|frame| {
                let frame_rms =
                    frame.iter().map(|&s| s * s).sum::<f32>() / frame.len().max(1) as f32;
                frame_rms.sqrt() < 0.005
            })
            .count();
        let silence_percent =
            (silence_frames as f32 / samples.chunks(frame_size).count().max(1) as f32) * 100.0;

        let zero_crossings = samples
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();
        let zero_crossings_per_second = zero_crossings as f32 / duration.max(1.0);

        Self {
            duration_seconds: duration,
            sample_rate,
            channels: 1,
            rms_db,
            peak_db,
            speech_estimate_percent,
            silence_percent,
            zero_crossings_per_second,
        }
    }
}
