    async fn create_plan(
        &self,
        Parameters(input): Parameters<tools::planner::CreatePlanInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("create_plan").await {
            tracing::warn!("Workflow enforcement blocked create_plan: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_create_plan(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("create_plan", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "add_plan_step", description = "Add a step to an existing plan")]
    async fn add_plan_step(
        &self,
        Parameters(input): Parameters<tools::planner::AddPlanStepInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("add_plan_step").await {
            tracing::warn!("Workflow enforcement blocked add_plan_step: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_add_plan_step(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("add_plan_step", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(
        name = "add_step_dependency",
        description = "Add a dependency between steps"
    )]
    async fn add_step_dependency(
        &self,
        Parameters(input): Parameters<tools::planner::AddStepDependencyInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("add_step_dependency").await {
            tracing::warn!("Workflow enforcement blocked add_step_dependency: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_add_step_dependency(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("add_step_dependency", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "get_plan", description = "Get a plan by ID")]
    async fn get_plan(
        &self,
        Parameters(input): Parameters<tools::planner::GetPlanInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_plan").await {
            tracing::warn!("Workflow enforcement blocked get_plan: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_get_plan(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("get_plan", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "list_plans", description = "List all active plans")]
    async fn list_plans(
        &self,
        Parameters(input): Parameters<tools::planner::ListPlansInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_plans").await {
            tracing::warn!("Workflow enforcement blocked list_plans: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_list_plans(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("list_plans", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "start_plan", description = "Start executing a plan")]
    async fn start_plan(
        &self,
        Parameters(input): Parameters<tools::planner::StartPlanInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("start_plan").await {
            tracing::warn!("Workflow enforcement blocked start_plan: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_start_plan(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("start_plan", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "complete_step", description = "Mark a step as completed")]
    async fn complete_step(
        &self,
        Parameters(input): Parameters<tools::planner::CompleteStepInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("complete_step").await {
            tracing::warn!("Workflow enforcement blocked complete_step: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_complete_step(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("complete_step", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "fail_step", description = "Mark a step as failed")]
    async fn fail_step(
        &self,
        Parameters(input): Parameters<tools::planner::FailStepInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("fail_step").await {
            tracing::warn!("Workflow enforcement blocked fail_step: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_fail_step(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("fail_step", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "cancel_plan", description = "Cancel a plan")]
    async fn cancel_plan(
        &self,
        Parameters(input): Parameters<tools::planner::CancelPlanInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("cancel_plan").await {
            tracing::warn!("Workflow enforcement blocked cancel_plan: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::planner::execute_cancel_plan(input, &self.context.planner).await;
        if result.success {
            self.record_tool_execution("cancel_plan", None).await;
        }
        tool_output_to_content(result)
    }
