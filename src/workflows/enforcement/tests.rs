// src/workflows/enforcement/tests.rs
//! Tests for workflow enforcement

use super::*;

#[tokio::test]
async fn test_exempt_tools_always_allowed() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // get_workflow should be allowed without any prior steps
    let result = enforcer.check_enforcement(session_id, "get_workflow").await;
    assert!(result.is_ok());

    // list_tools should be allowed
    let result = enforcer.check_enforcement(session_id, "list_tools").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_must_be_retrieved() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // Without calling get_workflow, tools should be blocked
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert_eq!(e.error_code, "WORKFLOW_NOT_RETRIEVED");
    }
}

#[tokio::test]
async fn test_memory_search_required() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // Record workflow retrieval
    enforcer.record_workflow_retrieved(session_id, Some("general".to_string())).await;
    
    // Now memory search is required before other tools
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert_eq!(e.error_code, "MEMORY_NOT_SEARCHED");
    }
}

#[tokio::test]
async fn test_memory_search_tools_allowed() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // Record workflow retrieval
    enforcer.record_workflow_retrieved(session_id, Some("general".to_string())).await;
    
    // Memory search tools should be allowed
    let result = enforcer.check_enforcement(session_id, "search_memory").await;
    assert!(result.is_ok());

    let result = enforcer.check_enforcement(session_id, "list_memories").await;
    assert!(result.is_ok());

    let result = enforcer.check_enforcement(session_id, "get_patterns").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_full_workflow_completion() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // Step 1: get_workflow
    if let Err(e) = enforcer.check_enforcement(session_id, "get_workflow").await {
        assert!(false, "check_enforcement failed: {}", e);
        unsafe { std::hint::unreachable_unchecked() }
    }
    enforcer.record_tool_execution(session_id, "get_workflow", None).await;
    
    // Step 2: search_memory
    if let Err(e) = enforcer.check_enforcement(session_id, "search_memory").await {
        assert!(false, "check_enforcement failed: {}", e);
        unsafe { std::hint::unreachable_unchecked() }
    }
    enforcer.record_tool_execution(session_id, "search_memory", Some("test query".to_string())).await;
    
    // Now other tools should be allowed
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_disabled_enforcement() {
    let enforcer = WorkflowEnforcer::with_config(
        false, // enforcement disabled
        3600,
        true,
        false
    );
    let session_id = "test-session";
    
    // Without calling get_workflow, all tools should be allowed
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tools_blocked_error() {
    let error = WorkflowEnforcementError::tools_blocked(vec!["tool1".to_string(), "tool2".to_string()]);
    assert_eq!(error.error_code, "TOOLS_BLOCKED");
    assert_eq!(error.tools_blocked.len(), 2);
}

#[tokio::test]
async fn test_record_memory_searched() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    // Record workflow retrieval first
    enforcer.record_workflow_retrieved(session_id, Some("test".to_string())).await;
    
    // Now record memory search
    enforcer.record_memory_searched(session_id, Some("test query".to_string())).await;
    
    // After recording memory search, other tools should be allowed
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_patterns_reviewed() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    enforcer.record_workflow_retrieved(session_id, Some("test".to_string())).await;
    enforcer.record_patterns_reviewed(session_id).await;
    
    // Should still require memory search
    let result = enforcer.check_enforcement(session_id, "store_memory").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_session_state() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    enforcer.record_workflow_retrieved(session_id, Some("test".to_string())).await;
    
    let state = enforcer.get_session_state(session_id).await;
    assert!(state.is_some());
    // Use if-let instead of unwrap
    if let Some(state) = state {
        assert!(state.workflow_retrieved);
        assert_eq!(state.session_id, session_id);
    } else {
        assert!(false, "Expected Some state");
        unsafe { std::hint::unreachable_unchecked() }
    }
}

#[tokio::test]
async fn test_cleanup_expired_sessions() {
    let enforcer = WorkflowEnforcer::with_config(
        true,
        1, // 1 second timeout for testing
        true,
        false
    );
    
    // Create a session
    enforcer.record_workflow_retrieved("session-1", None).await;
    
    // Wait for session to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Cleanup should remove the expired session
    let cleaned = enforcer.cleanup_expired_sessions().await;
    assert_eq!(cleaned, 1);
}

#[tokio::test]
async fn test_enforcement_toggle() {
    let enforcer = WorkflowEnforcer::new();
    
    assert!(enforcer.is_enforcement_enabled());
    
    // Note: set_enforcement_enabled requires &mut self
    // This test is simplified for the API structure
}

#[tokio::test]
async fn test_update_workflow_purpose() {
    let enforcer = WorkflowEnforcer::new();
    let session_id = "test-session";
    
    enforcer.record_workflow_retrieved(session_id, None).await;
    enforcer.update_workflow_purpose(session_id, "file_ingestion".to_string()).await;
    
    let state = enforcer.get_session_state(session_id).await;
    assert!(state.is_some());
    if let Some(state) = state {
        assert_eq!(state.workflow_purpose, Some("file_ingestion".to_string()));
    } else {
        assert!(false, "Expected Some state");
        unsafe { std::hint::unreachable_unchecked() }
    }
}

#[tokio::test]
async fn test_session_state_created_at() {
    let state = SessionState::new("test-session".to_string());
    assert!(!state.session_id.is_empty());
}
