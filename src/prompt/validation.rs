//! Prompt validation per Architecture Chapter 17.
//! Validates prompt safety and structure.

/// Validation result for a prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether the prompt is valid.
    pub valid: bool,
    /// Validation errors, if any.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful result.
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed result with errors.
    pub fn err(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// Validate a constructed prompt for safety and structure.
pub fn validate_prompt(text: &str, layers: &[String]) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if text.is_empty() {
        errors.push("Prompt text is empty".to_string());
    }

    if text.len() > 100_000 {
        warnings.push(format!(
            "Prompt text is very long ({} characters), may exceed model context",
            text.len()
        ));
    }

    if layers.is_empty() {
        warnings.push("No prompt layers defined".to_string());
    }

    // Check for potential injection patterns
    let lower = text.to_lowercase();
    if lower.contains("ignore previous instructions") || lower.contains("system override") {
        errors.push("Potential prompt injection detected".to_string());
    }

    if errors.is_empty() {
        ValidationResult::ok().with_warnings(warnings)
    } else {
        ValidationResult::err(errors).with_warnings(warnings)
    }
}

impl ValidationResult {
    fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}
