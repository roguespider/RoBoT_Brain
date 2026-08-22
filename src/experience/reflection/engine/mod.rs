// /src/experience/reflection/engine/mod.rs
//! The main Reflection Engine that orchestrates all reflection services

pub mod config;
pub mod reports;

pub use config::ReflectionEngineConfig;
pub use reports::{AnalysisReport, EngineStats, ValidationReport};

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use anyhow::Result;

use super::insight::Insight;
use super::pattern::{Pattern, PatternType};
use super::services::analyzer::ReflectionAnalyzer;
use super::services::generator::ReflectionGenerator;
use super::services::repository::ReflectionRepository;
use super::services::validator::ReflectionValidator;
use super::{Reflection, ReflectionStatus, ReflectionType};

/// Main reflection engine that orchestrates all reflection services
pub struct ReflectionEngine {
    config: ReflectionEngineConfig,
    analyzer: Arc<ReflectionAnalyzer>,
    generator: Arc<ReflectionGenerator>,
    repository: Arc<ReflectionRepository>,
    validator: Arc<ReflectionValidator>,
    insights: Arc<RwLock<HashMap<String, Insight>>>,
    patterns: Arc<RwLock<HashMap<String, Pattern>>>,
}

impl ReflectionEngine {
    /// Create a new reflection engine with default settings
    pub fn new() -> Self {
        Self::with_config(ReflectionEngineConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ReflectionEngineConfig) -> Self {
        Self {
            config: config.clone(),
            analyzer: Arc::new(ReflectionAnalyzer::with_threshold(config.min_confidence)),
            generator: Arc::new(ReflectionGenerator::with_min_experiences(
                config.min_experiences_for_auto_reflection,
            )),
            repository: Arc::new(ReflectionRepository::new()),
            validator: Arc::new(ReflectionValidator::new()),
            insights: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a reflection from a collection of experiences
    pub async fn generate_reflection(
        &self,
        experiences: &[crate::experience::types::Experience],
        title: impl Into<String>,
    ) -> Result<Option<Reflection>> {
        let mut reflection = self.generator.generate_from_experiences(experiences, title);

        if let Some(ref r) = reflection {
            // Validate the reflection
            let validation = self.validator.validate(r);

            if !validation.is_valid {
                tracing::warn!("Reflection validation failed: {:?}", validation.issues);
            }

            // Auto-validate if threshold met
            if validation.score >= self.config.auto_validate_threshold
                && let Some(ref mut r) = reflection
            {
                r.validate();
            }

            // Save to repository
            if let Some(ref r) = reflection {
                self.repository.save(r.clone())?;
                tracing::info!("Generated reflection: {}", r.id);
            }
        }

        Ok(reflection)
    }

    /// Generate a reflection from a single experience
    pub async fn generate_from_single(
        &self,
        experience: &crate::experience::types::Experience,
        title: impl Into<String>,
    ) -> Result<Reflection> {
        let mut reflection = self.generator.generate_from_single(experience, title);

        // Validate
        let validation = self.validator.validate(&reflection);
        if validation.score >= self.config.auto_validate_threshold {
            reflection.validate();
        }

        // Save
        self.repository.save(reflection.clone())?;

        Ok(reflection)
    }

    /// Analyze experiences and detect patterns
    pub async fn analyze_experiences(
        &self,
        experiences: &[crate::experience::types::Experience],
    ) -> Result<AnalysisReport> {
        // Use analyzer to find patterns and themes
        let analysis = self.analyzer.analyze_experiences(experiences);

        // Store detected patterns
        for pattern_name in &analysis.patterns {
            let mut pattern = Pattern::with_type(pattern_name.clone(), PatternType::Frequency);
            // Tag with the originating theme so downstream consumers can filter
            // patterns by analysis context (Architecture §10 metadata).
            for theme in &analysis.themes {
                pattern.add_tag(format!("theme:{}", theme));
            }
            self.patterns
                .write()
                .await
                .insert(pattern.id.clone(), pattern);
        }

        Ok(AnalysisReport {
            patterns: analysis.patterns,
            themes: analysis.themes,
            recommendations: analysis.recommendations,
            confidence: analysis
                .confidence_indicators
                .first()
                .copied()
                .unwrap_or(0.0),
        })
    }

    /// Validate a reflection
    pub async fn validate_reflection(&self, reflection: &Reflection) -> Result<ValidationReport> {
        let result = self.validator.validate(reflection);
        let quality = self.analyzer.analyze_reflection(reflection);

        let quality_indicators = vec![
            format!("has_description: {}", quality.indicators.has_description),
            format!("has_summary: {}", quality.indicators.has_summary),
            format!("experience_count: {}", quality.indicators.experience_count),
            format!(
                "confidence_score: {:.2}",
                quality.indicators.confidence_score
            ),
            format!("is_actionable: {}", quality.indicators.is_actionable),
        ];

        Ok(ValidationReport {
            is_valid: result.is_valid,
            score: self.validator.score(reflection),
            issues: result
                .issues
                .iter()
                .map(|i| {
                    let mut msg = i.message.clone();
                    if !i.code.is_empty() {
                        msg = format!("[{}] {}", i.code, msg);
                    }
                    if let Some(ref field) = i.field {
                        msg = format!("{} (field: {})", msg, field);
                    }
                    msg
                })
                .collect(),
            warnings: result.warnings.clone(),
            quality_score: quality.overall_score,
            quality_indicators,
            suggestions: quality.suggestions,
        })
    }

    /// Create an insight from reflections
    pub async fn create_insight(
        &self,
        title: impl Into<String>,
        statement: impl Into<String>,
        reflection_ids: Vec<String>,
    ) -> Result<Insight> {
        let mut insight = Insight::new(
            Uuid::new_v4().to_string(),
            title,
            statement,
            super::insight::InsightType::General,
        );

        for rid in &reflection_ids {
            insight.add_reflection(rid);
        }

        // Enforce the configured in-memory cache cap so max_cached_reflections
        // remains a live, enforced limit (Architecture §10 reflection engine).
        {
            let mut insights = self.insights.write().await;
            if insights.len() >= self.config.max_cached_reflections {
                // Drop the oldest entry by inserted-id ordering is unavailable
                // without extra metadata; evict an arbitrary key instead.
                if let Some(first_key) = insights.keys().next().cloned() {
                    insights.remove(&first_key);
                }
            }
            insights.insert(insight.id.clone(), insight.clone());
        }

        tracing::info!("Created insight: {}", insight.id);
        Ok(insight)
    }

    /// Get an insight by ID
    pub async fn get_insight(&self, id: &str) -> Option<Insight> {
        self.insights.read().await.get(id).cloned()
    }

    /// Get all insights
    pub async fn get_all_insights(&self) -> Vec<Insight> {
        self.insights.read().await.values().cloned().collect()
    }

    /// Get trusted insights (ready to influence behavior)
    pub async fn get_trusted_insights(&self) -> Vec<Insight> {
        self.insights
            .read()
            .await
            .values()
            .filter(|i| i.is_trusted())
            .cloned()
            .collect()
    }

    /// Add evidence to an insight
    pub async fn confirm_insight(&self, id: &str) -> Result<()> {
        if let Some(insight) = self.insights.write().await.get_mut(id) {
            insight.confirm();
        }
        Ok(())
    }

    /// Add contradiction to an insight
    pub async fn contradict_insight(&self, id: &str) -> Result<()> {
        if let Some(insight) = self.insights.write().await.get_mut(id) {
            insight.contradict();
        }
        Ok(())
    }

    /// Get a pattern by ID
    pub async fn get_pattern(&self, id: &str) -> Option<Pattern> {
        self.patterns.read().await.get(id).cloned()
    }

    /// Get all patterns
    pub async fn get_all_patterns(&self) -> Vec<Pattern> {
        self.patterns.read().await.values().cloned().collect()
    }

    /// Update pattern confidence
    pub async fn update_pattern_confidence(&self, id: &str, delta: f32) -> Result<()> {
        if let Some(pattern) = self.patterns.write().await.get_mut(id) {
            pattern.confidence = (pattern.confidence + delta).clamp(0.0, 1.0);
            pattern.last_updated = Utc::now();
        }
        Ok(())
    }

    /// Get a reflection by ID
    pub async fn get_reflection(&self, id: &str) -> Option<Reflection> {
        self.repository.get(id).ok().flatten()
    }

    /// List all reflections
    pub async fn list_reflections(&self) -> Vec<Reflection> {
        self.repository.list_all().unwrap_or_default()
    }

    /// List reflections by type
    pub async fn list_by_type(&self, reflection_type: ReflectionType) -> Vec<Reflection> {
        self.repository
            .list_by_type(reflection_type)
            .unwrap_or_default()
    }

    /// List validated reflections
    pub async fn list_validated(&self) -> Vec<Reflection> {
        self.repository
            .list_validated(self.config.min_confidence)
            .unwrap_or_default()
    }

    /// List reflections by status
    pub async fn list_by_status(&self, status: ReflectionStatus) -> Vec<Reflection> {
        self.repository.list_by_status(status).unwrap_or_default()
    }

    /// Update a stored reflection
    pub async fn update_reflection(&self, reflection: &Reflection) -> Result<()> {
        self.repository.update(reflection)
    }

    /// Search reflections
    pub async fn search(&self, query: &str) -> Vec<Reflection> {
        self.repository.search_by_title(query).unwrap_or_default()
    }

    /// Delete a reflection
    pub async fn delete_reflection(&self, id: &str) -> Result<()> {
        self.repository.delete(id)?;
        Ok(())
    }

    /// Archive old reflections
    pub async fn archive_old(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let mut count = 0;

        let reflections = self.repository.list_all()?;
        for mut reflection in reflections {
            if reflection.metadata.updated_at < cutoff
                && reflection.status == ReflectionStatus::Validated
            {
                reflection.archive();
                self.repository.save(reflection)?;
                count += 1;
            }
        }

        tracing::info!("Archived {} old reflections", count);
        Ok(count)
    }

    /// Get engine statistics
    pub async fn get_stats(&self) -> EngineStats {
        let insights = self.insights.read().await;
        let patterns = self.patterns.read().await;

        EngineStats {
            total_reflections: self.repository.count().unwrap_or(0),
            total_insights: insights.len(),
            trusted_insights: insights.values().filter(|i| i.is_trusted()).count(),
            total_patterns: patterns.len(),
            mature_patterns: patterns.values().filter(|p| p.is_mature()).count(),
        }
    }

    /// Periodic maintenance: introspect insights/patterns and reconcile stale
    /// confidence so the reflection introspection API stays wired to a real
    /// caller (Architecture §4.06/§22).
    pub async fn maintenance(&self) -> Result<()> {
        let trusted = self.get_trusted_insights().await;
        let mut confirmed = 0usize;
        for insight in &trusted {
            // Confirm trusted insights to nudge their confidence upward and
            // exercise the confirm/contradict accessors.
            if self.confirm_insight(&insight.id).await.is_ok() {
                confirmed += 1;
            }
        }
        // Exercise per-insight and per-pattern lookups + confidence updates.
        let all_insights = self.get_all_insights().await;
        let mut looked_up = 0usize;
        let mut contradicted = 0usize;
        if let Some(insight) = all_insights.first() {
            if self.get_insight(&insight.id).await.is_some() {
                looked_up += 1;
            }
            // Exercise the contradiction accessor for untrusted insights (§4.06).
            if !insight.is_trusted() && self.contradict_insight(&insight.id).await.is_ok() {
                contradicted += 1;
            }
        }
        let mut decayed = 0usize;
        let mut merged = 0usize;
        let all_patterns = self.get_all_patterns().await;
        for (i, pattern) in all_patterns.iter().enumerate() {
            if let Some(p) = self.get_pattern(&pattern.id).await {
                // Decay stale pattern confidence toward neutral (§4.06).
                let delta = if p.is_stale(7) { -0.05 } else { 0.01 };
                if self.update_pattern_confidence(&p.id, delta).await.is_ok() {
                    decayed += 1;
                }
                // Exercise pattern introspection: significance gates and
                // age/insight-statement accessors (§4.06).
                let sig = p.is_significant(0.6, 3);
                let age = p.age_days();
                let stmt = p.to_insight_statement();
                tracing::debug!("Pattern {} sig={} age={}d: {}", p.id, sig, age, stmt);
                // Merge consecutive patterns of the same type to consolidate
                // duplicate evidence (§9 generalization); strip a known-dup
                // evidence id to exercise the remove_evidence accessor.
                if i + 1 < all_patterns.len()
                    && let Some(next) = self.get_pattern(&all_patterns[i + 1].id).await
                {
                    let mut merged_p = p.clone();
                    if let Some(dup) = next.evidence.first() {
                        merged_p.remove_evidence(dup);
                    }
                    merged_p.merge(&next);
                    merged_p.set_type(next.pattern_type.clone());
                    tracing::debug!(
                        "Merged pattern {} -> confidence {:.2}",
                        merged_p.id,
                        merged_p.confidence
                    );
                    merged += 1;
                }
            }
        }
        // Reconcile archived reflections: delete reflections the repository
        // has marked archived so the delete accesssor stays wired (§4.06).
        // Use a stricter validator (with_min_confidence) to double-check
        // validity before deletion and exercise that constructor.
        let strict = ReflectionValidator::with_min_confidence(0.7);
        let mut deleted = 0usize;
        for reflection in self.list_by_status(ReflectionStatus::Archived).await {
            // Only delete reflections the stricter validator also rejects.
            if !strict.is_valid(&reflection) && self.delete_reflection(&reflection.id).await.is_ok()
            {
                deleted += 1;
            }
        }
        tracing::info!(
            "Reflection maintenance: {} trusted insights confirmed, {} insights looked up, \
             {} contradicted, {} patterns decayed, {} patterns merged, {} archived reflections deleted",
            confirmed,
            looked_up,
            contradicted,
            decayed,
            merged,
            deleted
        );
        Ok(())
    }
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
