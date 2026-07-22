// src/tools/ingestor/definitions.rs
// MCP tool definitions with JSON schemas

use crate::bridge::mcp::McpTool;

pub const INGEST_FILES: &str = "ingest_files";
pub const LIST_IMPORTABLE: &str = "list_importable";
pub const TRANSCRIBE_AUDIO: &str = "transcribe_audio";
pub const LIST_INGESTED_FILES: &str = "list_ingested_files";
pub const DELETE_INGESTED_FILES: &str = "delete_ingested_files";

pub fn all() -> Vec<McpTool> {
    vec![
        McpTool {
            name: INGEST_FILES.to_string(),
            description: "Ingest files into memory. Use file_path for single file, or folder with limit=1 for single file from folder. Files are stored relative to the executable location (exe dir/files_to_import/)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Folder path relative to exe. Defaults to 'files_to_import' in exe directory. The folder should contain files_to_import with the actual files."
                    },
                    "file_path": {
                        "type": "string",
                        "description": "SINGLE FILE MODE - Exact path to ingest one specific file. Use this OR folder+limit, not both."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "MAX 1 for single file mode. Number of files to ingest from folder (default: 1). Use limit=1 to ingest ONE file at a time."
                    },
                    "chunk_size": {
                        "type": "integer",
                        "description": "Chunk size for splitting text (default: 1000)"
                    },
                    "memory_type": {
                        "type": "string",
                        "description": "Memory type: file, conversation, code, note (default: file)"
                    }
                },
                "oneOf": [
                    {"required": ["file_path"]},
                    {"properties": {"limit": {"const": 1}}}
                ]
            }),
        },
        McpTool {
            name: LIST_IMPORTABLE.to_string(),
            description: "List files ready for import from files_to_import folder (in exe directory). Shows what files are available to ingest.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": {
                        "type": "string",
                        "description": "Folder path relative to exe directory. Defaults to 'files_to_import'"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max files to return (default: 5)"
                    }
                }
            }),
        },
        McpTool {
            name: TRANSCRIBE_AUDIO.to_string(),
            description: "Transcribe audio file to text.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to audio file"
                    },
                    "output": {
                        "type": "string",
                        "description": "Output path for transcription JSON"
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
                    }
                }
            }),
        },
        McpTool {
            name: DELETE_INGESTED_FILES.to_string(),
            description: "Delete original files after they have been ingested. ALWAYS ask user for confirmation before calling this tool!".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File paths to delete. MUST be files that were already ingested."
                    },
                    "confirmation": {
                        "type": "string",
                        "description": "VERIFICATION REQUIRED: Must be EXACTLY 'yes' to confirm deletion. Without this, deletion will NOT proceed."
                    }
                },
                "required": ["files", "confirmation"]
            }),
        },
    ]
}
