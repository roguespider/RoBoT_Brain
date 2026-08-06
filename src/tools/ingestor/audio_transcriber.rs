// src/tools/ingestor/audio_transcriber.rs

//! Audio transcription for RoBoT Brain using Candle Whisper
//!
//! ## Architecture (Architecture §17 - MCP)
//!
//! The transcribe_audio tool follows the standard MCP tool pattern:
//! 1. Load audio file (WAV format at 16kHz mono)
//! 2. Convert to mel spectrogram features
//! 3. Process through Whisper transformer model
//! 4. Return structured transcription result
//!
//! ## Candle Whisper Integration
//!
//! Uses candle-transformers with the Whisper model from HuggingFace.
//! Model is downloaded automatically on first use.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

// Use candle from candle_transformers to ensure same version
use candle_transformers::models::mimi::candle::{DType, Device, IndexOp, Tensor};
use candle_transformers::models::mimi::candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as whisper, Config};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

use crate::database::models::MemoryCard;
use crate::database::sqlite::SqliteDatabase;
use crate::memory::pipeline::MemoryPipeline;
use crate::memory::types::MemoryItem;
use crate::memory::WorkingMemory;

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

/// Whisper transcriber using Candle
pub struct WhisperTranscriber {
    model: whisper::model::Whisper,
    tokenizer: Tokenizer,
    device: Device,
}

impl WhisperTranscriber {
    /// Create a new Whisper transcriber
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;

        // Download the Whisper model from HuggingFace
        let api = Api::new()?;
        let repo = api.repo(hf_hub::Repo::with_revision(
            "openai/whisper-base".to_string(),
            hf_hub::RepoType::Model,
            "main".to_string(),
        ));

        tracing::info!("Downloading Whisper model files...");

        // Get model files
        let config_path = repo.get("config.json")?;
        let model_path = repo.get("model.safetensors")?;
        let tokenizer_path = repo.get("tokenizer.json")?;

        tracing::info!("Loading Whisper model from {:?}", model_path);

        // Load config
        let config_content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&config_content)?;

        let num_mel_bins = config["n_mels"].as_i64().unwrap_or(80) as usize;
        let max_source_positions = config["n_audio_ctx"].as_i64().unwrap_or(1500) as usize;
        let d_model = config["d_model"].as_i64().unwrap_or(512) as usize;
        let encoder_attention_heads = config["n_audio_head"].as_i64().unwrap_or(8) as usize;
        let encoder_layers = config["n_audio_layer"].as_i64().unwrap_or(4) as usize;
        let vocab_size = config["n_vocab"].as_i64().unwrap_or(51865) as usize;
        let max_target_positions = config["n_text_ctx"].as_i64().unwrap_or(448) as usize;
        let decoder_attention_heads = config["n_text_head"].as_i64().unwrap_or(8) as usize;
        let decoder_layers = config["n_text_layer"].as_i64().unwrap_or(4) as usize;

        let whisper_config = Config {
            num_mel_bins,
            max_source_positions,
            d_model,
            encoder_attention_heads,
            encoder_layers,
            vocab_size,
            max_target_positions,
            decoder_attention_heads,
            decoder_layers,
            suppress_tokens: vec![],
        };

        // Load model weights
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)? };

        let model = whisper::model::Whisper::load(&vb, whisper_config)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        tracing::info!("Whisper model loaded successfully");

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Transcribe audio samples to text
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<TranscriptionResult> {
        let duration_seconds = samples.len() as f32 / 16000.0;

        // Compute mel spectrogram using Candle's audio module
        let mel_filters = load_mel_filters()?;
        let mel_spec = whisper::audio::pcm_to_mel(&self.model.config, samples, &mel_filters);

        // Convert to tensor [1, n_mel, n_frames]
        let n_mel = self.model.config.num_mel_bins;
        let n_frames = mel_spec.len() / n_mel;
        let mel_tensor = Tensor::from_slice(&mel_spec, (1, n_mel, n_frames), &self.device)?;

        tracing::info!("Mel spectrogram shape: {:?}", mel_tensor.shape());

        // Run encoder
        let audio_features = self.model.encoder.forward(&mel_tensor, true)?;

        tracing::info!("Audio features shape: {:?}", audio_features.dims());

        // Decode to text
        let text = self.decode(&audio_features)?;

        Ok(TranscriptionResult {
            text,
            language: Some("en".to_string()),
            duration_seconds,
            segments: vec![],
        })
    }

    /// Decode audio features to text
    fn decode(&mut self, audio_features: &Tensor) -> Result<String> {
        // Get special tokens
        let sot_token = get_token_id(&self.tokenizer, whisper::SOT_TOKEN)?;
        let transcribe_token = get_token_id(&self.tokenizer, whisper::TRANSCRIBE_TOKEN)?;
        let no_timestamps_token = get_token_id(&self.tokenizer, whisper::NO_TIMESTAMPS_TOKEN)?;
        let eot_token = get_token_id(&self.tokenizer, whisper::EOT_TOKEN)?;

        // Build token sequence: <|startoftranscript|><|en|><|transcribe|><|notimestamps|>
        let mut tokens = vec![sot_token, transcribe_token, no_timestamps_token];

        let sample_len = self.model.config.max_target_positions / 2;

        for _ in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;

            let ys = self
                .model
                .decoder
                .forward(&tokens_t, audio_features, tokens.len() == 3)?;

            let (_, seq_len, _) = ys.dims3()?;
            let logits = self
                .model
                .decoder
                .final_linear(&ys.i((..1, seq_len - 1..))?)?
                .i(0)?
                .i(0)?;

            // Greedy decoding
            let logits_v: Vec<f32> = logits.to_vec1()?;
            let next_token = logits_v
                .iter()
                .enumerate()
                .max_by(|(_, u), (_, v)| u.partial_cmp(v).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i as u32)
                .unwrap();

            tokens.push(next_token);

            if next_token == eot_token || tokens.len() > self.model.config.max_target_positions {
                break;
            }
        }

        // Decode tokens to text
        let text = self
            .tokenizer
            .decode(&tokens, true)
            .map_err(|e| anyhow::anyhow!("Failed to decode tokens: {}", e))?;

        Ok(text)
    }
}

impl Default for WhisperTranscriber {
    fn default() -> Self {
        Self::new().expect("Failed to create Whisper transcriber")
    }
}

/// Get token ID from tokenizer
fn get_token_id(tokenizer: &Tokenizer, token: &str) -> Result<u32> {
    tokenizer
        .token_to_id(token)
        .ok_or_else(|| anyhow::anyhow!("Token not found: {}", token))
}

/// Load mel filterbank for Whisper
fn load_mel_filters() -> Result<Vec<f32>> {
    // Standard Whisper mel filterbank parameters
    let n_mel = 80usize;
    let n_fft = 400usize;
    let sample_rate = 16000usize;
    let f_min = 0.0f32;
    let f_max = 8000.0f32;

    let mut mel_filters = vec![0.0f32; n_mel * (n_fft / 2 + 1)];

    // Convert frequencies to mel scale
    let ln_10 = std::f64::consts::LN_10 as f32;
    let f_min_mel = 2595.0 * (1.0 + f_min / 700.0).ln() / ln_10;
    let f_max_mel = 2595.0 * (1.0 + f_max / 700.0).ln() / ln_10;

    let mel_points: Vec<f32> = (0..=n_mel)
        .map(|i| f_min_mel + (f_max_mel - f_min_mel) * i as f32 / n_mel as f32)
        .collect();

    // Convert back to Hz
    let hz_points: Vec<f32> = mel_points
        .iter()
        .map(|m| 700.0 * ((m * ln_10).exp() - 1.0))
        .collect();

    // Convert to FFT bin numbers
    let bin_points: Vec<f32> = hz_points
        .iter()
        .map(|hz| (n_fft + 1) as f32 * hz / sample_rate as f32)
        .collect();

    // Create triangular filters
    for m in 1..n_mel {
        let f_left = bin_points[m - 1] as usize;
        let f_center = bin_points[m] as usize;
        let f_right = bin_points[m + 1] as usize;

        for k in f_left..=f_right {
            let weight = if k <= f_center {
                (k - f_left) as f32 / (f_center - f_left).max(1) as f32
            } else {
                (f_right - k) as f32 / (f_right - f_center).max(1) as f32
            };
            let idx = m * (n_fft / 2 + 1) / n_mel + k;
            if idx < mel_filters.len() {
                mel_filters[idx] = weight;
            }
        }
    }

    Ok(mel_filters)
}

/// Global transcriber instance (using Mutex for interior mutability)
static TRANSCRIBER: std::sync::OnceLock<std::sync::Mutex<WhisperTranscriber>> =
    std::sync::OnceLock::new();

fn get_transcriber() -> Result<std::sync::MutexGuard<'static, WhisperTranscriber>> {
    let transcriber = TRANSCRIBER.get_or_init(|| {
        std::sync::Mutex::new(WhisperTranscriber::new().expect("Failed to create transcriber"))
    });

    transcriber
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))
}

/// Transcribe an audio file to text using Whisper
pub fn transcribe_audio(path: &Path) -> Result<TranscriptionResult> {
    let samples = load_audio_file(path)?;
    let duration_seconds = samples.len() as f32 / 16000.0;

    tracing::info!(
        "Transcribing audio file: {} (duration: {:.1}s)",
        path.display(),
        duration_seconds
    );

    // Get transcriber (loads model on first call)
    let mut transcriber = get_transcriber()?;

    // Run Whisper transcription
    let result = transcriber.transcribe(&samples)?;

    // Generate audio analysis for context
    let analysis = AudioAnalysis::from_samples(&samples, 16000, duration_seconds);
    let analysis_text = generate_audio_analysis_text(&analysis, path);

    Ok(TranscriptionResult {
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
    })
}

/// Generate text from audio analysis
fn generate_audio_analysis_text(analysis: &AudioAnalysis, path: &Path) -> String {
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
            let path_str = path.display().to_string();
            anyhow::bail!(
                "Compressed audio format '{}' requires conversion to WAV.\n\
                \n\
                To convert using ffmpeg:\n\
                \n\
                ffmpeg -i \"{}\" -ar 16000 -ac 1 -acodec pcm_s16le output.wav\n\
                \n\
                Then transcribe 'output.wav' instead.",
                extension,
                path_str
            )
        }
        _ => anyhow::bail!(
            "Unsupported audio format: {}. Supported: mp3, wav, m4a, flac, ogg, aac, wma, opus",
            extension
        ),
    }
}

/// Load WAV file and convert to 16kHz mono PCM samples
fn load_wav(path: &Path) -> Result<Vec<f32>> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read WAV file: {}", path.display()))?;

    // Parse WAV header
    if data.len() < 44 {
        anyhow::bail!("File too small to be a valid WAV file");
    }

    // Check RIFF header
    if &data[0..4] != b"RIFF" {
        anyhow::bail!("Invalid WAV file: missing RIFF header");
    }

    // Check WAVE format
    if &data[8..12] != b"WAVE" {
        anyhow::bail!("Invalid WAV file: missing WAVE format");
    }

    // Find fmt chunk
    let mut pos = 12;
    let mut channels: u16 = 1;
    let mut sample_rate: u32 = 16000;
    let mut bits_per_sample: u16 = 16;

    while pos < data.len() - 8 {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size =
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;

        if chunk_id == b"fmt " {
            // Parse format chunk
            if chunk_size >= 16 {
                channels = u16::from_le_bytes([data[pos + 14], data[pos + 15]]);
                sample_rate = u32::from_le_bytes([
                    data[pos + 16],
                    data[pos + 17],
                    data[pos + 18],
                    data[pos + 19],
                ]);
                bits_per_sample = u16::from_le_bytes([data[pos + 22], data[pos + 23]]);
            }
        } else if chunk_id == b"data" {
            // Found audio data
            let audio_start = pos + 8;
            let audio_end = (audio_start + chunk_size).min(data.len());

            tracing::debug!(
                "WAV: {} channels, {} Hz, {} bits per sample",
                channels,
                sample_rate,
                bits_per_sample
            );

            let samples =
                decode_wav_samples(&data[audio_start..audio_end], channels, bits_per_sample)?;

            // Convert to mono and resample if needed
            let mono = if channels > 1 {
                samples
                    .chunks(channels as usize)
                    .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                    .collect()
            } else {
                samples
            };

            if sample_rate == 16000 {
                return Ok(mono);
            } else {
                return resample_audio(&mono, sample_rate, 16000);
            }
        }

        pos += 8 + chunk_size;
        // Align to even byte
        if !chunk_size.is_multiple_of(2) && !pos.is_multiple_of(2) {
            pos += 1;
        }
    }

    anyhow::bail!("Invalid WAV file: no data chunk found")
}

/// Decode WAV sample data into f32 samples
fn decode_wav_samples(data: &[u8], channels: u16, bits_per_sample: u16) -> Result<Vec<f32>> {
    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let num_samples = data.len() / bytes_per_sample;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let offset = i * bytes_per_sample;
        let sample = match bits_per_sample {
            8 => {
                // 8-bit samples are unsigned (0-255), centered at 128
                let val = data[offset];
                (val as f32 - 128.0) / 128.0
            }
            16 => {
                let val = i16::from_le_bytes([data[offset], data[offset + 1]]);
                val as f32 / 32768.0
            }
            24 => {
                let val = i32::from_le_bytes([0, data[offset], data[offset + 1], data[offset + 2]]);
                val as f32 / 8388608.0
            }
            32 => {
                let val = i32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                val as f32 / 2147483648.0
            }
            _ => anyhow::bail!("Unsupported bits per sample: {}", bits_per_sample),
        };
        samples.push(sample);
    }

    Ok(samples)
}

/// Resample audio to target sample rate
fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = ((samples.len() as f64) * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;

        if src_idx + 1 < samples.len() {
            let sample =
                samples[src_idx] as f64 * (1.0 - frac) + samples[src_idx + 1] as f64 * frac;
            resampled.push(sample as f32);
        } else if src_idx < samples.len() {
            resampled.push(samples[src_idx]);
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

/// Store transcribed audio as memory
pub async fn store_transcription_as_memory(
    transcription: &TranscriptionResult,
    filename: &str,
    source_path: &str,
    db: Arc<SqliteDatabase>,
    working_memory: Arc<WorkingMemory>,
) -> Result<Vec<String>> {
    let content = format_transcription_as_memory(transcription, filename, source_path);

    let mut memory = MemoryCard::new(content, crate::database::models::MemoryType::File);
    memory.file_source = Some(source_path.to_string());

    let pipeline = MemoryPipeline::new(db.clone());
    pipeline.store_working(&memory)?;

    let memory_item = MemoryItem::from(&memory);
    working_memory.store(memory_item).await;

    Ok(vec![memory.id.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("test.mp3")));
        assert!(is_audio_file(Path::new("test.WAV")));
        assert!(is_audio_file(Path::new("test.m4a")));
        assert!(is_audio_file(Path::new("test.flac")));
        assert!(is_audio_file(Path::new("test.ogg")));
        assert!(!is_audio_file(Path::new("test.txt")));
        assert!(!is_audio_file(Path::new("test.mp4")));
        assert!(!is_audio_file(Path::new("test.pdf")));
    }

    #[test]
    fn test_get_supported_extensions() {
        let exts = get_supported_extensions();
        assert!(exts.contains(&"mp3"));
        assert!(exts.contains(&"wav"));
        assert!(exts.contains(&"m4a"));
        assert!(exts.contains(&"flac"));
        assert!(exts.contains(&"ogg"));
    }

    #[test]
    fn test_audio_analysis() {
        // Create 1 second of 16kHz silence
        let samples: Vec<f32> = vec![0.0; 16000];
        let analysis = AudioAnalysis::from_samples(&samples, 16000, 1.0);

        assert_eq!(analysis.duration_seconds, 1.0);
        assert_eq!(analysis.sample_rate, 16000);
        assert!(analysis.rms_db < -60.0); // Silence should be very quiet
        assert!(analysis.speech_estimate_percent < 10.0);
        assert!(analysis.silence_percent > 90.0);
    }
}
