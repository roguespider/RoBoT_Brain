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

/// Grouped subsystem handles passed to `create_app`.
pub struct AgentLoopSystems {
    pub mcp_context: Arc<McpContext>,
    pub shared_personality: Arc<std::sync::Mutex<crate::personality::Personality>>,
    pub hypothesis_engine: Arc<std::sync::Mutex<HypothesisEngine>>,
    pub experience_recorder: Arc<ExperienceRecorder>,
    pub reflection_pipeline: Arc<ReflectionPipeline>,
    pub memory_pipeline: Arc<MemoryPipeline>,
    pub acp_router: Arc<AcpRouter>,
    pub world_model: Arc<WorldModel>,
}

/// Create the goal-driven agent loop and the App struct.
///
/// Composes the already-initialized planner, memory retrieval, knowledge store,
/// coordinator and database into a single cognitive loop (Architecture §5.7),
/// then returns the App with all subsystems.
pub fn create_app(systems: AgentLoopSystems) -> App {
    let AgentLoopSystems {
        mcp_context,
        shared_personality,
        hypothesis_engine,
        experience_recorder,
        reflection_pipeline,
        memory_pipeline,
        acp_router,
        world_model,
    } = systems;
    // Goal-driven agent loop (Architecture §5.7, TASK-V2-04).
    let agent_deps =
        crate::agent::AgentDeps::from_context(&mcp_context, shared_personality.clone());
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
