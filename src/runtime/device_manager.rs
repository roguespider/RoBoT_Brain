//! Device Manager — Per Architecture Chapter 3X (AI Runtime and Model Execution).
//!
//! Manages hardware devices (CPU, GPU, accelerator) available for AI model execution.
//! Provides device enumeration, selection, and property reporting.

use std::collections::HashMap;
use std::sync::Mutex;

/// Type of compute device.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// CPU device.
    Cpu,
    /// NVIDIA CUDA GPU.
    Cuda,
    /// Apple Metal GPU.
    Metal,
    /// AMD ROCm GPU.
    Rocm,
    /// Vulkan compute device.
    Vulkan,
    /// OpenCL compute device.
    OpenCl,
    /// Other accelerator.
    Other(String),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cpu => write!(f, "CPU"),
            Self::Cuda => write!(f, "CUDA"),
            Self::Metal => write!(f, "Metal"),
            Self::Rocm => write!(f, "ROCm"),
            Self::Vulkan => write!(f, "Vulkan"),
            Self::OpenCl => write!(f, "OpenCL"),
            Self::Other(name) => write!(f, "Other({name})"),
        }
    }
}

/// Properties of a compute device.
#[derive(Debug, Clone)]
pub struct DeviceProperties {
    /// Device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Device type.
    pub device_type: DeviceType,
    /// Total memory in bytes.
    pub total_memory_bytes: u64,
    /// Available memory in bytes.
    pub available_memory_bytes: u64,
    /// Whether the device is currently available for use.
    pub available: bool,
    /// Compute capability major version (if applicable).
    pub compute_capability_major: Option<u32>,
    /// Compute capability minor version (if applicable).
    pub compute_capability_minor: Option<u32>,
}

/// Device Manager — enumerates and manages hardware devices for AI execution.
pub struct DeviceManager {
    devices: Mutex<HashMap<String, DeviceProperties>>,
    default_device: Mutex<Option<String>>,
}

impl DeviceManager {
    /// Create a new device manager with the CPU device registered.
    pub fn new() -> Self {
        let mut devices = HashMap::new();
        devices.insert(
            "cpu".to_string(),
            DeviceProperties {
                id: "cpu".to_string(),
                name: "CPU".to_string(),
                device_type: DeviceType::Cpu,
                total_memory_bytes: 0,
                available_memory_bytes: 0,
                available: true,
                compute_capability_major: None,
                compute_capability_minor: None,
            },
        );

        Self {
            devices: Mutex::new(devices),
            default_device: Mutex::new(Some("cpu".to_string())),
        }
    }

    /// Register a device.
    pub fn register_device(&self, properties: DeviceProperties) {
        let mut devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.insert(properties.id.clone(), properties);
    }

    /// List all registered device IDs.
    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.keys().cloned().collect()
    }

    /// Get properties for a device by ID.
    pub fn get_device(&self, id: &str) -> Option<DeviceProperties> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.get(id).cloned()
    }

    /// Get the default device ID.
    pub fn default_device(&self) -> Option<String> {
        self.default_device
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Set the default device by ID.
    pub fn set_default_device(&self, id: &str) -> Result<(), String> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        if devices.contains_key(id) {
            let mut default = self
                .default_device
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *default = Some(id.to_string());
            Ok(())
        } else {
            Err(format!("device '{id}' not registered"))
        }
    }

    /// Get the number of registered devices.
    pub fn device_count(&self) -> usize {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.len()
    }

    /// Check if a device is available.
    pub fn is_device_available(&self, id: &str) -> bool {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.get(id).map(|d| d.available).unwrap_or(false)
    }

    /// Get total memory in bytes for a device.
    pub fn device_total_memory(&self, id: &str) -> Option<u64> {
        let devices = self.devices.lock().unwrap_or_else(|e| e.into_inner());
        devices.get(id).map(|d| d.total_memory_bytes)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
