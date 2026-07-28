    #[tool(name = "register_skill", description = "Register a new skill in the skill registry. Skills represent reusable capabilities.")]
    async fn register_skill(
        &self,
        Parameters(input): Parameters<tools::skills::RegisterSkillInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("register_skill").await {
            tracing::warn!("Workflow enforcement blocked register_skill: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_register_skill(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("register_skill", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "discover_skill", description = "Create a skill discovered from an experience. Per Architecture §15: Skills emerge from experience.")]
    async fn discover_skill(
        &self,
        Parameters(input): Parameters<tools::skills::DiscoverSkillInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("discover_skill").await {
            tracing::warn!("Workflow enforcement blocked discover_skill: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_discover_skill(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("discover_skill", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_skill", description = "Get details of a specific skill including mastery level, usage statistics, and prerequisites.")]
    async fn get_skill(
        &self,
        Parameters(input): Parameters<tools::skills::GetSkillInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_skill").await {
            tracing::warn!("Workflow enforcement blocked get_skill: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_get_skill(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("get_skill", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "list_skills", description = "List all registered skills, optionally filtered by category or enabled status.")]
    async fn list_skills(
        &self,
        Parameters(input): Parameters<tools::skills::ListSkillsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("list_skills").await {
            tracing::warn!("Workflow enforcement blocked list_skills: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_list_skills(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("list_skills", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "update_skill_mastery", description = "Update skill mastery based on execution outcome. Records success or failure for the skill.")]
    async fn update_skill_mastery(
        &self,
        Parameters(input): Parameters<tools::skills::UpdateSkillMasteryInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("update_skill_mastery").await {
            tracing::warn!("Workflow enforcement blocked update_skill_mastery: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_update_skill_mastery(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("update_skill_mastery", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_skill_recommendations", description = "Get skill recommendations based on readiness and usage patterns. Per Architecture §15.")]
    async fn get_skill_recommendations(
        &self,
        Parameters(input): Parameters<tools::skills::GetSkillRecommendationsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_skill_recommendations").await {
            tracing::warn!("Workflow enforcement blocked get_skill_recommendations: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_get_skill_recommendations(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("get_skill_recommendations", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "execute_skill", description = "Execute a skill with provided task and parameters. Per Architecture §15: Skill::execute(&context).")]
    async fn execute_skill(
        &self,
        Parameters(input): Parameters<tools::skills::ExecuteSkillInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("execute_skill").await {
            tracing::warn!("Workflow enforcement blocked execute_skill: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_execute_skill(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("execute_skill", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_skill_stats", description = "Get comprehensive statistics about the skill registry including mastery distribution.")]
    async fn get_skill_stats(
        &self,
        Parameters(input): Parameters<tools::skills::GetSkillStatsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_skill_stats").await {
            tracing::warn!("Workflow enforcement blocked get_skill_stats: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_get_skill_stats(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("get_skill_stats", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "apply_skill_decay", description = "Apply mastery decay to unused skills. Per Architecture §15: Skills include decay.")]
    async fn apply_skill_decay(
        &self,
        Parameters(input): Parameters<tools::skills::ApplySkillDecayInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("apply_skill_decay").await {
            tracing::warn!("Workflow enforcement blocked apply_skill_decay: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_apply_skill_decay(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("apply_skill_decay", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "enable_disable_skill", description = "Enable or disable a skill. Disabled skills cannot be executed.")]
    async fn enable_disable_skill(
        &self,
        Parameters(input): Parameters<tools::skills::EnableDisableSkillInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("enable_disable_skill").await {
            tracing::warn!("Workflow enforcement blocked enable_disable_skill: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_enable_disable_skill(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("enable_disable_skill", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "search_skills", description = "Search skills by query, category, or minimum mastery level.")]
    async fn search_skills(
        &self,
        Parameters(input): Parameters<tools::skills::SearchSkillsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("search_skills").await {
            tracing::warn!("Workflow enforcement blocked search_skills: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::skills::execute_search_skills(input, &self.context).await {
            Ok(result) => {
                self.record_tool_execution("search_skills", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
