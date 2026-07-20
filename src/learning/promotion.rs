// src/learning/promotion.rs
//! Memory promotion policies and evaluation logic

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};

use super::memory_state::{MemoryState, StateTransition};

/// Promotion policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionPolicy {
    /// Minimum importance to be eligible for promotion
    pub min_importance: f32,
    
    /// Minimum access count for promotion
    pub min_access_count: u32,
    
    /// Maximum age in days before automatic rejection
    pub max_age_days: i64,
    
    /// Number of confirmations required for auto-promotion
    pub confirmations_for_auto_promote: u32,
    
    /// TTL in seconds for Active state (0 = no TTL)
    pub active_ttl_seconds: u64,
    
    /// TTL in seconds for Dormant state before Expired (0 = no TTL)
    pub dormant_ttl_seconds: u64,
    
    /// Maximum repeated count before forcing decision
    pub max_repeated_before_decision: u32,
    
    /// Confidence boost for repeated access
    pub repeated_confidence_boost: f32,
    
    /// Confidence boost for confirmed state
    pub confirmed_confidence_boost: f32,
}

impl Default for PromotionPolicy {
    fn default() -> Self {
        Self {
            min_importance: 0.6,
            min_access_count: 3,
            max_age_days: 30,
            confirmations_for_auto_promote: 3,
            active_ttl_seconds: 3600,      // 1 hour
            dormant_ttl_seconds: 86400,    // 24 hours
            max_repeated_before_decision: 5,
            repeated_confidence_boost: 0.1,
            confirmed_confidence_boost: 0.3,
        }
    }
}

/// Evaluation result for a memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionEvaluation {
    pub should_promote: bool,
    pub should_reject: bool,
    pub should_revive: bool,
    pub recommended_transition: Option<StateTransition>,
    pub confidence_delta: f32,
    pub reason: String,
}

impl PromotionPolicy {
    /// Evaluate a memory item for promotion
    pub fn evaluate(
        &self,
        state: MemoryState,
        importance: f32,
        access_count: u32,
        repeated_count: u32,
        confirmation_count: u32,
        created_at: DateTime<Utc>,
    ) -> PromotionEvaluation {
        let age = Utc::now() - created_at;
        let age_days = age.num_days();
        
        // Check if too old
        if age_days > self.max_age_days {
            return PromotionEvaluation {
                should_promote: false,
                should_reject: true,
                should_revive: false,
                recommended_transition: Some(StateTransition::Reject),
                confidence_delta: -0.5,
                reason: format!("Memory too old ({} days)", age_days),
            };
        }
        
        match state {
            MemoryState::Active => {
                // Check if should timeout to dormant
                if self.active_ttl_seconds > 0 && age.num_seconds() > self.active_ttl_seconds as i64 {
                    if access_count >= self.min_access_count {
                        PromotionEvaluation {
                            should_promote: true,
                            should_reject: false,
                            should_revive: false,
                            recommended_transition: Some(StateTransition::Promote),
                            confidence_delta: self.repeated_confidence_boost,
                            reason: "Auto-promote after sufficient access".to_string(),
                        }
                    } else {
                        PromotionEvaluation {
                            should_promote: false,
                            should_reject: false,
                            should_revive: false,
                            recommended_transition: Some(StateTransition::Timeout),
                            confidence_delta: 0.0,
                            reason: "TTL expired, moving to dormant".to_string(),
                        }
                    }
                } else if importance >= self.min_importance && access_count >= self.min_access_count {
                    PromotionEvaluation {
                        should_promote: true,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: Some(StateTransition::Promote),
                        confidence_delta: self.repeated_confidence_boost,
                        reason: "Meets promotion criteria".to_string(),
                    }
                } else {
                    PromotionEvaluation {
                        should_promote: false,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: None,
                        confidence_delta: 0.0,
                        reason: "Below promotion threshold".to_string(),
                    }
                }
            }
            
            MemoryState::Dormant => {
                if age.num_seconds() > self.dormant_ttl_seconds as i64 {
                    PromotionEvaluation {
                        should_promote: false,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: Some(StateTransition::Timeout),
                        confidence_delta: -0.1,
                        reason: "Dormant TTL expired".to_string(),
                    }
                } else {
                    PromotionEvaluation {
                        should_promote: false,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: None,
                        confidence_delta: 0.0,
                        reason: "Waiting in dormant state".to_string(),
                    }
                }
            }
            
            MemoryState::Expired => {
                if importance >= self.min_importance {
                    PromotionEvaluation {
                        should_promote: true,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: Some(StateTransition::Promote),
                        confidence_delta: 0.0,
                        reason: "Expired but important enough".to_string(),
                    }
                } else {
                    PromotionEvaluation {
                        should_promote: false,
                        should_reject: true,
                        should_revive: false,
                        recommended_transition: Some(StateTransition::Reject),
                        confidence_delta: -0.2,
                        reason: "Expired and below importance threshold".to_string(),
                    }
                }
            }
            
            MemoryState::Repeated => {
                if repeated_count >= self.max_repeated_before_decision {
                    if confirmation_count >= self.confirmations_for_auto_promote {
                        PromotionEvaluation {
                            should_promote: true,
                            should_reject: false,
                            should_revive: false,
                            recommended_transition: Some(StateTransition::Confirm),
                            confidence_delta: self.confirmed_confidence_boost,
                            reason: "Repeated and confirmed".to_string(),
                        }
                    } else {
                        PromotionEvaluation {
                            should_promote: false,
                            should_reject: true,
                            should_revive: false,
                            recommended_transition: Some(StateTransition::Reject),
                            confidence_delta: -0.3,
                            reason: "Too many repeats without confirmation".to_string(),
                        }
                    }
                } else {
                    PromotionEvaluation {
                        should_promote: false,
                        should_reject: false,
                        should_revive: false,
                        recommended_transition: Some(StateTransition::Observe),
                        confidence_delta: self.repeated_confidence_boost,
                        reason: "Still accumulating repeats".to_string(),
                    }
                }
            }
            
            MemoryState::Confirmed => {
                PromotionEvaluation {
                    should_promote: true,
                    should_reject: false,
                    should_revive: false,
                    recommended_transition: Some(StateTransition::Promote),
                    confidence_delta: self.confirmed_confidence_boost,
                    reason: "Confirmed, ready for promotion".to_string(),
                }
            }
            
            MemoryState::Contradicted => {
                PromotionEvaluation {
                    should_promote: false,
                    should_reject: true,
                    should_revive: false,
                    recommended_transition: Some(StateTransition::Reject),
                    confidence_delta: -0.5,
                    reason: "Contradicted, rejecting".to_string(),
                }
            }
            
            MemoryState::Promoted | MemoryState::Rejected => {
                PromotionEvaluation {
                    should_promote: false,
                    should_reject: false,
                    should_revive: false,
                    recommended_transition: None,
                    confidence_delta: 0.0,
                    reason: "Terminal state".to_string(),
                }
            }
        }
    }
    
    /// Calculate new confidence based on state and transitions
    pub fn calculate_confidence(
        &self,
        current_confidence: f32,
        state: MemoryState,
        access_count: u32,
        confirmations: u32,
    ) -> f32 {
        let mut confidence = current_confidence;
        
        // Boost from repeated access
        let repeat_boost = (access_count as f32 * self.repeated_confidence_boost).min(0.3);
        confidence += repeat_boost;
        
        // Boost from confirmations
        let confirm_boost = (confirmations as f32 * self.confirmed_confidence_boost).min(0.4);
        confidence += confirm_boost;
        
        // State-based adjustments
        match state {
            MemoryState::Confirmed => confidence += 0.1,
            MemoryState::Contradicted => confidence -= 0.3,
            MemoryState::Rejected => confidence = 0.0,
            _ => {}
        }
        
        // Clamp to [0.0, 1.0]
        confidence.max(0.0).min(1.0)
    }
}
