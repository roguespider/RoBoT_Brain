//! Prompt injection guard per Architecture Chapter 17.
//! Detects and prevents prompt injection attacks.

/// Known injection patterns.
const INJECTION_PATTERNS: [&str; 10] = [
    "ignore previous instructions",
    "system override",
    "jailbreak",
    "disregard safety",
    "forget your training",
    "you are now",
    "pretend to be",
    "roleplay as",
    "simulate being",
    "bypass restrictions",
];

/// Scan text for prompt injection patterns.
pub fn scan_for_injection(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut found = Vec::new();

    for pattern in INJECTION_PATTERNS {
        if lower.contains(pattern) {
            found.push(pattern.to_string());
        }
    }

    found
}

/// Check if text contains any injection patterns.
pub fn has_injection(text: &str) -> bool {
    !scan_for_injection(text).is_empty()
}

/// Sanitize text by removing detected injection patterns.
pub fn sanitize(text: &str) -> String {
    let mut result = text.to_string();
    for pattern in INJECTION_PATTERNS {
        result = result.replace(pattern, "[REDACTED]");
    }
    result
}
