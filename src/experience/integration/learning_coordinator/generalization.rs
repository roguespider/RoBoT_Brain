// src/experience/integration/learning_coordinator/generalization.rs
//! Generalization and transfer learning functions

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::metrics::MetricsCollector;
use crate::experience::types::Experience;
use crate::experience::types::outcome::OutcomeKind;
use crate::knowledge::{KnowledgeItem, KnowledgeStore};

use super::results::{GeneralizationResult, LearningPattern, PatternKind, TransferResult};

/// Generalization and transfer learning methods for LearningCoordinator
pub struct GeneralizationMethods<'a> {
    pub knowledge_store: &'a Arc<KnowledgeStore>,
    pub bus: &'a Arc<ExperienceBus>,
    pub metrics: &'a Arc<MetricsCollector>,
}

impl<'a> GeneralizationMethods<'a> {
    /// Per Architecture §9: Generalization - extracting patterns from specific experiences
    /// Generalize from a set of experiences to create broader patterns
    ///
    /// Per Architecture §9: "Generalization extracts common patterns from specific instances"
    pub async fn generalize(
        &self,
        experience_ids: Vec<Uuid>,
    ) -> Result<GeneralizationResult> {
        let mut result = GeneralizationResult::default();

        tracing::debug!("Generalizing from {} experience IDs", experience_ids.len());

        // Try to extract patterns from in-memory experiences
        // Note: In a full implementation, this would query the experience repository
        // For now, we create basic patterns from the experience IDs
        let mut patterns = Vec::new();

        for id in &experience_ids {
            let pattern = LearningPattern {
                description: format!("Pattern from experience {}", id),
                confidence: 0.5, // Default confidence for new patterns
                source_experience_count: 1,
                pattern_type: PatternKind::Sequential,
            };
            patterns.push(pattern);
        }

        result.patterns = patterns;

        // Create generalized knowledge from successful patterns
        for pattern in &result.patterns {
            // Only promote high-confidence patterns
            if pattern.confidence >= 0.6 {
                let generalized_knowledge = KnowledgeItem::from_reflection(
                    &pattern.description,
                    pattern.confidence,
                    Uuid::new_v4(),
                );
                let _ = self.knowledge_store.add(generalized_knowledge).await;
                result.generalized_knowledge_count += 1;
            }
        }

        self.metrics.increment("learning.generalizations").await;

        Ok(result)
    }

    /// Extract common patterns from experiences
    pub fn extract_common_patterns(&self, experiences: &[Experience]) -> Vec<LearningPattern> {
        let mut patterns = Vec::new();

        if experiences.len() < 2 {
            return patterns;
        }

        // Group by context/workflow
        let mut context_groups: std::collections::HashMap<String, Vec<&Experience>> =
            std::collections::HashMap::new();

        for exp in experiences {
            let key = exp
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            context_groups.entry(key).or_default().push(exp);
        }

        // Find patterns in groups
        for (context, exps) in context_groups {
            if exps.len() >= 2 {
                // Count successful vs failed
                let success_count = exps
                    .iter()
                    .filter(|e| {
                        matches!(
                            e.outcome.kind,
                            OutcomeKind::Success
                        )
                    })
                    .count();

                let confidence = success_count as f32 / exps.len() as f32;

                patterns.push(LearningPattern {
                    description: format!(
                        "In '{}' context, {} out of {} attempts succeeded",
                        context,
                        success_count,
                        exps.len()
                    ),
                    confidence,
                    source_experience_count: exps.len(),
                    pattern_type: PatternKind::Contextual,
                });
            }
        }

        patterns
    }

    /// Per Architecture §9: Transfer learning - applying knowledge from one domain to another
    /// Transfer knowledge from source domain to target domain
    ///
    /// Per Architecture §9: "Transfer learning applies knowledge from one domain to another"
    pub async fn transfer_knowledge(
        &self,
        source_domain: &str,
        target_domain: &str,
        knowledge_ids: Vec<Uuid>,
    ) -> Result<TransferResult> {
        let mut result = TransferResult {
            source_domain: source_domain.to_string(),
            target_domain: target_domain.to_string(),
            transferred_count: 0,
            adapted_count: 0,
            failed_count: 0,
            new_knowledge_ids: Vec::new(),
        };

        // Get source knowledge
        for knowledge_id in &knowledge_ids {
            if let Some(knowledge) = self.knowledge_store.get(*knowledge_id).await {
                // Adapt knowledge for target domain
                let adapted = self
                    .adapt_knowledge_for_domain(&knowledge, target_domain)
                    .await;

                if let Some(mut adapted_knowledge) = adapted {
                    // Lower confidence for transferred knowledge (needs validation)
                    adapted_knowledge.confidence.adjust_source_reliability(-0.2);

                    let new_id = self.knowledge_store.add(adapted_knowledge).await;
                    result.new_knowledge_ids.push(new_id);
                    result.transferred_count += 1;
                    result.adapted_count += 1;
                } else {
                    result.failed_count += 1;
                }
            }
        }

        // Publish transfer event
        let event = ExperienceEvent::knowledge_transferred(
            Uuid::new_v4(),
            source_domain.to_string(),
            target_domain.to_string(),
            result.transferred_count as u32,
        );
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish transfer event: {}", e);
        }

        self.metrics.increment("learning.transfers").await;

        tracing::info!(
            "Transferred {} knowledge items from {} to {}",
            result.transferred_count,
            source_domain,
            target_domain
        );

        Ok(result)
    }

    /// Adapt a knowledge item for a new domain
    pub async fn adapt_knowledge_for_domain(
        &self,
        knowledge: &KnowledgeItem,
        target_domain: &str,
    ) -> Option<KnowledgeItem> {
        // Check if domain is compatible
        let compatibility = Self::check_domain_compatibility(&knowledge.statement, target_domain);

        if compatibility < 0.3 {
            return None; // Not compatible enough
        }

        // Create adapted version
        let mut adapted = knowledge.clone();
        adapted.id = Uuid::new_v4();
        adapted
            .metadata
            .insert("transferred_from".to_string(), knowledge.id.to_string());
        adapted
            .metadata
            .insert("target_domain".to_string(), target_domain.to_string());
        adapted.metadata.insert(
            "original_confidence".to_string(),
            format!("{:.2}", knowledge.overall_confidence()),
        );

        // Scale confidence by compatibility
        let scaled_confidence = knowledge.overall_confidence() * compatibility;
        adapted
            .confidence
            .adjust_source_reliability(scaled_confidence - knowledge.overall_confidence());

        Some(adapted)
    }

    /// Check if knowledge is compatible with a domain
    pub fn check_domain_compatibility(knowledge_statement: &str, target_domain: &str) -> f32 {
        // Simple heuristic: check for domain-specific keywords
        let domain_keywords = match target_domain {
            "coding" => vec![
                "function",
                "variable",
                "class",
                "algorithm",
                "data",
                "process",
            ],
            "writing" => vec!["text", "content", "paragraph", "document", "sentence"],
            "analysis" => vec!["pattern", "trend", "compare", "evaluate", "assess"],
            _ => vec!["general", "common", "standard"],
        };

        let statement_lower = knowledge_statement.to_lowercase();
        let matches = domain_keywords
            .iter()
            .filter(|kw| statement_lower.contains(*kw))
            .count();

        let similarity = matches as f32 / domain_keywords.len() as f32;
        similarity.max(0.1) // Minimum 10% compatibility
    }
}
