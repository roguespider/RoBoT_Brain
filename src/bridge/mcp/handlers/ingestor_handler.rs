// src/bridge/tools/handlers/ingestor_handler.rs
// Ingestor tools handler - handles file ingestion tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::ingestor;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for ingestor-related tools
#[derive(Clone)]
pub struct IngestorToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl IngestorToolsHandler {
    /// Create a new ingestor tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "ingestor",
                "Database connection not available",
            ));
        }

        Ok(Self { context, enforcer })
    }

    /// Ingest files from import folder
    pub async fn execute_ingest_files(
        &self,
        input: ingestor::IngestFilesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        ingestor::ingest_file(
            input,
            self.context.database.clone(),
            self.context.working_memory.clone(),
        )
        .await
    }

    /// List importable files
    pub async fn execute_list_importable(
        &self,
        input: ingestor::ListImportableInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        ingestor::execute_list_importable(input).await
    }

    /// List ingested files
    pub async fn execute_list_ingested_files(
        &self,
        input: ingestor::ListIngestedFilesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        ingestor::execute_list_ingested_files(input).await
    }

    /// Delete ingested files
    pub async fn execute_delete_ingested_files(
        &self,
        input: ingestor::DeleteIngestedFilesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        ingestor::execute_delete_ingested_files(input).await
    }

    /// Transcribe audio file using Candle/Whisper
    #[cfg(feature = "audio")]
    pub async fn execute_transcribe_audio(
        &self,
        input: ingestor::TranscribeAudioInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        ingestor::execute_transcribe_audio(
            input,
            self.context.database.clone(),
            self.context.working_memory.clone(),
        )
        .await
    }

    /// Stub for transcribe_audio when audio feature is disabled
    pub async fn execute_transcribe_audio_disabled(
        &self,
        input: ingestor::TranscribeAudioInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        Ok(crate::bridge::tools::ToolOutput::error(format!(
            "Audio transcription is not available. Audio file not found: {}",
            input.path
        )))
    }
}

impl ToolHandler for IngestorToolsHandler {
    fn category(&self) -> &str {
        "ingestor"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "ingest_files".to_string(),
            "list_importable".to_string(),
            "list_ingested_files".to_string(),
            "delete_ingested_files".to_string(),
            "transcribe_audio".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "ingest_files",
                "Ingest files from files_to_import folder into memory",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": { "type": "string", "description": "Folder name (defaults to 'files_to_import')" },
                        "limit": { "type": "number", "description": "Number of files to ingest" },
                        "file_path": { "type": "string", "description": "Ingest specific file by path" },
                        "memory_type": { "type": "string", "description": "Memory type: file, conversation, code, note" }
                    }
                })),
            ).with_title("Ingest Files"),
            rmcp::model::Tool::new(
                "list_importable",
                "List files available for import",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": { "type": "string", "description": "Folder name" },
                        "limit": { "type": "number", "description": "Max files to return" }
                    }
                })),
            ).with_title("List Importable Files"),
            rmcp::model::Tool::new(
                "list_ingested_files",
                "List files that have been ingested",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "folder": { "type": "string", "description": "Folder name" },
                        "limit": { "type": "number", "description": "Max files to return" }
                    }
                })),
            ).with_title("List Ingested Files"),
            rmcp::model::Tool::new(
                "delete_ingested_files",
                "Delete original files after successful ingestion",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": { "type": "array", "items": { "type": "string" }, "description": "File paths to delete" },
                        "confirmation": { "type": "string", "description": "Must be 'yes'" }
                    },
                    "required": ["files", "confirmation"]
                })),
            ).with_title("Delete Ingested Files"),
            rmcp::model::Tool::new(
                "transcribe_audio",
                "Transcribe an audio file to text using Whisper AI",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Full path to the audio file" },
                        "store_as_memory": { "type": "boolean", "description": "Store as memory" }
                    },
                    "required": ["path"]
                })),
            ).with_title("Transcribe Audio"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "ingest_files" => {
                    let input: ingestor::IngestFilesInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_ingest_files(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_importable" => {
                    let input: ingestor::ListImportableInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_importable(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_ingested_files" => {
                    let input: ingestor::ListIngestedFilesInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_ingested_files(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "delete_ingested_files" => {
                    let input: ingestor::DeleteIngestedFilesInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_delete_ingested_files(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "transcribe_audio" => {
                    let input: ingestor::TranscribeAudioInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    #[cfg(feature = "audio")]
                    {
                        self.execute_transcribe_audio(input).await
                            .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                    }
                    #[cfg(not(feature = "audio"))]
                    {
                        self.execute_transcribe_audio_disabled(input).await
                            .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                    }
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
