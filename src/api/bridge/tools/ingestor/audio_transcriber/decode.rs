// src/api/bridge/tools/ingestor/audio_transcriber/decode.rs
//! Whisper token decoding helpers

use anyhow::Result;

use candle_core::{Device, IndexOp, Tensor};
use candle_transformers::models::whisper::{self, Config};
use tokenizers::Tokenizer;

/// Get token ID from tokenizer
pub fn get_token_id(tokenizer: &Tokenizer, token: &str) -> Result<u32> {
    tokenizer
        .token_to_id(token)
        .ok_or_else(|| anyhow::anyhow!("Token not found: {}", token))
}

/// Decode audio features to text (free function to avoid borrow conflicts)
pub fn decode_tokens_from_features(
    config: &Config,
    device: &Device,
    model: &mut whisper::model::Whisper,
    tokenizer: &Tokenizer,
    audio_features: &Tensor,
) -> Result<String> {
    // Get special tokens
    let sot_token = get_token_id(tokenizer, whisper::SOT_TOKEN)?;
    let transcribe_token = get_token_id(tokenizer, whisper::TRANSCRIBE_TOKEN)?;
    let no_timestamps_token = get_token_id(tokenizer, whisper::NO_TIMESTAMPS_TOKEN)?;
    let eot_token = get_token_id(tokenizer, whisper::EOT_TOKEN)?;

    // Build token sequence: <|startoftranscript|><|en|><|transcribe|><|notimestamps|>
    let mut tokens = vec![sot_token, transcribe_token, no_timestamps_token];

    let sample_len = config.max_target_positions / 2;

    for _ in 0..sample_len {
        let tokens_t = Tensor::new(tokens.as_slice(), device)?.unsqueeze(0)?;

        let ys = model
            .decoder
            .forward(&tokens_t, audio_features, tokens.len() == 3)?;

        let (_, seq_len, _) = ys.dims3()?;
        let logits = model
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
            .unwrap_or(0);

        tokens.push(next_token);

        if next_token == eot_token || tokens.len() > config.max_target_positions {
            break;
        }
    }

    // Decode tokens to text
    let text = tokenizer
        .decode(&tokens, true)
        .map_err(|e| anyhow::anyhow!("Failed to decode tokens: {}", e))?;

    Ok(text)
}
