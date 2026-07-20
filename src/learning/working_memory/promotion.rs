// src/learning/working_memory/promotion.rs
//! Promotion policies for working memory items

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::memory_state::MemoryState;
use super::WorkingMemoryItem;

/// Policy for promoting working memory to long-term memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionPolicy {
    /// Minimum access count before promotion is considered
    pub min_access_count: u32,
    
    /// Minimum confirmation count before promotion
    pub min_confirmation_count: u32,
    
    /// Minimum importance score (0.0 to 1.0)
    pub min_importance: f32,
    
    /// Minimum confidence score (0.0 to 1.0)
    pub min_confidence: f32,
    
    /// Maximum age in seconds before mandatory promotion or discard
    pub max_age_seconds: Option<u64>,
}

impl Default for PromotionPolicy {
    fn default() -> Self {
        Self {
            min_access_count: 3,
            min_confirmation_count: 2,
            min_importance: 0.6,
            min_confidence: 0.7,
            max_age_seconds: Some(86400), // 24 hours
        }
    }
}

impl PromotionPolicy {
    /// Create a strict policy requiring more evidence
    pub fn strict() -> Self {
        Self {
            min_access_count: 5,
            min_confirmation_count: 3,
            min_importance: 0.8,
            min_confidence: 0.85,
            max_age_seconds: Some(172800), // 48 hours
        }
    }
    
    /// Create a lenient policy for faster promotion
    pub fn lenient() -> Self {
        Self {
            min_access_count: 2,
            min_confirmation_count: 1,
            min_importance: 0.4,
            min_confidence: 0.5,
            max_age_seconds: None,
        }
    }
    
    /// Evaluate if an item should be promoted
    pub fn evaluate(&self, item: &WorkingMemoryItem) -> PromotionEvaluation {
        let mut reasons = Vec::new();
        let mut blockers = Vec::new();
        let mut score = 0.0;
        
        // Check access count
        if item.access_count >= self.min_access_count {
            score += 0.2;
            reasons.push(format!("Access count {} meets threshold", item.access_count));
        } else {
            blockers.push(format!("Access count {} below threshold {}", item.access_count, self.min_access_count));
        }
        
        // Check confirmation count
        if item.confirmation_count >= self.min_confirmation_count {
            score += 0.2;
            reasons.push(format!("Confirmation count {} meets threshold", item.confirmation_count));
        } else {
            blockers.push(format!("Confirmation count {} below threshold {}", item.confirmation_count, self.min_confirmation_count));
        }
        
        // Check importance
        if item.importance >= self.min_importance {
            score += 0.2;
            reasons.push(format!("Importance {:.2} meets threshold", item.importance));
        } else {
            blockers.push(format!("Importance {:.2} below threshold {:.2}", item.importance, self.min_importance));
        }
        
        // Check confidence
        if item.confidence >= self.min_confidence {
            score += 0.2;
            reasons.push(format!("Confidence {:.2} meets threshold", item.confidence));
        } else {
            blockers.push(format!("Confidence {:.2} below threshold {:.2}", item.confidence, self.min_confidence));
        }
        
        // Check state
        if item.state == MemoryState::Confirmed {
            score += 0.2;
            reasons.push("Memory is in Confirmed state".to_string());
        } else {
            blockers.push(format!("Memory is in {:?} state, not Confirmed", item.state));
        }
        
        let should_promote = blockers.is_empty() && score >= 0.8;
        
        PromotionEvaluation {
            should_promote,
            score,
            reasons,
            blockers,
        }
    }
    
    /// Calculate confidence adjustment based on access patterns
    pub fn calculate_confidence(&self, access_count: u32, confirmation_count: u32) -> f32 {
        let base = 0.5;
        let access_factor = (access_count as f32 * 0.05).min(0.3);
        let confirmation_factor = (confirmation_count as f32 * 0.1).min(0.2);
        (base + access_factor + confirmation_factor).min(1.0)
    }
}

/// Evaluation result for promotion consideration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionEvaluation {
    /// Whether the item should be promoted
    pub should_promote: bool,
    
    /// Overall promotion score (0.0 to 1.0)
    pub score: f32,
    
    /// Reasons for promotion decision
    pub reasons: Vec<String>,
    
    /// Issues or blockers for promotion
    pub blockers: Vec<String>,
}

impl PromotionEvaluation {
    /// Create a positive evaluation
    pub fn promote(score: f32, reasons: Vec<String>) -> Self {
        Self {
            should_promote: true,
            score,
            reasons,
            blockers: Vec::new(),
        }
    }
    
    /// Create a negative evaluation
    pub fn reject(score: f32, blockers: Vec<String>) -> Self {
        Self {
            should_promote: false,
            score,
            reasons: Vec::new(),
            blockers,
        }
    }
}
