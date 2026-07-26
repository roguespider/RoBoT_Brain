    #[tool(
        name = "get_workflow",
        description = "MANDATORY: Get workflow rules. MUST be called before any other tool. Returns the required workflow for this MCP server."
    )]
    async fn get_workflow(
        &self,
        Parameters(input): Parameters<tools::agent::GetWorkflowInput>,
    ) -> ContentBlock {
        // Record workflow retrieval
        self.enforcer.record_workflow_retrieved(&self.session_id, input.purpose.clone()).await;
        
        match tools::agent::execute_get_workflow(input).await {
            Ok(result) => {
                self.record_tool_execution("get_workflow", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "store_memory",
        description = "Store a new memory in the knowledge base"
    )]
    async fn store_memory(
        &self,
        Parameters(input): Parameters<tools::memory::StoreMemoryInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("store_memory").await {
            tracing::warn!("Workflow enforcement blocked store_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_store_memory(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("store_memory", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "search_memory", description = "Search memories by content")]
    async fn search_memory(
        &self,
        Parameters(input): Parameters<tools::memory::SearchMemoryInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (but search_memory is allowed after get_workflow)
        if let Err(e) = self.check_workflow_enforcement("search_memory").await {
            tracing::warn!("Workflow enforcement blocked search_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let query = Some(input.query.clone());
        match tools::memory::execute_search_memory(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("search_memory", query).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_memory", description = "Get a specific memory by ID")]
    async fn get_memory(
        &self,
        Parameters(input): Parameters<tools::memory::GetMemoryInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_memory").await {
            tracing::warn!("Workflow enforcement blocked get_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_get_memory(input, &self.context.database, &self.context.memory_retrieval).await {
            Ok(result) => {
                self.record_tool_execution("get_memory", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "list_memories", description = "List recent memories")]
    async fn list_memories(
        &self,
        Parameters(input): Parameters<tools::memory::ListMemoriesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_memories").await {
            tracing::warn!("Workflow enforcement blocked list_memories: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_list_memories(input, &self.context.database, &self.context.memory_retrieval).await {
            Ok(result) => {
                self.record_tool_execution("list_memories", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
