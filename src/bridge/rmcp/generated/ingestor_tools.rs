        #[tool(
        name = "ingest_files",
        description = "Ingest files from files_to_import folder into memory. One file at a time (limit=1). Returns memory IDs for stored content."
    )]
    async fn ingest_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::IngestFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("ingest_files").await {
            tracing::warn!("Workflow enforcement blocked ingest_files: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        // Per Architecture §6.3: Store ingested memories in Working Memory cache
        match tools::ingestor::ingest_file(
            input, 
            Arc::clone(&self.context.database),
            Arc::clone(&self.context.working_memory),
        ).await {
            Ok(result) => {
                self.record_tool_execution("ingest_files", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_importable",
        description = "List files available for import from files_to_import folder."
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
        description = "Transcribe an audio file to text using Whisper AI."
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
        
        match tools::ingestor::execute_transcribe_audio(input, self.context.database.clone(), self.context.working_memory.clone()).await {
            Ok(result) => {
                self.record_tool_execution("transcribe_audio", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_ingested_files",
        description = "List files that have been successfully ingested."
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
        description = "Delete original files after successful ingestion. Requires confirmation='yes'."
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
