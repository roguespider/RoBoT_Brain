// src/experience/integration/reflection_pipeline.rs

//! Reflection Pipeline - Wires reflection engine to the event system
//!
//! Per Architecture §10:
//! "Reflection transforms experience into understanding"
//! Reflection asks: What happened? Why did it happen? Was the result expected? What should change?

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::types::Experience;
use crate::experience::reflection::{ReflectionEngine, ReflectionType, ReflectionStatus};
use crate::experience::reflection::reflection::Reflection;

/// Reflection pipeline that processes experiences into insights
pub struct ReflectionPipeline {
    engine: Arc<ReflectionEngine>,
    bus: Arc<ExperienceBus>,
    min_experiences_for_pattern: usize,
}

impl ReflectionPipeline {
    /// Create a new reflection pipeline
    pub fn new(engine: Arc<ReflectionEngine>, bus: Arc<ExperienceBus>) -> Self {
        Self {
            engine,
            bus,
            min_experiences_for_pattern: 3,
        }
    }

    /// Process an experience and generate reflection
    ///
    /// Per Architecture §10:
    /// - What happened?
    /// - Why did it happen?
    /// - Was the result expected?
    /// - What should change?
    pub async fn process(&self, experience: &Experience) -> Result<Option<Reflection>> {
        // Determine reflection type based on outcome
        let reflection_type = self.determine_reflection_type(experience);

        // Generate reflection title
        let title = self.generate_title(experience, reflection_type.clone());

        // Create reflection
        let reflection = self.engine
            .generate_from_single(experience, title)
            .await?;

        // Add detailed description
        let mut reflection = reflection;
        reflection.description = self.generate_description(experience, reflection_type.clone());

        // Add lessons learned
        let lessons = self.extract_lessons(experience);
        for lesson in &lessons {
            reflection.tags.push(lesson.clone());
        }

        // Check if actionable
        if reflection.is_actionable() {
            reflection.status = ReflectionStatus::Validated;
            self.publish_insights(&reflection).await?;
        }

        // Publish ReflectionCompleted event
        let event = ExperienceEvent::reflection_completed(
            experience.id,
            Uuid::parse_str(&reflection.id).unwrap_or_default(),
        );
        let _ = self.bus.publish(event);

        tracing::info!("Generated {:?} reflection for experience {}", reflection_type, experience.id);
        Ok(Some(reflection))
    }

    /// Analyze multiple experiences to detect patterns
    pub async fn analyze_patterns(&self, experiences: &[Experience]) -> Result<Vec<String>> {
        if experiences.len() < self.min_experiences_for_pattern {
            return Ok(vec![]);
        }

        let report = self.engine.analyze_experiences(experiences).await?;
        Ok(report.patterns)
    }

    /// Determine reflection type from experience outcome
    fn determine_reflection_type(&self, experience: &Experience) -> ReflectionType {
        use crate::experience::types::outcome::OutcomeKind;
        match experience.outcome.kind {
            OutcomeKind::Success => ReflectionType::Success,
            OutcomeKind::Failure => ReflectionType::Failure,
            OutcomeKind::Interrupted => ReflectionType::Improvement,
            _ => ReflectionType::General,
        }
    }

    /// Generate reflection title
    fn generate_title(&self, experience: &Experience, reflection_type: ReflectionType) -> String {
        let prefix = match reflection_type {
            ReflectionType::Success => "Success:",
            ReflectionType::Failure => "Lesson from failure:",
            ReflectionType::Improvement => "Opportunity:",
            ReflectionType::Pattern => "Pattern:",
            ReflectionType::Anomaly => "Anomaly:",
            ReflectionType::Strategy => "Strategy:",
            ReflectionType::General => "Reflection:",
        };

        format!("{} {}", prefix, experience.title)
    }

    /// Generate detailed description for reflection
    fn generate_description(&self, experience: &Experience, reflection_type: ReflectionType) -> String {
        let mut desc = String::new();

        match reflection_type {
            ReflectionType::Success => {
                desc.push_str("What worked well:\n");
                desc.push_str(&format!("- {}\n", experience.description));
                desc.push_str("\nKey factors:\n");
                if let Some(source) = &experience.context.source {
                    desc.push_str(&format!("- Source: {}\n", source));
                }
            }
            ReflectionType::Failure => {
                desc.push_str("What went wrong:\n");
                desc.push_str(&format!("- {}\n", experience.description));
                if let Some(reason) = &experience.outcome.error {
                    desc.push_str(&format!("\nFailure reason: {}\n", reason));
                }
            }
            _ => {
                desc.push_str(&experience.description);
            }
        }

        desc
    }

    /// Extract lessons from experience
    fn extract_lessons(&self, experience: &Experience) -> Vec<String> {
        use crate::experience::types::outcome::OutcomeKind;
        let mut lessons = Vec::new();

        // Add outcome-based lessons
        match experience.outcome.kind {
            OutcomeKind::Success => {
                lessons.push("successful_outcome".to_string());
            }
            OutcomeKind::Failure => {
                lessons.push("failed_outcome".to_string());
                if let Some(reason) = &experience.outcome.error
                    && !reason.is_empty() {
                        lessons.push(format!("failure_reason:{}", reason));
                    }
            }
            _ => {}
        }

        // Add type-based lessons
        match experience.experience_type {
            crate::experience::types::ExperienceType::ToolExecution => {
                lessons.push("tool_execution".to_string());
            }
            crate::experience::types::ExperienceType::Planning => {
                lessons.push("planning".to_string());
            }
            crate::experience::types::ExperienceType::Workflow => {
                lessons.push("workflow".to_string());
            }
            _ => {}
        }

        lessons
    }

    /// Publish insights from reflection to the event bus
    async fn publish_insights(&self, reflection: &Reflection) -> Result<()> {
        // Create insights for validated reflections
        if reflection.status == ReflectionStatus::Validated {
            let insight = self.engine.create_insight(
                reflection.title.clone(),
                reflection.description.clone(),
                vec![reflection.id.clone()],
            ).await?;

            tracing::info!("Created insight from validated reflection: {}", insight.id);
        }

        Ok(())
    }
}
