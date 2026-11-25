// /src/CoObOpLoop/learning_pipeline.rs
// Learning pipeline for the CoObOpLoop system.

/// Learning update produced by processing an experience (§15 / T10.17).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LearningUpdate {
    pub capability_updates: Vec<CapabilityUpdate>,
    pub knowledge_additions: Vec<String>,
    pub strategy_refinements: Vec<String>,
    pub risk_adjustments: Vec<RiskAdjustment>,
}

/// Capability update from learning (§15).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityUpdate {
    pub capability_id: String,
    pub level_delta: f32,
    pub reason: String,
}

/// Risk adjustment from learning (§15).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RiskAdjustment {
    pub domain: String,
    pub adjustment: String,
    pub confidence_impact: f32,
}

/// Learning pipeline — processes recorded experiences into updates (§15 / T10.16).
pub struct LearningPipeline;

impl LearningPipeline {
    /// Process an experience and produce a LearningUpdate (§T10.16).
    /// Logs the transition: Experience -> Learning -> Capability/Knowledge Update (§T10.19).
    pub fn process(
        experience: &crate::experience::types::experience::Experience,
    ) -> LearningUpdate {
        let mut updates = LearningUpdate {
            capability_updates: Vec::new(),
            knowledge_additions: Vec::new(),
            strategy_refinements: Vec::new(),
            risk_adjustments: Vec::new(),
        };

        // Extract knowledge additions from lessons
        for lesson in &experience.lessons_learned {
            if !lesson.is_empty() {
                updates.knowledge_additions.push(lesson.clone());
            }
        }

        // Extract knowledge from description
        if !experience.description.is_empty() && experience.description.len() > 10 {
            updates
                .knowledge_additions
                .push(experience.description.clone());
        }

        // Capability updates based on outcome
        let delta: f32 = match experience.outcome.kind {
            crate::experience::types::outcome::OutcomeKind::Success => 0.05,
            crate::experience::types::outcome::OutcomeKind::Failure => -0.03,
            crate::experience::types::outcome::OutcomeKind::Partial => 0.01,
            crate::experience::types::outcome::OutcomeKind::Unknown => 0.0,
            crate::experience::types::outcome::OutcomeKind::Timeout => -0.01,
            crate::experience::types::outcome::OutcomeKind::Interrupted => -0.01,
        };

        if delta.abs() > 0.0f32 {
            updates.capability_updates.push(CapabilityUpdate {
                capability_id: "general".to_string(),
                level_delta: delta,
                reason: format!(
                    "experience {:?}: {}",
                    experience.experience_type, experience.title
                ),
            });
        }

        // Strategy refinements from successful/unsuccessful strategies
        for strategy in &experience.successful_strategies {
            updates
                .strategy_refinements
                .push(format!("reinforce: {}", strategy));
        }
        for strategy in &experience.unsuccessful_strategies {
            updates
                .strategy_refinements
                .push(format!("avoid: {}", strategy));
        }

        // Risk adjustments based on confidence
        let risk_adj = if experience.confidence > 0.7 {
            RiskAdjustment {
                domain: experience.title.clone(),
                adjustment: "increase_trust".to_string(),
                confidence_impact: experience.confidence,
            }
        } else {
            RiskAdjustment {
                domain: experience.title.clone(),
                adjustment: "decrease_trust".to_string(),
                confidence_impact: experience.confidence,
            }
        };
        updates.risk_adjustments.push(risk_adj);

        updates
    }
}
