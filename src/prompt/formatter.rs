//! Prompt formatting per Architecture Chapter 17.
//! Formats constructed prompts for output to language models.

/// Format a constructed prompt into a single string suitable for model input.
pub fn format_prompt(text: &str, layers: &[String]) -> String {
    let mut result = String::new();
    result.push_str("--- Prompt Layers ---\n");
    for layer in layers {
        result.push_str(&format!("- {layer}\n"));
    }
    result.push_str("--- Prompt Content ---\n");
    result.push_str(text);
    result.push_str("\n--- End ---\n");
    result
}

/// Format sources for attribution.
pub fn format_sources(sources: &[String]) -> String {
    if sources.is_empty() {
        return "No sources".to_string();
    }
    let mut result = String::from("Sources:\n");
    for (i, source) in sources.iter().enumerate() {
        result.push_str(&format!("{}. {}\n", i + 1, source));
    }
    result
}
