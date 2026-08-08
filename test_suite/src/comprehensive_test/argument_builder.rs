//! Argument builder for test requirements.
//!
//! Generates test arguments for each MCP tool based on its requirements.

use crate::function_registry::TestRequirement;
use crate::test_environment::TestEnvironment;

/// Build test arguments based on the requirement
pub fn build_test_arguments(
    requirement: &TestRequirement,
    env: &TestEnvironment,
) -> serde_json::Value {
    match requirement.id.as_str() {
        // Agent tools
        "agent_get_workflow_default" => serde_json::json!({
            "purpose": "default"
        }),
        "agent_get_workflow_general" => serde_json::json!({
            "purpose": "general"
        }),
        "agent_list_tools" => serde_json::json!({}),
        "agent_list_tools_memory" => serde_json::json!({
            "category": "memory"
        }),
        "agent_get_tool" => serde_json::json!({
            "name": "store_memory"
        }),
        "agent_connect_mcp" => serde_json::json!({
            "name": "test_server",
            "command": "echo",
            "args": []
        }),
        "agent_call_tool" => serde_json::json!({
            "tool_name": "get_workflow",
            "arguments": "{\"purpose\": \"general\"}"
        }),

        // Memory tools
        "memory_store_basic" => serde_json::json!({
            "content": "Test memory content",
            "memory_type": "note"
        }),
        "memory_store_with_metadata" => serde_json::json!({
            "content": "Test memory with metadata",
            "memory_type": "fact",
            "confidence": 0.9,
            "importance": 0.8
        }),
        "memory_search" => serde_json::json!({
            "query": "test"
        }),
        "memory_get" => serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        }),
        "memory_get_invalid" => serde_json::json!({
            "id": "not-a-valid-uuid"
        }),
        "memory_list" => serde_json::json!({}),
        "memory_list_filtered" => serde_json::json!({
            "memory_type": "note"
        }),

        // Vector Index tools (Embedding operations)
        "vector_store_embedding" => serde_json::json!({
            "memory_id": "00000000-0000-0000-0000-000000000001",
            "embedding": [0.1, 0.2, 0.3, 0.4, 0.5],
            "model": "test-model"
        }),
        "vector_get_embedding" => serde_json::json!({
            "memory_id": "00000000-0000-0000-0000-000000000001"
        }),
        "vector_search_similar" => serde_json::json!({
            "query_embedding": [0.1, 0.2, 0.3, 0.4, 0.5],
            "limit": 5,
            "min_similarity": 0.5
        }),
        "vector_list_embeddings" => serde_json::json!({
            "limit": 100
        }),
        "vector_delete_embedding" => serde_json::json!({
            "memory_id": "00000000-0000-0000-0000-000000000001"
        }),
        "vector_get_embedding_stats" => serde_json::json!({}),

        // Experience tools
        "experience_record" => serde_json::json!({
            "action": "Test Action",
            "outcome": "Success",
            "tool_name": "test_tool"
        }),
        "experience_get" => serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000000"
        }),
        "experience_list" => serde_json::json!({}),
        "experience_stats" => serde_json::json!({}),

        // Background Workers tools (per Architecture §22)
        "worker_get_stats" => serde_json::json!({}),
        "worker_get_stats_filtered" => serde_json::json!({
            "observer_name": "ExperienceScorer"
        }),
        "worker_get_count" => serde_json::json!({}),

        // Reflection tools
        "reflection_create" => serde_json::json!({
            "title": "Test Reflection",
            "reflection_type": "analysis"
        }),
        "reflection_get_patterns" => serde_json::json!({}),
        "reflection_get_insights" => serde_json::json!({}),
        "reflection_analyze" => serde_json::json!({}),

        // Search tools
        "search_global" => serde_json::json!({
            "query": "test"
        }),
        "search_recommendations" => serde_json::json!({}),
        "search_reputation" => serde_json::json!({
            "tool_name": "store_memory"
        }),

        // Ingestor tools
        "ingestor_list_importable" => serde_json::json!({}),
        "ingestor_list_importable_recursive" => serde_json::json!({
            "recursive": true,
            "list_all": true
        }),
        "ingestor_ingest_text" => serde_json::json!({
            "file_path": env.files_folder.join("readme.txt").to_string_lossy()
        }),
        "ingestor_ingest_json" => serde_json::json!({
            "file_path": env.files_folder.join("config_files/data.json").to_string_lossy()
        }),
        "ingestor_ingest_code" => serde_json::json!({
            "file_path": env.files_folder.join("code_samples/sample.rs").to_string_lossy()
        }),
        "ingestor_list_ingested" => serde_json::json!({}),
        "ingestor_delete_blocked" => serde_json::json!({
            "file_ids": ["test_file_id"]
        }),
        "ingestor_transcribe_audio" => serde_json::json!({
            "path": env.files_folder.join("sample.mp3").to_string_lossy().to_string(),
            "store_as_memory": true
        }),

        // Hypothesis tools
        "hypothesis_record_observation" => serde_json::json!({
            "observation_type": "pattern",
            "content": "Test observation content",
            "context": "test_context"
        }),
        "hypothesis_create" => serde_json::json!({
            "statement": "Users prefer memory-first approach",
            "domain": "testing"
        }),
        "hypothesis_add_evidence" => serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000001",
            "content": "Test evidence content",
            "evidence_type": "support",
            "direction": "support",
            "strength": 0.8
        }),
        "hypothesis_get" => serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000001"
        }),
        "hypothesis_list" => serde_json::json!({}),
        "hypothesis_evaluate" => serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000001"
        }),
        "hypothesis_extract" => serde_json::json!({
            "hypothesis_id": "00000000-0000-0000-0000-000000000001",
            "knowledge_content": "Extracted knowledge"
        }),

        // Exploration tools
        "exploration_start" => serde_json::json!({
            "title": "Test Exploration",
            "purpose": "Testing purposes"
        }),
        "exploration_status" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001"
        }),
        "exploration_record_attempt" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001",
            "action": "Test attempt",
            "expected_result": "Expected outcome",
            "actual_result": "Actual outcome"
        }),
        "exploration_add_hypothesis" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001",
            "statement": "Test hypothesis",
            "initial_confidence": 0.7
        }),
        "exploration_complete" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001",
            "findings": [{"description": "Test finding", "confidence": 0.9}]
        }),
        "exploration_abandon" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001"
        }),
        "exploration_evaluate_hypothesis" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001",
            "hypothesis_id": "00000000-0000-0000-0000-000000000002",
            "result": "supported"
        }),
        "exploration_promote_finding" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001",
            "finding_id": "00000000-0000-0000-0000-000000000003"
        }),
        "exploration_pause" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001"
        }),
        "exploration_resume" => serde_json::json!({
            "exploration_id": "00000000-0000-0000-0000-000000000001"
        }),

        // Knowledge tools
        "knowledge_add" => serde_json::json!({
            "statement": "Test knowledge content"
        }),
        "knowledge_query" => serde_json::json!({
            "query": "test"
        }),
        "knowledge_mature" => serde_json::json!({
            "min_applications": 5
        }),
        "knowledge_stats" => serde_json::json!({}),
        "knowledge_record_application" => serde_json::json!({
            "knowledge_id": "00000000-0000-0000-0000-000000000000",
            "success": true
        }),

        // Planner tools
        "planner_create" => serde_json::json!({
            "description": "Test Plan"
        }),
        "planner_add_step" => serde_json::json!({
            "description": "Step 1"
        }),
        "planner_add_dependency" => serde_json::json!({
            "step_id": "00000000-0000-0000-0000-000000000001",
            "depends_on": "00000000-0000-0000-0000-000000000002"
        }),
        "planner_get" => serde_json::json!({
            "plan_id": "00000000-0000-0000-0000-000000000000"
        }),
        "planner_start" => serde_json::json!({
            "plan_id": "00000000-0000-0000-0000-000000000000"
        }),
        "planner_complete_step" => serde_json::json!({
            "plan_id": "00000000-0000-0000-0000-000000000000",
            "step_id": "00000000-0000-0000-0000-000000000001",
            "result": "Success"
        }),
        "planner_fail_step" => serde_json::json!({
            "plan_id": "00000000-0000-0000-0000-000000000000",
            "step_id": "00000000-0000-0000-0000-000000000002",
            "error": "Test failure"
        }),
        "planner_cancel" => serde_json::json!({}),
        "planner_list" => serde_json::json!({}),

        // Workflow tools
        // Note: create_workflow creates a workflow, subsequent tests need that workflow_id
        // For now, we use a static test workflow ID since data tracking isn't fully implemented
        "workflow_create" => serde_json::json!({
            "name": "Test Workflow"
        }),
        "workflow_add_step" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001",
            "name": "Step 1",
            "action": "store_memory",
            "parameters": null
        }),
        "workflow_status" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_start" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_pause" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_resume" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_cancel" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_delete" => serde_json::json!({
            "workflow_id": "00000000-0000-0000-0000-000000000001"
        }),
        "workflow_list" => serde_json::json!({}),

        // Skills tools
        "skills_register" => serde_json::json!({
            "name": "test_skill",
            "description": "A test skill",
            "category": "file_operation"
        }),
        "skills_discover" => serde_json::json!({
            "name": "discovered_skill",
            "description": "Discovered from experience",
            "category": "search",
            "source_experience_id": "00000000-0000-0000-0000-000000000000"
        }),
        "skills_get" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000"
        }),
        "skills_list" => serde_json::json!({}),
        "skills_update_mastery" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "success": true
        }),
        "skills_recommendations" => serde_json::json!({}),
        "skills_execute" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "task": "test task",
            "parameters": null
        }),
        "skills_stats" => serde_json::json!({}),
        "skills_decay" => serde_json::json!({
            "decay_rate": 0.05
        }),
        "skills_enable_disable" => serde_json::json!({
            "skill_id": "00000000-0000-0000-0000-000000000000",
            "enable": false
        }),
        "skills_search" => serde_json::json!({
            "query": "test"
        }),

        // Default: empty arguments
        _ => serde_json::json!({}),
    }
}
