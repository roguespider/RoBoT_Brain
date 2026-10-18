//! Enumerations for the LoopRunner module.

/// Stage of the loop cycle.
#[derive(Clone, PartialEq, Eq)]
pub enum LoopStage {
    ObserveState,
    CollectObjectives,
    EvaluateQueue,
    SelectObjective,
    Plan,
    Execute,
    Verify,
    RecordExperience,
    UpdateKnowledge,
    Reflect,
    EvaluateCurrentState,
    GenerateNewObjectives,
}

/// Architecture-level stages in the continuous cognitive cycle (§20).
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CognitiveCycleStage {
    Observe,
    Evaluate,
    Prioritize,
    Plan,
    Execute,
    Verify,
    Learn,
    Reflect,
    FindNextObjective,
}

impl std::fmt::Debug for CognitiveCycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CognitiveCycleStage::Observe => write!(f, "Observe"),
            CognitiveCycleStage::Evaluate => write!(f, "Evaluate"),
            CognitiveCycleStage::Prioritize => write!(f, "Prioritize"),
            CognitiveCycleStage::Plan => write!(f, "Plan"),
            CognitiveCycleStage::Execute => write!(f, "Execute"),
            CognitiveCycleStage::Verify => write!(f, "Verify"),
            CognitiveCycleStage::Learn => write!(f, "Learn"),
            CognitiveCycleStage::Reflect => write!(f, "Reflect"),
            CognitiveCycleStage::FindNextObjective => write!(f, "FindNextObjective"),
        }
    }
}

impl LoopStage {
    pub fn cognitive_cycle_stage(&self) -> CognitiveCycleStage {
        match self {
            LoopStage::ObserveState => CognitiveCycleStage::Observe,
            LoopStage::CollectObjectives => CognitiveCycleStage::Evaluate,
            LoopStage::EvaluateQueue => CognitiveCycleStage::Prioritize,
            LoopStage::SelectObjective => CognitiveCycleStage::Plan,
            LoopStage::Plan => CognitiveCycleStage::Execute,
            LoopStage::Execute => CognitiveCycleStage::Verify,
            LoopStage::Verify => CognitiveCycleStage::Learn,
            LoopStage::RecordExperience => CognitiveCycleStage::Reflect,
            LoopStage::UpdateKnowledge => CognitiveCycleStage::FindNextObjective,
            LoopStage::Reflect => CognitiveCycleStage::Observe,
            LoopStage::EvaluateCurrentState => CognitiveCycleStage::Evaluate,
            LoopStage::GenerateNewObjectives => CognitiveCycleStage::Prioritize,
        }
    }
}

impl std::fmt::Debug for LoopStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopStage::ObserveState => write!(f, "ObserveState"),
            LoopStage::CollectObjectives => write!(f, "CollectObjectives"),
            LoopStage::EvaluateQueue => write!(f, "EvaluateQueue"),
            LoopStage::SelectObjective => write!(f, "SelectObjective"),
            LoopStage::Plan => write!(f, "Plan"),
            LoopStage::Execute => write!(f, "Execute"),
            LoopStage::Verify => write!(f, "Verify"),
            LoopStage::RecordExperience => write!(f, "RecordExperience"),
            LoopStage::UpdateKnowledge => write!(f, "UpdateKnowledge"),
            LoopStage::Reflect => write!(f, "Reflect"),
            LoopStage::EvaluateCurrentState => write!(f, "EvaluateCurrentState"),
            LoopStage::GenerateNewObjectives => write!(f, "GenerateNewObjectives"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CyclePhase {
    Observe,
    FindNextObjective,
    Wait,
}

impl std::fmt::Debug for CyclePhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => formatter.write_str("Observe"),
            Self::FindNextObjective => formatter.write_str("FindNextObjective"),
            Self::Wait => formatter.write_str("Wait"),
        }
    }
}
