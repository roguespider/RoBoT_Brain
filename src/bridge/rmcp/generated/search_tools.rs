    #[tool(
        name = "global_search",
        description = "Search across all memories and experiences"
    )]
    async fn global_search(
        &self,
        Parameters(input): Parameters<tools::search::GlobalSearchInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (but global_search counts as memory search)
        if let Err(e) = self.check_workflow_enforcement("global_search").await {
            tracing::warn!("Workflow enforcement blocked global_search: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let query = Some(input.query.clone());
        match tools::search::execute_global_search(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("global_search", query).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_recommendations",
        description = "Get recommendations based on learned patterns"
    )]
    async fn get_recommendations(
        &self,
        Parameters(input): Parameters<tools::search::GetRecommendationsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_recommendations").await {
            tracing::warn!("Workflow enforcement blocked get_recommendations: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::search::execute_get_recommendations(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_recommendations", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_reputation",
        description = "Get reputation score for a target"
    )]
    async fn get_reputation(
        &self,
        Parameters(input): Parameters<tools::search::GetReputationInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_reputation").await {
            tracing::warn!("Workflow enforcement blocked get_reputation: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::search::execute_get_reputation(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_reputation", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
