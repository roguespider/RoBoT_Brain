// /src/experience/evolution/behavior.rs
// Represents a behavior that can be adopted by the agent

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A behavior is an actionable change derived from validated insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the behavior
    pub description: String,

    /// The action or pattern to follow
    pub action: BehaviorAction,

    /// Current status
    pub status: BehaviorStatus,

    /// Priority level
    pub priority: BehaviorPriority,

    /// Source insight IDs that led to this behavior
    pub source_insights: Vec<String>,

    /// Number of times this behavior was applied
    pub application_count: u32,

    /// Number of successful applications
    pub success_count: u32,

    /// Confidence in this behavior (0.0 - 1.0)
    pub confidence: f32,

    /// When the behavior was created
    pub created_at: DateTime<Utc>,

    /// Last time this behavior was applied
    pub last_applied: Option<DateTime<Utc>>,

    /// When the behavior was last updated
    pub updated_at: DateTime<Utc>,
}

/// What the behavior actually does
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorAction {
    /// Prefer this approach/tool
    PreferTool { tool_name: String, reason: String },
    
    /// Avoid this approach/tool
    AvoidTool { tool_name: String, reason: String },
    
    /// Use this workflow pattern
    UseWorkflow { workflow: String, conditions: Vec<String> },
    
    /// Set this parameter/setting
    SetParameter { name: String, value: String },
    
    /// Apply this heuristic
    ApplyHeuristic { rule: String, priority: u8 },
    
    /// Change confidence threshold
    AdjustThreshold { metric: String, threshold: f32 },
    
    /// Custom action
    Custom { action_type: String, details: String },
}

/// Current lifecycle state of a behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BehaviorStatus {
    /// Candidate awaiting evaluation
    Candidate,
    
    /// Active and available for use
    Active,
    
    /// Currently in practice
    Practicing,
    
    /// Deprecated and should not be used
    Deprecated,
    
    /// Fully integrated into agent behavior
    Integrated,
}

/// Priority level for behavior selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BehaviorPriority {
    Critical = 5,
    High = 4,
    Medium = 3,
    Low = 2,
    Background = 1,
}

impl Behavior {
    /// Create a new behavior candidate
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        action: BehaviorAction,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            action,
            status: BehaviorStatus::Candidate,
            priority: BehaviorPriority::Medium,
            source_insights: Vec::new(),
            application_count: 0,
            success_count: 0,
            confidence: 0.5,
            created_at: now,
            last_applied: None,
            updated_at: now,
        }
    }

    /// Add a source insight
    pub fn add_source_insight(&mut self, insight_id: impl Into<String>) {
        let id = insight_id.into();
        if !self.source_insights.contains(&id) {
            self.source_insights.push(id);
            self.updated_at = Utc::now();
        }
    }

    /// Record a successful application
    pub fn record_success(&mut self) {
        self.application_count += 1;
        self.success_count += 1;
        self.last_applied = Some(Utc::now());
        self.updated_at = Utc::now();
        self.recalculate_confidence();
    }

    /// Record a failed application
    pub fn record_failure(&mut self) {
        self.application_count += 1;
        self.last_applied = Some(Utc::now());
        self.updated_at = Utc::now();
        self.recalculate_confidence();
    }

    /// Recalculate confidence based on success rate
    fn recalculate_confidence(&mut self) {
        if self.application_count > 0 {
            let success_rate = self.success_count as f32 / self.application_count as f32;
            // Weight by number of applications (more data = more confidence)
            let weight = (self.application_count as f32 / 10.0).min(1.0);
            self.confidence = (self.confidence * (1.0 - weight)) + (success_rate * weight);
        }
    }

    /// Promote to active status
    pub fn activate(&mut self) {
        self.status = BehaviorStatus::Active;
        self.updated_at = Utc::now();
    }

    /// Start practicing this behavior
    pub fn start_practicing(&mut self) {
        self.status = BehaviorStatus::Practicing;
        self.updated_at = Utc::now();
    }

    /// Mark as deprecated
    pub fn deprecate(&mut self) {
        self.status = BehaviorStatus::Deprecated;
        self.updated_at = Utc::now();
    }

    /// Mark as integrated
    pub fn integrate(&mut self) {
        self.status = BehaviorStatus::Integrated;
        self.updated_at = Utc::now();
    }

    /// Check if behavior is ready to be promoted
    pub fn is_ready_for_promotion(&self, min_applications: u32, min_confidence: f32) -> bool {
        self.application_count >= min_applications && self.confidence >= min_confidence
    }

    /// Check if behavior should be deprecated
    pub fn should_deprecate(&self, failure_threshold: f32, unused_days: i64) -> bool {
        // Check failure rate
        if self.application_count > 0 {
            let failure_rate = 1.0 - (self.success_count as f32 / self.application_count as f32);
            if failure_rate >= failure_threshold {
                return true;
            }
        }

        // Check if unused
        if let Some(last_applied) = self.last_applied {
            let days_since_use = (Utc::now() - last_applied).num_days();
            if days_since_use >= unused_days && self.status != BehaviorStatus::Integrated {
                return true;
            }
        }

        false
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f32 {
        if self.application_count > 0 {
            self.success_count as f32 / self.application_count as f32
        } else {
            0.0
        }
    }
}
