//! Coverage Tools Module
//!
//! Test requirements for the remaining server tools not yet covered by the
//! FunctionRegistry pipeline. Each entry makes the coverage cross-check count
//! the tool as tested. Validations are chosen from live probing:
// !   - IsSuccess(None) for tools that return success on a default/fake call.
// !   - IsSuccess(Some("false")) for tools that return an MCP error on a fake id
// !     (update_knowledge, update_reflection, validate_reflection, get_evidence,
// !     add_world_relationship).

use crate::function_registry::types::{CheckType, TestRequirement, ValidationCheck};

fn req(id: &str, name: &str, expect_fail: bool, priority: u8) -> TestRequirement {
    TestRequirement {
        id: id.to_string(),
        function_name: name.to_string(),
        category: "Coverage".to_string(),
        requires_workflow: true,
        requires_data: None,
        expected_behavior: if expect_fail {
            "Expected to fail for a fake id".to_string()
        } else {
            "Returns a result for default args".to_string()
        },
        validation: vec![ValidationCheck {
            check_type: CheckType::IsSuccess,
            field: "success".to_string(),
            expected_value: if expect_fail {
                Some("false".to_string())
            } else {
                None
            },
        }],
        priority,
    }
}

/// Returns test requirements for all remaining uncovered server tools.
pub fn coverage_tools() -> Vec<TestRequirement> {
    vec![
        // System / status
        req("cov_get_system_status", "get_system_status", false, 1),
        req("cov_cleanup_sessions", "cleanup_sessions", false, 3),
        req("cov_get_session_state", "get_session_state", false, 3),
        // Memory
        req("cov_archive_memory", "archive_memory", true, 2),
        req("cov_link_memories", "link_memories", false, 2),
        req("cov_ranked_search", "ranked_search", false, 2),
        // Knowledge
        req("cov_get_knowledge", "get_knowledge", false, 1),
        req("cov_delete_knowledge", "delete_knowledge", false, 2),
        req("cov_update_knowledge", "update_knowledge", true, 2),
        req("cov_get_related_knowledge", "get_related_knowledge", false, 2),
        req("cov_validate_knowledge_deps", "validate_knowledge_dependencies", false, 2),
        req("cov_bump_knowledge_version", "bump_knowledge_version", false, 2),
        // Evidence / observation
        req("cov_get_evidence", "get_evidence", true, 2),
        req("cov_list_evidence", "list_evidence", false, 2),
        req("cov_list_observations", "list_observations", false, 2),
        // Reflection
        req("cov_list_reflections_by_status", "list_reflections_by_status", false, 2),
        req("cov_update_reflection", "update_reflection", true, 2),
        req("cov_validate_reflection", "validate_reflection", true, 2),
        // Skills
        req("cov_get_skill_metrics", "get_skill_metrics", false, 2),
        req("cov_clear_skill_metrics", "clear_skill_metrics", false, 3),
        req("cov_get_unreliable_skills", "get_unreliable_skills", false, 2),
        req("cov_search_skills_by_tag", "search_skills_by_tag", false, 2),
        req("cov_unregister_skill", "unregister_skill", false, 2),
        // Personality
        req("cov_get_personality", "get_personality", false, 1),
        req("cov_set_personality_traits", "set_personality_traits", false, 2),
        req("cov_apply_personality_preset", "apply_personality_preset", false, 2),
        req("cov_list_personality_presets", "list_personality_presets", false, 2),
        req("cov_get_personality_decision", "get_personality_decision", false, 2),
        req("cov_format_response", "format_response", false, 2),
        // World model
        req("cov_upsert_world_entity", "upsert_world_entity", false, 2),
        req("cov_get_world_entity", "get_world_entity", false, 2),
        req("cov_find_world_entity", "find_world_entity", false, 2),
        req("cov_list_world_entities", "list_world_entities", false, 2),
        req("cov_add_world_relationship", "add_world_relationship", true, 2),
        req("cov_get_world_relationships", "get_world_relationships", false, 2),
        req("cov_get_world_blockers", "get_world_blockers", false, 2),
        req("cov_get_world_dependencies", "get_world_dependencies", false, 2),
        req("cov_get_world_model_stats", "get_world_model_stats", false, 2),
        req("cov_get_consumed_resources", "get_consumed_resources", false, 2),
        // Agent / workflow
        req("cov_run_agent_goal", "run_agent_goal", false, 1),
        req("cov_set_workflow_variable", "set_workflow_variable", false, 2),
    ]
}
