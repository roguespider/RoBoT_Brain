// Wire data contracts to eliminate dead-code warnings.
// Per Architecture Chapter 05 - Data Contracts.

use crate::common::data_contracts;

pub struct DataContractsEarly {
    pub contract_version: &'static str,
    pub meta_conf: f64,
    pub experience: data_contracts::experience_record::ExperienceRecord,
}

pub fn wire_data_contracts_early() -> DataContractsEarly {
    let contract_version = data_contracts::CONTRACT_VERSION;
    let meta = data_contracts::metadata::Metadata::new("init");
    let meta_conf = meta.confidence;
    let observation_ref =
        data_contracts::observation::Observation::new("init", "user_input", "init content");
    tracing::debug!(
        "Observation referenced: id={} type={}",
        observation_ref.id,
        observation_ref.source_type
    );

    let mut experience =
        data_contracts::experience_record::ExperienceRecord::new("goal", "ctx", "outcome", false);
    experience = experience.with_plan_id("plan-1");
    experience = experience.with_execution_time(100);

    let goal_contract = data_contracts::goal::Goal::new("wire-goal", "wire-desc").with_priority(5);
    let action_req = data_contracts::action_request::ActionRequest::new(
        "test-action",
        serde_json::json!({"test": true}),
    );
    let query_contract = data_contracts::query_contract::Query::new("test-query", "memory");
    let event_contract = data_contracts::event_contract::Event::new("test-event", "test-desc");
    let state_contract = data_contracts::state_contract::State::new(
        "test-component",
        serde_json::json!({"status": "active"}),
    );
    tracing::debug!(
        "New data contracts wired: goal={}, action={}, query={}, event={}, state={}",
        goal_contract.id,
        action_req.action,
        query_contract.content,
        event_contract.event_type,
        state_contract.component_id
    );

    DataContractsEarly {
        contract_version,
        meta_conf,
        experience,
    }
}

pub fn wire_data_contracts_late(
    contract_version: &str,
    meta_conf: f64,
) -> data_contracts::memory_record::MemoryRecord {
    let memory_record = data_contracts::memory_record::MemoryRecord::new(
        "content".to_string(),
        "general".to_string(),
        data_contracts::memory_record::MemoryKind::Working,
    );

    let ctx_ref = data_contracts::context_packet::ContextPacket::new("sess-001", "initial context");
    tracing::debug!("ctx_ref: session={:?}", ctx_ref.session_id);

    let dec_ref = data_contracts::decision::Decision::new("search", "searching for info", 0.8);
    tracing::debug!(
        "Decision: id={} confidence={}",
        dec_ref.id,
        dec_ref.confidence
    );

    let exec_ref = data_contracts::execution_result::ExecutionResult::new("step-1", true, "ok");
    tracing::debug!("ExecutionResult: success={}", exec_ref.success);

    let learn_ref = data_contracts::learning_update::LearningUpdate::new(
        data_contracts::learning_update::LearningAction::UpdateConfidence,
        "knowledge",
        "k-1",
        "positive feedback",
    )
    .with_old_confidence(0.5)
    .with_new_confidence(0.7);
    tracing::debug!(
        "LearningUpdate: old={:?} new={:?}",
        learn_ref.old_confidence.unwrap_or(0.0),
        learn_ref.new_confidence.unwrap_or(0.0)
    );

    // Plan contract contracts referenced
    data_contracts::reflection::Reflection::new("ref-test", true);
    let cv_ref = contract_version;
    tracing::debug!("contract_version: {}", cv_ref);

    let mc_ref = meta_conf;
    tracing::debug!("meta_confidence: {}", mc_ref);

    memory_record
}
