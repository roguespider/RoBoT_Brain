// src/skills/registry/types.rs
//! Skill types and metadata

use serde::{Deserialize, Serialize};

/// Skill category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCategory {
    FileOperation,
    CodeAnalysis,
    Search,
    Memory,
    Learning,
    Planning,
    Communication,
    Web,
    Database,
    System,
    Custom,
}

impl SkillCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillCategory::FileOperation => "file_operation",
            SkillCategory::CodeAnalysis => "code_analysis",
            SkillCategory::Search => "search",
            SkillCategory::Memory => "memory",
            SkillCategory::Learning => "learning",
            SkillCategory::Planning => "planning",
            SkillCategory::Communication => "communication",
            SkillCategory::Web => "web",
            SkillCategory::Database => "database",
            SkillCategory::System => "system",
            SkillCategory::Custom => "custom",
        }
    }
}

/// Metadata about a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub version: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

/// Source of a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillSource {
    /// Manually registered skill
    Manual,
    /// Discovered from experience
    Discovered { experience_id: uuid::Uuid },
    /// Learned from external source
    Learned { source_name: String },
}

/// Statistics about skill discovery
#[derive(Debug)]
pub struct SkillDiscoveryStats {
    pub total_skills: usize,
    pub manual_skills: usize,
    pub discovered_skills: usize,
    pub learned_skills: usize,
    pub avg_mastery: f32,
    pub mastered_skills: usize,
}
