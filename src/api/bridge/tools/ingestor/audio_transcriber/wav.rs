// src/api/bridge/tools/ingestor/audio_transcriber/wav.rs
//! WAV sample decoding

use anyhow::Result;

/// Decode WAV sample data into f32 samples, downmixing to mono when multi-channel
pub fn decode_wav_samples(data: &[u8], channels: u16, bits_per_sample: u16) -> Result<Vec<f32>> {
    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let frame_size = bytes_per_sample * channels.max(1) as usize;
    let num_frames = data.len().checked_div(frame_size).unwrap_or(0);
    let mut samples = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let frame_offset = f * frame_size;
        let mut sum = 0.0f32;
        for c in 0..channels.max(1) as usize {
            let offset = frame_offset + c * bytes_per_sample;
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
                    let val =
                        i32::from_le_bytes([0, data[offset], data[offset + 1], data[offset + 2]]);
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
            sum += sample;
        }
        // downmix: average across channels (mono when channels == 1)
        samples.push(sum / channels.max(1) as f32);
    }

    Ok(samples)
}
