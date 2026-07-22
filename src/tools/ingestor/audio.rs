// src/tools/ingestor/audio.rs
// Audio transcription using whisper-cli

use std::path::Path;

use anyhow::Result;

use crate::tools::ToolOutput;

use super::TranscribeAudioInput;

// ============================================================================
// AUDIO TRANSCRIPTION
// ============================================================================

pub async fn execute_transcribe_audio(
    input: TranscribeAudioInput,
) -> Result<ToolOutput> {
    let path = Path::new(&input.path);
    
    if !path.exists() {
        return Ok(ToolOutput::error(format!("File not found: {}", input.path)));
    }
    
    // Check if whisper-cli is available
    let output = std::process::Command::new("which")
        .arg("whisper")
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            // Use whisper-cli for transcription
            let output_path = input.output.unwrap_or_else(|| {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                format!("{}_transcription.json", stem)
            });
            
            let result = std::process::Command::new("whisper")
                .arg(&input.path)
                .arg("--output_format")
                .arg("json")
                .arg("--output_dir")
                .arg(
                    std::path::Path::new(&output_path)
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                )
                .output();
            
            match result {
                Ok(result) if result.status.success() => {
                    Ok(ToolOutput::success(serde_json::json!({
                        "transcription": format!(
                            "{}.json",
                            path.file_stem().unwrap_or_default().to_string_lossy()
                        ),
                        "path": input.path
                    })))
                }
                Ok(result) => {
                    Ok(ToolOutput::error(format!(
                        "Whisper failed: {}",
                        String::from_utf8_lossy(&result.stderr)
                    )))
                }
                Err(e) => {
                    Ok(ToolOutput::error(format!("Failed to run whisper: {}", e)))
                }
            }
        }
        _ => {
            Ok(ToolOutput::error(
                "Audio transcription requires whisper-cli.\n\
                 Install with: pip install whisper\n\
                 Or: brew install whisper".to_string()
            ))
        }
    }
}
