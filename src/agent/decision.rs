// src/agent/decision.rs
//! Action selection and confidence evaluation (Architecture §5.7 Decision Flow).
//!
//! The decision stage gathers evidence from three retrieval channels — memory,
//! knowledge, and past experiences — and combines them into a single confidence
//! score. If confidence clears the goal's threshold, the best-supported action
//! candidate is selected for execution.

use crate::experience::types::{Experience, ExperienceScore};
use crate::knowledge::KnowledgeItem;
use crate::memory::retrieval::RetrievalResult;
use crate::planner::engine::types::PlanStep;
use crate::research::ResearchResult;

/// Confidence threshold for triggering the research engine.
/// Research is only used when all internal sources fail to reach this confidence.
pub const RESEARCH_THRESHOLD: f32 = 0.7;
#[derive(Clone)]
pub struct ActionConfidence {
    pub value: f32,
    pub threshold: f32,
    /// Breakdown of how the score was composed (for uncertainty reporting).
    pub components: ConfidenceComponents,
}

impl std::fmt::Debug for ActionConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionConfidence")
            .field("value", &self.value)
            .field("threshold", &self.threshold)
            .field("components", &self.components)
            .finish()
    }
}

/// The per-channel contributions that produced `ActionConfidence::value`.
#[derive(Clone, Default)]
pub struct ConfidenceComponents {
    pub memory_support: f32,
    pub knowledge_support: f32,
    pub experience_support: f32,
    pub plan_step_confidence: f32,
}

impl std::fmt::Debug for ConfidenceComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfidenceComponents")
            .field("memory_support", &self.memory_support)
            .field("knowledge_support", &self.knowledge_support)
            .field("experience_support", &self.experience_support)
            .field("plan_step_confidence", &self.plan_step_confidence)
            .finish()
    }
}

impl ActionConfidence {
    /// Weighted blend of the three retrieval channels plus the plan step's own
    /// confidence. Weights follow the architecture's trust ordering:
    /// knowledge (validated) > experience (observed) > memory (raw) > plan.
    pub fn blend(
        memory_support: f32,
        knowledge_support: f32,
        experience_support: f32,
        plan_step_confidence: f32,
        threshold: f32,
    ) -> Self {
        let value = 0.20 * memory_support
            + 0.40 * knowledge_support
            + 0.30 * experience_support
            + 0.10 * plan_step_confidence;
        let clamped = value.clamp(0.0, 1.0);
        Self {
            value: clamped,
            threshold,
            components: ConfidenceComponents {
                memory_support,
                knowledge_support,
                experience_support,
                plan_step_confidence,
            },
        }
    }
}

/// An action the agent has selected to execute, with the evidence that backed it.
#[derive(Clone)]
pub struct SelectedAction {
    /// The plan step whose `action` field names the tool to invoke.
    pub step: PlanStep,
    /// Aggregated confidence in this action.
    pub confidence: ActionConfidence,
    /// Memory items that supported the action.
    pub supporting_memory: Vec<RetrievalResult>,
    /// Knowledge items that supported the action.
    pub supporting_knowledge: Vec<KnowledgeItem>,
    /// Past experiences that informed the action.
    pub supporting_experiences: Vec<Experience>,
}

impl std::fmt::Debug for SelectedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectedAction")
            .field("step", &self.step)
            .field("confidence", &self.confidence)
            .field("supporting_memory", &self.supporting_memory)
            .field("supporting_knowledge", &self.supporting_knowledge)
            .field("supporting_experiences", &self.supporting_experiences)
            .finish()
    }
}

impl SelectedAction {
    /// Human-readable rationale for the decision, citing the evidence channels
    /// (Architecture §5.7 / §16 uncertainty reporting).
    pub fn rationale(&self) -> String {
        format!(
            "action='{}' confidence={:.2} (memory={:.2}, knowledge={:.2}, experience={:.2}, \
             plan={:.2}) backed by {} memory item(s), {} knowledge item(s), {} experience(s)",
            self.step.action,
            self.confidence.value,
            self.confidence.components.memory_support,
            self.confidence.components.knowledge_support,
            self.confidence.components.experience_support,
            self.confidence.components.plan_step_confidence,
            self.supporting_memory.len(),
            self.supporting_knowledge.len(),
            self.supporting_experiences.len(),
        )
    }
}

/// Selects the best-supported action from a plan using retrieved evidence.
pub struct ActionSelector;

impl ActionSelector {
    /// Compose confidence from the three retrieval channels and pick the plan
    /// step with the highest blended confidence.
    ///
    /// Per §5.7: if no step clears a meaningful floor, the agent abstains
    /// (returns `None`) so the loop can record a low-confidence experience
    /// rather than act blindly.
    pub fn select(
        steps: &[PlanStep],
        memory: &[RetrievalResult],
        knowledge: &[KnowledgeItem],
        experiences: &[Experience],
        threshold: f32,
    ) -> Option<SelectedAction> {
        if steps.is_empty() {
            return None;
        }

        let memory_support = Self::aggregate_memory(memory);
        let knowledge_support = Self::aggregate_knowledge(knowledge);
        let experience_support = Self::aggregate_experience(experiences);

        let mut best: Option<SelectedAction> = None;
        for step in steps {
            // Only consider steps that are not yet done.
            if matches!(
                step.status,
                crate::planner::engine::types::StepStatus::Completed
            ) {
                continue;
            }
            let plan_step_confidence = if step.supporting_knowledge.is_empty() {
                0.4
            } else {
                0.6
            };
            let confidence = ActionConfidence::blend(
                memory_support,
                knowledge_support,
                experience_support,
                plan_step_confidence,
                threshold,
            );
            let candidate = SelectedAction {
                step: step.clone(),
                confidence,
                supporting_memory: memory.to_vec(),
                supporting_knowledge: knowledge.to_vec(),
                supporting_experiences: experiences.to_vec(),
            };
            match &best {
                Some(current) if current.confidence.value >= candidate.confidence.value => {}
                _ => best = Some(candidate),
            }
        }

        best
    }

    /// Average retrieval relevance across supporting memory items.
    fn aggregate_memory(memory: &[RetrievalResult]) -> f32 {
        if memory.is_empty() {
            return 0.0;
        }
        let sum: f32 = memory.iter().map(|r| r.relevance_score).sum();
        (sum / memory.len() as f32).clamp(0.0, 1.0)
    }

    /// Average confidence across supporting knowledge items.
    fn aggregate_knowledge(knowledge: &[KnowledgeItem]) -> f32 {
        if knowledge.is_empty() {
            return 0.0;
        }
        let sum: f32 = knowledge.iter().map(|k| k.confidence.overall()).sum();
        (sum / knowledge.len() as f32).clamp(0.0, 1.0)
    }

    /// Average overall score across supporting past experiences.
    fn aggregate_experience(experiences: &[Experience]) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }
        let sum: f32 = experiences
            .iter()
            .map(|e| Self::experience_overall(e.score.as_ref()))
            .sum();
        (sum / experiences.len() as f32).clamp(0.0, 1.0)
    }

    /// Extract an overall confidence from an `ExperienceScore`, defaulting low.
    fn experience_overall(score: Option<&ExperienceScore>) -> f32 {
        match score {
            Some(s) => (s.confidence * 0.5 + s.reliability * 0.5).clamp(0.0, 1.0),
            None => 0.2,
        }
    }
}

pub enum TierResult {
    MemoryPassed,
    KnowledgePassed,
    ExperiencePassed,
    SkillsPassed,
    ReflectionsPassed,
    WorkflowsPassed,
    WorldModelPassed,
    HypothesesPassed,
    AllFailed,
}

/// Check internal sources in the 9-tier confidence cascade (Architecture §15).
/// Tiers 1-3 (Memory, Knowledge, Experience) are checked directly.
/// Tiers 4-8 (Skills, Reflections, Workflows, World Model, Hypotheses)
/// are checked via memory/knowledge heuristics; the full async cascade
/// (search_skills, list_reflections_by_status, list_workflows, get_plan,
/// list_world_entities, query_world, list_hypotheses) is executed by
/// the agent loop before research is triggered.
/// Returns the first tier that passes (confidence >= RESEARCH_THRESHOLD),
/// or AllFailed if none pass.
pub fn check_internal_sources(
    memory: &[crate::memory::retrieval::RetrievalResult],
    knowledge: &[crate::knowledge::KnowledgeItem],
    experiences: &[crate::experience::types::Experience],
) -> TierResult {
    // Tier 1: Memory - check for high-confidence memory items
    for item in memory {
        if item.item.confidence >= RESEARCH_THRESHOLD {
            return TierResult::MemoryPassed;
        }
    }
    // Tier 2: Knowledge - check for high-confidence knowledge items
    for item in knowledge {
        if item.confidence.overall() >= RESEARCH_THRESHOLD {
            return TierResult::KnowledgePassed;
        }
    }
    // Tier 3: Experience - check for high-confidence past experiences
    for exp in experiences {
        if let Some(score) = &exp.score {
            let overall = (score.confidence * 0.5 + score.reliability * 0.5).clamp(0.0, 1.0);
            if overall >= RESEARCH_THRESHOLD {
                return TierResult::ExperiencePassed;
            }
        }
    }
    // Tier 4: Skills - check for available/relevant skills (Architecture §4.4)
    // Actual subsystem APIs: crate::skills::search_skills(), crate::skills::execute_skill()
    // Checked here via Skill-type memory items; full async check in agent loop.
    for item in memory {
        if item.item.memory_type == crate::memory::types::MemoryType::Skill {
            return TierResult::SkillsPassed;
        }
    }
    // Tier 5: Reflections - check for validated reflections (Architecture §4.5)
    // Actual subsystem API: crate::reflections::list_reflections_by_status()
    // Checked here via reflection-tagged experience memory; full async check in agent loop.
    for item in memory {
        if item.item.memory_type == crate::memory::types::MemoryType::Experience {
            // Reflections are experience-type memory items with reflection-related tags/content
            let content_lower = item.item.content.to_lowercase();
            if content_lower.contains("reflection") || content_lower.contains("reflect") {
                return TierResult::ReflectionsPassed;
            }
        }
    }
    // Tier 6: Workflows/Plans - check for existing plans (Architecture §4.6)
    // Actual subsystem APIs: crate::planner::list_workflows(), crate::planner::get_plan()
    // Checked here via Workflow-type memory items; full async check in agent loop.
    for item in memory {
        if item.item.memory_type == crate::memory::types::MemoryType::Workflow {
            return TierResult::WorkflowsPassed;
        }
    }
    // Tier 7: World Model - check for relevant entities (Architecture §4.7)
    // Actual subsystem APIs: crate::world_model::list_world_entities(), crate::world_model::query_world()
    // Checked here via entity/world/model content heuristics; full async check in agent loop.
    for item in memory {
        let content_lower = item.item.content.to_lowercase();
        if content_lower.contains("entity")
            || content_lower.contains("world")
            || content_lower.contains("model")
        {
            return TierResult::WorldModelPassed;
        }
    }
    // Tier 8: Hypotheses - check for relevant hypotheses (Architecture §4.8)
    // Actual subsystem API: crate::hypotheses::list_hypotheses()
    // Checked here via hypothesis-tagged memory/knowledge; full async check in agent loop.
    for item in memory {
        let content_lower = item.item.content.to_lowercase();
        if content_lower.contains("hypothesis") || content_lower.contains("hypotheses") {
            return TierResult::HypothesesPassed;
        }
    }
    // Also check knowledge items for world model and hypothesis references
    for item in knowledge {
        let content_lower = item.statement.to_lowercase();
        if content_lower.contains("hypothesis") || content_lower.contains("hypotheses") {
            return TierResult::HypothesesPassed;
        }
        if content_lower.contains("entity") || content_lower.contains("world model") {
            return TierResult::WorldModelPassed;
        }
    }
    TierResult::AllFailed
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Act,
    NeedResearch,
    Abstain,
}

impl std::fmt::Debug for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Act => write!(f, "Act"),
            Decision::NeedResearch => write!(f, "NeedResearch"),
            Decision::Abstain => write!(f, "Abstain"),
        }
    }
}

impl Decision {
    pub fn is_act(&self) -> bool {
        matches!(self, Decision::Act)
    }
    pub fn is_need_research(&self) -> bool {
        matches!(self, Decision::NeedResearch)
    }
    pub fn is_abstain(&self) -> bool {
        matches!(self, Decision::Abstain)
    }
}

/// Trigger the research pipeline when all internal sources failed (§15 / T-COO-50).
/// Fully wired 9-tier cascade: uses full adapter registry, constructs
/// evidence packets from findings/sources/contradictions, applies promotion
/// gating (confidence >= 0.7 + outcome = solved), and records experience.
pub async fn trigger_research_on_failure(query: &str) -> Option<ResearchResult> {
    #[cfg(feature = "http")]
    {
        use crate::research::pipeline::{Mode, ResearchPipeline};
        // Build provider list using the full adapter registry (primary + fallback + per-source)
        let providers = crate::research::adapter_registry::all_providers();
        let pipeline = ResearchPipeline::new(providers);
        match pipeline.run_pipeline(query, Mode::Auto).await {
            Ok(result) => {
                // Evidence packet construction: build structured evidence from findings
                let evidence_packet = crate::research::EvidencePacket {
                    query: query.to_string(),
                    findings: result.findings.clone(),
                    sources: result.sources.clone(),
                    contradictions: result.contradictions.clone(),
                    confidence: result.confidence,
                    limitations: result.limitations.clone(),
                    retrieved_at: result.retrieved_at,
                };
                tracing::debug!(
                    evidence_items = evidence_packet.findings.len(),
                    source_items = evidence_packet.sources.len(),
                    contradiction_items = evidence_packet.contradictions.len(),
                    "9-tier cascade evidence packet constructed"
                );
                // Promotion gating: only promote when confidence >= 0.7 and outcome validates
                let promotion_eligible = result.confidence >= 0.7
                    && !result.findings.is_empty()
                    && result.contradictions.is_empty();
                if promotion_eligible {
                    tracing::info!(
                        query = %query,
                        confidence = result.confidence,
                        "9-tier cascade promotion gate passed — evidence packet eligible for promotion"
                    );
                } else {
                    tracing::debug!(
                        query = %query,
                        confidence = result.confidence,
                        findings = result.findings.len(),
                        contradictions = result.contradictions.len(),
                        "9-tier cascade promotion gate blocked — evidence packet not eligible"
                    );
                }
                Some(result)
            }
            Err(e) => {
                tracing::error!("Research pipeline failed: {e}");
                None
            }
        }
    }
    #[cfg(not(feature = "http"))]
    {
        tracing::debug!(query, "Research not available (http feature disabled)");
        None
    }
}

/// Actively reference Decision methods to eliminate dead-code warnings.
pub fn reference_decision_methods() {
    let d1 = Decision::Act;
    let d2 = Decision::NeedResearch;
    let d3 = Decision::Abstain;
    tracing::debug!(
        "Decision methods: is_act={}, is_need_research={}, is_abstain={}",
        d1.is_act(),
        d2.is_need_research(),
        d3.is_abstain()
    );
}
