//! Prompt templates per Architecture Chapter 17.
//! Pre-defined prompt structures for common task types.

/// A reusable prompt template with named slots.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptTemplate {
    /// Template name.
    pub name: String,
    /// Template text with {slot} placeholders.
    pub pattern: String,
    /// Required slot names.
    pub required_slots: Vec<String>,
}

impl PromptTemplate {
    /// Create a new template.
    pub fn new(name: &str, pattern: &str, required_slots: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            pattern: pattern.to_string(),
            required_slots,
        }
    }

    /// Fill the template with provided values.
    pub fn fill(&self, values: &[(&str, &str)]) -> String {
        let mut result = self.pattern.clone();
        for (slot, value) in values {
            result = result.replace(&format!("{{{slot}}}"), value);
        }
        result
    }

    /// Check if all required slots are provided.
    pub fn has_required_slots(&self, provided: &[&str]) -> bool {
        self.required_slots
            .iter()
            .all(|slot| provided.contains(&slot.as_str()))
    }
}

/// Pre-defined templates for common task types.
pub struct StandardTemplates;

impl StandardTemplates {
    /// Coding task template.
    pub fn coding() -> PromptTemplate {
        PromptTemplate::new(
            "coding",
            "You are an expert programmer. Task: {task}\nContext: {context}\nRequirements: {requirements}\nOutput: {output}",
            vec!["task".to_string(), "output".to_string()],
        )
    }

    /// Research task template.
    pub fn research() -> PromptTemplate {
        PromptTemplate::new(
            "research",
            "Research question: {question}\nSources: {sources}\nFindings: {findings}\nConclusion: {conclusion}",
            vec!["question".to_string(), "sources".to_string()],
        )
    }

    /// Planning task template.
    pub fn planning() -> PromptTemplate {
        PromptTemplate::new(
            "planning",
            "Goal: {goal}\nConstraints: {constraints}\nSteps: {steps}\nSuccess criteria: {criteria}",
            vec!["goal".to_string(), "steps".to_string()],
        )
    }

    /// Debugging task template.
    pub fn debugging() -> PromptTemplate {
        PromptTemplate::new(
            "debugging",
            "Error: {error}\nStack trace: {trace}\nContext: {context}\nExpected: {expected}",
            vec!["error".to_string(), "expected".to_string()],
        )
    }
}
