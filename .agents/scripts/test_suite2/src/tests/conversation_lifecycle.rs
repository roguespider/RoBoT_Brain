//! Conversation lifecycle test — T2-133 verification.
use robot_brain::conversation::ConversationEngine;

#[test]
fn test_full_lifecycle_advances_states() {
    let mut engine = ConversationEngine::new();
    let identity = engine.start_session("conv-001", "sess-001");
    assert_eq!(identity.conversation_id, "conv-001");
    let state = engine.process_full_lifecycle("conv-001", "input-001");
    assert!(state.is_some());
    assert_eq!(
        state.unwrap(),
        robot_brain::conversation::ConversationState::Completed
    );
}
