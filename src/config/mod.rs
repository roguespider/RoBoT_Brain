//! Configuration and Runtime Management (Architecture Chapter 29).

/// Runtime profile for different environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeProfile {
    #[default]
    Development,
    Testing,
    Production,
}

/// System configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub profile: RuntimeProfile,
    pub memory_enabled: bool,
    pub learning_enabled: bool,
    pub planning_enabled: bool,
    pub workers_enabled: bool,
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
