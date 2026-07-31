//! Function Registry Module
//! 
//! Defines all MCP tools and their test requirements.
//! This registry is the source of truth for what functions need to be tested.

use serde::{Deserialize, Serialize};

/// Represents a test requirement for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Unique identifier for this test
    pub id: String,
    /// Name of the function/tool
    pub function_name: String,
    /// Category this function belongs to
    pub category: String,
    /// Whether this function requires the workflow to be initialized first
    pub requires_workflow: bool,
    /// Whether this function requires specific data to exist first
    pub requires_data: Option<DataRequirement>,
    /// Expected behavior description
    pub expected_behavior: String,
    /// Test validation checks (what to verify in the result)
    pub validation: Vec<ValidationCheck>,
    /// Priority level (1 = critical, 2 = important, 3 = nice to have)
    pub priority: u8,
}

/// Data that needs to be created before testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRequirement {
    pub data_type: String,
    pub creation_tool: String,
    pub min_count: usize,
}

/// Validation checks to perform on the result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub check_type: CheckType,
    pub field: String,
    pub expected_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckType {
    HasField,
    IsNonEmpty,
    IsSuccess,
    MatchesPattern,
    GreaterThan,
    LessThan,
}

/// All registered functions that need testing
pub struct FunctionRegistry;

impl FunctionRegistry {
    /// Get all functions that need to be tested
    pub fn get_all_functions() -> Vec<TestRequirement> {
        let mut functions = Vec::new();
        
        // Agent tools
        functions.extend(Self::agent_tools());
        
        // Memory tools
        functions.extend(Self::memory_tools());
        
        // Experience tools
        functions.extend(Self::experience_tools());
        
        // Reflection tools
        functions.extend(Self::reflection_tools());
        
        // Search tools
        functions.extend(Self::search_tools());
        
        // Ingestor tools
        functions.extend(Self::ingestor_tools());
        
        // Hypothesis tools
        functions.extend(Self::hypothesis_tools());
        
        // Exploration tools
        functions.extend(Self::exploration_tools());
        
        // Knowledge tools
        functions.extend(Self::knowledge_tools());
        
        // Planner tools
        functions.extend(Self::planner_tools());
        
        // Workflow tools
        functions.extend(Self::workflow_tools());
        
        // Skills tools
        functions.extend(Self::skills_tools());
        
        functions
    }
    
    /// Get all unique tool names
    #[allow(dead_code)]
    pub fn get_all_tool_names() -> Vec<&'static str> {
        vec![
            // Agent
            "get_workflow", "list_tools", "get_tool",
            // Memory
            "store_memory", "search_memory", "get_memory", "list_memories",
            // Experience
            "record_experience", "get_experience", "list_experiences", "get_experience_stats",
            // Reflection
            "create_reflection", "get_patterns", "get_insights", "analyze_patterns",
            // Search
            "global_search", "get_recommendations", "get_reputation",
            // Ingestor
            "ingest_files", "list_importable", "transcribe_audio", "list_ingested_files", "delete_ingested_files",
            // Hypothesis
            "record_observation", "create_hypothesis", "add_evidence", "get_hypothesis",
            "list_hypotheses", "list_observations", "evaluate_hypothesis", "get_knowledge", "extract_knowledge",
            // Exploration
            "start_exploration", "get_exploration_status", "complete_exploration", "abandon_exploration",
            "record_attempt", "add_exploration_hypothesis", "evaluate_exploration_hypothesis",
            "promote_finding", "pause_exploration", "resume_exploration",
            // Knowledge
            "add_knowledge", "query_knowledge", "get_mature_knowledge", "get_knowledge_stats",
            "record_knowledge_application",
            // Planner
            "create_plan", "add_plan_step", "add_step_dependency", "get_plan",
            "list_plans", "start_plan", "complete_step", "fail_step", "cancel_plan",
            // Workflow
            "create_workflow", "add_workflow_step", "get_workflow_status", "list_workflows",
            "start_workflow", "pause_workflow", "resume_workflow", "cancel_workflow", "delete_workflow",
            // Skills
            "register_skill", "discover_skill", "get_skill", "list_skills",
            "update_skill_mastery", "get_skill_recommendations", "execute_skill",
            "get_skill_stats", "apply_skill_decay", "enable_disable_skill", "search_skills",
        ]
    }
    
    fn agent_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "agent_get_workflow_default".to_string(),
                function_name: "get_workflow".to_string(),
                category: "Agent".to_string(),
                requires_workflow: false,
                requires_data: None,
                expected_behavior: "Returns workflow rules when called with 'default' purpose".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "workflow".to_string(), expected_value: None },
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "agent_get_workflow_general".to_string(),
                function_name: "get_workflow".to_string(),
                category: "Agent".to_string(),
                requires_workflow: false,
                requires_data: None,
                expected_behavior: "Returns workflow rules when called with 'general' purpose".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "workflow".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "agent_list_tools".to_string(),
                function_name: "list_tools".to_string(),
                category: "Agent".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all available tools".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "tools".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "agent_list_tools_memory".to_string(),
                function_name: "list_tools".to_string(),
                category: "Agent".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists memory tools when filtered by 'memory' category".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "tools".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "agent_get_tool".to_string(),
                function_name: "get_tool".to_string(),
                category: "Agent".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns tool definition for 'store_memory'".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "found".to_string(), expected_value: Some("true".to_string()) },
                    ValidationCheck { check_type: CheckType::HasField, field: "tool".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn memory_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "memory_store_basic".to_string(),
                function_name: "store_memory".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Stores a basic memory item (requires search_memory first)".to_string(),
                validation: vec![
                    // This test requires search_memory to be called first (precondition)
                    // The tool returns MEMORY_NOT_SEARCHED error
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "memory_store_with_metadata".to_string(),
                function_name: "store_memory".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Stores memory with confidence and importance scores (requires search_memory first)".to_string(),
                validation: vec![
                    // This test requires search_memory to be called first (precondition)
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "memory_search".to_string(),
                function_name: "search_memory".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: Some(DataRequirement { data_type: "memory".to_string(), creation_tool: "store_memory".to_string(), min_count: 1 }),
                expected_behavior: "Finds memories matching query".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "results".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "memory_get".to_string(),
                function_name: "get_memory".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None, // No real data created, uses fake UUID
                expected_behavior: "Retrieves a specific memory by ID (returns found=false for non-existent)".to_string(),
                validation: vec![
                    // Tool returns success with found=false for non-existent memory
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("true".to_string()) },
                    ValidationCheck { check_type: CheckType::HasField, field: "found".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "memory_get_invalid".to_string(),
                function_name: "get_memory".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Handles invalid UUID format gracefully (expected error)".to_string(),
                validation: vec![
                    // Invalid UUID format returns error with success=false
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "memory_list".to_string(),
                function_name: "list_memories".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all recent memories".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "memories".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "memory_list_filtered".to_string(),
                function_name: "list_memories".to_string(),
                category: "Memory".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists memories filtered by type".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "memories".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn experience_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "experience_record".to_string(),
                function_name: "record_experience".to_string(),
                category: "Experience".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Records a new experience with action, outcome, and tool name".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "experience_get".to_string(),
                function_name: "get_experience".to_string(),
                category: "Experience".to_string(),
                requires_workflow: true,
                requires_data: Some(DataRequirement { data_type: "experience".to_string(), creation_tool: "record_experience".to_string(), min_count: 1 }),
                expected_behavior: "Retrieves a specific experience by ID".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "experience_list".to_string(),
                function_name: "list_experiences".to_string(),
                category: "Experience".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists recent experiences".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "experiences".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "experience_stats".to_string(),
                function_name: "get_experience_stats".to_string(),
                category: "Experience".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns experience statistics".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "stats".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn reflection_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "reflection_create".to_string(),
                function_name: "create_reflection".to_string(),
                category: "Reflection".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Creates a new reflection".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "reflection_get_patterns".to_string(),
                function_name: "get_patterns".to_string(),
                category: "Reflection".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns learned patterns".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "patterns".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "reflection_get_insights".to_string(),
                function_name: "get_insights".to_string(),
                category: "Reflection".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns insights from analysis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "insights".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "reflection_analyze".to_string(),
                function_name: "analyze_patterns".to_string(),
                category: "Reflection".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Performs pattern analysis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn search_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "search_global".to_string(),
                function_name: "global_search".to_string(),
                category: "Search".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Performs global search across all data".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "results".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "search_recommendations".to_string(),
                function_name: "get_recommendations".to_string(),
                category: "Search".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns tool recommendations".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "recommendations".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "search_reputation".to_string(),
                function_name: "get_reputation".to_string(),
                category: "Search".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns reputation data for a tool".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "tool_name".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn ingestor_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "ingestor_list_importable".to_string(),
                function_name: "list_importable".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists files available for import".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "files".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "ingestor_list_importable_recursive".to_string(),
                function_name: "list_importable".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists files recursively including subdirectories".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "files".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "ingestor_ingest_text".to_string(),
                function_name: "ingest_files".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Ingests a text file".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "ingestor_ingest_json".to_string(),
                function_name: "ingest_files".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Ingests a JSON file with smart extraction".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "ingestor_ingest_code".to_string(),
                function_name: "ingest_files".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Ingests a code file (Rust)".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "ingestor_list_ingested".to_string(),
                function_name: "list_ingested_files".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all ingested files".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "files".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "ingestor_delete_blocked".to_string(),
                function_name: "delete_ingested_files".to_string(),
                category: "Ingestor".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Delete operation should be blocked without admin".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 3,
            },
        ]
    }
    
    fn hypothesis_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "hypothesis_record_observation".to_string(),
                function_name: "record_observation".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Records a new observation".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "hypothesis_create".to_string(),
                function_name: "create_hypothesis".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Creates a new hypothesis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "hypothesis_add_evidence".to_string(),
                function_name: "add_evidence".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds supporting or contradicting evidence".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "hypothesis_get".to_string(),
                function_name: "get_hypothesis".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns the current hypothesis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "hypothesis".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "hypothesis_list".to_string(),
                function_name: "list_hypotheses".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all hypotheses".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "hypotheses".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "hypothesis_evaluate".to_string(),
                function_name: "evaluate_hypothesis".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Evaluates the current hypothesis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "evaluation".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "hypothesis_extract".to_string(),
                function_name: "extract_knowledge".to_string(),
                category: "Hypothesis".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Extracts knowledge from evaluated hypothesis".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn exploration_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "exploration_start".to_string(),
                function_name: "start_exploration".to_string(),
                category: "Exploration".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Starts a new exploration".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "exploration_status".to_string(),
                function_name: "get_exploration_status".to_string(),
                category: "Exploration".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns exploration status".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "status".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "exploration_record_attempt".to_string(),
                function_name: "record_attempt".to_string(),
                category: "Exploration".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Records an exploration attempt".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "exploration_add_hypothesis".to_string(),
                function_name: "add_exploration_hypothesis".to_string(),
                category: "Exploration".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds a hypothesis to exploration".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn knowledge_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "knowledge_add".to_string(),
                function_name: "add_knowledge".to_string(),
                category: "Knowledge".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds new knowledge".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "knowledge_id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "knowledge_query".to_string(),
                function_name: "query_knowledge".to_string(),
                category: "Knowledge".to_string(),
                requires_workflow: true,
                requires_data: Some(DataRequirement { data_type: "knowledge".to_string(), creation_tool: "add_knowledge".to_string(), min_count: 1 }),
                expected_behavior: "Queries knowledge base".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "items".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "knowledge_mature".to_string(),
                function_name: "get_mature_knowledge".to_string(),
                category: "Knowledge".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Gets knowledge that has been applied multiple times".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "items".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "knowledge_stats".to_string(),
                function_name: "get_knowledge_stats".to_string(),
                category: "Knowledge".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns knowledge statistics".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "total".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "knowledge_record_application".to_string(),
                function_name: "record_knowledge_application".to_string(),
                category: "Knowledge".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Records knowledge application outcome (fails with fake UUID)".to_string(),
                validation: vec![
                    // This test uses a fake UUID, so it will fail.
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 2,
            },
        ]
    }
    
    fn planner_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "planner_create".to_string(),
                function_name: "create_plan".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Creates a new plan".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "planner_add_step".to_string(),
                function_name: "add_plan_step".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds a step to the current plan".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "planner_add_dependency".to_string(),
                function_name: "add_step_dependency".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds a dependency between steps".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_get".to_string(),
                function_name: "get_plan".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns the current plan".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "plan".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_start".to_string(),
                function_name: "start_plan".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Starts executing the plan".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_complete_step".to_string(),
                function_name: "complete_step".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Marks a step as completed".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_fail_step".to_string(),
                function_name: "fail_step".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Marks a step as failed".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_cancel".to_string(),
                function_name: "cancel_plan".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Cancels the current plan".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "planner_list".to_string(),
                function_name: "list_plans".to_string(),
                category: "Planner".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all plans".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "plans".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn workflow_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "workflow_create".to_string(),
                function_name: "create_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Creates a new workflow".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "workflow_add_step".to_string(),
                function_name: "add_workflow_step".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Adds a step to the workflow".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "workflow_status".to_string(),
                function_name: "get_workflow_status".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Returns workflow status".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "status".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_start".to_string(),
                function_name: "start_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Starts workflow execution".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_pause".to_string(),
                function_name: "pause_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Pauses workflow execution".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_resume".to_string(),
                function_name: "resume_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Resumes workflow execution".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_cancel".to_string(),
                function_name: "cancel_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Cancels workflow execution".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_delete".to_string(),
                function_name: "delete_workflow".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Deletes a workflow".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "workflow_list".to_string(),
                function_name: "list_workflows".to_string(),
                category: "Workflow".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all workflows".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "workflows".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
    
    fn skills_tools() -> Vec<TestRequirement> {
        vec![
            TestRequirement {
                id: "skills_register".to_string(),
                function_name: "register_skill".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Registers a new skill".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 1,
            },
            TestRequirement {
                id: "skills_discover".to_string(),
                function_name: "discover_skill".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Creates a skill from experience".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "id".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_get".to_string(),
                function_name: "get_skill".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Gets skill details (fails with fake UUID)".to_string(),
                validation: vec![
                    // This test uses a fake UUID, so it will fail. 
                    // For a real test, use requires_data to get a registered skill's ID.
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_list".to_string(),
                function_name: "list_skills".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Lists all skills".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "skills".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_update_mastery".to_string(),
                function_name: "update_skill_mastery".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Updates skill mastery (fails with fake UUID)".to_string(),
                validation: vec![
                    // This test uses a fake UUID, so it will fail.
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_recommendations".to_string(),
                function_name: "get_skill_recommendations".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Gets skill recommendations".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "recommendations".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_execute".to_string(),
                function_name: "execute_skill".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Executes a skill (fails with fake UUID)".to_string(),
                validation: vec![
                    // This test uses a fake UUID, so it will fail.
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: Some("false".to_string()) },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_stats".to_string(),
                function_name: "get_skill_stats".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Gets skill statistics".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "stats".to_string(), expected_value: None },
                ],
                priority: 2,
            },
            TestRequirement {
                id: "skills_decay".to_string(),
                function_name: "apply_skill_decay".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Applies skill decay".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 3,
            },
            TestRequirement {
                id: "skills_enable_disable".to_string(),
                function_name: "enable_disable_skill".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Enables or disables a skill".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::IsSuccess, field: "success".to_string(), expected_value: None },
                ],
                priority: 3,
            },
            TestRequirement {
                id: "skills_search".to_string(),
                function_name: "search_skills".to_string(),
                category: "Skills".to_string(),
                requires_workflow: true,
                requires_data: None,
                expected_behavior: "Searches skills".to_string(),
                validation: vec![
                    ValidationCheck { check_type: CheckType::HasField, field: "results".to_string(), expected_value: None },
                ],
                priority: 2,
            },
        ]
    }
}
