// /src/CoObOpLoop/inspection.rs
// Inspection system for the CoObOpLoop system.

/// Inspection target categories (§13 / T8.13).
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InspectionTarget {
    OperatingSystem,
    Drivers,
    Runtime,
    InferenceEngine,
    McpLayer,
    Libraries,
    Services,
    Configuration,
    Storage,
    Databases,
    Logs,
    SourceCode,
    Tests,
    Dependencies,
    HardwareInterfaces,
}

impl std::fmt::Display for InspectionTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::OperatingSystem => "operating_system",
            Self::Drivers => "drivers",
            Self::Runtime => "runtime",
            Self::InferenceEngine => "inference_engine",
            Self::McpLayer => "mcp_layer",
            Self::Libraries => "libraries",
            Self::Services => "services",
            Self::Configuration => "configuration",
            Self::Storage => "storage",
            Self::Databases => "databases",
            Self::Logs => "logs",
            Self::SourceCode => "source_code",
            Self::Tests => "tests",
            Self::Dependencies => "dependencies",
            Self::HardwareInterfaces => "hardware_interfaces",
        };
        formatter.write_str(name)
    }
}

impl std::str::FromStr for InspectionTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        match normalized.as_str() {
            "operating_system" => Ok(Self::OperatingSystem),
            "drivers" => Ok(Self::Drivers),
            "runtime" => Ok(Self::Runtime),
            "inference_engine" => Ok(Self::InferenceEngine),
            "mcp_layer" => Ok(Self::McpLayer),
            "libraries" => Ok(Self::Libraries),
            "services" => Ok(Self::Services),
            "configuration" => Ok(Self::Configuration),
            "storage" => Ok(Self::Storage),
            "databases" => Ok(Self::Databases),
            "logs" => Ok(Self::Logs),
            "source_code" => Ok(Self::SourceCode),
            "tests" => Ok(Self::Tests),
            "dependencies" => Ok(Self::Dependencies),
            "hardware_interfaces" => Ok(Self::HardwareInterfaces),
            _ => Err(format!("Unknown inspection target: {value}")),
        }
    }
}

/// Inspection issue (§13 / T8.14).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectionIssue {
    pub target: InspectionTarget,
    pub severity: String,
    pub description: String,
    pub recommended_action: String,
}

/// Inspector that runs inspections.
pub struct Inspector {
    issues: Vec<InspectionIssue>,
}

impl Inspector {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Inspect the active operating environment and return detected issues (§T8.15).
    pub fn scan(&mut self) -> Vec<InspectionIssue> {
        self.issues.clear();
        self.inspect_runtime();
        self.inspect_configuration();
        self.inspect_storage_and_database();
        self.inspect_checkout_layout();
        self.issues.clone()
    }

    fn record_issue(
        &mut self,
        target: InspectionTarget,
        severity: &str,
        description: String,
        recommended_action: &str,
    ) {
        self.issues.push(InspectionIssue {
            target,
            severity: severity.to_string(),
            description,
            recommended_action: recommended_action.to_string(),
        });
    }

    fn inspect_runtime(&mut self) {
        match std::env::current_exe() {
            Ok(path) => match std::fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => {}
                Ok(metadata) => self.record_issue(
                    InspectionTarget::Runtime,
                    "high",
                    format!(
                        "Current executable path is not a regular file ({} bytes): {}",
                        metadata.len(),
                        path.display()
                    ),
                    "Restore or redeploy the RoBoT Brain executable",
                ),
                Err(error) => self.record_issue(
                    InspectionTarget::Runtime,
                    "high",
                    format!("Cannot inspect executable {}: {error}", path.display()),
                    "Verify executable permissions and filesystem integrity",
                ),
            },
            Err(error) => self.record_issue(
                InspectionTarget::OperatingSystem,
                "high",
                format!("Operating system did not expose the current executable: {error}"),
                "Verify process and operating-system runtime configuration",
            ),
        }
    }

    fn inspect_configuration(&mut self) {
        match std::env::var("ROBOT_BRAIN_CONFIG") {
            Ok(path) => {
                let config_path = std::path::Path::new(&path);
                if !config_path.is_file() {
                    self.record_issue(
                        InspectionTarget::Configuration,
                        "medium",
                        format!("ROBOT_BRAIN_CONFIG does not reference a file: {path}"),
                        "Correct ROBOT_BRAIN_CONFIG or create the referenced configuration file",
                    );
                }
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(std::env::VarError::NotUnicode(value)) => self.record_issue(
                InspectionTarget::Configuration,
                "medium",
                format!("ROBOT_BRAIN_CONFIG contains non-Unicode data: {value:?}"),
                "Set ROBOT_BRAIN_CONFIG to a valid Unicode filesystem path",
            ),
        }
    }

    fn inspect_storage_and_database(&mut self) {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                self.record_issue(
                    InspectionTarget::Storage,
                    "high",
                    format!("Cannot resolve storage location from executable: {error}"),
                    "Verify process filesystem access",
                );
                return;
            }
        };
        let Some(directory) = executable.parent() else {
            self.record_issue(
                InspectionTarget::Storage,
                "high",
                format!(
                    "Executable has no parent directory: {}",
                    executable.display()
                ),
                "Deploy the executable inside a writable directory",
            );
            return;
        };
        match std::fs::metadata(directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(metadata) => self.record_issue(
                InspectionTarget::Storage,
                "high",
                format!(
                    "Executable parent is not a directory ({} bytes): {}",
                    metadata.len(),
                    directory.display()
                ),
                "Move the executable to a valid application directory",
            ),
            Err(error) => self.record_issue(
                InspectionTarget::Storage,
                "high",
                format!(
                    "Cannot access application directory {}: {error}",
                    directory.display()
                ),
                "Restore application-directory permissions",
            ),
        }
        let database_path = directory.join("robot_brain.db");
        match std::fs::metadata(&database_path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(metadata) => self.record_issue(
                InspectionTarget::Databases,
                "high",
                format!(
                    "Database path is not a regular file ({} bytes): {}",
                    metadata.len(),
                    database_path.display()
                ),
                "Restore robot_brain.db from backup or recreate the database",
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.record_issue(
                InspectionTarget::Databases,
                "medium",
                format!("Database file is missing: {}", database_path.display()),
                "Initialize the database and run all migrations",
            ),
            Err(error) => self.record_issue(
                InspectionTarget::Databases,
                "high",
                format!(
                    "Cannot inspect database {}: {error}",
                    database_path.display()
                ),
                "Verify database ownership and permissions",
            ),
        }
    }

    fn inspect_checkout_layout(&mut self) {
        let root = match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                self.record_issue(
                    InspectionTarget::OperatingSystem,
                    "medium",
                    format!("Cannot inspect current working directory: {error}"),
                    "Restore access to the process working directory",
                );
                return;
            }
        };
        if !root.join("Cargo.toml").is_file() {
            return;
        }
        let expected = [
            (
                InspectionTarget::SourceCode,
                "src",
                "Restore the src directory",
            ),
            (
                InspectionTarget::Tests,
                "test_suite",
                "Restore the independent test_suite project",
            ),
            (
                InspectionTarget::Dependencies,
                "Cargo.lock",
                "Regenerate Cargo.lock from the reviewed dependency set",
            ),
        ];
        for (target, relative_path, action) in expected {
            let path = root.join(relative_path);
            if !path.exists() {
                self.record_issue(
                    target,
                    "high",
                    format!("Required project path is missing: {}", path.display()),
                    action,
                );
            }
        }
    }

    /// Convert a list of inspection issues into AgentGoals (§T8.16).
    pub fn issues_to_objectives(
        issues: &[InspectionIssue],
    ) -> Vec<crate::cooboploop::queue::AgentGoal> {
        use crate::cooboploop::queue::{AgentGoal, GoalStatus};
        use crate::cooboploop::sources::ObjectiveSource;
        issues
            .iter()
            .map(|issue| {
                let label = issue.target.to_string();
                AgentGoal {
                    id: format!("inspection_{label}_{}", uuid::Uuid::new_v4()),
                    title: format!("Resolve inspection: {} ({})", label, issue.severity),
                    description: format!(
                        "{} Recommended action: {}",
                        issue.description, issue.recommended_action
                    ),
                    status: GoalStatus::Discovered,
                    priority: match issue.severity.to_ascii_lowercase().as_str() {
                        "critical" => 1.0,
                        "high" => 0.9,
                        "medium" => 0.6,
                        "low" => 0.3,
                        _ => 0.2,
                    },
                    source: ObjectiveSource::SystemTrigger,
                    expected_value: 0.5,
                    risk: 0.3,
                    learning_value: 0.4,
                    required_capabilities: Vec::new(),
                    dependencies: Vec::new(),
                    deadline: None,
                    execution_history: Vec::new(),
                    completion_state: None,
                }
            })
            .collect()
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
