// src/skills/registry/executor.rs
//! Skill executor implementation

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;

use super::context::ExecutionContext;
use super::metrics::ExecutionMetrics;
use super::registry::SkillRegistry;
use super::result::ExecutionResult;
use super::skill::Skill;
use super::types::SkillCategory;

/// Skill executor for running skills
/// Per Architecture §15: "Skill::execute(&context)"
pub struct SkillExecutor {
    registry: Arc<SkillRegistry>,
    metrics: std::sync::Mutex<std::collections::HashMap<String, ExecutionMetrics>>,
}

impl SkillExecutor {
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        Self {
            registry,
            metrics: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Execute a skill by ID with context
    /// Per Architecture §15: "Skill::execute(&context)"
    pub async fn execute_skill(
        &self,
        skill_id: &str,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        // Get skill
        let skill = self.registry.get(skill_id).await
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_id))?;
        
        // Check prerequisites
        if !skill.prerequisites_met(&[]) {
            return Ok(ExecutionResult::failure(
                skill_id.to_string(),
                format!("Prerequisites not met for skill: {}", skill.metadata.name),
                start.elapsed().as_millis() as u64,
                skill.mastery,
                -0.05, // Small penalty
            ));
        }
        
        // Execute the skill logic based on category
        let result = self.execute_by_category(&skill, context).await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Record execution
        self.record_execution(skill_id, &result, duration_ms).await;
        
        Ok(result)
    }

    /// Execute skill based on its category
    async fn execute_by_category(
        &self,
        skill: &Skill,
        context: ExecutionContext,
    ) -> ExecutionResult {
        let start = Instant::now();
        
        // Dispatch based on category
        let output = match skill.metadata.category {
            SkillCategory::FileOperation => {
                serde_json::json!({
                    "task": context.task,
                    "category": "file_operation",
                    "status": "simulated"
                })
            }
            SkillCategory::CodeAnalysis => {
                serde_json::json!({
                    "task": context.task,
                    "category": "code_analysis", 
                    "status": "simulated"
                })
            }
            SkillCategory::Search => {
                serde_json::json!({
                    "task": context.task,
                    "category": "search",
                    "status": "simulated"
                })
            }
            SkillCategory::Memory => {
                serde_json::json!({
                    "task": context.task,
                    "category": "memory",
                    "status": "simulated"
                })
            }
            SkillCategory::Learning => {
                serde_json::json!({
                    "task": context.task,
                    "category": "learning",
                    "status": "simulated"
                })
            }
            SkillCategory::Planning => {
                serde_json::json!({
                    "task": context.task,
                    "category": "planning",
                    "status": "simulated"
                })
            }
            _ => {
                serde_json::json!({
                    "task": context.task,
                    "category": format!("{:?}", skill.metadata.category),
                    "status": "simulated"
                })
            }
        };

        ExecutionResult::success(
            skill.id.clone(),
            output,
            start.elapsed().as_millis() as u64,
            skill.mastery,
            0.0, // Will be calculated by caller
        )
    }

    /// Record execution in registry and update metrics
    async fn record_execution(
        &self,
        skill_id: &str,
        result: &ExecutionResult,
        duration_ms: u64,
    ) {
        // Update registry
        let _ = self.registry.record_usage(skill_id, result.success).await;
        
        // Update local metrics
        match self.metrics.lock() {
            Ok(mut metrics) => {
                let metrics_entry = metrics.entry(skill_id.to_string()).or_default();
                if result.success {
                    metrics_entry.record_success(duration_ms);
                } else {
                    metrics_entry.record_failure(duration_ms);
                }
            }
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during record_execution");
                let mut metrics = poisoned.into_inner();
                let metrics_entry = metrics.entry(skill_id.to_string()).or_default();
                if result.success {
                    metrics_entry.record_success(duration_ms);
                } else {
                    metrics_entry.record_failure(duration_ms);
                }
            }
        }
    }

    /// Get execution metrics for a skill
    /// Per Architecture §15: "Skill::track_execution_metrics()"
    pub fn get_execution_metrics(&self, skill_id: &str) -> Option<ExecutionMetrics> {
        match self.metrics.lock() {
            Ok(metrics) => metrics.get(skill_id).cloned(),
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during get_execution_metrics");
                poisoned.into_inner().get(skill_id).cloned()
            }
        }
    }

    /// Get all execution metrics
    pub fn get_all_metrics(&self) -> std::collections::HashMap<String, ExecutionMetrics> {
        match self.metrics.lock() {
            Ok(metrics) => metrics.clone(),
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during get_all_metrics");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Get skills sorted by success rate
    pub fn get_skills_by_success_rate(&self) -> Vec<(String, f32)> {
        match self.metrics.lock() {
            Ok(metrics) => {
                let mut result: Vec<_> = metrics.iter()
                    .map(|(id, m)| (id.clone(), m.success_rate()))
                    .collect();
                result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                result
            }
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during get_skills_by_success_rate");
                let metrics = poisoned.into_inner();
                let mut result: Vec<_> = metrics.iter()
                    .map(|(id, m)| (id.clone(), m.success_rate()))
                    .collect();
                result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                result
            }
        }
    }

    /// Get unreliable skills
    pub fn get_unreliable_skills(&self) -> Vec<String> {
        match self.metrics.lock() {
            Ok(metrics) => {
                metrics.iter()
                    .filter(|(_, m)| m.is_unreliable())
                    .map(|(id, _)| id.clone())
                    .collect()
            }
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during get_unreliable_skills");
                let metrics = poisoned.into_inner();
                metrics.iter()
                    .filter(|(_, m)| m.is_unreliable())
                    .map(|(id, _)| id.clone())
                    .collect()
            }
        }
    }

    /// Clear metrics for a skill
    pub fn clear_metrics(&self, skill_id: &str) {
        match self.metrics.lock() {
            Ok(mut metrics) => {
                metrics.remove(skill_id);
            }
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during clear_metrics");
                poisoned.into_inner().remove(skill_id);
            }
        }
    }

    /// Clear all metrics
    pub fn clear_all_metrics(&self) {
        match self.metrics.lock() {
            Ok(mut metrics) => {
                metrics.clear();
            }
            Err(poisoned) => {
                tracing::error!("Metrics mutex poisoned during clear_all_metrics");
                poisoned.into_inner().clear();
            }
        }
    }
}
