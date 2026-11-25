// src/skills/registry/skill.rs
//! Skill struct and implementation

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{SkillCategory, SkillMetadata, SkillSource};

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
