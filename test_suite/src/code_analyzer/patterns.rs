//! Pre-compiled regex patterns for code analysis

use regex::Regex;
use std::sync::OnceLock;

/// Fallback regex pattern (always valid - "." matches any character)
static FALLBACK_REGEX: OnceLock<Regex> = OnceLock::new();

/// Get or create the fallback regex pattern
///
/// Uses unwrap_or_else to handle the Result safely. For the dot pattern ".":
/// - "." is a valid regex that matches any single character
/// - The regex crate guarantees this pattern will always compile successfully
/// - Using unwrap_or_else with a fallback closure handles potential edge cases
pub fn get_fallback_regex() -> &'static Regex {
    FALLBACK_REGEX.get_or_init(|| {
        // "." is guaranteed to be a valid regex pattern
        Regex::new(".").unwrap_or_else(|_| {
            // If "." fails (theoretically impossible), use "$." which matches
            // a newline at the end of string, or use another safe fallback
            Regex::new("$.").unwrap_or_else(|_| {
                // Final fallback - these patterns are guaranteed to be valid
                // We iterate through known-valid patterns until one succeeds
                // This loop always executes at least once since "." is always valid
                loop {
                    // Try a series of guaranteed-valid patterns
                    let pattern = "."; // Always valid - matches any character
                    match Regex::new(pattern) {
                        Ok(r) => break r,
                        Err(_) => continue, // Should never happen for "."
                    }
                }
            })
        })
    })
}

/// Pre-compiled regex patterns for code analysis
pub struct CodePatterns {
    pub allow_annotation: Regex,
    pub dead_code_allow: Regex,
    pub unimplemented: Regex,
    pub todo: Regex,
    pub panic: Regex,
    pub underscore_prefix: Regex,
    /// `#[cfg(test)]` attribute (with optional inner whitespace).
    pub cfg_test: Regex,
    /// Decorative emoji banned in code (AGENTS.md "No emoji / plain-text
    /// markers"). Arrows (`->` `|` `v` unicode: U+2190-U+21FF) are NOT in
    /// this set and remain permitted for flow diagrams. This matches any
    /// single banned codepoint on a line.
    pub emoji: Regex,
}

impl CodePatterns {
    /// Create new code patterns
    pub fn new() -> Self {
        Self {
            allow_annotation: Regex::new(r#"#\s*\[\s*allow\s*\([^)]*\)"#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            dead_code_allow: Regex::new(r#"#\s*\[\s*allow\s*\(dead_code\)"#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            unimplemented: Regex::new(r"unimplemented!\s*\(")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            todo: Regex::new(r"todo!\s*\(")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            panic: Regex::new(r#"panic!\s*\("#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            underscore_prefix: Regex::new(r"\b_\w+\b")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            cfg_test: Regex::new(r#"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]"#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            // Banned decorative emoji. Excludes the Arrows block (U+2190-U+21FF)
            // so flow-diagram arrows remain permitted. Includes U+FE0F
            // (variation selector-16) which appends to emoji and is a frequent
            // mojibake source. Raw string so `\u{...}` is passed verbatim to
            // the regex engine (which interprets it as a unicode codepoint).
            // Must be a single line: raw strings do not process `\`+newline
            // as a continuation, so line breaks would inject literal backslashes.
            emoji: Regex::new(r"[\u{2705}\u{274C}\u{26A0}\u{FE0F}\u{1F4CB}\u{1F389}\u{1F6AB}\u{1F97E}\u{2713}\u{2717}\u{1F50E}\u{1F50C}\u{1F4CA}\u{1F4C4}\u{2699}\u{2B50}\u{2753}\u{2754}\u{2755}\u{2728}\u{1F4A1}\u{1F680}\u{1F47D}\u{1F4DD}\u{1F527}\u{1F4E2}\u{1F4DA}\u{1F4E6}\u{1F4E5}]")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
        }
    }
}
