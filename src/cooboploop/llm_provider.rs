// /src/CoObOpLoop/llm_provider.rs
// Replaceable subsystem contracts and default implementations (§22 / T15.3-T15.12).

use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::cooboploop::learning_pipeline::{LearningPipeline, LearningUpdate};
use crate::cooboploop::queue::AgentGoal;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::Experience;
use crate::memory::repository::{MemoryRepository, SqliteMemoryRepository};
use crate::memory::types::{MemoryItem, MemoryLayer, MemoryType};
use crate::planner::engine::{Plan, Planner};

/// LLM provider trait — generates responses from prompts.
pub trait LlmProvider {
    /// Generate a response from a prompt.
    fn generate(&self, prompt: &str) -> String;
}

/// Replaceable inference runtime contract (§22 / T15.4).
pub trait InferenceRuntime {
    fn run(&self, model: &str, input: &[f32]) -> Vec<f32>;
}

/// Operating-system details exposed through a replaceable adapter.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub family: String,
}

/// Replaceable operating-system detection contract (§22 / T15.6).
pub trait OperatingSystemAdapter {
    fn detect(&self) -> OsInfo;
}

/// Canonical tool descriptor exposed by replaceable tool registries.
pub type Tool = crate::bridge::mcp::McpTool;

/// Replaceable tool registry contract (§22 / T15.7).
pub trait ToolRegistry {
    fn list_tools(&self) -> Vec<Tool>;
}

/// Replaceable MCP server registry contract (§22 / T15.8).
pub trait McpServerRegistry {
    fn connect(&self, url: &str) -> Result<(), String>;
}

/// Replaceable memory system contract (§22 / T15.9).
pub trait MemorySystem {
    fn store(&self, content: &str) -> Result<(), String>;
}

/// Replaceable planning algorithm contract (§22 / T15.10).
pub trait PlanningAlgorithm {
    fn plan(&self, objective: &AgentGoal) -> Plan;
}

/// Replaceable learning system contract (§22 / T15.11).
pub trait LearningSystem {
    fn learn(&self, experience: &Experience) -> LearningUpdate;
}

/// Safe default when no text-generation backend has been configured.
///
/// The provider acknowledges the request without fabricating an inferred answer.
#[derive(Clone, Default)]
pub struct DefaultLlmProvider;

impl LlmProvider for DefaultLlmProvider {
    fn generate(&self, prompt: &str) -> String {
        format!(
            "No LLM backend is configured; received a prompt containing {} characters",
            prompt.chars().count()
        )
    }
}

/// Deterministic pass-through runtime used until a model runtime is configured.
///
/// Non-finite values are replaced with zero so downstream consumers receive a
/// stable vector. The model name remains part of the replaceable contract.
#[derive(Clone, Default)]
pub struct DefaultInferenceRuntime;

impl InferenceRuntime for DefaultInferenceRuntime {
    fn run(&self, model: &str, input: &[f32]) -> Vec<f32> {
        if model.trim().is_empty() {
            return Vec::new();
        }
        input
            .iter()
            .map(|value| if value.is_finite() { *value } else { 0.0 })
            .collect()
    }
}

/// Default operating-system adapter backed by standard library constants and
/// platform-native version probes.
#[derive(Clone, Default)]
pub struct DefaultOperatingSystemAdapter;

impl DefaultOperatingSystemAdapter {
    fn command_output(program: &str, arguments: &[&str]) -> Option<String> {
        match std::process::Command::new(program).args(arguments).output() {
            Ok(output) if output.status.success() => {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if value.is_empty() { None } else { Some(value) }
            }
            Ok(output) => {
                tracing::debug!(program, status = %output.status, "operating-system version probe failed");
                None
            }
            Err(error) => {
                tracing::debug!(program, error = %error, "operating-system version probe could not start");
                None
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn version() -> String {
        Self::command_output(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_OperatingSystem).Version",
            ],
        )
        .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(target_os = "linux")]
    fn version() -> String {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content.lines().find_map(|line| {
                    line.strip_prefix("VERSION_ID=")
                        .map(|value| value.trim_matches('"').to_string())
                })
            })
            .filter(|value| !value.is_empty())
            .or_else(|| Self::command_output("uname", &["-r"]))
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(target_os = "macos")]
    fn version() -> String {
        Self::command_output("sw_vers", &["-productVersion"])
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    fn version() -> String {
        "unknown".to_string()
    }
}

impl OperatingSystemAdapter for DefaultOperatingSystemAdapter {
    fn detect(&self) -> OsInfo {
        OsInfo {
            name: std::env::consts::OS.to_string(),
            version: Self::version(),
            architecture: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        }
    }
}

/// Default view over the application's registered MCP tools.
#[derive(Clone, Default)]
pub struct DefaultToolRegistry;

impl ToolRegistry for DefaultToolRegistry {
    fn list_tools(&self) -> Vec<Tool> {
        crate::bridge::tools::registered_tools_snapshot()
    }
}

/// Default URL-based MCP endpoint registry.
///
/// This adapter establishes and retains TCP connections for `tcp://host:port`
/// endpoints. Child-process MCP servers continue to use `McpClient`, whose
/// command-based contract is intentionally distinct from this URL contract.
#[derive(Default)]
pub struct DefaultMcpServerRegistry {
    connections: Mutex<Vec<TcpStream>>,
}

impl DefaultMcpServerRegistry {
    fn socket_address(url: &str) -> Result<&str, String> {
        let trimmed = url.trim();
        let Some(address) = trimmed.strip_prefix("tcp://") else {
            return Err("MCP server URL must use the tcp:// scheme".to_string());
        };
        let authority = address.split('/').next().unwrap_or(address).trim();
        if authority.is_empty() || !authority.contains(':') {
            return Err("MCP server URL must include a host and port".to_string());
        }
        Ok(authority)
    }
}

impl McpServerRegistry for DefaultMcpServerRegistry {
    fn connect(&self, url: &str) -> Result<(), String> {
        let authority = Self::socket_address(url)?;
        let addresses = authority
            .to_socket_addrs()
            .map_err(|error| format!("resolve MCP server URL {url}: {error}"))?;
        let mut last_error = None;
        for address in addresses {
            match TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
                Ok(stream) => {
                    stream
                        .set_nodelay(true)
                        .map_err(|error| format!("configure MCP server connection: {error}"))?;
                    match self.connections.lock() {
                        Ok(mut connections) => connections.push(stream),
                        Err(poisoned) => poisoned.into_inner().push(stream),
                    }
                    return Ok(());
                }
                Err(error) => last_error = Some(error.to_string()),
            }
        }
        Err(format!(
            "connect to MCP server {url}: {}",
            last_error.unwrap_or_else(|| "URL resolved to no socket addresses".to_string())
        ))
    }
}

/// Default memory adapter backed by the application's SQLite repository.
pub struct DefaultMemorySystem {
    repository: SqliteMemoryRepository,
}

impl DefaultMemorySystem {
    pub fn new(database: SqliteDatabase) -> Self {
        Self {
            repository: SqliteMemoryRepository::new(database),
        }
    }
}

impl MemorySystem for DefaultMemorySystem {
    fn store(&self, content: &str) -> Result<(), String> {
        if content.trim().is_empty() {
            return Err("memory content cannot be empty".to_string());
        }
        let item = MemoryItem::new(
            MemoryLayer::Working,
            MemoryType::Context,
            content.to_string(),
            "cooboploop_default_memory_system".to_string(),
        );
        self.repository
            .store(&item)
            .map_err(|error| format!("store memory through default repository: {error}"))
    }
}

/// Default planning adapter backed by the current rule-based planner draft.
#[derive(Clone, Default)]
pub struct DefaultPlanningAlgorithm;

impl PlanningAlgorithm for DefaultPlanningAlgorithm {
    fn plan(&self, objective: &AgentGoal) -> Plan {
        let goal = if objective.description.trim().is_empty() {
            objective.title.as_str()
        } else {
            objective.description.as_str()
        };
        Planner::draft_plan(goal)
    }
}

/// Default learning adapter backed by the current CoObOpLoop learning pipeline.
#[derive(Clone, Default)]
pub struct DefaultLearningSystem;

impl LearningSystem for DefaultLearningSystem {
    fn learn(&self, experience: &Experience) -> LearningUpdate {
        LearningPipeline::process(experience)
    }
}

/// Default replaceable subsystem set used by the CoObOpLoop composition root.
pub struct DefaultRuntimeAdapters {
    pub llm: Arc<dyn LlmProvider + Send + Sync>,
    pub inference: Arc<dyn InferenceRuntime + Send + Sync>,
    pub hardware: Arc<dyn crate::cooboploop::hardware::HardwareAdapter + Send + Sync>,
    pub operating_system: Arc<dyn OperatingSystemAdapter + Send + Sync>,
    pub tools: Arc<dyn ToolRegistry + Send + Sync>,
    pub mcp_servers: Arc<dyn McpServerRegistry + Send + Sync>,
    pub memory: Arc<dyn MemorySystem + Send + Sync>,
    pub planning: Arc<dyn PlanningAlgorithm + Send + Sync>,
    pub learning: Arc<dyn LearningSystem + Send + Sync>,
}

impl DefaultRuntimeAdapters {
    pub fn new(database: SqliteDatabase) -> Self {
        Self {
            llm: Arc::new(DefaultLlmProvider),
            inference: Arc::new(DefaultInferenceRuntime),
            hardware: Arc::new(crate::cooboploop::hardware::DefaultHardwareAdapter),
            operating_system: Arc::new(DefaultOperatingSystemAdapter),
            tools: Arc::new(DefaultToolRegistry),
            mcp_servers: Arc::new(DefaultMcpServerRegistry::default()),
            memory: Arc::new(DefaultMemorySystem::new(database)),
            planning: Arc::new(DefaultPlanningAlgorithm),
            learning: Arc::new(DefaultLearningSystem),
        }
    }

    /// Run non-destructive contract checks across all nine default adapters.
    pub fn is_complete(&self) -> bool {
        let llm_response = self.llm.generate("CoObOpLoop adapter health check");
        let inference_output = self.inference.run("identity", &[1.0, f32::NAN]);
        let hardware_profile = self.hardware.detect();
        let os_info = self.operating_system.detect();
        let registered_tools = self.tools.list_tools();
        let invalid_mcp_url_rejected = self.mcp_servers.connect("").is_err();
        let empty_memory_rejected = self.memory.store("").is_err();

        let objective = AgentGoal {
            id: "adapter-health-check".to_string(),
            title: "Verify default planning adapter".to_string(),
            description: "Create a plan for the adapter health check".to_string(),
            status: crate::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.0,
            source: crate::cooboploop::sources::ObjectiveSource::SystemTrigger,
            expected_value: 0.0,
            risk: 0.0,
            learning_value: 0.0,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
        };
        let plan = self.planning.plan(&objective);
        let experience = Experience::new(
            "Default adapter health check".to_string(),
            "Verified replaceable CoObOpLoop subsystem contracts".to_string(),
            crate::experience::types::ExperienceType::System,
            Vec::new(),
        );
        let learning_update = self.learning.learn(&experience);

        !llm_response.is_empty()
            && inference_output == vec![1.0, 0.0]
            && hardware_profile.cpu_cores > 0
            && !os_info.name.is_empty()
            && !os_info.architecture.is_empty()
            && registered_tools.iter().all(|tool| !tool.name.is_empty())
            && invalid_mcp_url_rejected
            && empty_memory_rejected
            && !plan.steps.is_empty()
            && !learning_update.risk_adjustments.is_empty()
    }
}
