//! Configuration and Runtime Management (Architecture Chapter 29).
//!
//! Configuration layers per Architecture §29: system defaults, user overrides,
//! and runtime overrides. Supports loading from file, environment variables,
//! and hot-reload via ConfigWatcher.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Runtime profile for different environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RuntimeProfile {
    #[default]
    Development,
    Testing,
    Production,
}

/// System configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub profile: RuntimeProfile,
    pub memory_enabled: bool,
    pub learning_enabled: bool,
    pub planning_enabled: bool,
    pub workers_enabled: bool,
}

impl Config {
    /// Load configuration from a JSON file at the given path.
    pub fn from_file(path: &str) -> Result<Config, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("failed to read config file {}: {}", path, e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("failed to parse config file {}: {}", path, e))
    }

    /// Load configuration from environment variables.
    ///
    /// Reads ROBOT_BRAIN_* environment variables with sensible defaults for
    /// any that are not set.
    pub fn from_env() -> Config {
        let profile = std::env::var("ROBOT_BRAIN_PROFILE")
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();

        let memory_enabled = std::env::var("ROBOT_BRAIN_MEMORY_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let learning_enabled = std::env::var("ROBOT_BRAIN_LEARNING_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let planning_enabled = std::env::var("ROBOT_BRAIN_PLANNING_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let workers_enabled = std::env::var("ROBOT_BRAIN_WORKERS_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        Config {
            profile,
            memory_enabled,
            learning_enabled,
            planning_enabled,
            workers_enabled,
        }
    }
}

/// Load configuration for a profile.
pub fn load_config(profile: RuntimeProfile) -> Config {
    Config {
        profile,
        memory_enabled: true,
        learning_enabled: true,
        planning_enabled: true,
        workers_enabled: true,
    }
}

/// Validate a configuration.
pub fn validate_config(config: &Config) -> Result<(), String> {
    if !config.memory_enabled && !config.learning_enabled && !config.planning_enabled {
        return Err("at least one subsystem must be enabled".to_string());
    }
    Ok(())
}

/// Watches a configuration file for changes and reloads on demand.
/// Per Architecture §29: hot-reload support for config layers.
pub struct ConfigWatcher {
    path: PathBuf,
    config: Config,
}

impl ConfigWatcher {
    /// Create a new watcher from a config file path.
    pub fn new(path: &str) -> Result<ConfigWatcher, String> {
        let config = Config::from_file(path)?;
        Ok(ConfigWatcher {
            path: PathBuf::from(path),
            config,
        })
    }

    /// Get a reference to the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Reload the configuration from the watched file.
    pub fn reload(&mut self) -> Result<(), String> {
        let path_str = self
            .path
            .to_str()
            .ok_or_else(|| "invalid config path".to_string())?;
        self.config = Config::from_file(path_str)?;
        Ok(())
    }
}
