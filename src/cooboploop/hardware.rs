// /src/CoObOpLoop/hardware.rs
// Hardware profiling for the CoObOpLoop system.

use rusqlite::OptionalExtension;

/// Hardware profile (§12 / T8.6).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareProfile {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub memory_total_mb: u64,
    pub memory_available_mb: u64,
    pub storage_total_gb: f64,
    pub storage_available_gb: f64,
    pub gpu_model: Option<String>,
    pub network_interfaces: Vec<String>,
    pub thermal_state: String,
    pub supported_runtimes: Vec<String>,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            cpu_model: "unknown".to_string(),
            cpu_cores: 0,
            memory_total_mb: 0,
            memory_available_mb: 0,
            storage_total_gb: 0.0,
            storage_available_gb: 0.0,
            gpu_model: None,
            network_interfaces: Vec::new(),
            thermal_state: "unknown".to_string(),
            supported_runtimes: Vec::new(),
        }
    }
}

/// Replaceable hardware detection contract (§22 / T15.5).
pub trait HardwareAdapter {
    fn detect(&self) -> HardwareProfile;
}

/// Default hardware adapter backed by the platform probes in this module (§22 / T15.12).
#[derive(Clone, Default)]
pub struct DefaultHardwareAdapter;

impl HardwareAdapter for DefaultHardwareAdapter {
    fn detect(&self) -> HardwareProfile {
        detect_profile()
    }
}

fn command_stdout(program: &str, arguments: &[&str]) -> Option<String> {
    match std::process::Command::new(program).args(arguments).output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        Ok(output) => {
            tracing::debug!(program, status = %output.status, "hardware probe was unavailable");
            None
        }
        Err(error) => {
            tracing::debug!(program, error = %error, "hardware probe could not start");
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn detect_cpu_model() -> String {
    std::env::var("PROCESSOR_IDENTIFIER")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

#[cfg(target_os = "linux")]
fn detect_cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|value| value.split_once(':').map(|pair| pair.1.trim().to_string()))
            })
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

#[cfg(target_os = "macos")]
fn detect_cpu_model() -> String {
    command_stdout("sysctl", &["-n", "machdep.cpu.brand_string"])
        .unwrap_or_else(|| std::env::consts::ARCH.to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn detect_cpu_model() -> String {
    std::env::consts::ARCH.to_string()
}

#[cfg(target_os = "windows")]
fn detect_memory_mb() -> (u64, u64) {
    let script = "$os = Get-CimInstance Win32_OperatingSystem; Write-Output \"$($os.TotalVisibleMemorySize) $($os.FreePhysicalMemory)\"";
    command_stdout("powershell", &["-NoProfile", "-Command", script])
        .and_then(|output| {
            let mut values = output
                .split_whitespace()
                .filter_map(|value| value.parse::<u64>().ok());
            match (values.next(), values.next()) {
                (Some(total_kib), Some(available_kib)) => {
                    Some((total_kib / 1024, available_kib / 1024))
                }
                _ => None,
            }
        })
        .unwrap_or((0, 0))
}

#[cfg(target_os = "linux")]
fn detect_memory_mb() -> (u64, u64) {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(content) => content,
        Err(error) => {
            tracing::debug!(error = %error, "could not read Linux memory information");
            return (0, 0);
        }
    };
    let value_kib = |name: &str| {
        content.lines().find_map(|line| {
            line.strip_prefix(name).and_then(|value| {
                value
                    .split_whitespace()
                    .next()
                    .and_then(|number| number.parse::<u64>().ok())
            })
        })
    };
    (
        value_kib("MemTotal:").unwrap_or(0) / 1024,
        value_kib("MemAvailable:").unwrap_or(0) / 1024,
    )
}

#[cfg(target_os = "macos")]
fn detect_memory_mb() -> (u64, u64) {
    let total_bytes = command_stdout("sysctl", &["-n", "hw.memsize"])
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let available_pages = command_stdout("vm_stat", &[])
        .map(|output| {
            output
                .lines()
                .filter_map(|line| {
                    if line.starts_with("Pages free") || line.starts_with("Pages inactive") {
                        line.split_once(':').and_then(|pair| {
                            pair.1.trim().trim_end_matches('.').parse::<u64>().ok()
                        })
                    } else {
                        None
                    }
                })
                .sum::<u64>()
        })
        .unwrap_or(0);
    (
        total_bytes / 1_048_576,
        available_pages.saturating_mul(4096) / 1_048_576,
    )
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn detect_memory_mb() -> (u64, u64) {
    (0, 0)
}

#[cfg(target_os = "windows")]
fn detect_storage_gb() -> (f64, f64) {
    let script = "$drives = Get-PSDrive -PSProvider FileSystem; Write-Output \"$(($drives.Used | Measure-Object -Sum).Sum) $(($drives.Free | Measure-Object -Sum).Sum)\"";
    command_stdout("powershell", &["-NoProfile", "-Command", script])
        .and_then(|output| {
            let mut values = output
                .split_whitespace()
                .filter_map(|value| value.parse::<f64>().ok());
            match (values.next(), values.next()) {
                (Some(used), Some(free)) => {
                    Some(((used + free) / 1_073_741_824.0, free / 1_073_741_824.0))
                }
                _ => None,
            }
        })
        .unwrap_or((0.0, 0.0))
}

#[cfg(not(target_os = "windows"))]
fn detect_storage_gb() -> (f64, f64) {
    command_stdout("df", &["-k", "."])
        .and_then(|output| output.lines().last().map(str::to_string))
        .and_then(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            if columns.len() < 4 {
                return None;
            }
            match (columns[1].parse::<f64>(), columns[3].parse::<f64>()) {
                (Ok(total_kib), Ok(available_kib)) => {
                    Some((total_kib / 1_048_576.0, available_kib / 1_048_576.0))
                }
                _ => None,
            }
        })
        .unwrap_or((0.0, 0.0))
}

#[cfg(target_os = "windows")]
fn detect_gpu_model() -> Option<String> {
    command_stdout(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
        ],
    )
    .and_then(|output| output.lines().next().map(str::to_string))
}

#[cfg(target_os = "linux")]
fn detect_gpu_model() -> Option<String> {
    command_stdout("lspci", &[]).and_then(|output| {
        output
            .lines()
            .find(|line| line.contains("VGA") || line.contains("3D controller"))
            .map(str::to_string)
    })
}

#[cfg(target_os = "macos")]
fn detect_gpu_model() -> Option<String> {
    command_stdout("system_profiler", &["SPDisplaysDataType"]).and_then(|output| {
        output.lines().find_map(|line| {
            line.trim()
                .strip_prefix("Chipset Model:")
                .map(|value| value.trim().to_string())
        })
    })
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn detect_gpu_model() -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
fn detect_network_interfaces() -> Vec<String> {
    command_stdout(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object -ExpandProperty Name",
        ],
    )
    .map(|output| output.lines().map(str::to_string).collect())
    .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn detect_network_interfaces() -> Vec<String> {
    match std::fs::read_dir("/sys/class/net") {
        Ok(entries) => entries
            .filter_map(|entry_result| entry_result.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect(),
        Err(error) => {
            tracing::debug!(error = %error, "could not enumerate Linux network interfaces");
            Vec::new()
        }
    }
}

#[cfg(target_os = "macos")]
fn detect_network_interfaces() -> Vec<String> {
    command_stdout("ifconfig", &["-l"])
        .map(|output| output.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn detect_network_interfaces() -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn detect_thermal_state() -> String {
    std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .map(|millidegrees| format!("{:.1} C", millidegrees / 1000.0))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(not(target_os = "linux"))]
fn detect_thermal_state() -> String {
    "unknown".to_string()
}

fn detect_supported_runtimes() -> Vec<String> {
    let candidates = [("rust", "rustc"), ("python", "python"), ("node", "node")];
    candidates
        .into_iter()
        .filter_map(|(name, executable)| {
            command_stdout(executable, &["--version"]).map(|version| format!("{name}: {version}"))
        })
        .collect()
}

fn detect_profile() -> HardwareProfile {
    let (memory_total_mb, memory_available_mb) = detect_memory_mb();
    let (storage_total_gb, storage_available_gb) = detect_storage_gb();
    let mut network_interfaces = detect_network_interfaces();
    network_interfaces.sort();
    HardwareProfile {
        cpu_model: detect_cpu_model(),
        cpu_cores: std::thread::available_parallelism()
            .map(|count| u32::try_from(count.get()).unwrap_or(u32::MAX))
            .unwrap_or(1),
        memory_total_mb,
        memory_available_mb,
        storage_total_gb,
        storage_available_gb,
        gpu_model: detect_gpu_model(),
        network_interfaces,
        thermal_state: detect_thermal_state(),
        supported_runtimes: detect_supported_runtimes(),
    }
}

/// Hardware discovery for detecting system capabilities.
pub struct HardwareDiscovery {
    current_profile: Option<HardwareProfile>,
    snapshots: Vec<HardwareProfile>,
}

impl HardwareDiscovery {
    pub fn new() -> Self {
        Self {
            current_profile: None,
            snapshots: Vec::new(),
        }
    }

    /// Detect the current hardware profile (§T8.7).
    ///
    /// Metrics that an operating system cannot expose without elevated privileges
    /// retain an explicit `unknown`/empty value rather than fabricating data.
    pub fn detect(&mut self) -> Option<HardwareProfile> {
        let profile = detect_profile();
        self.current_profile = Some(profile.clone());
        Some(profile)
    }

    pub fn current_profile(&self) -> Option<&HardwareProfile> {
        self.current_profile.as_ref()
    }

    pub fn set_profile(&mut self, profile: HardwareProfile) {
        self.current_profile = Some(profile);
    }

    /// Retain a detected profile for in-process change comparison.
    pub fn update_snapshots(&mut self) -> usize {
        if let Some(profile) = self.current_profile.clone() {
            self.snapshots.push(profile);
        }
        self.snapshots.len()
    }

    /// Latest snapshot.
    pub fn latest_snapshot(&self) -> Option<&HardwareProfile> {
        self.snapshots.last()
    }

    /// Compare the current profile against the latest retained snapshot (§T8.11).
    pub fn detect_hardware_changes(&self) -> Vec<String> {
        let mut changes = Vec::new();
        let (current, previous) = match (self.current_profile(), self.latest_snapshot()) {
            (Some(current), Some(previous)) => (current, previous),
            (Some(current), None) => {
                changes.push(format!(
                    "no prior snapshot to compare against current {}-core profile",
                    current.cpu_cores
                ));
                return changes;
            }
            (None, Some(previous)) => {
                changes.push(format!(
                    "current hardware profile unavailable; latest snapshot has {} cores",
                    previous.cpu_cores
                ));
                return changes;
            }
            (None, None) => {
                changes.push(
                    "current hardware profile and prior snapshot are unavailable".to_string(),
                );
                return changes;
            }
        };

        if current.cpu_model != previous.cpu_model {
            changes.push(format!(
                "cpu_model: {} -> {}",
                previous.cpu_model, current.cpu_model
            ));
        }
        if current.cpu_cores != previous.cpu_cores {
            changes.push(format!(
                "cpu_cores: {} -> {}",
                previous.cpu_cores, current.cpu_cores
            ));
        }
        if current.memory_total_mb != previous.memory_total_mb {
            changes.push(format!(
                "memory_total_mb: {} -> {}",
                previous.memory_total_mb, current.memory_total_mb
            ));
        }
        if current.memory_available_mb != previous.memory_available_mb {
            changes.push(format!(
                "memory_available_mb: {} -> {}",
                previous.memory_available_mb, current.memory_available_mb
            ));
        }
        if (current.storage_total_gb - previous.storage_total_gb).abs() > 0.01 {
            changes.push(format!(
                "storage_total_gb: {:.2} -> {:.2}",
                previous.storage_total_gb, current.storage_total_gb
            ));
        }
        if (current.storage_available_gb - previous.storage_available_gb).abs() > 0.01 {
            changes.push(format!(
                "storage_available_gb: {:.2} -> {:.2}",
                previous.storage_available_gb, current.storage_available_gb
            ));
        }
        if current.gpu_model != previous.gpu_model {
            changes.push(format!(
                "gpu_model: {} -> {}",
                previous.gpu_model.as_deref().unwrap_or("unavailable"),
                current.gpu_model.as_deref().unwrap_or("unavailable")
            ));
        }
        if current.network_interfaces != previous.network_interfaces {
            changes.push(format!(
                "network_interfaces: {:?} -> {:?}",
                previous.network_interfaces, current.network_interfaces
            ));
        }
        if current.thermal_state != previous.thermal_state {
            changes.push(format!(
                "thermal_state: {} -> {}",
                previous.thermal_state, current.thermal_state
            ));
        }
        if current.supported_runtimes != previous.supported_runtimes {
            changes.push(format!(
                "supported_runtimes: {:?} -> {:?}",
                previous.supported_runtimes, current.supported_runtimes
            ));
        }
        changes
    }
}

/// Persistent hardware snapshot registry (§12 / T8.9).
pub struct HardwareRegistry {
    connection: rusqlite::Connection,
}

impl HardwareRegistry {
    pub fn open(path: &str) -> Result<Self, String> {
        let connection = rusqlite::Connection::open(path)
            .map_err(|error| format!("open hardware registry: {error}"))?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS hardware_snapshots (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    snapshot_at TEXT DEFAULT (datetime('now')),
                    cpu_model TEXT,
                    cpu_cores INTEGER,
                    memory_total_mb INTEGER,
                    memory_available_mb INTEGER,
                    storage_total_gb INTEGER,
                    storage_available_gb REAL,
                    gpu_model TEXT,
                    network_interfaces TEXT,
                    thermal_state TEXT,
                    other TEXT
                );",
            )
            .map_err(|error| format!("initialize hardware registry: {error}"))?;
        Ok(Self { connection })
    }

    pub fn update(&self, profile: &HardwareProfile) -> Result<i64, String> {
        let network_interfaces = serde_json::to_string(&profile.network_interfaces)
            .map_err(|error| format!("serialize network interfaces: {error}"))?;
        let other = serde_json::to_string(&serde_json::json!({
            "supported_runtimes": profile.supported_runtimes,
        }))
        .map_err(|error| format!("serialize additional hardware data: {error}"))?;
        let cpu_cores = i64::from(profile.cpu_cores);
        let memory_total_mb = i64::try_from(profile.memory_total_mb).unwrap_or(i64::MAX);
        let memory_available_mb = i64::try_from(profile.memory_available_mb).unwrap_or(i64::MAX);
        let storage_total_gb = profile.storage_total_gb.round() as i64;

        self.connection
            .execute(
                "INSERT INTO hardware_snapshots (
                    cpu_model, cpu_cores, memory_total_mb, memory_available_mb,
                    storage_total_gb, storage_available_gb, gpu_model,
                    network_interfaces, thermal_state, other
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    profile.cpu_model,
                    cpu_cores,
                    memory_total_mb,
                    memory_available_mb,
                    storage_total_gb,
                    profile.storage_available_gb,
                    profile.gpu_model,
                    network_interfaces,
                    profile.thermal_state,
                    other,
                ],
            )
            .map_err(|error| format!("save hardware snapshot: {error}"))?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn latest(&self) -> Result<Option<HardwareProfile>, String> {
        let row = self
            .connection
            .query_row(
                "SELECT cpu_model, cpu_cores, memory_total_mb, memory_available_mb,
                        storage_total_gb, storage_available_gb, gpu_model,
                        network_interfaces, thermal_state, other
                 FROM hardware_snapshots ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<usize, String>(0)?,
                        row.get::<usize, i64>(1)?,
                        row.get::<usize, i64>(2)?,
                        row.get::<usize, i64>(3)?,
                        row.get::<usize, f64>(4)?,
                        row.get::<usize, f64>(5)?,
                        row.get::<usize, Option<String>>(6)?,
                        row.get::<usize, String>(7)?,
                        row.get::<usize, String>(8)?,
                        row.get::<usize, String>(9)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("load latest hardware snapshot: {error}"))?;

        let Some((
            cpu_model,
            cpu_cores,
            memory_total_mb,
            memory_available_mb,
            storage_total_gb,
            storage_available_gb,
            gpu_model,
            network_json,
            thermal_state,
            other_json,
        )) = row
        else {
            return Ok(None);
        };
        let network_interfaces = serde_json::from_str(&network_json)
            .map_err(|error| format!("decode hardware network interfaces: {error}"))?;
        let other: serde_json::Value = serde_json::from_str(&other_json)
            .map_err(|error| format!("decode additional hardware data: {error}"))?;
        let supported_runtimes = other
            .get("supported_runtimes")
            .and_then(serde_json::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(HardwareProfile {
            cpu_model,
            cpu_cores: u32::try_from(cpu_cores)
                .map_err(|error| format!("invalid stored CPU core count: {error}"))?,
            memory_total_mb: u64::try_from(memory_total_mb)
                .map_err(|error| format!("invalid stored total memory: {error}"))?,
            memory_available_mb: u64::try_from(memory_available_mb)
                .map_err(|error| format!("invalid stored available memory: {error}"))?,
            storage_total_gb,
            storage_available_gb,
            gpu_model,
            network_interfaces,
            thermal_state,
            supported_runtimes,
        }))
    }
}

impl Default for HardwareDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
