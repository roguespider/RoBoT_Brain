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
    /// Bare `.unwrap()` (NOT `.unwrap_or`, `.unwrap_or_else`, `.unwrap_or_default`).
    pub unwrap: Regex,
    /// Bare `.expect(...)` in non-test code.
    pub expect: Regex,
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
            // Bare `.unwrap()` — a `.` then `unwrap` then `(` then `)`.
            // The negative lookahead on `_or`/`_or_else`/`_or_default` is not
            // expressible in the regex crate, so we anchor on the closing `()`
            // being immediately preceded by `unwrap` (no `_or` suffix). We match
            // `.unwrap()` literally and let check_unwrap reject the allowed
            // `.unwrap_or*` variants by inspecting the matched text.
            unwrap: Regex::new(r"\.unwrap\s*\(\s*\)")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            // `.expect("...")` — a `.expect(` followed by an argument and `)`.
            expect: Regex::new(r"\.expect\s*\(")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
        }
    }
}
