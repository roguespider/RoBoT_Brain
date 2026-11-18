// src/api/bridge/tools/knowledge/handler.rs
//! Knowledge tool handler functions

use std::sync::Arc;
use uuid::Uuid;

use crate::api::bridge::tools::ToolOutput;
use crate::knowledge::{
    KnowledgeItem, KnowledgeQuery, KnowledgeResult, KnowledgeStore, apply_query, rank_items,
    types::{KnowledgeConfidence, KnowledgeSource, KnowledgeStatus, KnowledgeType},
};

/// Execute add knowledge tool
pub async fn execute_add_knowledge(
    input: super::types::AddKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let knowledge_type = match input.knowledge_type.as_deref() {
        Some("fact") => KnowledgeType::Fact,
        Some("procedure") => KnowledgeType::Procedure,
        Some("causality") => KnowledgeType::Causality,
        Some("pattern") => KnowledgeType::Pattern,
        Some("insight") => KnowledgeType::Insight,
        Some("rule") => KnowledgeType::Rule,
        Some("concept") => KnowledgeType::Concept,
        Some(t) => KnowledgeType::Custom(t.to_string()),
        None => KnowledgeType::Insight,
    };

    let source = match input.source.as_deref() {
        Some("user") => KnowledgeSource::User,
        Some("tool") => KnowledgeSource::Tool,
        Some("planner") => KnowledgeSource::Planner,
        Some("reflection") => KnowledgeSource::Reflection(Uuid::new_v4()),
        Some("hypothesis") => KnowledgeSource::Hypothesis(Uuid::new_v4()),
        Some("exploration") => KnowledgeSource::Exploration(Uuid::new_v4()),
        Some(s) => KnowledgeSource::External(s.to_string()),
        None => KnowledgeSource::User,
    };

    let confidence = input.confidence.unwrap_or(0.5);

    let item = KnowledgeItem {
        id: Uuid::new_v4(),
        statement: input.statement,
        knowledge_type,
        confidence: KnowledgeConfidence::new(confidence),
        status: KnowledgeStatus::Active,
        source,
        supporting_evidence: Vec::new(),
        contradicting_evidence: Vec::new(),
        relations: Vec::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        success_count: 0,
        failure_count: 0,
        tags: input.tags.unwrap_or_default(),
        metadata: std::collections::HashMap::new(),
    };

    let id = knowledge.add(item);

    ToolOutput::success(serde_json::json!({
        "status": "added",
        "id": id.to_string(),
        "knowledge_id": id.to_string(),
        "message": "Knowledge added successfully"
    }))
}

/// Execute query knowledge tool
pub async fn execute_query_knowledge(
    input: super::types::QueryKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let query_text = input.query.clone();

    let ktype = input.knowledge_type.as_ref().map(|t| match t.as_str() {
        "fact" => KnowledgeType::Fact,
        "procedure" => KnowledgeType::Procedure,
        "causality" => KnowledgeType::Causality,
        "pattern" => KnowledgeType::Pattern,
        "insight" => KnowledgeType::Insight,
        "rule" => KnowledgeType::Rule,
        "concept" => KnowledgeType::Concept,
        t => KnowledgeType::Custom(t.to_string()),
    });

    // Use the by_type index when a type filter is present (avoids a full scan),
    // otherwise fall back to the full item set.
    let all_items = match &ktype {
        Some(kt) => knowledge.get_by_type(kt),
        None => knowledge.get_all(),
    };

    let query = KnowledgeQuery {
        text: Some(query_text.clone()),
        knowledge_type: ktype,
        status: None,
        min_confidence: input.min_confidence,
        tags: None,
        mature_only: input.mature_only.unwrap_or(false),
        include_related: false,
        limit: input.limit,
    };

    let filtered = apply_query(&all_items, &query);
    let ranked = rank_items(filtered, &query);

    let result = KnowledgeResult::new(ranked, query.clone());

    let items_json: Vec<serde_json::Value> = result
        .items
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id.to_string(),
                "statement": item.statement,
                "type": format!("{:?}", item.knowledge_type),
                "confidence": item.overall_confidence(),
                "status": format!("{:?}", item.status),
                "tags": item.tags,
                "success_count": item.success_count,
                "failure_count": item.failure_count,
            })
        })
        .collect();

    let best_match = result.best().map(|item| {
        serde_json::json!({
            "id": item.id.to_string(),
            "statement": item.statement,
            "confidence": item.overall_confidence(),
        })
    });

    ToolOutput::success(serde_json::json!({
        "items": items_json,
        "total_matches": result.total_matches,
        "returned": result.items.len(),
        "query": query_text,
        "best_match": best_match,
    }))
}

/// Execute record knowledge application tool
pub async fn execute_record_knowledge_application(
    input: super::types::RecordKnowledgeApplicationInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(id) => id,
        Err(_) => return ToolOutput::error("Invalid knowledge ID format"),
    };

    let success = if input.success {
        knowledge.record_success(id)
    } else {
        knowledge.record_failure(id)
    };

    if success {
        ToolOutput::success(serde_json::json!({
            "status": "recorded",
            "knowledge_id": input.knowledge_id,
            "result": if input.success { "success_recorded" } else { "failure_recorded" },
        }))
    } else {
        ToolOutput::error(format!("Knowledge item {} not found", input.knowledge_id))
    }
}

/// Execute get knowledge stats tool
pub async fn execute_get_knowledge_stats(
    _: super::types::GetKnowledgeStatsInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let stats = knowledge.stats();

    ToolOutput::success(serde_json::json!({
        "total": stats.total,
        "active": stats.active,
        "mature": stats.mature,
        "needs_review": stats.needs_review,
        "average_confidence": stats.average_confidence,
    }))
}

/// Execute get mature knowledge tool
pub async fn execute_get_mature_knowledge(
    input: super::types::GetMatureKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let mut mature = knowledge.get_mature();

    if let Some(l) = input.limit {
        mature.truncate(l);
    }

    let items_json: Vec<serde_json::Value> = mature
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id.to_string(),
                "statement": item.statement,
                "type": format!("{:?}", item.knowledge_type),
                "confidence": item.overall_confidence(),
                "tags": item.tags,
                "success_count": item.success_count,
            })
        })
        .collect();

    ToolOutput::success(serde_json::json!({
        "items": items_json,
        "count": items_json.len(),
    }))
}

/// Execute update knowledge tool
pub async fn execute_update_knowledge(
    input: super::types::UpdateKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let mut item = match knowledge.get(id) {
        Some(it) => it,
        None => return ToolOutput::error("Knowledge item not found"),
    };
    if let Some(stmt) = input.statement {
        item.statement = stmt;
    }
    if let Some(conf) = input.confidence {
        item.confidence = KnowledgeConfidence::new(conf);
    }
    if let Some(tags) = input.tags {
        item.tags = tags;
    }
    let ok = knowledge.update(item);
    ToolOutput::success(serde_json::json!({
        "status": if ok { "updated" } else { "failed" },
        "knowledge_id": id.to_string(),
    }))
}

/// Execute delete knowledge tool
pub async fn execute_delete_knowledge(
    input: super::types::DeleteKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let ok = knowledge.delete(id);
    ToolOutput::success(serde_json::json!({
        "status": if ok { "deleted" } else { "not_found" },
        "knowledge_id": id.to_string(),
    }))
}

/// Execute get related knowledge tool
pub async fn execute_get_related_knowledge(
    input: super::types::GetRelatedKnowledgeInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let related = knowledge.get_related(id);
    let items_json: Vec<serde_json::Value> = related
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id.to_string(),
                "statement": item.statement,
                "type": format!("{:?}", item.knowledge_type),
                "confidence": item.overall_confidence(),
            })
        })
        .collect();
    ToolOutput::success(serde_json::json!({
        "items": items_json,
        "count": items_json.len(),
    }))
}

/// Execute validate knowledge dependencies tool
pub async fn execute_validate_knowledge_dependencies(
    _: super::types::ValidateKnowledgeDependenciesInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let validations = knowledge.validate_all_dependencies();
    let results_json: Vec<serde_json::Value> = validations
        .iter()
        .map(|v| {
            serde_json::json!({
                "knowledge_id": v.knowledge_id.to_string(),
                "unsatisfied_count": v.check_result.unsatisfied.len(),
                "conflicts_count": v.check_result.conflicts.len(),
                "unsatisfied": v.check_result.unsatisfied.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
                "conflicts": v.check_result.conflicts.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
            })
        })
        .collect();
    ToolOutput::success(serde_json::json!({
        "items_with_issues": results_json.len(),
        "validations": results_json,
    }))
}

/// Execute bump knowledge version tool
pub async fn execute_bump_knowledge_version(
    input: super::types::BumpKnowledgeVersionInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let bump_type = match input.bump_type.as_deref().unwrap_or("minor") {
        "major" => crate::knowledge::store::VersionBumpType::Major,
        "patch" => crate::knowledge::store::VersionBumpType::Patch,
        _ => crate::knowledge::store::VersionBumpType::Minor,
    };
    if let Some(init_ver) = input.initial_version.as_deref()
        && knowledge.get_version_info(&id).is_none()
    {
        knowledge.init_version(id, init_ver);
    }
    let ok = knowledge.bump_version(&id, bump_type);
    let new_version = knowledge.get_version_info(&id);
    ToolOutput::success(serde_json::json!({
        "status": if ok { "bumped" } else { "failed" },
        "knowledge_id": id.to_string(),
        "current_version": new_version.map(|v| v.current_version).unwrap_or_default(),
    }))
}

/// Execute set knowledge status tool
pub async fn execute_set_knowledge_status(
    input: super::types::SetKnowledgeStatusInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let ok = match input.action.as_str() {
        "activate" => knowledge.activate(id),
        "suspend" => knowledge.suspend(id),
        "disprove" => knowledge.disprove(id),
        other => return ToolOutput::error(format!("Unknown action: {other}")),
    };
    ToolOutput::success(serde_json::json!({
        "status": if ok { "ok" } else { "failed" },
        "knowledge_id": id.to_string(),
        "action": input.action,
    }))
}

/// Execute manage knowledge dependency tool
pub async fn execute_manage_knowledge_dependency(
    input: super::types::ManageKnowledgeDependencyInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    match input.action.as_str() {
        "add" => {
            let dep_id_str = match input.depends_on_id.as_deref() {
                Some(s) => s,
                None => {
                    return ToolOutput::error(
                        "depends_on_id is required for add action".to_string(),
                    );
                }
            };
            let dep_id = match Uuid::parse_str(dep_id_str) {
                Ok(u) => u,
                Err(e) => return ToolOutput::error(format!("Invalid depends_on_id: {e}")),
            };
            let dep_type = match input.dependency_type.as_deref().unwrap_or("required") {
                "optional" => crate::knowledge::types::DependencyType::Optional,
                "conflict" => crate::knowledge::types::DependencyType::Conflict,
                "replaces" => crate::knowledge::types::DependencyType::Replaces,
                _ => crate::knowledge::types::DependencyType::Required,
            };
            let ok =
                knowledge.add_dependency(id, dep_id, dep_type, input.version_constraint.clone());
            ToolOutput::success(serde_json::json!({
                "status": if ok { "added" } else { "failed" },
                "knowledge_id": id.to_string(),
                "depends_on_id": input.depends_on_id,
            }))
        }
        "remove" => {
            let dep_id_str = match input.depends_on_id.as_deref() {
                Some(s) => s,
                None => {
                    return ToolOutput::error(
                        "depends_on_id is required for remove action".to_string(),
                    );
                }
            };
            let dep_id = match Uuid::parse_str(dep_id_str) {
                Ok(u) => u,
                Err(e) => return ToolOutput::error(format!("Invalid depends_on_id: {e}")),
            };
            let ok = knowledge.remove_dependency(&id, &dep_id);
            ToolOutput::success(serde_json::json!({
                "status": if ok { "removed" } else { "failed" },
                "knowledge_id": id.to_string(),
                "depends_on_id": input.depends_on_id,
            }))
        }
        "get" => {
            let deps = knowledge.get_dependencies(&id);
            let deps_json: Vec<serde_json::Value> = deps
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "depends_on_id": d.depends_on_id.to_string(),
                        "dependency_type": format!("{:?}", d.dependency_type),
                        "version_constraint": d.version_constraint,
                    })
                })
                .collect();
            ToolOutput::success(serde_json::json!({
                "knowledge_id": id.to_string(),
                "dependencies": deps_json,
                "count": deps_json.len(),
            }))
        }
        "impact" => {
            let impact = knowledge.get_impact_set(&id);
            let impact_ids: Vec<String> = impact.iter().map(|u| u.to_string()).collect();
            ToolOutput::success(serde_json::json!({
                "knowledge_id": id.to_string(),
                "impact_set": impact_ids,
                "count": impact_ids.len(),
            }))
        }
        other => ToolOutput::error(format!("Unknown action: {other}")),
    }
}

/// Execute add knowledge relation tool
pub async fn execute_add_knowledge_relation(
    input: super::types::AddKnowledgeRelationInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let id = match Uuid::parse_str(&input.knowledge_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid knowledge_id: {e}")),
    };
    let related_id = match Uuid::parse_str(&input.related_id) {
        Ok(u) => u,
        Err(e) => return ToolOutput::error(format!("Invalid related_id: {e}")),
    };
    let relation_type = match input.relation_type.as_deref().unwrap_or("related") {
        "supports" => crate::knowledge::types::RelationType::Supports,
        "contradicts" => crate::knowledge::types::RelationType::Contradicts,
        "specializes" => crate::knowledge::types::RelationType::Specializes,
        "generalizes" => crate::knowledge::types::RelationType::Generalizes,
        "prerequisite" => crate::knowledge::types::RelationType::Prerequisite,
        _ => crate::knowledge::types::RelationType::Related,
    };
    let confidence = input.confidence.unwrap_or(0.5);
    let ok = knowledge.add_relation(id, related_id, relation_type, confidence);
    ToolOutput::success(serde_json::json!({
        "status": if ok { "added" } else { "failed" },
        "knowledge_id": id.to_string(),
        "related_id": related_id.to_string(),
        "relation_type": input.relation_type.unwrap_or_else(|| "related".to_string()),
        "confidence": confidence,
    }))
}

/// Execute search knowledge by tag tool
pub async fn execute_search_knowledge_by_tag(
    input: super::types::SearchKnowledgeByTagInput,
    knowledge: &Arc<KnowledgeStore>,
) -> ToolOutput {
    let items = match input.tag.as_deref() {
        Some(tag) => knowledge.get_by_tag(tag),
        None => knowledge.get_needing_review(),
    };
    let items_json: Vec<serde_json::Value> = items
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id.to_string(),
                "statement": item.statement,
                "type": format!("{:?}", item.knowledge_type),
                "confidence": item.overall_confidence(),
                "status": format!("{:?}", item.status),
                "tags": item.tags,
            })
        })
        .collect();
    let mode = if input.tag.is_some() {
        "by_tag"
    } else {
        "needing_review"
    };
    ToolOutput::success(serde_json::json!({
        "mode": mode,
        "items": items_json,
        "count": items_json.len(),
    }))
}
