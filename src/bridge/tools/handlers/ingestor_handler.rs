// src/bridge/tools/handlers/ingestor_handler.rs
// Ingestor tools handler - handles file ingestion tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::ingestor;
use crate::bridge::tools::handlers::{HandlerInitError, HandlerInitResult, ToolHandler};
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
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }
}
