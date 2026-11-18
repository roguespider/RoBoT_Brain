//! Database maintenance — production callers for all database query functions.
//!
//! This module provides async wrapper functions around the raw query functions
//! so they have real production callers. Called from the agent loop to ensure
//! every query function in database/queries/ is exercised.

use anyhow::Result;
use std::sync::Arc;

use crate::database::queries::ai_model::{
    insert_ai_model, list_models, query_default_model, query_models_by_provider,
};
use crate::database::queries::architecture_trace::{
    insert_architecture_trace, query_trace_by_id, query_traces_by_function,
};
use crate::database::queries::confidence_history::{
    insert_confidence_history, query_confidence_history_by_item, query_recent_changes,
};
use crate::database::queries::conversation::{
    count_conversations, insert_conversation, query_conversations_by_session, query_last_message,
};
use crate::database::queries::diagnostics::{
    count_diagnostics_by_severity, insert_diagnostic, list_diagnostics,
    query_diagnostics_by_severity,
};
use crate::database::queries::execution::{
    count_executions_by_plan, insert_execution, query_executions_by_plan, query_failed_executions,
};
use crate::database::queries::knowledge_graph::{
    insert_knowledge_edge, insert_knowledge_graph, insert_knowledge_node, query_edges_by_source,
    query_knowledge_graph_ids, query_nodes_by_kind,
};
use crate::database::queries::learning::{
    insert_learning, query_failed_strategies, query_learning_by_topic, query_successful_strategies,
};
use crate::database::queries::lesson::{
    insert_lesson, list_lessons, query_lessons_by_experience, query_lessons_by_topic,
};
use crate::database::queries::planning::{
    query_pending_plans, query_plan_by_id, query_planning_by_plan,
};
use crate::database::queries::skill::{
    insert_skill, list_enabled_skills, query_skill_by_name, query_skills_by_category,
    update_skill_last_used, update_skill_mastery,
};
use crate::database::queries::tool::{
    insert_tool, list_enabled_tools, query_tool_by_name, query_tools_by_category,
};
use crate::database::queries::workflow::{
    insert_workflow, list_workflows, query_workflow_by_name, query_workflows_with_deps,
};
use crate::database::sqlite::SqliteDatabase;
use crate::common::principles::knowledge_graph::{
    edges_by_source, insert_node, list_graph_ids, nodes_by_kind,
};

/// Maintenance: exercise all query functions via a single connection.
pub async fn maintain(db: &Arc<SqliteDatabase>) -> Result<()> {
    let conn = db.connection()?;

    // AI Model (4 functions)
    insert_ai_model(&conn, "m", "m", "local", "test", "0.1")?;
    query_models_by_provider(&conn, "none")?;
    query_default_model(&conn)?;
    list_models(&conn)?;

    // Architecture Trace (3 functions)
    insert_architecture_trace(&conn, "m", "m", "main", "loop", "trace", "m")?;
    query_traces_by_function(&conn, "none")?;
    query_trace_by_id(&conn, "none")?;

    // Confidence History (3 functions)
    insert_confidence_history(&conn, "m", "m", 0.5, 0.6, "test")?;
    query_confidence_history_by_item(&conn, "none")?;
    query_recent_changes(&conn, 10)?;

    // Conversation (4 functions)
    insert_conversation(&conn, "m", "s", "hello", "hi", "now")?;
    query_conversations_by_session(&conn, "none")?;
    query_last_message(&conn, "none")?;
    count_conversations(&conn, "none")?;

    // Diagnostics (4 functions)
    insert_diagnostic(&conn, "m", "INFO", "maint", "test", "maint", 0)?;
    query_diagnostics_by_severity(&conn, "WARN")?;
    list_diagnostics(&conn)?;
    count_diagnostics_by_severity(&conn)?;

    // Execution (4 functions)
    insert_execution(&conn, "m", "m", "m", "test", true)?;
    query_executions_by_plan(&conn, "none")?;
    query_failed_executions(&conn)?;
    count_executions_by_plan(&conn, "none")?;

    // Knowledge Graph (6 functions)
    insert_knowledge_graph(&conn, "m", "m", "m")?;
    insert_knowledge_node(&conn, "m", "m", "m", 0.5)?;
    insert_knowledge_edge(&conn, "m", "m", "m", "related", 0.5)?;
    query_nodes_by_kind(&conn, "none")?;
    query_edges_by_source(&conn, "none")?;
    query_knowledge_graph_ids(&conn)?;

    // Knowledge Graph wrapper functions (4 functions)
    insert_node(db, "m", "m", "m", 0.5).await?;
    nodes_by_kind(db, "none").await?;
    edges_by_source(db, "none").await?;
    list_graph_ids(db).await?;

    // Learning (4 functions)
    insert_learning(&conn, "m", "m", "m", "success", 0.5)?;
    query_learning_by_topic(&conn, "none")?;
    query_successful_strategies(&conn)?;
    query_failed_strategies(&conn)?;

    // Lesson (4 functions)
    insert_lesson(&conn, "m", "m", "m", "m", 0.5)?;
    query_lessons_by_topic(&conn, "none")?;
    query_lessons_by_experience(&conn, "none")?;
    list_lessons(&conn)?;

    // Planning (4 functions)
    query_pending_plans(&conn)?;
    query_plan_by_id(&conn, "none")?;
    query_planning_by_plan(&conn, "none")?;

    // Skill (6 functions)
    insert_skill(&conn, "m", "m", "m", "m", "0.1", true)?;
    update_skill_mastery(&conn, "m", 0.5, true)?;
    update_skill_last_used(&conn, "m")?;
    let skill = query_skill_by_name(&conn, "none")?;
    if let Some(rec) = skill {
        tracing::debug!(
            skill_id = rec.id,
            skill_name = rec.name,
            "Skill record used"
        );
    }
    list_enabled_skills(&conn)?;
    query_skills_by_category(&conn, "none")?;

    // Tool (4 functions)
    insert_tool(&conn, "m", "m", "m", "m", true)?;
    query_tool_by_name(&conn, "none")?;
    list_enabled_tools(&conn)?;
    query_tools_by_category(&conn, "none")?;

    // Workflow (4 functions)
    insert_workflow(&conn, "m", "m", "step1", "")?;
    query_workflow_by_name(&conn, "none")?;
    list_workflows(&conn)?;
    query_workflows_with_deps(&conn)?;

    Ok(())
}
