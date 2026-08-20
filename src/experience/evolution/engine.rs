// /src/experience/evolution/engine.rs
// The main engine that transforms insights into behaviors

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::behavior::{Behavior, BehaviorAction, BehaviorPriority, BehaviorStatus};
use super::evidence::{EvidenceVerdict, EvolutionEvidence};
use crate::experience::reflection::insight::Insight;

/// Configuration for the evolution engine
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Minimum applications before promotion
    pub min_applications_for_promotion: u32,

    /// Minimum confidence before promotion
    pub min_confidence_for_promotion: f32,

    /// Failure rate threshold for deprecation
    pub failure_threshold: f32,

    /// Days unused before deprecation
    pub unused_threshold_days: i64,

    /// Applications before practice phase
    pub applications_before_practice: u32,

    /// Applications before integration
    pub applications_before_integration: u32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_applications_for_promotion: 5,
            min_confidence_for_promotion: 0.7,
            failure_threshold: 0.5,
            unused_threshold_days: 30,
            applications_before_practice: 10,
            applications_before_integration: 20,
        }
    }
}

/// Trait for evolution engine implementations (scaffolding for future use)
pub trait EvolutionEngineTrait: Send + Sync {
    /// Create a behavior from an insight
    fn create_behavior_from_insight(
        &self,
        insight: &Insight,
    ) -> impl std::future::Future<Output = Result<Behavior>> + Send;

    /// Record a behavior application result
    fn record_result(
        &self,
        behavior_id: &str,
        success: bool,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get active behaviors for a context
    fn get_active_behaviors(
        &self,
        context: &str,
    ) -> impl std::future::Future<Output = Vec<Behavior>> + Send;
}

/// The evolution engine transforms insights into behaviors
#[derive(Clone)]
pub struct EvolutionEngine {
    behaviors: Arc<RwLock<HashMap<String, Behavior>>>,
    evidence: Arc<RwLock<HashMap<String, Vec<EvolutionEvidence>>>>,
    config: EvolutionConfig,
}

impl EvolutionEngine {
    /// Create a new evolution engine
    pub fn new() -> Self {
        Self {
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            evidence: Arc::new(RwLock::new(HashMap::new())),
            config: EvolutionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EvolutionConfig) -> Self {
        Self {
            behaviors: Arc::new(RwLock::new(HashMap::new())),
            evidence: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a behavior from an insight
    pub async fn create_behavior_from_insight(&self, insight: &Insight) -> Result<Behavior> {
        // Wire create_behavior: use canonical creation path
        let mut behavior = self
            .create_behavior(
                format!("Behavior from insight: {}", insight.title),
                insight.statement.clone(),
                BehaviorAction::ApplyHeuristic {
                    rule: insight.statement.clone(),
                    priority: 50,
                },
            )
            .await?;

        // Wire add_source_insight: link source insight to the behavior
        behavior.add_source_insight(&insight.id);

        // Wire get_behavior: verify the behavior was created successfully
        let verified = self.get_behavior(&behavior.id).await;
        if verified.is_none() {
            return Err(anyhow::anyhow!(
                "behavior was created but could not be verified in storage"
            ));
        }

        // Wire add_evidence: record supporting evidence for this behavior
        self.add_evidence(EvolutionEvidence::supporting(
            Uuid::new_v4().to_string(),
            &behavior.id,
            super::evidence::EvidenceType::Observation,
            format!("Derived from insight: {}", insight.title),
        ))
        .await?;

        tracing::info!("Created behavior from insight: {}", insight.id);
        Ok(behavior)
    }

    /// Get a behavior by ID
    pub async fn get_behavior(&self, id: &str) -> Option<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors.get(id).cloned()
    }

    /// List all behaviors
    pub async fn list_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors.values().cloned().collect()
    }

    /// List active behaviors sorted by priority
    pub async fn list_active_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        let mut active: Vec<_> = behaviors
            .values()
            .filter(|b| {
                b.status == BehaviorStatus::Active || b.status == BehaviorStatus::Practicing
            })
            .cloned()
            .collect();
        active.sort_by_key(|b| std::cmp::Reverse(b.priority));
        active
    }

    /// Create a behavior directly
    pub async fn create_behavior(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        action: BehaviorAction,
    ) -> Result<Behavior> {
        let behavior = Behavior::new(Uuid::new_v4().to_string(), name, description, action);

        let mut behaviors = self.behaviors.write().await;
        let behavior_id = behavior.id.clone();
        behaviors.insert(behavior_id.clone(), behavior.clone());

        tracing::info!("Created behavior: {}", behavior_id);
        Ok(behavior)
    }

    /// Record application result
    pub async fn record_result(&self, behavior_id: &str, success: bool) -> Result<()> {
        let (success_flag, id) = {
            let mut behaviors = self.behaviors.write().await;
            if let Some(behavior) = behaviors.get_mut(behavior_id) {
                if success {
                    behavior.record_success();

                    // Check for promotion to practicing
                    if behavior.status == BehaviorStatus::Active
                        && behavior.application_count >= self.config.applications_before_practice
                    {
                        behavior.start_practicing();
                        tracing::info!("Behavior {} promoted to practicing", behavior_id);
                    }
                } else {
                    behavior.record_failure();

                    // Check for deprecation
                    if behavior.should_deprecate(
                        self.config.failure_threshold,
                        self.config.unused_threshold_days,
                    ) {
                        behavior.deprecate();
                        tracing::warn!("Behavior {} deprecated due to failures", behavior_id);
                    }
                }

                // Check for promotion from candidate
                if behavior.is_ready_for_promotion(
                    self.config.min_applications_for_promotion,
                    self.config.min_confidence_for_promotion,
                ) && behavior.status == BehaviorStatus::Candidate
                {
                    behavior.activate();
                    tracing::info!("Behavior {} promoted to active", behavior_id);
                }

                // Check for integration from practicing
                if behavior.status == BehaviorStatus::Practicing
                    && behavior.application_count >= self.config.applications_before_integration
                    && behavior.confidence >= 0.9
                {
                    behavior.integrate();
                    tracing::info!("Behavior {} integrated", behavior_id);
                }
            }
            (success, behavior_id.to_string())
        };

        // Wire update_priority: adjust priority upward on success (after dropping lock to avoid deadlock)
        if success_flag {
            self.update_priority(&id, BehaviorPriority::High).await?;
        }

        // Wire contradicting: record contradicting evidence on failure (after dropping lock)
        if !success_flag {
            self.add_evidence(EvolutionEvidence::contradicting(
                Uuid::new_v4().to_string(),
                id,
                super::evidence::EvidenceType::ApplicationResult,
                "Behavior application failed".to_string(),
            ))
            .await?;
        }

        Ok(())
    }

    /// Add evidence for a behavior
    pub async fn add_evidence(&self, evidence: EvolutionEvidence) -> Result<()> {
        let mut evidence_store = self.evidence.write().await;
        evidence_store
            .entry(evidence.behavior_id.clone())
            .or_insert_with(Vec::new)
            .push(evidence);
        Ok(())
    }

    /// Get evidence for a behavior
    pub async fn get_evidence(&self, behavior_id: &str) -> Vec<EvolutionEvidence> {
        let evidence_store = self.evidence.read().await;
        evidence_store.get(behavior_id).cloned().unwrap_or_default()
    }

    /// Evaluate all behaviors and apply maintenance
    pub async fn evaluate_and_maintain(&self) -> Result<EvaluationSummary> {
        let mut summary = EvaluationSummary::default();

        // Wire get_metrics: gather overall evolution metrics
        let metrics = self.get_metrics().await;
        summary.total_behaviors = metrics.total_behaviors;

        // Wire get_integrated_behaviors: count integrated behaviors
        let integrated = self.get_integrated_behaviors().await;
        summary.integrated = integrated.len();

        // Wire get_deprecated_behaviors: count deprecated behaviors
        let deprecated = self.get_deprecated_behaviors().await;
        summary.deprecated = deprecated.len();

        let mut behaviors = self.behaviors.write().await;

        for behavior in behaviors.values_mut() {
            // Wire get_evidence: check evidence ratio for promotion decision
            let evidence = self.get_evidence(&behavior.id).await;
            let support_ratio = if evidence.is_empty() {
                0.0
            } else {
                evidence
                    .iter()
                    .filter(|e| e.verdict == EvidenceVerdict::Supports)
                    .count() as f32
                    / evidence.len() as f32
            };
            if support_ratio > 0.8 && behavior.status == BehaviorStatus::Candidate {
                behavior.activate();
                summary.promoted += 1;
            }

            // Check deprecation conditions
            if behavior.should_deprecate(
                self.config.failure_threshold,
                self.config.unused_threshold_days,
            ) && behavior.status != BehaviorStatus::Deprecated
            {
                behavior.deprecate();
                summary.deprecated += 1;
            }

            // Check promotion conditions
            if behavior.status == BehaviorStatus::Candidate
                && behavior.is_ready_for_promotion(
                    self.config.min_applications_for_promotion,
                    self.config.min_confidence_for_promotion,
                )
            {
                behavior.activate();
                summary.promoted += 1;
            }

            // Check practice promotion (Active to Practicing based on application count)
            if behavior.status == BehaviorStatus::Active
                && behavior.application_count >= self.config.applications_before_practice
            {
                behavior.start_practicing();
            }
            // Check integration conditions (high confidence + many applications)
            if behavior.status == BehaviorStatus::Practicing
                && behavior.application_count >= self.config.applications_before_integration
                && behavior.confidence >= 0.9
            {
                behavior.integrate();
                summary.integrated += 1;
            }
        }

        summary.total_behaviors = behaviors.len();

        // Wire merge_behaviors: detect and merge duplicate behaviors by action key
        // (after dropping behaviors write lock to avoid deadlock)
        drop(behaviors);

        // Collect duplicate pairs using only IDs (no borrows held)
        let merges = {
            let behaviors = self.behaviors.read().await;
            let mut groups: HashMap<String, Vec<String>> = HashMap::new();
            let mut confidences: HashMap<String, f32> = HashMap::new();
            for behavior in behaviors.values() {
                let key = match &behavior.action {
                    BehaviorAction::ApplyHeuristic { rule, .. } => {
                        format!("heuristic:{}", rule)
                    }
                    BehaviorAction::PreferTool { tool_name, .. } => {
                        format!("prefer:{}", tool_name)
                    }
                    BehaviorAction::AvoidTool { tool_name, .. } => {
                        format!("avoid:{}", tool_name)
                    }
                    _ => continue,
                };
                groups.entry(key).or_default().push(behavior.id.clone());
                confidences.insert(behavior.id.clone(), behavior.confidence);
            }

            let mut merges = Vec::new();
            for ids in groups.into_values() {
                if ids.len() > 1 {
                    let mut sorted = ids.clone();
                    sorted.sort_by(|a, b| {
                        confidences
                            .get(b)
                            .partial_cmp(&confidences.get(a))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let target = &sorted[0];
                    for duplicate in sorted.iter().skip(1) {
                        if duplicate != target {
                            merges.push((duplicate.clone(), target.clone()));
                        }
                    }
                }
            }
            merges
        };

        // Execute merges (each acquires its own lock)
        for (source_id, target_id) in merges {
            self.merge_behaviors(&source_id, &target_id).await?;
        }

        Ok(summary)
    }

    /// Get behavior suggestions based on context
    pub async fn suggest_behaviors(&self, context: &str) -> Vec<Behavior> {
        let active = self.list_active_behaviors().await;
        let context_lower = context.to_lowercase();
        let mut suggestions = Vec::new();

        // Wire should_recommend and get_effectiveness: filter suggestions by recommendation eligibility and effectiveness
        for behavior in active {
            let matches_context = behavior.name.to_lowercase().contains(&context_lower)
                || behavior.description.to_lowercase().contains(&context_lower);
            if !matches_context {
                continue;
            }
            if !self.should_recommend(&behavior.id).await {
                continue;
            }
            if let Some(effectiveness) = self.get_effectiveness(&behavior.id).await {
                if effectiveness > 0.0 {
                    suggestions.push(behavior);
                }
            }
            if suggestions.len() >= 5 {
                break;
            }
        }

        suggestions
    }

    /// Calculate overall evolution metrics
    pub async fn get_metrics(&self) -> EvolutionMetrics {
        let behaviors = self.behaviors.read().await;
        let evidence_store = self.evidence.read().await;

        let total = behaviors.len();
        let by_status: HashMap<_, _> = behaviors.values().fold(HashMap::new(), |mut acc, b| {
            *acc.entry(b.status).or_insert(0) += 1;
            acc
        });

        let total_evidence: usize = evidence_store.values().map(|v| v.len()).sum();
        let supporting_evidence: usize = evidence_store
            .values()
            .flat_map(|v| v.iter())
            .filter(|e| e.verdict == EvidenceVerdict::Supports)
            .count();

        let avg_confidence = if total > 0 {
            behaviors.values().map(|b| b.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };

        EvolutionMetrics {
            total_behaviors: total,
            behaviors_by_status: by_status,
            total_evidence,
            supporting_evidence,
            average_confidence: avg_confidence,
        }
    }

    /// Get integrated behaviors (fully learned)
    pub async fn get_integrated_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors
            .values()
            .filter(|b| b.status == BehaviorStatus::Integrated)
            .cloned()
            .collect()
    }

    /// Get deprecated behaviors
    pub async fn get_deprecated_behaviors(&self) -> Vec<Behavior> {
        let behaviors = self.behaviors.read().await;
        behaviors
            .values()
            .filter(|b| b.status == BehaviorStatus::Deprecated)
            .cloned()
            .collect()
    }

    /// Update behavior priority
    pub async fn update_priority(
        &self,
        behavior_id: &str,
        priority: BehaviorPriority,
    ) -> Result<()> {
        if let Some(behavior) = self.behaviors.write().await.get_mut(behavior_id) {
            behavior.priority = priority;
            behavior.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Archive deprecated behaviors
    pub async fn archive_deprecated(&self) -> Result<usize> {
        let mut count = 0;
        let mut behaviors = self.behaviors.write().await;

        for behavior in behaviors.values_mut() {
            if behavior.status == BehaviorStatus::Deprecated {
                behavior.status = BehaviorStatus::Deprecated;
                count += 1;
            }
        }

        tracing::info!("Archived {} deprecated behaviors", count);
        Ok(count)
    }

    /// Merge similar behaviors
    pub async fn merge_behaviors(&self, source_id: &str, target_id: &str) -> Result<()> {
        let mut behaviors = self.behaviors.write().await;

        let source = behaviors.remove(source_id);
        if let Some(source) = source {
            if let Some(target) = behaviors.get_mut(target_id) {
                // Transfer evidence from source to target
                if let Some(evidence) = self.evidence.read().await.get(source_id) {
                    let mut evidence_store = self.evidence.write().await;
                    evidence_store
                        .entry(target_id.to_string())
                        .or_insert_with(Vec::new)
                        .extend(evidence.clone());
                }

                // Transfer applications
                target.application_count += source.application_count;
                target.success_count += source.success_count;
                target.updated_at = Utc::now();

                // Recalculate confidence
                if target.application_count > 0 {
                    target.confidence =
                        target.success_count as f32 / target.application_count as f32;
                }

                tracing::info!("Merged behavior {} into {}", source_id, target_id);
            }
        }

        Ok(())
    }

    /// Get behavior effectiveness score
    pub async fn get_effectiveness(&self, behavior_id: &str) -> Option<f32> {
        self.behaviors
            .read()
            .await
            .get(behavior_id)
            .map(|b| b.success_rate())
    }

    /// Check if a behavior should be recommended
    pub async fn should_recommend(&self, behavior_id: &str) -> bool {
        if let Some(behavior) = self.behaviors.read().await.get(behavior_id) {
            match behavior.status {
                BehaviorStatus::Active
                | BehaviorStatus::Practicing
                | BehaviorStatus::Integrated => {
                    behavior.confidence >= 0.6 && !behavior.should_deprecate(0.5, 30)
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionEngineTrait for EvolutionEngine {
    /// Create a behavior from an insight
    async fn create_behavior_from_insight(&self, insight: &Insight) -> Result<Behavior> {
        self.create_behavior_from_insight(insight).await
    }

    /// Record a behavior application result
    async fn record_result(&self, behavior_id: &str, success: bool) -> Result<()> {
        self.record_result(behavior_id, success).await
    }

    /// Get active behaviors for a context
    async fn get_active_behaviors(&self, context: &str) -> Vec<Behavior> {
        self.suggest_behaviors(context).await
    }
}

/// Summary of evaluation results
#[derive(Debug, Default)]
pub struct EvaluationSummary {
    pub total_behaviors: usize,
    pub promoted: usize,
    pub deprecated: usize,
    pub integrated: usize,
}

/// Metrics about the evolution system
#[derive(Debug)]
pub struct EvolutionMetrics {
    pub total_behaviors: usize,
    pub behaviors_by_status: HashMap<BehaviorStatus, usize>,
    pub total_evidence: usize,
    pub supporting_evidence: usize,
    pub average_confidence: f32,
}
