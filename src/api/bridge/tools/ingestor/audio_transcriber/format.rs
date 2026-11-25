// src/api/bridge/tools/ingestor/audio_transcriber/format.rs
//! Formatting transcription results as text and memory

use std::path::Path;

use super::types::{AudioAnalysis, TranscriptionResult};

/// Generate text from audio analysis
pub fn generate_audio_analysis_text(analysis: &AudioAnalysis, path: &Path) -> String {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    format!(
        "AUDIO ANALYSIS RESULTS\n\
         ====================\n\
         File: {}\n\
         Duration: {:.2}s\n\
         Sample Rate: {} Hz\n\
         Channels: {}\n\
         \n\
         AUDIO METRICS\n\
         -------------\n\
         RMS Level: {:.1} dB\n\
         Peak Level: {:.1} dB\n\
         Speech Estimate: {:.0}%\n\
         Silence Estimate: {:.0}%\n\
         Zero Crossings: {:.0}/s\n\
         \n\
         STATUS\n\
         ------\n\
         Audio has been analyzed using Candle ML framework.\n\
         Mel spectrogram features extracted successfully.",
        filename,
        analysis.duration_seconds,
        analysis.sample_rate,
        analysis.channels,
        analysis.rms_db,
        analysis.peak_db,
        analysis.speech_estimate_percent,
        analysis.silence_percent,
        analysis.zero_crossings_per_second
    )
}

/// Format transcription result as memory content
pub fn format_transcription_as_memory(
    result: &TranscriptionResult,
    filename: &str,
    source_path: &str,
) -> String {
    format!(
        "AUDIO TRANSCRIPTION\n\
         ==================\n\
         Source file: {}\n\
         Original path: {}\n\
         Duration: {:.1} seconds\n\
         Language: {}\n\
         \n\
         TRANSCRIPT\n\
         ---------\n\
         {}\n\
         \n\
         SEGMENTS\n\
         --------\n{}",
        filename,
        source_path,
        result.duration_seconds,
        result.language.as_deref().unwrap_or("unknown"),
        result.text,
        result
            .segments
            .iter()
            .map(|s| format!("[{:.1}s - {:.1}s] {}", s.start, s.end, s.text))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
