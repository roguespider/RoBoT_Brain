//! Data contract chain test — T2-132 verification.
use robot_brain::pipeline::{
    LifecyclePipeline, LifecycleStep, run_pipeline, validate_contract_chain,
};

#[test]
fn test_contract_chain_valid() {
    let pipeline = LifecyclePipeline {
        steps: vec![
            LifecycleStep::Observation,
            LifecycleStep::ContextConstruction,
            LifecycleStep::MemoryRetrieval,
        ],
        correlation_id: "test-corr-003".to_string(),
    };
    let trace = run_pipeline(&pipeline).expect("pipeline runs");
    assert!(
        validate_contract_chain(&trace),
        "contract chain should be valid"
    );
}
