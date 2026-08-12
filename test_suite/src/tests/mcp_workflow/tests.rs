//! Unit tests for MCP Workflow helpers

#[cfg(test)]
mod tests {
    use super::super::helpers::extract_content_text;
    use super::super::results::{WorkflowDiscoveryResults, WorkflowExecutionResults};

    #[test]
    fn test_extract_content_text() {
        // Test direct text field
        let result = serde_json::json!({
            "text": "Hello World"
        });
        assert_eq!(
            extract_content_text(&result),
            Some("Hello World".to_string())
        );

        // Test content array format
        let result = serde_json::json!({
            "content": [{
                "text": "Hello from content"
            }]
        });
        assert_eq!(
            extract_content_text(&result),
            Some("Hello from content".to_string())
        );
    }

    #[test]
    fn test_workflow_results_structs() {
        // Test that all result structs can be created
        let discovery = WorkflowDiscoveryResults {
            get_workflow_available: true,
            default_workflow_retrieved: true,
            purpose_based_workflows: vec!["test".to_string()],
            workflow_rules_understood: true,
        };
        assert!(discovery.get_workflow_available);

        let execution = WorkflowExecutionResults {
            create_workflow_succeeds: true,
            workflow_id_generated: Some("test-id".to_string()),
            add_step_succeeds: true,
            start_workflow_succeeds: true,
            workflow_completes: true,
            pause_resume_works: true,
        };
        assert!(execution.create_workflow_succeeds);
    }
}
