#![allow(dead_code)]

// src/tools/ingestor/definitions.rs
// MCP tool definitions with JSON schemas

use crate::bridge::mcp::McpTool;

pub const INGEST_FILES: &str = "ingest_files";
pub const LIST_IMPORTABLE: &str = "list_importable";
pub const LIST_INGESTED_FILES: &str = "list_ingested_files";
pub const DELETE_INGESTED_FILES: &str = "delete_ingested_files";
pub const TRANSCRIBE_AUDIO: &str = "transcribe_audio";

pub fn all() -> Vec<McpTool> {
    vec![
        McpTool {
            name: INGEST_FILES.to_string(),
            description: "INGEST FILES INTO MEMORY. REQUIRED WORKFLOW: 1) Call get_workflow with purpose='file_ingestion' first. 2) Call list_importable to see files. 3) Call ingest_files with limit=1 (ONE file at a time). 4) SUMMARIZE what was ingested (filename, size, chunks, memory IDs). 5) ASK USER: 'Can I delete the original file?' 6) Only delete if user says YES. DO NOT batch ingest or auto-delete.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Defaults to 'files_to_import' - it's ALREADY next to robot_brain.exe. You don't need to specify this unless using a different folder. Example: 'files_to_import'"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "SINGLE FILE MODE - Ingest one specific file by full path. Example: 'C:\\robot_brain\\files_to_import\\notes.txt'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "REQUIRED: Must be 1. Ingest ONE file at a time, then ASK USER about deletion before continuing. Default is 1. Example: limit=1"
                    },
                    "chunk_size": {
                        "type": "integer",
                        "description": "Chunk size for splitting text (default: 1000, JSON: 16384)"
                    },
                    "memory_type": {
                        "type": "string",
                        "description": "Memory type: file, conversation, code, note (default: file)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: true). Set to false to only look in the root folder."
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force re-ingestion of already-ingested files (default: false). Use when user confirms they want to add a file again."
                    }
                }
            }),
        },
        McpTool {
            name: LIST_IMPORTABLE.to_string(),
            description: "LIST FILES READY FOR IMPORT. Automatically looks in 'files_to_import' folder (same directory as robot_brain.exe). Returns list of files with full paths. No need to search - just call this tool.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Leave empty - defaults to 'files_to_import' which is already next to robot_brain.exe"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return (default: 5)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: true). Set to false to only look in the root folder."
                    }
                }
            }),
        },
        McpTool {
            name: TRANSCRIBE_AUDIO.to_string(),
            description: "TRANSCRIBE AUDIO FILE. Transcribes an audio file (MP3, WAV, M4A, FLAC, etc.) to text using Whisper AI. The transcription is automatically stored as memory for later retrieval. Audio files in files_to_import will be automatically transcribed when ingested.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Full path to the audio file to transcribe. Example: 'C:\\robot_brain\\files_to_import\\recording.wav'"
                    },
                    "store_as_memory": {
                        "type": "boolean",
                        "description": "Whether to store the transcription as memory (default: true)"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: LIST_INGESTED_FILES.to_string(),
            description: "List files that have been successfully ingested and can now be deleted.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Import folder path (default: files_to_import)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: true). Set to false to only look in the root folder."
                    }
                }
            }),
        },
        McpTool {
            name: DELETE_INGESTED_FILES.to_string(),
            description: "DELETE ORIGINAL FILES after ingestion. ⚠️ CRITICAL: You MUST have asked the user 'Can I delete the original file?' and received a YES before calling this tool. Do NOT auto-delete. The tool will block deletion without user confirmation.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File paths to delete. MUST be files that were already ingested by ingest_files."
                    },
                    "confirmation": {
                        "type": "string",
                        "description": "VERIFICATION REQUIRED: Must be EXACTLY 'yes' to confirm deletion. Without this, deletion will NOT proceed. The user must have explicitly said YES to deletion."
                    }
                },
                "required": ["files", "confirmation"]
            }),
        },
    ]
}
