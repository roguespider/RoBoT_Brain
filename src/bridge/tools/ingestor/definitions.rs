
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
            description: "Ingest files from files_to_import folder into memory. When no path is specified, automatically ingests from the files_to_import folder. Returns memory IDs for stored content.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Optional. Import folder name (defaults to 'files_to_import' if not specified). The folder should be next to robot_brain executable."
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Ingest one specific file by full path. Example: '/path/to/file.txt'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of files to ingest (default: 1). Ingest one at a time."
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
                        "description": "Search subfolders recursively (default: true)"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force re-ingestion of already-ingested files (default: false)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: LIST_IMPORTABLE.to_string(),
            description: "List files available for import. Shows files in the files_to_import folder by default.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Optional. Import folder name (defaults to 'files_to_import' if not specified)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return (default: 5)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: true)"
                    },
                    "all": {
                        "type": "boolean",
                        "description": "List all files without limit (default: false)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: TRANSCRIBE_AUDIO.to_string(),
            description: "Transcribe an audio file to text using Whisper AI.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Full path to the audio file to transcribe"
                    },
                    "store_as_memory": {
                        "type": "boolean",
                        "description": "Store the transcription as memory (default: true)"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: LIST_INGESTED_FILES.to_string(),
            description: "List files that have been successfully ingested.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Optional. Import folder name (defaults to 'files_to_import' if not specified)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search subfolders recursively (default: true)"
                    }
                }
            }),
        },
        McpTool {
            name: DELETE_INGESTED_FILES.to_string(),
            description: "Delete original files after successful ingestion. Requires confirmation='yes'.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File paths to delete (must be ingested files)"
                    },
                    "confirmation": {
                        "type": "string",
                        "description": "Must be 'yes' to confirm deletion"
                    }
                },
                "required": ["files", "confirmation"]
            }),
        },
    ]
}
