// /src/experience/metrics.rs
// Metrics collection for performance and learning tracking


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A single metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional labels/tags
    pub labels: HashMap<String, String>,
}

/// Aggregated metric over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std_dev: Option<f64>,
}

/// System-wide metrics collection
/// 
/// Per Architecture: Provides centralized metrics for monitoring system health,
/// learning progress, and performance characteristics.
pub struct Metrics {
    /// Internal metrics collector
    collector: Arc<MetricsCollector>,
    
    /// Experience count gauge
    experience_count: Arc<RwLock<u64>>,
    
    /// Knowledge count gauge  
    knowledge_count: Arc<RwLock<u64>>,
    
    /// Learning rate (insights per experience)
    learning_rate: Arc<RwLock<f64>>,
    
    /// Reputation scores by source
    reputation_scores: Arc<RwLock<HashMap<String, f64>>>,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            collector: Arc::new(MetricsCollector::new()),
            experience_count: Arc::new(RwLock::new(0)),
            knowledge_count: Arc::new(RwLock::new(0)),
            learning_rate: Arc::new(RwLock::new(0.0)),
            reputation_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Collect all system metrics
    ///
    /// Per Architecture §4: Gather current state of all subsystems
    pub async fn collect(&self) -> SystemMetrics {
        let counters = self.collector.get_all_counters().await;
        let gauges = self.collector.get_all_gauges().await;
        let summary = self.collector.summary().await;
        
        // Get subsystem-specific metrics
        let experience_count = *self.experience_count.read().await;
        let knowledge_count = *self.knowledge_count.read().await;
        let learning_rate = *self.learning_rate.read().await;
        let reputation_scores = self.reputation_scores.read().await.clone();
        
        SystemMetrics {
            timestamp: Utc::now(),
            experience_count,
            knowledge_count,
            learning_rate,
            reputation_scores,
            counters,
            gauges,
            aggregated: summary.metrics,
        }
    }
    
    /// Record experience count
    pub async fn set_experience_count(&self, count: u64) {
        let mut exp_count = self.experience_count.write().await;
        *exp_count = count;
        self.collector.set_gauge("system.experiences.total", count as f64).await;
    }
    
    /// Increment experience count
    pub async fn increment_experience_count(&self) {
        let mut exp_count = self.experience_count.write().await;
        *exp_count += 1;
        self.collector.set_gauge("system.experiences.total", *exp_count as f64).await;
        self.collector.increment("experiences.recorded").await;
    }
    
    /// Get experience count
    pub async fn get_experience_count(&self) -> u64 {
        *self.experience_count.read().await
    }
    
    /// Record knowledge count
    pub async fn set_knowledge_count(&self, count: u64) {
        let mut know_count = self.knowledge_count.write().await;
        *know_count = count;
        self.collector.set_gauge("system.knowledge.total", count as f64).await;
    }
    
    /// Increment knowledge count
    pub async fn increment_knowledge_count(&self) {
        let mut know_count = self.knowledge_count.write().await;
        *know_count += 1;
        self.collector.set_gauge("system.knowledge.total", *know_count as f64).await;
        self.collector.increment("knowledge.created").await;
    }
    
    /// Get knowledge count
    pub async fn get_knowledge_count(&self) -> u64 {
        *self.knowledge_count.read().await
    }
    
    /// Update learning rate (insights generated per experience)
    pub async fn update_learning_rate(&self, insights: u64, experiences: u64) {
        let rate = if experiences > 0 {
            insights as f64 / experiences as f64
        } else {
            0.0
        };
        
        let mut lr = self.learning_rate.write().await;
        *lr = rate;
        
        self.collector.record("learning.rate", rate).await;
        self.collector.set_gauge("learning.rate.current", rate).await;
    }
    
    /// Get current learning rate
    pub async fn get_learning_rate(&self) -> f64 {
        *self.learning_rate.read().await
    }
    
    /// Update reputation score for a source
    pub async fn update_reputation_score(&self, source: &str, score: f64) {
        let mut scores = self.reputation_scores.write().await;
        scores.insert(source.to_string(), score);
        
        let key = format!("reputation.{}", source);
        self.collector.set_gauge(&key, score).await;
    }
    
    /// Get all reputation scores
    pub async fn get_reputation_scores(&self) -> HashMap<String, f64> {
        self.reputation_scores.read().await.clone()
    }
    
    /// Get reputation score for a specific source
    pub async fn get_reputation_score(&self, source: &str) -> Option<f64> {
        self.reputation_scores.read().await.get(source).copied()
    }
    
    /// Record metric for a specific subsystem
    pub async fn record(&self, name: &str, value: f64) {
        self.collector.record(name, value).await;
    }
    
    /// Increment counter
    pub async fn increment(&self, name: &str) {
        self.collector.increment(name).await;
    }
    
    /// Get internal collector for direct access
    pub fn collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.collector)
    }
    
    /// Get aggregated metric
    pub async fn get_aggregated(&self, name: &str) -> Option<AggregatedMetric> {
        self.collector.aggregate(name).await
    }
    
    /// Calculate and return learning statistics
    pub async fn get_learning_stats(&self) -> LearningMetrics {
        let counters = self.collector.get_all_counters().await;
        
        let reflections = *counters.get("reflections.created").unwrap_or(&0);
        let insights = *counters.get("insights.generated").unwrap_or(&0);
        let hypotheses = *counters.get("hypotheses.generated").unwrap_or(&0);
        let validated = *counters.get("hypotheses.confirmed").unwrap_or(&0);
        let rejected = *counters.get("hypotheses.rejected").unwrap_or(&0);
        
        let validation_rate = if hypotheses > 0 {
            validated as f64 / hypotheses as f64
        } else {
            0.0
        };
        
        LearningMetrics {
            reflections_generated: reflections,
            insights_extracted: insights,
            hypotheses_formed: hypotheses,
            hypotheses_confirmed: validated,
            hypotheses_rejected: rejected,
            validation_rate,
            learning_rate: *self.learning_rate.read().await,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// System-wide metrics snapshot
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub experience_count: u64,
    pub knowledge_count: u64,
    pub learning_rate: f64,
    pub reputation_scores: HashMap<String, f64>,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub aggregated: HashMap<String, AggregatedMetric>,
}

/// Learning-specific metrics
#[derive(Debug, Clone)]
pub struct LearningMetrics {
    pub reflections_generated: u64,
    pub insights_extracted: u64,
    pub hypotheses_formed: u64,
    pub hypotheses_confirmed: u64,
    pub hypotheses_rejected: u64,
    pub validation_rate: f64,
    pub learning_rate: f64,
}

/// Metrics collector for tracking system performance
pub struct MetricsCollector {
    /// In-memory storage for current metrics
    metrics: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,

    /// Counters for discrete events
    counters: Arc<RwLock<HashMap<String, u64>>>,

    /// Gauges for current values
    gauges: Arc<RwLock<HashMap<String, f64>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a metric value
    pub async fn record(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let point = MetricPoint {
            name: name.clone(),
            value,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.entry(name).or_insert_with(Vec::new).push(point);
    }

    /// Record a metric with labels
    pub async fn record_with_labels(
        &self,
        name: impl Into<String>,
        value: f64,
        labels: HashMap<String, String>,
    ) {
        let name = name.into();
        let point = MetricPoint {
            name: name.clone(),
            value,
            timestamp: Utc::now(),
            labels,
        };

        let mut metrics = self.metrics.write().await;
        metrics.entry(name).or_insert_with(Vec::new).push(point);
    }

    /// Increment a counter
    pub async fn increment(&self, name: impl Into<String>) {
        let name = name.into();
        let mut counters = self.counters.write().await;
        *counters.entry(name).or_insert(0) += 1;
    }

    /// Increment a counter by value
    pub async fn increment_by(&self, name: impl Into<String>, value: u64) {
        let name = name.into();
        let mut counters = self.counters.write().await;
        *counters.entry(name).or_insert(0) += value;
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().await;
        counters.get(name).copied().unwrap_or(0)
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let mut gauges = self.gauges.write().await;
        gauges.insert(name, value);
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }

    /// Get all values for a metric
    pub async fn get_metric(&self, name: &str) -> Vec<MetricPoint> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned().unwrap_or_default()
    }

    /// Get aggregated metric
    pub async fn aggregate(&self, name: &str) -> Option<AggregatedMetric> {
        let metrics = self.metrics.read().await;
        let points = metrics.get(name)?;

        if points.is_empty() {
            return None;
        }

        let count = points.len() as u64;
        let sum: f64 = points.iter().map(|p| p.value).sum();
        let min = points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
        let max = points
            .iter()
            .map(|p| p.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let avg = sum / count as f64;

        let std_dev = if count > 1 {
            let variance: f64 =
                points.iter().map(|p| (p.value - avg).powi(2)).sum::<f64>() / (count - 1) as f64;
            Some(variance.sqrt())
        } else {
            None
        };

        Some(AggregatedMetric {
            name: name.to_string(),
            count,
            sum,
            min,
            max,
            avg,
            std_dev,
        })
    }

    /// Get all counters
    pub async fn get_all_counters(&self) -> HashMap<String, u64> {
        let counters = self.counters.read().await;
        counters.clone()
    }

    /// Get all gauges
    pub async fn get_all_gauges(&self) -> HashMap<String, f64> {
        let gauges = self.gauges.read().await;
        gauges.clone()
    }

    /// Clear old metrics (older than specified hours)
    pub async fn clear_old(&self, hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let mut metrics = self.metrics.write().await;

        for points in metrics.values_mut() {
            points.retain(|p| p.timestamp > cutoff);
        }
    }

    /// Reset all counters
    pub async fn reset_counters(&self) {
        let mut counters = self.counters.write().await;
        counters.clear();
    }

    /// Get summary of all metrics
    pub async fn summary(&self) -> MetricsSummary {
        let counters = self.get_all_counters().await;
        let gauges = self.get_all_gauges().await;

        let mut metric_summaries = HashMap::new();
        let metrics = self.metrics.read().await;

        for (name, points) in &*metrics {
            if !points.is_empty() {
                let values: Vec<f64> = points.iter().map(|p| p.value).collect();
                let sum: f64 = values.iter().sum();
                let count = values.len() as u64;
                metric_summaries.insert(
                    name.clone(),
                    AggregatedMetric {
                        name: name.clone(),
                        count,
                        sum,
                        min: values.iter().copied().fold(f64::INFINITY, f64::min),
                        max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        avg: sum / count as f64,
                        std_dev: None,
                    },
                );
            }
        }

        MetricsSummary {
            counters,
            gauges,
            metrics: metric_summaries,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of all metrics
#[derive(Debug)]
pub struct MetricsSummary {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub metrics: HashMap<String, AggregatedMetric>,
}

/// Predefined metric names for consistency
pub mod metric_names {
    // Experience metrics
    pub const EXPERIENCES_RECORDED: &str = "experiences.recorded";
    pub const EXPERIENCES_SUCCESS: &str = "experiences.success";
    pub const EXPERIENCES_FAILURE: &str = "experiences.failure";

    // Reflection metrics
    pub const REFLECTIONS_CREATED: &str = "reflections.created";
    pub const REFLECTIONS_VALIDATED: &str = "reflections.validated";
    pub const PATTERNS_DETECTED: &str = "patterns.detected";

    // Hypothesis metrics
    pub const HYPOTHESES_GENERATED: &str = "hypotheses.generated";
    pub const HYPOTHESES_CONFIRMED: &str = "hypotheses.confirmed";
    pub const HYPOTHESES_REJECTED: &str = "hypotheses.rejected";

    // Exploration metrics
    pub const EXPLORATIONS_STARTED: &str = "explorations.started";
    pub const EXPLORATIONS_COMPLETED: &str = "explorations.completed";
    pub const FINDINGS_DISCOVERED: &str = "explorations.findings";

    // Evolution metrics
    pub const BEHAVIORS_CREATED: &str = "behaviors.created";
    pub const BEHAVIORS_ACTIVATED: &str = "behaviors.activated";
    pub const BEHAVIORS_DEPRECATED: &str = "behaviors.deprecated";

    // Reputation metrics
    pub const REPUTATION_UPDATES: &str = "reputation.updates";

    // Performance metrics
    pub const PROCESSING_TIME_MS: &str = "processing.time_ms";
    pub const DATABASE_OPERATIONS: &str = "database.operations";
    pub const DATABASE_LATENCY_MS: &str = "database.latency_ms";

    // Learning metrics
    pub const INSIGHTS_GENERATED: &str = "insights.generated";
    pub const LEARNING_ITERATIONS: &str = "learning.iterations";
    pub const KNOWLEDGE_CONFIDENCE: &str = "knowledge.confidence";
}
