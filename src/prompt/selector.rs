//! Prompt template selection per Architecture Chapter 17.
//! Selects the appropriate template based on task type and context.

/// Task type for template selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Coding,
    Research,
    Planning,
    Debugging,
    General,
}

/// Select the best template for the given task type.
pub fn select_template(task_type: TaskType) -> &'static str {
    match task_type {
        TaskType::Coding => "coding",
        TaskType::Research => "research",
        TaskType::Planning => "planning",
        TaskType::Debugging => "debugging",
        TaskType::General => "general",
    }
}

/// Detect the task type from a query string.
pub fn detect_task_type(query: &str) -> TaskType {
    let lower = query.to_lowercase();
    if lower.contains("code") || lower.contains("program") || lower.contains("function") {
        TaskType::Coding
    } else if lower.contains("research") || lower.contains("find") || lower.contains("search") {
        TaskType::Research
    } else if lower.contains("plan") || lower.contains("strategy") || lower.contains("goal") {
        TaskType::Planning
    } else if lower.contains("debug") || lower.contains("error") || lower.contains("fix") {
        TaskType::Debugging
    } else {
        TaskType::General
    }
}
