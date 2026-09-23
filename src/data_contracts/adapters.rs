/// Adapters that convert legacy subsystem types into shared data contracts
/// without losing provenance (Chapter 5.1 "API boundaries").
use crate::data_contracts::experience_record::ExperienceRecord;
use crate::data_contracts::memory_record::{MemoryKind, MemoryRecord};
use crate::data_contracts::plan_contract::{Plan, PlanStep};
use crate::database::models::MemoryCard;

/// Active reference to adapter contracts.
pub fn reference_adapters() {
    let memory_record_from_legacy_fn = memory_record_from_legacy;
    let memory_record_from_card_fn = memory_record_from_card;
    let experience_record_from_legacy_fn = experience_record_from_legacy;
    let plan_from_legacy_fn = plan_from_legacy;
    let adapter_fn_names = [
        std::any::type_name_of_val(&memory_record_from_legacy_fn),
        std::any::type_name_of_val(&memory_record_from_card_fn),
        std::any::type_name_of_val(&experience_record_from_legacy_fn),
        std::any::type_name_of_val(&plan_from_legacy_fn),
    ];
    tracing::info!(
        adapter_count = adapter_fn_names.len(),
        "Data contract adapters actively referenced"
    );
}

/// Convert a legacy memory item (`MemoryItem` from `src/memory/types.rs`)
/// into the canonical `MemoryRecord` contract.
///
/// Provenance is preserved by copying the `source` field into `metadata.source`
/// and adding the original memory ID to `metadata.provenance`.
pub fn memory_record_from_legacy(m: &crate::memory::types::MemoryItem) -> MemoryRecord {
    MemoryRecord {
        id: m.id,
        memory_type: m.memory_type.to_string(),
        kind: MemoryKind::Working,
        title: String::new(),
        content: m.content.clone(),
        summary: String::new(),
        embedding: None,
        confidence: m.confidence,
        created_at: m.created_at.timestamp(),
        updated_at: m.modified_at.timestamp(),
        relationships: m.related_ids.clone(),
        tags: m.tags.clone(),
        source: m.source.clone(),
        version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
        importance: m.importance,
        access_count: m.access_count,
        metadata: crate::data_contracts::metadata::Metadata {
            version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
            source: m.source.clone(),
            created_at: m.created_at.timestamp(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            confidence: m.confidence,
            provenance: vec![m.id.to_string()],
        },
    }
}

/// Convert a database `MemoryCard` into the canonical `MemoryRecord` contract.
///
/// The memory_type enum is mapped from the database `MemoryType` to the
/// memory module's `MemoryType` using the standard mapping table.
pub fn memory_record_from_card(card: &MemoryCard) -> MemoryRecord {
    let memory_type_str = match card.memory_type {
        crate::database::models::MemoryType::Note => "experience".to_string(),
        crate::database::models::MemoryType::Fact => "knowledge".to_string(),
        crate::database::models::MemoryType::Task => "skill".to_string(),
        crate::database::models::MemoryType::File => "workflow".to_string(),
        crate::database::models::MemoryType::Conversation => "context".to_string(),
        crate::database::models::MemoryType::Code => "skill".to_string(),
        crate::database::models::MemoryType::Decision => "experience".to_string(),
        crate::database::models::MemoryType::Event => "observation".to_string(),
        crate::database::models::MemoryType::Encounter => "observation".to_string(),
        crate::database::models::MemoryType::Experience => "experience".to_string(),
    };

    let kind = match card.layer {
        crate::database::models::MemoryLayer::Working => MemoryKind::Working,
        crate::database::models::MemoryLayer::Permanent => MemoryKind::Permanent,
    };

    MemoryRecord {
        id: card.id,
        memory_type: memory_type_str,
        kind,
        title: String::new(),
        content: card.content.clone(),
        summary: String::new(),
        embedding: None,
        confidence: card.confidence,
        created_at: card.created_at.timestamp(),
        updated_at: card.updated_at.timestamp(),
        relationships: Vec::new(),
        tags: Vec::new(),
        source: card
            .file_source
            .clone()
            .unwrap_or_else(|| "database".to_string()),
        version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
        importance: card.importance,
        access_count: card.access_count,
        metadata: crate::data_contracts::metadata::Metadata {
            version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
            source: card
                .file_source
                .clone()
                .unwrap_or_else(|| "database".to_string()),
            created_at: card.created_at.timestamp(),
            correlation_id: card.id.to_string(),
            confidence: card.confidence,
            provenance: vec![card.id.to_string()],
        },
    }
}

/// Convert a legacy experience (`Experience` from `src/experience/types/experience.rs`)
/// into the canonical `ExperienceRecord` contract.
///
/// The `plan` string is mapped to `plan_id` when non-empty; provenance
/// includes the original experience UUID.
pub fn experience_record_from_legacy(
    e: &crate::experience::types::experience::Experience,
) -> ExperienceRecord {
    ExperienceRecord {
        metadata: crate::data_contracts::metadata::Metadata {
            version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
            source: "experience_engine".to_string(),
            created_at: e.timestamp.timestamp(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            confidence: e.confidence,
            provenance: vec![e.id.to_string()],
        },
        goal: e.objective.clone(),
        plan_id: if e.plan.is_empty() {
            None
        } else {
            Some(e.plan.clone())
        },
        outcome: e.final_outcome.clone(),
        success: e.outcome.kind == crate::experience::types::outcome::OutcomeKind::Success,
        execution_time_ms: 0,
        cost: 0.0,
        confidence_change: 0.0,
        tool_usage: Vec::new(),
        lessons_learned: e.lessons_learned.clone(),
        context_signature: format!("{}:{:?}", e.objective, e.outcome.kind),
        id: e.id.to_string(),
        result: e.final_outcome.clone(),
    }
}

/// Convert a legacy planner `Plan` (`Plan` from `src/planner/engine/types.rs`)
/// into the canonical `Plan` contract.
///
/// Each `PlanStep` is mapped 1:1; `dependencies` become `params` when present,
/// and provenance carries the original plan UUID.
pub fn plan_from_legacy(p: &crate::planner::engine::types::Plan) -> Plan {
    Plan {
        metadata: crate::data_contracts::metadata::Metadata {
            version: crate::data_contracts::version::CONTRACT_VERSION.to_string(),
            source: "planner_engine".to_string(),
            created_at: p.created_at.timestamp(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            confidence: p.confidence,
            provenance: vec![p.id.clone()],
        },
        goal: p.goal.clone(),
        steps: p
            .steps
            .iter()
            .map(|s| PlanStep {
                id: s.id.clone(),
                action: s.action.clone(),
                params: serde_json::json!({
                    "description": s.description,
                    "dependencies": s.dependencies,
                    "status": format!("{:?}", s.status),
                }),
                dependencies: s.dependencies.clone(),
                status: format!("{:?}", s.status),
            })
            .collect(),
        confidence: p.confidence,
        objectives: Vec::new(),
        required_skills: Vec::new(),
        estimated_cost: 0.0,
        alternative_branches: Vec::new(),
        knowledge_used: p.knowledge_used.clone(),
        experiences_used: p.experiences_used.clone(),
    }
}
