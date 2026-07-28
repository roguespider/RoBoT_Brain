// src/safety/mod.rs

//! Safety Layer
//!
//! Per Architecture: Provides safety checks, validation, and constraint enforcement
//! to prevent harmful or unintended actions.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    /// Whether the action passed the check
    pub passed: bool,
    
    /// Risk level if not passed
    pub risk_level: RiskLevel,
    
    /// Description of the issue if not passed
    pub reason: Option<String>,
    
    /// Suggested safe alternative if applicable
    pub suggested_alternative: Option<String>,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// A safety rule
#[derive(Debug, Clone)]
pub struct SafetyRule {
    /// Rule identifier
    pub id: String,
    
    /// Rule description
    pub description: String,
    
    /// Risk level for violations
    pub risk_level: RiskLevel,
    
    /// Enabled flag
    pub enabled: bool,
    
    /// Check function type
    pub check_fn: SafetyCheckType,
}

/// Types of safety checks
#[derive(Debug, Clone)]
pub enum SafetyCheckType {
    /// Block dangerous commands
    CommandBlocked(Vec<String>),
    
    /// Require confirmation for sensitive operations
    RequireConfirmation(Vec<String>),
    
    /// Limit resource usage
    ResourceLimit { max_cpu_percent: f32, max_memory_mb: u64 },
    
    /// Validate input/output content
    ContentValidation { blocked_patterns: Vec<String> },
    
    /// Rate limiting
    RateLimit { max_per_minute: u32 },
}

/// Safety layer for enforcing constraints
pub struct SafetyLayer {
    /// Active safety rules
    rules: Vec<SafetyRule>,
    
    /// Blocked commands list
    blocked_commands: HashSet<String>,
    
    /// Rate limiting state
    request_counts: std::collections::HashMap<String, Vec<std::time::Instant>>,
}

impl SafetyLayer {
    /// Create a new safety layer with default rules
    pub fn new() -> Self {
        let mut layer = Self {
            rules: Vec::new(),
            blocked_commands: HashSet::new(),
            request_counts: std::collections::HashMap::new(),
        };
        
        // Add default safety rules
        layer.add_default_rules();
        
        layer
    }
    
    /// Add default safety rules
    fn add_default_rules(&mut self) {
        // Block dangerous system commands
        self.rules.push(SafetyRule {
            id: "block_dangerous_cmds".to_string(),
            description: "Block execution of dangerous system commands".to_string(),
            risk_level: RiskLevel::Critical,
            enabled: true,
            check_fn: SafetyCheckType::CommandBlocked(vec![
                "rm -rf /".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "mkfs".to_string(),
                "dd".to_string(),
            ]),
        });
        
        // Require confirmation for file deletion
        self.rules.push(SafetyRule {
            id: "confirm_file_delete".to_string(),
            description: "Require confirmation before file deletion".to_string(),
            risk_level: RiskLevel::High,
            enabled: true,
            check_fn: SafetyCheckType::RequireConfirmation(vec![
                "delete".to_string(),
                "remove".to_string(),
                "unlink".to_string(),
            ]),
        });
        
        // Rate limiting
        self.rules.push(SafetyRule {
            id: "rate_limit".to_string(),
            description: "Limit request rate to prevent abuse".to_string(),
            risk_level: RiskLevel::Medium,
            enabled: true,
            check_fn: SafetyCheckType::RateLimit { max_per_minute: 60 },
        });
        
        // Resource limits
        self.rules.push(SafetyRule {
            id: "resource_limit".to_string(),
            description: "Limit resource usage".to_string(),
            risk_level: RiskLevel::High,
            enabled: true,
            check_fn: SafetyCheckType::ResourceLimit { 
                max_cpu_percent: 80.0, 
                max_memory_mb: 4096 
            },
        });
    }
    
    /// Check if an action is safe
    pub async fn check_action(&self, action: &SafetyAction) -> SafetyCheckResult {
        // Check against blocked commands
        if self.is_command_blocked(&action.action_type) {
            return SafetyCheckResult {
                passed: false,
                risk_level: RiskLevel::Critical,
                reason: Some(format!("Command '{}' is blocked for safety", action.action_type)),
                suggested_alternative: Some("Use a safer alternative command".to_string()),
            };
        }
        
        // Check rate limits
        if !self.check_rate_limit(&action.source) {
            return SafetyCheckResult {
                passed: false,
                risk_level: RiskLevel::Medium,
                reason: Some("Rate limit exceeded. Please slow down.".to_string()),
                suggested_alternative: None,
            };
        }
        
        // Check content for dangerous patterns
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            
            if let SafetyCheckType::ContentValidation { blocked_patterns } = &rule.check_fn {
                for pattern in blocked_patterns {
                    if action.parameters.iter().any(|p| p.contains(pattern)) {
                        return SafetyCheckResult {
                            passed: false,
                            risk_level: rule.risk_level,
                            reason: Some(format!("Content matches blocked pattern: {}", pattern)),
                            suggested_alternative: Some("Remove or modify the blocked content".to_string()),
                        };
                    }
                }
            }
        }
        
        SafetyCheckResult {
            passed: true,
            risk_level: RiskLevel::Low,
            reason: None,
            suggested_alternative: None,
        }
    }
    
    /// Check if a command is blocked
    pub fn is_command_blocked(&self, command: &str) -> bool {
        self.blocked_commands.contains(command) ||
        self.blocked_commands.iter().any(|blocked| command.contains(blocked))
    }
    
    /// Block a command
    pub fn block_command(&mut self, command: &str) {
        self.blocked_commands.insert(command.to_string());
    }
    
    /// Unblock a command
    pub fn unblock_command(&mut self, command: &str) {
        self.blocked_commands.remove(command);
    }
    
    /// Check and update rate limit
    pub fn check_rate_limit(&self, source: &str) -> bool {
        // Default 60 per minute
        let max_per_minute = 60u32;
        
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        
        if let Some(times) = self.request_counts.get(source) {
            let recent: Vec<_> = times.iter().filter(|&&t| t > one_minute_ago).collect();
            recent.len() < max_per_minute as usize
        } else {
            true
        }
    }
    
    /// Record a request for rate limiting
    pub fn record_request(&mut self, source: &str) {
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        
        let times = self.request_counts.entry(source.to_string()).or_insert_with(Vec::new);
        times.retain(|&t| t > one_minute_ago);
        times.push(now);
    }
    
    /// Add a custom safety rule
    pub fn add_rule(&mut self, rule: SafetyRule) {
        self.rules.push(rule);
    }
    
    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            true
        } else {
            false
        }
    }
    
    /// Get all rules
    pub fn get_rules(&self) -> &[SafetyRule] {
        &self.rules
    }
    
    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&SafetyRule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }
    
    /// Validate input content
    pub fn validate_content(&self, content: &str) -> bool {
        let dangerous_patterns = [
            "..",  // Path traversal
            "&&",  // Command chaining
            "||",  // Command chaining
            "|",   // Pipe
            ";",   // Command separator
            "$",   // Variable expansion
            "`",   // Command substitution
            "\n",  // Newline injection
            "\r",  // Carriage return
        ];
        
        !dangerous_patterns.iter().any(|p| content.contains(p))
    }
    
    /// Get safety statistics
    pub fn get_stats(&self) -> SafetyStats {
        SafetyStats {
            total_rules: self.rules.len(),
            enabled_rules: self.rules.iter().filter(|r| r.enabled).count(),
            blocked_commands: self.blocked_commands.len(),
        }
    }
}

/// An action to check for safety
#[derive(Debug, Clone)]
pub struct SafetyAction {
    /// Action type (command, operation, etc.)
    pub action_type: String,
    
    /// Action parameters
    pub parameters: Vec<String>,
    
    /// Source of the action
    pub source: String,
}

/// Safety statistics
#[derive(Debug, Clone)]
pub struct SafetyStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub blocked_commands: usize,
}

impl Default for SafetyLayer {
    fn default() -> Self {
        Self::new()
    }
}
