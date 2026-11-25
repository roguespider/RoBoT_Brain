// src/bridge/tools/search/research.rs
// Research pipeline functions: research, quick_research, deep_research, find_error_resolution.

use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::agent::decision::{CascadeDeps, ResearchCascade};
use crate::api::bridge::tools::ToolOutput;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::types::Experience;
use crate::knowledge::store::KnowledgeStore;
use crate::knowledge::world_model::store::WorldModel;
use crate::memory::permanent::PermanentMemory;
use crate::memory::repository::SqliteMemoryRepository;
use crate::planning::engine::types::Plan;
use crate::research;
use crate::scheduler::cooboploop::capability::CapabilityRegistry;

/// Build cascade dependencies from available sources and check if research should proceed.
/// Per research_engine.md Rule 1: "Research is NEVER triggered if any internal source
/// has confidence >= 0.7. The cascade must run in full order."
/// Returns `true` if research should proceed (all tiers 1-8 failed), `false` if research
/// should be skipped because a tier 1-8 internal source can answer the query.
async fn should_bypass_cascade(database: &Arc<SqliteDatabase>, query: &str) -> bool {
    let permanent_memory = PermanentMemory::new_with_db(10000, database.clone());
    let knowledge_store = KnowledgeStore::new(10000);
    let capability_registry = StdMutex::new(CapabilityRegistry::new());
    let world_model = WorldModel::new();
    let reflection_engine = Mutex::new(ReflectionEngine::new());
    let hypothesis_engine = HypothesisEngine::new();
    let current_plan: Option<Plan> = None;
    let latest_experience: Option<Experience> = None;

    let deps = CascadeDeps {
        capability_registry: Some(&capability_registry),
        knowledge_store: &knowledge_store,
        latest_experience: latest_experience.as_ref(),
        reflection_engine: &reflection_engine,
        current_plan: &current_plan,
        permanent_memory: &permanent_memory,
        world_model: &world_model,
        hypothesis_engine: &hypothesis_engine,
    };

    ResearchCascade::should_research(query, &deps).await
}

/// Build a research pipeline wired to the memory repository.
///
/// Validated findings must be durable, not merely constructed (R12), so every
/// entry point that can run research builds its pipeline through this helper
/// rather than constructing one directly.
#[cfg(feature = "http")]
fn pipeline_with_memory(database: &Arc<SqliteDatabase>) -> research::pipeline::ResearchPipeline {
    let providers = research::adapter_registry::all_providers();
    let repository = Arc::new(SqliteMemoryRepository::new((**database).clone()));
    research::pipeline::ResearchPipeline::new(providers).with_memory_repository(repository)
}

/// Render a completed research result as the MCP payload for a research tool.
///
/// The payload is the bounded evidence packet the LLM reasons over: findings,
/// sources with full provenance, contradictions and limitations. Raw provider
/// HTML never crosses this boundary (Rule 2, Rule 9). The accompanying summary
/// is explicitly marked as third-party data so the reasoning step has an
/// unambiguous trust boundary.
#[cfg(feature = "http")]
fn research_payload(result: &research::ResearchResult, mode: &str) -> ToolOutput {
    let packet = result.to_evidence_packet();
    // The packet is serialized once to confirm it round-trips across the MCP
    // boundary (R8). A result that cannot be serialized is reported as an error
    // rather than returned as a partially-formed payload.
    let serialized = research::evidence::serialize_result(result);
    let round_trips = research::evidence::deserialize_result(&serialized).is_some();
    ToolOutput::success(serde_json::json!({
        "mode": mode,
        "question": packet.query,
        "findings": packet.findings,
        "sources": packet.sources,
        "contradictions": packet.contradictions,
        "limitations": packet.limitations,
        "confidence": packet.confidence,
        "queries": result.queries,
        "retrieved_at": packet.retrieved_at,
        "findings_count": packet.findings.len(),
        "sources_count": packet.sources.len(),
        "contradictions_count": packet.contradictions.len(),
        "evidence_summary": research::evidence::build_evidence_summary(result),
        "evidence_packet_serialized": serialized,
        "evidence_packet_round_trips": round_trips,
    }))
}

/// Execute research tool (auto mode)
pub async fn execute_research(
    input: super::types::ResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        if !research::is_research_ready() {
            return Ok(ToolOutput::error(format!(
                "research is not available. Request was: {}. {}",
                input.query,
                research::adapter_registry::readiness_report()
            )));
        }
        // 9-tier confidence cascade check (Rule 1: research is last resort).
        // If any tier 1-8 internal source can answer the query, skip research.
        if !should_bypass_cascade(database, &input.query).await {
            return Ok(ToolOutput::error(format!(
                "research skipped: internal knowledge suffices for query '{}'",
                input.query
            )));
        }
        let pipeline = pipeline_with_memory(database);
        let result = pipeline
            .run_pipeline(&input.query, research::pipeline::Mode::Auto)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "auto"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "research",
            &input.query,
        )))
    }
}

/// Execute quick research tool
pub async fn execute_quick_research(
    input: super::types::QuickResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // 9-tier confidence cascade check (Rule 1: research is last resort).
        if !should_bypass_cascade(database, &input.query).await {
            return Ok(ToolOutput::error(format!(
                "research skipped: internal knowledge suffices for query '{}'",
                input.query
            )));
        }
        let quick =
            research::quick_research::QuickMode::new(Arc::new(pipeline_with_memory(database)));
        let result = quick
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "quick"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "quick_research",
            &input.query,
        )))
    }
}

/// Execute deep research tool
pub async fn execute_deep_research(
    input: super::types::DeepResearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        // 9-tier confidence cascade check (Rule 1: research is last resort).
        if !should_bypass_cascade(database, &input.query).await {
            return Ok(ToolOutput::error(format!(
                "research skipped: internal knowledge suffices for query '{}'",
                input.query
            )));
        }
        let deep = research::deep_research::DeepMode::new(Arc::new(pipeline_with_memory(database)));
        let result = deep
            .run(&input.query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(research_payload(&result, "deep"))
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "deep_research",
            &input.query,
        )))
    }
}

/// Execute find error resolution tool
pub async fn execute_find_error_resolution(
    input: super::types::FindErrorResolutionInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    #[cfg(feature = "http")]
    {
        let query = format!("error resolution for: {}", input.error);
        // 9-tier confidence cascade check (Rule 1: research is last resort).
        if !should_bypass_cascade(database, &query).await {
            return Ok(ToolOutput::error(format!(
                "research skipped: internal knowledge suffices for error '{}'",
                input.error
            )));
        }
        let pipeline = pipeline_with_memory(database);
        match pipeline
            .run_pipeline(&query, research::pipeline::Mode::Quick)
            .await
        {
            Ok(result) => {
                let mut payload = research_payload(&result, "quick");
                if let Some(data) = payload.data.as_object_mut() {
                    data.insert("error_text".to_string(), serde_json::json!(input.error));
                }
                Ok(payload)
            }
            Err(e) => {
                // Do NOT fabricate a resolution. Returning success here with
                // "Check logs and retry" tells the LLM that a search happened
                // and produced a verdict, which invites it to invent a fix.
                // Propagate the real cause instead (Rule 6, structured errors).
                tracing::warn!(error = %e, "error-resolution research pipeline failed");
                Err(anyhow::anyhow!(
                    "error-resolution research failed for '{}': {}",
                    input.error,
                    e
                ))
            }
        }
    }
    #[cfg(not(feature = "http"))]
    {
        Ok(ToolOutput::error(super::http_feature_unavailable_for(
            "find_error_resolution",
            &input.error,
        )))
    }
}
