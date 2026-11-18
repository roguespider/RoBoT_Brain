//! Autonomous Navigation Framework (Stage Eight: Robotics and Physical Interaction)
//!
//! Per v0.0.2 Ch 33 "Stage Eight: Robotics and Physical Interaction":
//! - Future versions may interact directly with the physical world
//! - Potential capabilities include: autonomous navigation, sensor fusion
//! - Planning remains the central decision-making component regardless of physical platform
//!
//! This module provides a feature-gated scaffold for future robotics integration.
//! When the `navigation` feature is enabled, the framework provides basic navigation
//! abstractions that can be extended with real sensor data and actuator control.
//!
//! # Architecture
//!
//! ```text
//! navigation/
//! ├── mod.rs          — Feature-gated module entry point
//! ├── types.rs        — Navigation types (Pose, Waypoint, Path, SensorReading)
//! ├── planner.rs      — Path planning (A*, RRT scaffolds)
//! ├── sensor_fusion.rs — Sensor fusion (IMU, GPS, LiDAR, camera)
//! └── driver.rs       — Driver abstraction (mobile platform, smart home, lab)
//! ```
//!
//! # Feature Gate
//!
//! Add to `Cargo.toml`:
//! ```toml
//! [features]
//! navigation = []
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! // Requires `navigation` feature
//! #[cfg(feature = "navigation")]
//! use crate::navigation;
//! ```

use serde::{Deserialize, Serialize};

/// Current pose in the environment (x, y, z coordinates + orientation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    /// X coordinate (meters).
    pub x: f64,
    /// Y coordinate (meters).
    pub y: f64,
    /// Z coordinate (meters, optional for 2D navigation).
    pub z: f64,
    /// Orientation angle in radians (0 to 2*pi).
    pub yaw: f64,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
        }
    }
}

impl Pose {
    /// Create a new pose at the origin.
    pub fn new(x: f64, y: f64, z: f64, yaw: f64) -> Self {
        Self { x, y, z, yaw }
    }

    /// Distance to another pose (Euclidean distance in X-Y plane).
    pub fn distance_to(&self, other: &Pose) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// A waypoint in a navigation path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Waypoint {
    /// Target pose.
    pub pose: Pose,
    /// Whether this waypoint requires stopping (e.g., for sensor scan).
    pub stop_required: bool,
    /// Estimated time to reach this waypoint (seconds).
    pub eta_seconds: f64,
}

impl Waypoint {
    /// Create a new waypoint.
    pub fn new(pose: Pose, stop_required: bool, eta_seconds: f64) -> Self {
        Self {
            pose,
            stop_required,
            eta_seconds,
        }
    }
}

/// A path as a sequence of waypoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    /// Waypoints in order from start to goal.
    pub waypoints: Vec<Waypoint>,
    /// Total estimated path length (meters).
    pub total_length_meters: f64,
    /// Total estimated time (seconds).
    pub total_time_seconds: f64,
    /// Whether the path is feasible (obstacle-free).
    pub feasible: bool,
}

impl Path {
    /// Create an empty path.
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            total_length_meters: 0.0,
            total_time_seconds: 0.0,
            feasible: true,
        }
    }

    /// Check if path has waypoints.
    pub fn has_waypoints(&self) -> bool {
        !self.waypoints.is_empty()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

/// Sensor reading type for navigation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SensorType {
    /// Inertial Measurement Unit (accelerometer + gyroscope).
    Imu,
    /// Global Positioning System.
    Gps,
    /// Light Detection and Ranging.
    Lidar,
    /// Camera/vision.
    Camera,
    /// Sonar/ultrasonic distance.
    Sonar,
}

/// A single sensor reading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// Type of sensor.
    pub sensor_type: SensorType,
    /// Timestamp (Unix epoch seconds).
    pub timestamp: f64,
    /// Raw data as a vector of floats (sensor-dependent format).
    pub data: Vec<f64>,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

impl SensorReading {
    /// Create a new sensor reading.
    pub fn new(sensor_type: SensorType, timestamp: f64, data: Vec<f64>, confidence: f64) -> Self {
        Self {
            sensor_type,
            timestamp,
            data,
            confidence,
        }
    }
}

/// Sensor fusion result after combining multiple sensor inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionResult {
    /// Estimated pose after fusion.
    pub estimated_pose: Pose,
    /// Covariance matrix (simplified as diagonal).
    pub covariance: [f64; 4], // x, y, z, yaw variances
    /// Sensors used in fusion.
    pub sensors_used: Vec<SensorType>,
    /// Fusion confidence (0.0 to 1.0).
    pub confidence: f64,
}

impl FusionResult {
    /// Create a fusion result with zero pose and low confidence.
    pub fn unknown() -> Self {
        Self {
            estimated_pose: Pose::default(),
            covariance: [1.0, 1.0, 1.0, 1.0],
            sensors_used: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// Navigation driver abstraction for different physical platforms.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlatformType {
    /// Mobile robot platform (wheels/tracks).
    MobileRobot,
    /// Smart home device.
    SmartHome,
    /// Laboratory automation platform.
    LabAutomation,
    /// Industrial automation robot.
    IndustrialRobot,
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MobileRobot => write!(f, "MobileRobot"),
            Self::SmartHome => write!(f, "SmartHome"),
            Self::LabAutomation => write!(f, "LabAutomation"),
            Self::IndustrialRobot => write!(f, "IndustrialRobot"),
        }
    }
}

/// Navigation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationConfig {
    /// Platform type.
    pub platform: PlatformType,
    /// Maximum speed (meters/second).
    pub max_speed_mps: f64,
    /// Navigation update interval (seconds).
    pub update_interval_secs: f64,
    /// Whether sensor fusion is enabled.
    pub sensor_fusion_enabled: bool,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            platform: PlatformType::MobileRobot,
            max_speed_mps: 1.0,
            update_interval_secs: 0.1,
            sensor_fusion_enabled: true,
        }
    }
}

/// Navigation state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NavigationState {
    /// Current pose.
    pub current_pose: Pose,
    /// Current path (if any).
    pub active_path: Option<Path>,
    /// Current waypoint index.
    pub current_waypoint_index: usize,
    /// Whether navigation is active.
    pub is_active: bool,
}
