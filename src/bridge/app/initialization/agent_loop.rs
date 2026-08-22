// src/bridge/app/initialization/agent_loop.rs
//! Create the goal-driven agent loop and App struct.

use std::sync::Arc;

use crate::bridge::acp::AcpRouter;
use crate::bridge::app::state::App;
use crate::bridge::mcp::McpContext;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::reflection_pipeline::ReflectionPipeline;
use crate::memory::pipeline::MemoryPipeline;
use crate::world_model::WorldModel;

/// Create the goal-driven agent loop and the App struct.
///
/// Composes the already-initialized planner, memory retrieval, knowledge store,
/// coordinator and database into a single cognitive loop (Architecture §5.7),
/// then returns the App with all subsystems.
pub fn create_app(
    mcp_context: Arc<McpContext>,
    shared_personality: Arc<std::sync::Mutex<crate::personality::Personality>>,
    hypothesis_engine: Arc<std::sync::Mutex<HypothesisEngine>>,
    experience_recorder: Arc<ExperienceRecorder>,
    reflection_pipeline: Arc<ReflectionPipeline>,
    memory_pipeline: Arc<MemoryPipeline>,
    acp_router: Arc<AcpRouter>,
    world_model: Arc<WorldModel>,
) -> App {
    // Goal-driven agent loop (Architecture §5.7, TASK-V2-04).
    let agent_safety_gate = mcp_context.safety_gate.clone();
    let agent_deps = crate::agent::AgentDeps::new(
        mcp_context.planner.clone(),
        mcp_context.memory_retrieval.clone(),
        mcp_context.knowledge.clone(),
        mcp_context.coordinator.clone(),
        mcp_context.database.clone(),
        agent_safety_gate,
        shared_personality.clone(),
        mcp_context.metrics.clone(),
    );
    let agent_loop = Arc::new(crate::agent::AgentLoop::new(agent_deps));

    App {
        hypothesis_engine,
        experience_recorder,
        reflection_pipeline,
        memory_pipeline,
        mcp_context,
        personality: shared_personality,
        acp_router,
        agent_loop,
        world_model,
    }
}
