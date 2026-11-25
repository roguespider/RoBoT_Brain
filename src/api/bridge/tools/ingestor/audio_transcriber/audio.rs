// src/api/bridge/tools/ingestor/audio_transcriber/audio.rs
//! Audio processing: mel filters, WAV loading, resampling

use std::path::Path;

use anyhow::{Context, Result};

/// Load mel filterbank for Whisper
pub fn load_mel_filters() -> Result<Vec<f32>> {
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

/// Load audio file and convert to 16kHz mono PCM samples
pub fn load_audio_file(path: &Path) -> Result<Vec<f32>> {
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

            let samples = super::wav::decode_wav_samples(
                &data[audio_start..audio_end],
                channels,
                bits_per_sample,
            )?;

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
