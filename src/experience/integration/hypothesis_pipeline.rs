// src/experience/integration/hypothesis_pipeline.rs
#![allow(dead_code)]
//! Hypothesis Pipeline - Wires hypothesis engine to reflection and exploration
//!
//! Per Architecture §11:
//! "Hypotheses Enable Discovery"
//! A hypothesis is: A proposed explanation, Supported by evidence, Assigned confidence, Tested through future experience

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::types::Experience;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::hypothesis::core::hypothesis::{Hypothesis, HypothesisCategory, HypothesisConfidence, HypothesisStatus};
use crate::experience::hypothesis::services::generator::HypothesisGenerator;

/// Configuration for hypothesis generation
#[derive(Debug, Clone)]
pub struct HypothesisPipelineConfig {
    /// Minimum confidence to generate hypothesis
    pub min_confidence: f32,
    /// Whether to auto-explore validated hypotheses
    pub auto_explore: bool,
    /// Confidence threshold for validation
    pub validation_threshold: f32,
    /// Evidence weight for confidence update
    pub supporting_evidence_weight: f32,
    /// Contradicting evidence weight
    pub contradicting_evidence_weight: f32,
}

impl Default for HypothesisPipelineConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            auto_explore: false,
            validation_threshold: 0.75,
            supporting_evidence_weight: 0.1,
            contradicting_evidence_weight: 0.15,
        }
    }
}

/// Hypothesis pipeline that processes experiences into hypotheses
///
/// Per Architecture §2.5:
/// - Generate explanations
/// - Track confidence
/// - Compare competing ideas
/// - Request exploration
/// - Validate assumptions
pub struct HypothesisPipeline {
    config: HypothesisPipelineConfig,
    engine: Arc<HypothesisEngine>,
    generator: Arc<HypothesisGenerator>,
    bus: Arc<ExperienceBus>,
    hypotheses: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Hypothesis>>>,
}

impl HypothesisPipeline {
    /// Create a new hypothesis pipeline
    pub fn new(
        engine: Arc<HypothesisEngine>,
        bus: Arc<ExperienceBus>,
    ) -> Self {
        let generator = Arc::new(HypothesisGenerator::new());
        
        Self {
            config: HypothesisPipelineConfig::default(),
            engine,
            generator,
            bus,
            hypotheses: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        config: HypothesisPipelineConfig,
        engine: Arc<HypothesisEngine>,
        bus: Arc<ExperienceBus>,
    ) -> Self {
        let generator = Arc::new(HypothesisGenerator::new());
        
        Self {
            config,
            engine,
            generator,
            bus,
            hypotheses: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Process an experience and generate hypotheses
    ///
    /// Per Architecture §11:
    /// "A hypothesis is a temporary model waiting for evidence"
    pub async fn process(&self, experience: &Experience) -> Result<Vec<String>> {
        // Generate hypothesis from experience
        if let Some(mut hypothesis) = self.generator.generate(experience)? {
            // Set appropriate category based on experience type
            hypothesis.category = self.categorize_from_experience(experience);
            
            // Calculate initial confidence
            hypothesis.confidence = self.calculate_initial_confidence(experience);
            
            // Only store if confidence meets threshold
            if hypothesis.confidence.value >= self.config.min_confidence {
                let id = hypothesis.id.0.clone();
                
                let mut store = self.hypotheses.write().await;
                store.insert(id.clone(), hypothesis);
                
                // Publish HypothesisGenerated event
                let event = ExperienceEvent::hypothesis_generated(
                    experience.id,
                    Uuid::new_v4(),
                );
                let _ = self.bus.publish(event);
                
                tracing::info!("Generated hypothesis: {}", id);
                return Ok(vec![id]);
            }
        }

        Ok(vec![])
    }

    /// Add supporting evidence to a hypothesis
    ///
    /// Per Architecture §11:
    /// "Evidence strengthens beliefs"
    pub async fn add_supporting_evidence(&self, hypothesis_id: &str, evidence: &str) -> Result<()> {
        let mut store = self.hypotheses.write().await;
        
        if let Some(hypothesis) = store.get_mut(hypothesis_id) {
            hypothesis.add_supporting_evidence(evidence);
            hypothesis.confirmations += 1;
            
            // Update confidence
            hypothesis.confidence.increase(self.config.supporting_evidence_weight);
            hypothesis.touch();
            
            // Check if validated
            if hypothesis.confidence.value >= self.config.validation_threshold {
                hypothesis.status = HypothesisStatus::Supported;
                self.publish_validation(hypothesis_id, true).await?;
            }
            
            tracing::debug!("Added supporting evidence to hypothesis {}, new confidence: {}", 
                hypothesis_id, hypothesis.confidence.value);
        }
        
        Ok(())
    }

    /// Add contradicting evidence to a hypothesis
    ///
    /// Per Architecture §11:
    /// "Evidence can weaken beliefs"
    pub async fn add_contradicting_evidence(&self, hypothesis_id: &str, evidence: &str) -> Result<()> {
        let mut store = self.hypotheses.write().await;
        
        if let Some(hypothesis) = store.get_mut(hypothesis_id) {
            hypothesis.add_contradicting_evidence(evidence);
            hypothesis.contradictions += 1;
            
            // Update confidence (decrease more than supporting evidence increases)
            hypothesis.confidence.decrease(self.config.contradicting_evidence_weight);
            hypothesis.touch();
            
            // Check if rejected
            if hypothesis.confidence.is_uncertain() {
                hypothesis.status = HypothesisStatus::Rejected;
                self.publish_validation(hypothesis_id, false).await?;
            }
            
            tracing::debug!("Added contradicting evidence to hypothesis {}, new confidence: {}", 
                hypothesis_id, hypothesis.confidence.value);
        }
        
        Ok(())
    }

    /// Get a hypothesis by ID
    pub async fn get(&self, hypothesis_id: &str) -> Option<Hypothesis> {
        let store = self.hypotheses.read().await;
        store.get(hypothesis_id).cloned()
    }

    /// List all active hypotheses
    pub async fn list_active(&self) -> Vec<Hypothesis> {
        let store = self.hypotheses.read().await;
        store.values()
            .filter(|h| h.status == HypothesisStatus::Active || h.status == HypothesisStatus::Draft)
            .cloned()
            .collect()
    }

    /// List validated hypotheses (ready for knowledge promotion)
    pub async fn list_validated(&self) -> Vec<Hypothesis> {
        let store = self.hypotheses.read().await;
        store.values()
            .filter(|h| h.status == HypothesisStatus::Supported)
            .filter(|h| h.confidence.is_confident())
            .cloned()
            .collect()
    }

    /// Archive old hypotheses
    pub async fn archive_old(&self, max_age_days: i64) -> Result<usize> {
        use chrono::{Duration, Utc};
        
        let cutoff = Utc::now() - Duration::days(max_age_days);
        let mut store = self.hypotheses.write().await;
        let mut archived = 0;
        
        store.retain(|_id, hypothesis| {
            if hypothesis.updated_at < cutoff && hypothesis.status != HypothesisStatus::Archived {
                hypothesis.status = HypothesisStatus::Archived;
                archived += 1;
                false // Remove from active store
            } else {
                true
            }
        });
        
        tracing::info!("Archived {} old hypotheses", archived);
        Ok(archived)
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    /// Categorize hypothesis based on experience type
    fn categorize_from_experience(&self, experience: &Experience) -> HypothesisCategory {
        match experience.experience_type {
            crate::experience::types::ExperienceType::ToolExecution => HypothesisCategory::Behavioral,
            crate::experience::types::ExperienceType::Planning => HypothesisCategory::Workflow,
            crate::experience::types::ExperienceType::Exploration => HypothesisCategory::Prediction,
            crate::experience::types::ExperienceType::Hypothesis => HypothesisCategory::Knowledge,
            _ => HypothesisCategory::Other,
        }
    }

    /// Calculate initial confidence for hypothesis
    fn calculate_initial_confidence(&self, experience: &Experience) -> HypothesisConfidence {
        let mut confidence = 0.5;

        // Factor in experience confidence
        confidence = confidence * 0.6 + experience.confidence * 0.4;

        // Factor in evidence count
        let evidence_bonus = (experience.evidence_count as f32 * 0.03).min(0.2);
        confidence += evidence_bonus;

        HypothesisConfidence::new(confidence)
    }

    /// Publish hypothesis validation event
    async fn publish_validation(&self, hypothesis_id: &str, validated: bool) -> Result<()> {
        let event = ExperienceEvent::hypothesis_validated(
            Uuid::new_v4(),
            hypothesis_id.to_string(),
            validated,
        );
        let _ = self.bus.publish(event);
        Ok(())
    }
}
