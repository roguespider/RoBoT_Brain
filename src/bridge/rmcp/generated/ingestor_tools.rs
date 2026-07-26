    async fn ingest_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::IngestFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("ingest_files").await {
            tracing::warn!("Workflow enforcement blocked ingest_files: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::ingest_file(input, Arc::clone(&self.context.database)).await {
            Ok(result) => {
                self.record_tool_execution("ingest_files", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_importable",
        description = "List files available for import"
    )]
    async fn list_importable(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListImportableInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_importable").await {
            tracing::warn!("Workflow enforcement blocked list_importable: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::execute_list_importable(input).await {
            Ok(result) => {
                self.record_tool_execution("list_importable", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "transcribe_audio",
        description = "Transcribe an audio file to text"
    )]
    async fn transcribe_audio(
        &self,
        Parameters(input): Parameters<tools::ingestor::TranscribeAudioInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("transcribe_audio").await {
            tracing::warn!("Workflow enforcement blocked transcribe_audio: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::execute_transcribe_audio(input).await {
            Ok(result) => {
                self.record_tool_execution("transcribe_audio", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_ingested_files",
        description = "List files that have been ingested"
    )]
    async fn list_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListIngestedFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_ingested_files").await {
            tracing::warn!("Workflow enforcement blocked list_ingested_files: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::execute_list_ingested_files(input).await {
            Ok(result) => {
                self.record_tool_execution("list_ingested_files", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "delete_ingested_files",
        description = "Delete successfully ingested files"
    )]
    async fn delete_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::DeleteIngestedFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (destructive operation - requires full workflow)
        if let Err(e) = self.check_workflow_enforcement("delete_ingested_files").await {
            tracing::warn!("Workflow enforcement blocked delete_ingested_files: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::execute_delete_ingested_files(input).await {
            Ok(result) => {
                self.record_tool_execution("delete_ingested_files", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
