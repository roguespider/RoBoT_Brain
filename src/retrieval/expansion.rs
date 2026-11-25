//! Query expansion per Architecture Chapter 16.
//! Expands queries with synonyms, related terms, and alternate phrasings.

/// Expand a query with related terms and synonyms.
pub fn expand_query(query: &str) -> Vec<String> {
    let mut expanded = Vec::new();
    expanded.push(query.to_string());

    // Add common expansions based on query terms
    let lower = query.to_lowercase();
    if lower.contains("error") {
        expanded.push(format!("troubleshooting {query}"));
        expanded.push(format!("fix {query}"));
    }
    if lower.contains("how") {
        expanded.push(format!("guide {query}"));
        expanded.push(format!("tutorial {query}"));
    }
    if lower.contains("what") {
        expanded.push(format!("definition {query}"));
        expanded.push(format!("overview {query}"));
    }

    expanded
}

/// Apply expansion to a retrieval query, adding expanded terms.
pub fn apply_expansion(original: &str, expansions: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    result.push(original.to_string());
    for expansion in expansions {
        if expansion != original {
            result.push(expansion.clone());
        }
    }
    result
}

/// Simple query expansion: returns the query, its lowercase form, and each word.
pub fn expand_query_simple(query: &str) -> Vec<String> {
    let mut expanded = Vec::new();
    expanded.push(query.to_string());
    expanded.push(query.to_lowercase());
    for word in query.split_whitespace() {
        expanded.push(word.to_string());
    }
    expanded
}
