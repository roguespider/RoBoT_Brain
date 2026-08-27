//! T1-10B-07 — migrated from `src/bridge/tools/ingestor/audio_transcriber.rs`
//! `#[cfg(test)] mod tests` (test_is_audio_file, test_get_supported_extensions).
//! test_audio_analysis reclassified to Group B (see PLAN.md note).
//!
//! The src/ unit tests exercised `is_audio_file` and `get_supported_extensions`
//! directly. test_suite cannot import robot_brain source, so the behavior is
//! re-expressed through the public MCP surface that invokes those functions:
//!   - `transcribe_audio` calls `is_audio_file(path)` (which calls
//!     `get_supported_extensions()`) and returns "Not a supported audio file"
//!     if the extension isn't in the supported list.
//!
//! MCP-reachable (migrated here):
//!   - test_is_audio_file / test_get_supported_extensions: create temp files
//!     with audio (.mp3, .wav, .m4a, .flac, .ogg) and non-audio (.txt, .mp4)
//!     extensions, call transcribe_audio, verify audio extensions pass the
//!     is_audio_file gate (error != "Not a supported audio file") while
//!     non-audio extensions are rejected with "Not a supported audio file".
//!
//! Group B (internal-only, LEAVE as Rust unit test):
//!
//!   - test_audio_analysis: AudioAnalysis::from_samples requires valid audio
//!     samples loaded from a real WAV file; not practical to create via MCP.

use crate::TestMcpClient;
use crate::TestStats;
use std::io::Write;

fn payload_text(result: &serde_json::Value) -> Option<String> {
    result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Create a temp file with the given extension and dummy content.
fn make_temp_file(ext: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir();
    let path = dir.join(format!("t1_10b_07_{:x}.{}", nanos, ext));
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(b"dummy content");
    }
    path
}

pub async fn run_audio_transcriber_tests(
    client: &mut TestMcpClient,
    stats: &mut TestStats,
) -> anyhow::Result<()> {
    crate::teeprintln!("\n--- Audio transcriber is_audio_file + extensions (T1-10B-07) ---");

    // Audio extensions that is_audio_file should accept (pass the gate).
    // They'll fail later (load_audio_file) but with a DIFFERENT error than
    // "Not a supported audio file".
    let audio_exts = ["mp3", "wav", "m4a", "flac", "ogg"];
    // Non-audio extensions that is_audio_file should reject.
    let non_audio_exts = ["txt", "mp4"];

    let mut all_pass = true;

    // --- test_is_audio_file + test_get_supported_extensions (combined) ---
    // Audio extensions: must NOT get "Not a supported audio file" error.
    for ext in &audio_exts {
        let path = make_temp_file(ext);
        let result = client
            .call_tool(
                "transcribe_audio",
                serde_json::json!({
                    "path": path.to_string_lossy(),
                    "store_as_memory": false
                }),
            )
            .await;
        // The tool returns errors as Err (TestMcpClient converts error
        // responses). Extract the message from either Ok content or Err text.
        let text = match result {
            Ok(r) => payload_text(&r).unwrap_or_default(),
            Err(e) => format!("{}", e),
        };
        let _ = std::fs::remove_file(&path);
        // Audio extension should pass is_audio_file gate. The error (if any)
        // should be about conversion/loading, NOT "Not a supported audio file".
        let rejected_as_non_audio = text.contains("Not a supported audio file");
        if rejected_as_non_audio {
            crate::teeprintln!(
                "  [FAIL] is_audio_file(.{}): rejected as non-audio (expected to pass gate)",
                ext
            );
            all_pass = false;
        }
    }

    // Non-audio extensions: must get "Not a supported audio file" error.
    for ext in &non_audio_exts {
        let path = make_temp_file(ext);
        let result = client
            .call_tool(
                "transcribe_audio",
                serde_json::json!({
                    "path": path.to_string_lossy(),
                    "store_as_memory": false
                }),
            )
            .await;
        let text = match result {
            Ok(r) => payload_text(&r).unwrap_or_default(),
            Err(e) => format!("{}", e),
        };
        let _ = std::fs::remove_file(&path);
        let rejected_as_non_audio = text.contains("Not a supported audio file");
        if !rejected_as_non_audio {
            crate::teeprintln!(
                "  [FAIL] is_audio_file(.{}): NOT rejected as non-audio (expected rejection)",
                ext
            );
            all_pass = false;
        }
    }

    if all_pass {
        crate::teeprintln!(
            "  [OK] is_audio_file + get_supported_extensions: audio exts (mp3/wav/m4a/flac/ogg) pass gate, non-audio (txt/mp4) rejected"
        );
        stats.passed += 1;
    } else {
        crate::teeprintln!(
            "  [FAIL] is_audio_file: one or more extension checks failed (see above)"
        );
        stats.failed += 1;
    }

    Ok(())
}
