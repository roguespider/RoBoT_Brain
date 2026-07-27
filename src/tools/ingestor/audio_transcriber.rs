// src/tools/ingestor/audio_transcriber.rs
//! Audio transcription using Whisper
//! 
//! This module provides audio transcription capabilities using whisper-rs,
//! a Rust binding for OpenAI's Whisper model.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::database::models::MemoryCard;
use crate::database::sqlite::SqliteDatabase;
use crate::memory::pipeline::MemoryPipeline;
use crate::memory::types::MemoryItem;
use crate::memory::WorkingMemory;

/// Model size to download - options: tiny, tiny.en, base, base.en, small, small.en, 
/// medium, medium.en, large-v1, large-v2, large
const DEFAULT_MODEL_SIZE: &str = "base.en";

/// Global whisper context (lazily initialized)
static WHISPER_CONTEXT: std::sync::OnceLock<Arc<WhisperContext>> = std::sync::OnceLock::new();

/// Initialize whisper model - downloads if not cached
pub fn init_whisper_model() -> Result<&'static WhisperContext> {
    WHISPER_CONTEXT
        .get_or_try_init(|| {
            tracing::info!("Initializing Whisper model: {}", DEFAULT_MODEL_SIZE);
            
            let model_path = get_model_path();
            
            // Download model if not exists
            if !model_path.exists() {
                tracing::info!("Downloading Whisper model: {}", DEFAULT_MODEL_SIZE);
                download_model(DEFAULT_MODEL_SIZE, &model_path)?;
            }
            
            // Load the model
            let ctx = WhisperContext::new(&model_path)
                .context("Failed to load Whisper model")?;
            
            tracing::info!("Whisper model loaded successfully");
            Ok(Arc::new(ctx))
        })
        .map_err(|e| anyhow::anyhow!("Failed to initialize Whisper: {}", e))
}

/// Get the path where Whisper models are stored
fn get_model_path() -> std::path::PathBuf {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    
    cache_dir.join("whisper-rs").join(format!("{}.bin", DEFAULT_MODEL_SIZE))
}

/// Download a Whisper model
fn download_model(model_name: &str, dest_path: &Path) -> Result<()> {
    // Create parent directory
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}.bin",
        model_name
    );
    
    tracing::info!("Downloading from: {}", url);
    
    let response = ureq::get(&url)
        .call()
        .context("Failed to download model")?;
    
    let mut file = std::fs::File::create(dest_path)?;
    
    // Copy response body to file
    std::io::copy(&mut response.into_reader(), &mut file)?;
    
    tracing::info!("Model downloaded to: {:?}", dest_path);
    Ok(())
}

/// Check if whisper model is available
pub fn is_model_available() -> bool {
    get_model_path().exists() || WHISPER_CONTEXT.get().is_some()
}

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

/// Transcribe an audio file to text
pub fn transcribe_audio(path: &Path) -> Result<TranscriptionResult> {
    let ctx = init_whisper_model()?;
    
    let audio_path = path.to_string_lossy();
    tracing::info!("Transcribing audio file: {}", audio_path);
    
    // Load audio file and convert to 16kHz mono PCM
    let samples = load_audio_file(path)?;
    
    let duration_seconds = samples.len() as f32 / 16000.0;
    tracing::info!("Audio duration: {:.1}s", duration_seconds);
    
    // Create transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_language(Some("en"));
    params.set_translate_task(false);
    params.set_n_threads(4);
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Run transcription
    let mut state = ctx.create_state()?;
    state
        .full(params, &samples)
        .context("Transcription failed")?;
    
    // Extract results
    let num_segments = state
        .len()
        .context("Failed to get segment count")?;
    
    let mut full_text = String::new();
    let mut segments = Vec::new();
    
    for i in 0..num_segments {
        let segment = state
            .get_segment(i)
            .context("Failed to get segment")?;
        
        let text = segment.get_text().to_string();
        let start = segment.get_timestamp().0 as f32 / 100.0;
        let end = segment.get_timestamp().1 as f32 / 100.0;
        
        segments.push(TranscriptionSegment {
            text: text.clone(),
            start,
            end,
        });
        
        if !full_text.is_empty() {
            full_text.push(' ');
        }
        full_text.push_str(&text);
    }
    
    tracing::info!(
        "Transcription complete: {} segments, {} chars",
        segments.len(),
        full_text.len()
    );
    
    Ok(TranscriptionResult {
        text: full_text,
        language: Some("en".to_string()),
        duration_seconds,
        segments,
    })
}

/// Load audio file and convert to 16kHz mono PCM samples
fn load_audio_file(path: &Path) -> Result<Vec<f32>> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "wav" => load_wav(path),
        "mp3" | "m4a" | "aac" | "ogg" | "flac" | "opus" | "wma" => {
            // For compressed formats, we'd need additional libraries
            // For now, fall back to a placeholder or error
            anyhow::bail!(
                "Compressed audio format '{}' not directly supported. \
                Please convert to WAV format first, or use a pre-converted file.",
                extension
            )
        }
        _ => anyhow::bail!("Unsupported audio format: {}", extension),
    }
}

/// Load WAV file and convert to 16kHz mono PCM samples
fn load_wav(path: &Path) -> Result<Vec<f32>> {
    use hound::{WavReader, WavSpec, SampleFormat};
    
    let reader = WavReader::open(path)?;
    let spec = reader.spec();
    
    tracing::info!(
        "WAV: {} channels, {} Hz, {} bits per sample",
        spec.channels,
        spec.sample_rate,
        spec.bits_per_sample
    );
    
    // Convert to mono 16kHz if needed
    let samples: Vec<f32> = if spec.channels == 1 && spec.sample_rate == 16000 {
        match spec.sample_format {
            SampleFormat::Int => reader
                .into_samples::<i16>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / 32768.0)
                .collect(),
            SampleFormat::Float => reader
                .into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect(),
        }
    } else {
        // Resample and mix to mono 16kHz
        resample_audio(reader, spec)?
    };
    
    Ok(samples)
}

/// Resample audio to 16kHz mono
fn resample_audio(
    reader: hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: hound::WavSpec,
) -> Result<Vec<f32>> {
    let target_rate = 16000u32;
    
    // Calculate resampling ratio
    let ratio = target_rate as f64 / spec.sample_rate as f64;
    
    // Read all samples
    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Int => reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect(),
        SampleFormat::Float => reader
            .into_samples::<f32>()
            .filter_map(|s| s.ok())
            .collect(),
    };
    
    // Mix channels to mono
    let mono: Vec<f32> = if spec.channels > 1 {
        samples
            .chunks(spec.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / spec.channels as f32)
            .collect()
    } else {
        samples
    };
    
    // Resample
    let new_len = ((mono.len() as f64) * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;
        
        if src_idx + 1 < mono.len() {
            // Linear interpolation
            let sample = mono[src_idx] as f64 * (1.0 - frac)
                + mono[src_idx + 1] as f64 * frac;
            resampled.push(sample as f32);
        } else if src_idx < mono.len() {
            resampled.push(mono[src_idx]);
        }
    }
    
    Ok(resampled)
}

/// Format transcription result as memory content
pub fn format_transcription_as_memory(
    result: &TranscriptionResult,
    filename: &str,
    source_path: &str,
) -> String {
    let mut content = format!(
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
         --------\n",
        filename,
        source_path,
        result.duration_seconds,
        result.language.as_deref().unwrap_or("unknown"),
        result.text
    );
    
    // Add segment timestamps
    for (i, segment) in result.segments.iter().enumerate() {
        content.push_str(&format!(
            "[{:.1}s - {:.1}s] {}\n",
            segment.start, segment.end, segment.text
        ));
    }
    
    content
}

/// Store transcribed audio as memory
pub async fn store_transcription_as_memory(
    transcription: &TranscriptionResult,
    filename: &str,
    source_path: &str,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<Vec<String>> {
    let content = format_transcription_as_memory(transcription, filename, source_path);
    
    // Store as memory card
    let mut memory = MemoryCard::new(content, crate::database::models::MemoryType::File);
    memory.file_source = Some(source_path.to_string());
    
    // Store via pipeline (SQLite persistence)
    let pipeline = MemoryPipeline::new(db.clone());
    pipeline.store_working(&memory)?;
    
    // Store in working memory cache
    let memory_item = MemoryItem::from(&memory);
    working_memory.store(memory_item).await;
    
    Ok(vec![memory.id.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("test.mp3")));
        assert!(is_audio_file(Path::new("test.WAV")));
        assert!(is_audio_file(Path::new("test.m4a")));
        assert!(!is_audio_file(Path::new("test.txt")));
        assert!(!is_audio_file(Path::new("test.mp4")));
    }

    #[test]
    fn test_get_supported_extensions() {
        let exts = get_supported_extensions();
        assert!(exts.contains(&"mp3"));
        assert!(exts.contains(&"wav"));
        assert!(exts.contains(&"m4a"));
    }
}
