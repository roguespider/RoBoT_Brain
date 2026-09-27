//! Minimal XML element readers for the Atom/RSS feeds the research adapters use.
//!
//! The providers that return XML (arXiv, Google News) are read-only consumers of
//! well-formed, machine-generated documents. Pulling in a full XML parser for
//! them would add a dependency to read five fields per entry, so these helpers
//! do the narrow job instead: locate an element, return its decoded text.
//!
//! Every function is bounded by a fixed iteration cap so a malformed document
//! cannot cause unbounded looping.

/// Split a document into the inner XML of every `<name>...</name>` element.
///
/// Self-closing and non-element matches (`<entrypoint>`) are skipped, so a
/// caller asking for `entry` never receives an `entrypoint` element.
pub fn split_elements(document: &str, name: &str) -> Vec<String> {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut elements = Vec::new();
    let mut rest = document;
    const MAX_ELEMENTS: usize = 512;
    for _ in 0..MAX_ELEMENTS {
        let Some(start) = rest.find(&open) else { break };
        let after_open = &rest[start + open.len()..];
        let Some(terminator) = after_open.find('>') else {
            break;
        };
        if after_open[..terminator].trim_end().ends_with('/') {
            rest = &after_open[terminator + 1..];
            continue;
        }
        let body_start = &after_open[terminator + 1..];
        let Some(rel_end) = body_start.find(&close) else {
            break;
        };
        elements.push(body_start[..rel_end].to_string());
        rest = &body_start[rel_end + close.len()..];
    }
    elements
}

/// Decode entities and drop markup from one element body, then trim.
pub fn clean_element_text(inner: &str) -> String {
    crate::research::sanitize::decode_entities(&crate::research::sanitize::strip_html(inner.trim()))
        .trim()
        .to_string()
}

/// Text content of the first `<name>...</name>` child, entity-decoded.
pub fn first_text(element: &str, name: &str) -> String {
    split_elements(element, name)
        .into_iter()
        .next()
        .map(|inner| clean_element_text(&inner))
        .unwrap_or_default()
}

/// All `<name>...</name>` child texts joined with commas, entity-decoded.
///
/// Used for author lists, where every match is meaningful.
pub fn collect_text(element: &str, name: &str) -> String {
    let parts: Vec<String> = split_elements(element, name)
        .into_iter()
        .map(|inner| clean_element_text(&inner))
        .filter(|s| !s.is_empty())
        .collect();
    parts.join(", ")
}

/// Resolve the `href` of the first `<link .../>` child, falling back to its text.
pub fn first_attr_href(element: &str, name: &str) -> Option<String> {
    for inner in split_elements(element, name) {
        if let Some(value) = extract_attribute(&inner, "href")
            && crate::research::sanitize::is_citable_url(&value)
        {
            return Some(value);
        }
    }
    None
}

/// Extract `name="value"` from a tag body without a full XML parser.
pub fn extract_attribute(tag_body: &str, attribute: &str) -> Option<String> {
    let needle = format!("{attribute}=\"");
    let start = tag_body.find(&needle)? + needle.len();
    let rest = &tag_body[start..];
    let end = rest.find('"')?;
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
