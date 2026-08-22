// src/skills/registry/registry.rs
//! Skill registry implementation

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use super::skill::Skill;
use super::types::{SkillCategory, SkillDiscoveryStats, SkillMetadata, SkillSource};

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
        // Safety: last() always returns Some after push
        let skill_id = if let Some(last_skill) = skills.last() {
            last_skill.id.clone()
        } else {
            // This branch is unreachable after push, but handle it safely
            tracing::error!("Unexpected: skills vector empty after push");
            anyhow::bail!("Internal error: failed to get skill ID after registration");
        };
        Ok(skill_id)
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

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
