// src/skills/registry.rs
//! Skill registry for managing available skills
//!
//! Per Architecture §2.9, §12, §15:
//! Skills represent reusable capabilities discovered through experience.
//! A skill is not simply stored code.
//! Skills allow RoBoT to improve through repetition.
//!
//! Skills are different from knowledge:
//! - Knowledge: "I know SQL." (information)
//! - Skill: "I can optimize a query." (capability)

#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

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

/// A registered skill with execution capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub metadata: SkillMetadata,
    pub enabled: bool,
    
    /// Usage metrics
    pub usage_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Mastery and decay per Architecture §15
    pub mastery: f32,  // 0.0-1.0 based on successful usage
    pub last_practiced: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Prerequisites for this skill
    pub prerequisites: Vec<String>,
    
    /// Source of this skill (manual or discovered)
    pub source: SkillSource,
}

impl Skill {
    /// Create a new skill
    pub fn new(metadata: SkillMetadata) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            metadata,
            enabled: true,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            last_used: None,
            mastery: 0.5, // Start with neutral mastery
            last_practiced: None,
            prerequisites: Vec::new(),
            source: SkillSource::Manual,
        }
    }

    /// Create a skill discovered from experience
    ///
    /// Per Architecture §2.9:
    /// "Skills represent reusable capabilities discovered through experience"
    pub fn discovered(
        name: String,
        description: String,
        category: SkillCategory,
        source_experience_id: uuid::Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            metadata: SkillMetadata {
                name,
                description: description.clone(),
                category,
                version: "1.0.0-discovered".to_string(),
                author: Some("RoBoT-Experience".to_string()),
                tags: vec!["discovered".to_string(), "learned".to_string()],
                examples: vec![],
            },
            enabled: true,
            usage_count: 0,
            success_count: 0,
            failure_count: 0,
            last_used: None,
            mastery: 0.3, // Start low until proven
            last_practiced: None,
            prerequisites: Vec::new(),
            source: SkillSource::Discovered { experience_id: source_experience_id },
        }
    }

    /// Record successful usage
    ///
    /// Per Architecture §15:
    /// "Skills include: practice, execution metrics"
    pub fn record_success(&mut self) {
        self.usage_count += 1;
        self.success_count += 1;
        self.last_used = Some(chrono::Utc::now());
        self.last_practiced = Some(chrono::Utc::now());
        
        // Increase mastery on success (diminishing returns)
        let improvement = (0.1 * (1.0 - self.mastery)).min(0.05);
        self.mastery = (self.mastery + improvement).min(1.0);
    }

    /// Record failed usage
    pub fn record_failure(&mut self) {
        self.usage_count += 1;
        self.failure_count += 1;
        self.last_used = Some(chrono::Utc::now());
        
        // Decrease mastery on failure
        self.mastery = (self.mastery - 0.05).max(0.0);
    }

    /// Record usage
    pub fn record_usage(&mut self, success: bool) {
        if success {
            self.record_success();
        } else {
            self.record_failure();
        }
    }

    /// Apply decay to mastery over time
    ///
    /// Per Architecture §15:
    /// "Skills include: decay"
    pub fn apply_decay(&mut self, decay_rate: f32, days_idle: i64) {
        if days_idle > 7 {
            // Gradual decay for unused skills
            let decay = decay_rate * days_idle as f32 / 30.0;
            self.mastery = (self.mastery - decay).max(0.0);
        }
    }

    /// Check if skill is ready to use
    pub fn is_ready(&self) -> bool {
        self.enabled && self.mastery >= 0.3
    }

    /// Check if prerequisites are met
    pub fn prerequisites_met(&self, learned_skills: &[String]) -> bool {
        self.prerequisites.iter().all(|p| learned_skills.contains(p))
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.usage_count == 0 {
            return 0.5; // Neutral for unused skills
        }
        self.success_count as f32 / self.usage_count as f32
    }

    /// Get execution score (combines mastery and success rate)
    pub fn execution_score(&self) -> f32 {
        (self.mastery * 0.6) + (self.success_rate() * 0.4)
    }
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

/// Skill registry for managing available skills
pub struct SkillRegistry {
    skills: Arc<RwLock<Vec<Skill>>>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new skill
    pub async fn register(&self, skill: Skill) -> Result<String> {
        let mut skills = self.skills.write().await;
        
        // Check for duplicate name
        if skills.iter().any(|s| s.metadata.name == skill.metadata.name) {
            anyhow::bail!("Skill '{}' is already registered", skill.metadata.name);
        }
        
        skills.push(skill);
        Ok(skills.last().expect("Skill was just pushed, should exist").id.clone())
    }

    /// Unregister a skill by ID
    pub async fn unregister(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        skills.retain(|s| s.id != skill_id);
        Ok(())
    }

    /// Enable a skill
    pub async fn enable(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.enabled = true;
        }
        Ok(())
    }

    /// Disable a skill
    pub async fn disable(&self, skill_id: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.enabled = false;
        }
        Ok(())
    }

    /// Get a skill by ID
    pub async fn get(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.iter().find(|s| s.id == skill_id).cloned()
    }

    /// Get a skill by name
    pub async fn get_by_name(&self, name: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.iter().find(|s| s.metadata.name == name).cloned()
    }

    /// List all skills
    pub async fn list(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.clone()
    }

    /// List enabled skills
    pub async fn list_enabled(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter().filter(|s| s.enabled).cloned().collect()
    }

    /// List skills by category
    pub async fn list_by_category(&self, category: SkillCategory) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter().filter(|s| s.metadata.category == category).cloned().collect()
    }

    /// Search skills by tag
    pub async fn search_by_tag(&self, tag: &str) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter()
            .filter(|s| s.metadata.tags.iter().any(|t| t.contains(tag)))
            .cloned()
            .collect()
    }

    /// Record skill usage
    pub async fn record_usage(&self, skill_id: &str, success: bool) -> Result<()> {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
            skill.record_usage(success);
        }
        Ok(())
    }

    /// Get most used skills
    pub async fn get_most_used(&self, limit: usize) -> Vec<Skill> {
        let mut skills = self.skills.read().await.clone();
        skills.sort_by_key(|b| std::cmp::Reverse(b.usage_count));
        skills.truncate(limit);
        skills
    }

    /// Get most successful skills
    pub async fn get_most_successful(&self, min_uses: u64) -> Vec<Skill> {
        let skills = self.skills.read().await;
        let mut result: Vec<Skill> = skills.iter()
            .filter(|s| s.usage_count >= min_uses)
            .cloned()
            .collect();
        result.sort_by(|a, b| {
            b.success_rate().partial_cmp(&a.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    /// Get ready skills (enabled and above mastery threshold)
    pub async fn get_ready_skills(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter()
            .filter(|s| s.is_ready())
            .cloned()
            .collect()
    }

    /// Get mastered skills (high execution score)
    pub async fn get_mastered_skills(&self, min_score: f32) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.iter()
            .filter(|s| s.execution_score() >= min_score)
            .cloned()
            .collect()
    }

    /// Apply decay to all skills
    ///
    /// Per Architecture §15:
    /// "Skills include: decay"
    pub async fn apply_decay_all(&self, decay_rate: f32) -> usize {
        let mut skills = self.skills.write().await;
        let now = chrono::Utc::now();
        let mut decayed_count = 0;
        
        for skill in skills.iter_mut() {
            if let Some(last_used) = skill.last_used {
                let days_idle = (now - last_used).num_days();
                let old_mastery = skill.mastery;
                skill.apply_decay(decay_rate, days_idle);
                if (old_mastery - skill.mastery).abs() > 0.001 {
                    decayed_count += 1;
                }
            }
        }
        
        decayed_count
    }

    /// Load default skills
    pub async fn load_defaults(&self) {
        let defaults = vec![
            Skill::new(SkillMetadata {
                name: "file_read".to_string(),
                description: "Read contents of a file".to_string(),
                category: SkillCategory::FileOperation,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["file".to_string(), "read".to_string(), "io".to_string()],
                examples: vec!["Read file at path /src/main.rs".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "file_write".to_string(),
                description: "Write contents to a file".to_string(),
                category: SkillCategory::FileOperation,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["file".to_string(), "write".to_string(), "io".to_string()],
                examples: vec!["Write content to /src/output.txt".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "search".to_string(),
                description: "Search for patterns in files".to_string(),
                category: SkillCategory::Search,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["search".to_string(), "grep".to_string(), "find".to_string()],
                examples: vec!["Search for 'TODO' in all .rs files".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "memory_store".to_string(),
                description: "Store information in memory".to_string(),
                category: SkillCategory::Memory,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["memory".to_string(), "store".to_string(), "persist".to_string()],
                examples: vec!["Store that project uses Rust edition 2024".to_string()],
            }),
            Skill::new(SkillMetadata {
                name: "memory_recall".to_string(),
                description: "Recall information from memory".to_string(),
                category: SkillCategory::Memory,
                version: "1.0.0".to_string(),
                author: Some("RoBoT".to_string()),
                tags: vec!["memory".to_string(), "recall".to_string(), "retrieve".to_string()],
                examples: vec!["Recall all information about the database schema".to_string()],
            }),
        ];

        let mut skills = self.skills.write().await;
        *skills = defaults;
    }

    /// Get skill discovery statistics
    pub async fn get_discovery_stats(&self) -> SkillDiscoveryStats {
        let skills = self.skills.read().await;
        
        let mut by_source = std::collections::HashMap::new();
        let mut total_mastery = 0.0;
        let mut mastered_count = 0;
        
        for skill in skills.iter() {
            // Count by source
            let source_key = match &skill.source {
                SkillSource::Manual => "manual",
                SkillSource::Discovered { .. } => "discovered",
                SkillSource::Learned { .. } => "learned",
            };
            *by_source.entry(source_key.to_string()).or_insert(0) += 1;
            
            total_mastery += skill.mastery;
            if skill.mastery >= 0.8 {
                mastered_count += 1;
            }
        }
        
        let count = skills.len();
        SkillDiscoveryStats {
            total_skills: count,
            manual_skills: *by_source.get("manual").unwrap_or(&0),
            discovered_skills: *by_source.get("discovered").unwrap_or(&0),
            learned_skills: *by_source.get("learned").unwrap_or(&0),
            avg_mastery: if count > 0 { total_mastery / count as f32 } else { 0.0 },
            mastered_skills: mastered_count,
        }
    }
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

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
