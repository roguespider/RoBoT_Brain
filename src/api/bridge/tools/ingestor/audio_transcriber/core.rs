// src/api/bridge/tools/ingestor/audio_transcriber/core.rs
//! WhisperTranscriber - Candle Whisper model loading and inference

use anyhow::Result;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as whisper, Config};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

use super::types::TranscriptionResult;

/// Whisper transcriber using Candle
pub struct WhisperTranscriber {
    pub model: Option<whisper::model::Whisper>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    config: Config,
}

impl WhisperTranscriber {
    /// Create a new Whisper transcriber (assumes model is already cached)
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;

        // Use HuggingFace cache - model should already be downloaded
        let api = Api::new()?;
        let repo = api.repo(hf_hub::Repo::with_revision(
            "openai/whisper-base".to_string(),
            hf_hub::RepoType::Model,
            "main".to_string(),
        ));

        tracing::info!("Loading Whisper model...");

        // Get model files from cache
        let config_path = repo.get("config.json")?;
        let model_path = repo.get("model.safetensors")?;
        let tokenizer_path = repo.get("tokenizer.json")?;

        tracing::info!("Loading Whisper model from {:?}", model_path);

        // Load config
        let config_content = std::fs::read_to_string(&config_path)?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)?;

        let num_mel_bins = config_json["n_mels"].as_i64().unwrap_or(80) as usize;
        let max_source_positions = config_json["n_audio_ctx"].as_i64().unwrap_or(1500) as usize;
        let d_model = config_json["d_model"].as_i64().unwrap_or(512) as usize;
        let encoder_attention_heads = config_json["n_audio_head"].as_i64().unwrap_or(8) as usize;
        let encoder_layers = config_json["n_audio_layer"].as_i64().unwrap_or(4) as usize;
        let vocab_size = config_json["n_vocab"].as_i64().unwrap_or(51865) as usize;
        let max_target_positions = config_json["n_text_ctx"].as_i64().unwrap_or(448) as usize;
        let decoder_attention_heads = config_json["n_text_head"].as_i64().unwrap_or(8) as usize;
        let decoder_layers = config_json["n_text_layer"].as_i64().unwrap_or(4) as usize;

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

        let model = whisper::model::Whisper::load(&vb, whisper_config.clone())?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        tracing::info!("Whisper model loaded successfully");

        Ok(Self {
            model: Some(model),
            tokenizer: Some(tokenizer),
            device,
            config: whisper_config,
        })
    }

    /// Transcribe audio samples to text
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<TranscriptionResult> {
        let duration_seconds = samples.len() as f32 / 16000.0;

        // Compute mel spectrogram using Candle's audio module
        let mel_filters = super::audio::load_mel_filters()?;
        let mel_spec = whisper::audio::pcm_to_mel(&self.config, samples, &mel_filters);

        // Convert to tensor [1, n_mel, n_frames]
        let n_mel = self.config.num_mel_bins;
        let n_frames = mel_spec.len() / n_mel;
        let mel_tensor = Tensor::from_slice(&mel_spec, (1, n_mel, n_frames), &self.device)?;

        tracing::info!("Mel spectrogram shape: {:?}", mel_tensor.shape());

        // Run encoder and decode - do everything in one borrow
        let text = {
            let model = self
                .model
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Whisper model not loaded"))?;
            let tokenizer = self
                .tokenizer
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Tokenizer not loaded"))?;

            // Run encoder
            let audio_features = model.encoder.forward(&mel_tensor, true)?;
            tracing::info!("Audio features shape: {:?}", audio_features.dims());

            // Decode tokens - pass config and device explicitly to avoid borrow conflict
            super::decode::decode_tokens_from_features(
                &self.config,
                &self.device,
                model,
                tokenizer,
                &audio_features,
            )?
        };

        Ok(TranscriptionResult {
            text,
            language: Some("en".to_string()),
            duration_seconds,
            segments: vec![],
        })
    }
}

impl Default for WhisperTranscriber {
    fn default() -> Self {
        Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            config: Config {
                num_mel_bins: 80,
                max_source_positions: 1500,
                d_model: 512,
                encoder_attention_heads: 8,
                encoder_layers: 4,
                vocab_size: 51865,
                max_target_positions: 448,
                decoder_attention_heads: 8,
                decoder_layers: 4,
                suppress_tokens: vec![],
            },
        }
    }
}
