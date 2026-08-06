// ingestor_tools.rs - File ingestion and processing tools

use crate::bridge::rmcp::generated::tool_traits::{
    IngestorToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for ingestor tools - implements IngestorToolsHandlerTrait
pub struct IngestorToolsHandler;

impl IngestorToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IngestorToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl IngestorToolsHandlerTrait for IngestorToolsHandler {
    async fn execute_ingest_files(
        &self,
        context: &ToolContext,
        input: tools::ingestor::IngestFilesInput,
    ) -> ToolOutput {
        match tools::ingestor::execute_ingest_files(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_importable(
        &self,
        context: &ToolContext,
        input: tools::ingestor::ListImportableInput,
    ) -> ToolOutput {
        match tools::ingestor::execute_list_importable(input).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_ingested_files(
        &self,
        context: &ToolContext,
        input: tools::ingestor::ListIngestedFilesInput,
    ) -> ToolOutput {
        match tools::ingestor::execute_list_ingested_files(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_ingested_files(
        &self,
        context: &ToolContext,
        input: tools::ingestor::DeleteIngestedFilesInput,
    ) -> ToolOutput {
        match tools::ingestor::execute_delete_ingested_files(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_transcribe_audio(
        &self,
        context: &ToolContext,
        input: tools::ingestor::TranscribeAudioInput,
    ) -> ToolOutput {
        match tools::ingestor::execute_transcribe_audio(input).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::ingestor::ingest_files_tool(),
            tools::ingestor::list_importable_tool(),
            tools::ingestor::list_ingested_files_tool(),
            tools::ingestor::delete_ingested_files_tool(),
            tools::ingestor::transcribe_audio_tool(),
        ]
    }
}
