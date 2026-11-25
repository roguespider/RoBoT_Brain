//! Reflection logic for the CoObOpLoop.

use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};

/// Reflect on completed objectives and generate insights.
/// Per CoObOpLoop.md §20: loop includes Reflect stage.
pub fn reflect(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::Reflect;

    // Use the ReflectionEngine to generate insights from the latest experience.
    // Per Architecture §15: Learning feedback generates insights.
    if let Some(ref latest) = runner.latest_experience {
        // Generate a reflection title and content from the experience.
        // Per CoObOpLoop.md §20: Reflect stage generates insights.
        let title = format!("Reflection on {}", latest.title);
        let outcome_str = format!("{:?}", latest.outcome.kind);
        let content = format!(
            "Experience: {}. Outcome: {}. Lessons: {:?}",
            latest.title, outcome_str, latest.lessons_learned
        );

        // Log the reflection for observability.
        tracing::info!(
            "reflect: generated reflection - {} ({})",
            title,
            content.chars().take(60).collect::<String>()
        );

        // Store learning lessons in strategy tracker.
        // Per Architecture §15: Experience -> Learning -> Strategy refinement.
        if let Ok(mut tracker) = runner.strategy_tracker.lock()
            && !latest.lessons_learned.is_empty()
        {
            for lesson in &latest.lessons_learned {
                if lesson.contains("Successfully") {
                    let strategy = lesson
                        .strip_prefix("Successfully completed: ")
                        .unwrap_or(lesson);
                    if !tracker.reinforced.contains(&strategy.to_string()) {
                        tracker.reinforced.push(strategy.to_string());
                    }
                }
            }
        }
    } else {
        tracing::debug!("reflect: no latest experience to reflect on");
    }
}
