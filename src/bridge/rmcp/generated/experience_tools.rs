    #[tool(name = "record_experience", description = "Record a new experience")]
    async fn record_experience(
        &self,
        Parameters(input): Parameters<tools::experience::RecordExperienceInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("record_experience").await {
            tracing::warn!("Workflow enforcement blocked record_experience: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::experience::execute_record_experience(
            input,
            &self.context.coordinator,
            &self.context.database,
        )
        .await
        {
            Ok(result) => {
                self.record_tool_execution("record_experience", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_experience_stats",
        description = "Get experience statistics"
    )]
    async fn get_experience_stats(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceStatsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_experience_stats").await {
            tracing::warn!("Workflow enforcement blocked get_experience_stats: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::experience::execute_get_experience_stats(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_experience_stats", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "list_experiences", description = "List recent experiences")]
    async fn list_experiences(
        &self,
        Parameters(input): Parameters<tools::experience::ListExperiencesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_experiences").await {
            tracing::warn!("Workflow enforcement blocked list_experiences: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::experience::execute_list_experiences(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("list_experiences", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_experience",
        description = "Get a specific experience by ID"
    )]
    async fn get_experience(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_experience").await {
            tracing::warn!("Workflow enforcement blocked get_experience: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::experience::execute_get_experience(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_experience", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_worker_stats",
        description = "Get background worker statistics for observers"
    )]
    async fn get_worker_stats(
        &self,
        Parameters(input): Parameters<tools::experience::GetWorkerStatsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_worker_stats").await {
            tracing::warn!("Workflow enforcement blocked get_worker_stats: {}", e.message);
            return enforcement_error_to_content(e);
        }

        match tools::experience::execute_get_worker_stats(input, &self.context.worker_manager).await {
            Ok(result) => {
                self.record_tool_execution("get_worker_stats", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_worker_count",
        description = "Get the number of active background workers"
    )]
    async fn get_worker_count(
        &self,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_worker_count").await {
            tracing::warn!("Workflow enforcement blocked get_worker_count: {}", e.message);
            return enforcement_error_to_content(e);
        }

        match tools::experience::execute_get_worker_count(&self.context.worker_manager).await {
            Ok(result) => {
                self.record_tool_execution("get_worker_count", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
