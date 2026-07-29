
// src/tools/ingestor/semantic_chunker.rs
// Semantic document chunking - splits documents at natural boundaries
// preserving full document structure for hierarchical storage


use crate::database::models::HierarchyLevel;
use crate::database::models::MemoryType;
use crate::database::models::MemoryCard;
use uuid::Uuid;

/// A node in the document hierarchy tree
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub content: String,
    pub level: HierarchyLevel,
    pub order_index: usize,
    pub path: String,
    pub children: Vec<HierarchyNode>,
}

impl HierarchyNode {
    /// Create a new node
    pub fn new(content: String, level: HierarchyLevel, order_index: usize, path: String) -> Self {
        Self {
            content,
            level,
            order_index,
            path,
            children: Vec::new(),
        }
    }

    /// Flatten the tree into a list of memories with parent relationships
    pub fn flatten(&self) -> Vec<MemoryCard> {
        let mut memories = Vec::new();
        self.flatten_recursive(None, &mut memories);
        memories
    }

    fn flatten_recursive(&self, parent_id: Option<Uuid>, memories: &mut Vec<MemoryCard>) {
        let memory = MemoryCard::new_hierarchical(
            self.content.clone(),
            MemoryType::File,  // Default for file ingestion
            parent_id,
            self.level.clone(),
            self.order_index,
            self.path.clone(),
            None,  // file_source set at root
        );
        
        let current_id = Some(memory.id);
        memories.push(memory);
        
        for child in &self.children {
            child.flatten_recursive(current_id, memories);
        }
    }
}

/// Parse a document and create a hierarchy tree
pub fn parse_document(content: &str, file_name: &str, file_type: &str) -> HierarchyNode {
    match file_type {
        "markdown" | "md" => parse_markdown(content, file_name),
        "json" | "jsonl" => parse_json_document(content, file_name),
        "text" | "txt" | "log" | "rst" => parse_plain_text(content, file_name),
        "html" | "htm" => parse_html(content, file_name),
        "xml" => parse_xml(content, file_name),
        "code" => parse_code(content, file_name),  // Code files
        _ => parse_plain_text(content, file_name),  // Default to plain text
    }
}

/// Parse Markdown document into hierarchy
fn parse_markdown(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Document: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );

    let lines: Vec<&str> = content.lines().collect();
    let mut current_section: Option<HierarchyNode> = None;
    let mut current_paragraph: Vec<String> = Vec::new();
    let mut section_index: usize = 0;
    let mut _paragraph_index = 0;

    // Collect code blocks separately
    let mut in_code_block = false;
    let mut code_block_content: Vec<String> = Vec::new();

    for line in lines.iter() {
        let trimmed = line.trim();

        // Check for code blocks
        if trimmed.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                code_block_content.clear();
            } else {
                in_code_block = false;
                if !code_block_content.is_empty() {
                    let code_content = code_block_content.join("\n");
                    let path = format!("{}/section[{}]/code_block[{}]",
                        file_name,
                        current_section.as_ref().map(|s| s.order_index).unwrap_or(0),
                        _paragraph_index
                    );
                    let code_node = HierarchyNode::new(
                        code_content,
                        HierarchyLevel::Paragraph,
                        _paragraph_index,
                        path
                    );
                    if let Some(ref mut sec) = current_section {
                        sec.children.push(code_node);
                    } else {
                        root.children.push(code_node);
                    }
                    _paragraph_index += 1;
                }
                code_block_content.clear();
            }
            continue;
        }

        if in_code_block {
            code_block_content.push(line.to_string());
            continue;
        }

        if trimmed.starts_with('#') && trimmed.contains(' ') {
            // Push previous section to root before creating new one
            if let Some(prev_section) = current_section.take() {
                root.children.push(prev_section);
            }
            
            if !current_paragraph.is_empty() {
                let para_text = current_paragraph.join(" ").trim().to_string();
                if !para_text.is_empty() {
                    let path = format!("{}/section[{}]/paragraph[{}]",
                        file_name,
                        section_index.saturating_sub(1),
                        _paragraph_index
                    );
                    let node = HierarchyNode::new(para_text, HierarchyLevel::Paragraph, _paragraph_index, path);
                    root.children.push(node);
                    _paragraph_index += 1;
                }
                current_paragraph.clear();
            }

            let header_level = trimmed.find('#').unwrap_or(0);
            let header_text = trimmed.trim_start_matches('#').trim().to_string();
            // Treat level 1 and 2 headers as sections for test compatibility
            let level = if header_level <= 2 { HierarchyLevel::Section } else { HierarchyLevel::Subsection };
            let path = format!("{}/section[{}]", file_name, section_index);
            let section = HierarchyNode::new(header_text, level, section_index, path);
            section_index += 1;
            _paragraph_index = 0;
            current_section = Some(section);
        } else if trimmed.is_empty() {
            if !current_paragraph.is_empty() {
                let para_text = current_paragraph.join(" ").trim().to_string();
                if !para_text.is_empty() {
                    let path = format!("{}/section[{}]/paragraph[{}]",
                        file_name,
                        current_section.as_ref().map(|s| s.order_index).unwrap_or(0),
                        _paragraph_index
                    );
                    let node = HierarchyNode::new(para_text, HierarchyLevel::Paragraph, _paragraph_index, path);
                    if let Some(ref mut sec) = current_section {
                        sec.children.push(node);
                    } else {
                        root.children.push(node);
                    }
                    _paragraph_index += 1;
                }
                current_paragraph.clear();
            }
        } else {
            current_paragraph.push(trimmed.to_string());
        }
    }

    if !current_paragraph.is_empty() {
        let para_text = current_paragraph.join(" ").trim().to_string();
        if !para_text.is_empty() {
            let path = format!("{}/section[{}]/paragraph[{}]",
                file_name,
                current_section.as_ref().map(|s| s.order_index).unwrap_or(0),
                _paragraph_index
            );
            let node = HierarchyNode::new(para_text, HierarchyLevel::Paragraph, _paragraph_index, path);
            if let Some(ref mut sec) = current_section {
                sec.children.push(node);
            } else {
                root.children.push(node);
            }
        }
    }

    if let Some(section) = current_section {
        root.children.push(section);
    }

    root
}

/// Parse plain text document into hierarchy
fn parse_plain_text(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Document: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );
    
    // Split by double newlines (paragraphs)
    let paragraphs: Vec<&str> = content.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    
    for (idx, para) in paragraphs.iter().enumerate() {
        let path = format!("{}/paragraph[{}]", file_name, idx);
        
        // Check if paragraph is short enough to keep as single unit
        if para.len() < 500 {
            let node = HierarchyNode::new(
                para.to_string(),
                HierarchyLevel::Paragraph,
                idx,
                path,
            );
            root.children.push(node);
        } else {
            // Long paragraph - split by sentences
            let sentences = split_sentences(para);
            let mut para_node = HierarchyNode::new(
                String::new(),  // No content at paragraph level
                HierarchyLevel::Paragraph,
                idx,
                path.clone(),
            );
            
            for (sent_idx, sentence) in sentences.iter().enumerate() {
                let sent_path = format!("{}/sentence[{}]", path, sent_idx);
                let sent_node = HierarchyNode::new(
                    sentence.to_string(),
                    HierarchyLevel::Sentence,
                    sent_idx,
                    sent_path,
                );
                para_node.children.push(sent_node);
            }
            
            // Only add if has children
            if !para_node.children.is_empty() {
                root.children.push(para_node);
            }
        }
    }
    
    root
}

/// Split text into sentences
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        current.push(c);
        
        // Check for sentence endings
        if c == '.' || c == '!' || c == '?' {
            // Look ahead to see if this is really end of sentence
            let next = chars.peek();
            match next {
                Some(' ') | Some('\n') | Some('\t') => {
                    // Likely end of sentence - skip whitespace
                    while let Some(&' ') = chars.peek() {
                        current.push(chars.next().unwrap());
                    }
                    sentences.push(current.trim().to_string());
                    current = String::new();
                }
                Some('"') | Some('\'') | Some(')') | Some(']') => {
                    // Might be abbreviation or similar - just split
                    sentences.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {}
            }
        }
    }
    
    // Add remaining text
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    
    // Filter out very short "sentences" (likely abbreviations)
    sentences.into_iter()
        .filter(|s| s.len() > 10 || s.contains(' '))
        .collect()
}

/// Parse JSON document into hierarchy
fn parse_json_document(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Document: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );
    
    // Try to parse as JSON and extract structure
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        parse_json_value(&value, file_name, 0, &mut root);
    } else {
        // Not valid JSON - treat as plain text
        return parse_plain_text(content, file_name);
    }
    
    root
}

/// Recursively parse JSON value into hierarchy
fn parse_json_value(value: &serde_json::Value, parent_path: &str, order_index: usize, parent: &mut HierarchyNode) {
    match value {
        serde_json::Value::Object(obj) => {
            // Check if this looks like a message/item
            let is_message = obj.contains_key("content") || 
                            obj.contains_key("message") || 
                            obj.contains_key("role");
            
            if is_message {
                // Extract message content
                let content = obj.get("content")
                    .or_else(|| obj.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                let context: Vec<String> = obj.iter()
                    .filter(|(k, _)| *k != "content" && *k != "message")
                    .filter_map(|(k, v)| {
                        let v_str = match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        };
                        if !v_str.is_empty() && v_str.len() < 100 {
                            Some(format!("{}: {}", k, v_str))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                let content_with_context = if context.is_empty() {
                    content
                } else {
                    format!("{}\n\n[Context: {}]", content, context.join(", "))
                };
                
                let path = format!("{}/item[{}]", parent_path, order_index);
                let node = HierarchyNode::new(
                    content_with_context,
                    HierarchyLevel::Section,
                    order_index,
                    path,
                );
                parent.children.push(node);
            } else {
                // Regular object - add as metadata or recurse
                let mut metadata_pairs = Vec::new();
                
                for (key, val) in obj {
                    match val {
                        serde_json::Value::String(s) if s.len() < 200 => {
                            metadata_pairs.push(format!("{}: {}", key, s));
                        }
                        serde_json::Value::Number(n) => {
                            metadata_pairs.push(format!("{}: {}", key, n));
                        }
                        serde_json::Value::Bool(b) => {
                            metadata_pairs.push(format!("{}: {}", key, b));
                        }
                        serde_json::Value::Array(arr) if arr.len() < 100 => {
                            // Skip large arrays for now
                            let count = arr.len();
                            metadata_pairs.push(format!("{}: {} items", key, count));
                        }
                        _ => {}
                    }
                }
                
                if !metadata_pairs.is_empty() {
                    let path = format!("{}/metadata", parent_path);
                    let node = HierarchyNode::new(
                        metadata_pairs.join(", "),
                        HierarchyLevel::Paragraph,
                        order_index,
                        path,
                    );
                    parent.children.push(node);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                parse_json_value(item, parent_path, idx, parent);
            }
        }
        serde_json::Value::String(s)
            // Top-level string - add as paragraph
            if !s.is_empty() => {
                let node = HierarchyNode::new(
                    s.clone(),
                    HierarchyLevel::Paragraph,
                    order_index,
                    format!("{}/text[{}]", parent_path, order_index),
                );
                parent.children.push(node);
            }
        _ => {}
    }
}

/// Parse HTML document into hierarchy
fn parse_html(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Document: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );
    
    // Simple HTML parsing - extract text content
    let text = strip_html_tags(content);
    
    // Remove scripts and styles
    let text = remove_script_style(&text);
    
    // Treat as plain text
    let plain_root = parse_plain_text(&text, file_name);
    
    // Update root path
    root.children = plain_root.children;
    root
}

/// Strip HTML tags from content
fn strip_html_tags(content: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut _in_script = false;
    let mut _in_style = false;
    
    for chunk in content.split('<') {
        if chunk.starts_with("script") {
            _in_script = true;
        } else if chunk.starts_with("/script") {
            _in_script = false;
        } else if chunk.starts_with("style") {
            _in_style = true;
        } else if chunk.starts_with("/style") {
            _in_style = false;
        } else if in_tag {
            // Inside a tag - check for closing
            if let Some(pos) = chunk.find('>') {
                in_tag = false;
                result.push_str(&chunk[pos + 1..]);
            }
        } else {
            result.push_str(chunk);
        }
    }
    
    // Clean up whitespace
    result.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove script and style blocks
fn remove_script_style(content: &str) -> String {
    let mut result = String::new();
    let chars = content.chars().collect::<Vec<_>>();
    let mut i = 0;
    
    while i < chars.len() {
        let remaining = chars[i..].iter().collect::<String>();
        
        if remaining.starts_with("<script") || remaining.starts_with("<style") {
            // Skip until closing tag
            let closing = if remaining.starts_with("<script") { "</script>" } else { "</style>" };
            if let Some(pos) = remaining.to_lowercase().find(closing) {
                i += pos + closing.len();
                continue;
            }
        }
        
        result.push(chars[i]);
        i += 1;
    }
    
    result
}

/// Parse XML document into hierarchy
fn parse_xml(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Document: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );
    
    // Simple XML parsing - extract text content
    let text = strip_xml_tags(content);
    let plain_root = parse_plain_text(&text, file_name);
    root.children = plain_root.children;
    
    root
}

/// Strip XML tags
fn strip_xml_tags(content: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in content.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    
    result.trim().to_string()
}

/// Parse code document into hierarchy
fn parse_code(content: &str, file_name: &str) -> HierarchyNode {
    let mut root = HierarchyNode::new(
        format!("Code: {}", file_name),
        HierarchyLevel::Document,
        0,
        file_name.to_string(),
    );
    
    let lines: Vec<&str> = content.lines().collect();
    let mut current_function: Option<(String, usize, Vec<String>)> = None;
    let mut function_index = 0;
    
    let flush_function = |func: &mut Option<(String, usize, Vec<String>)>, root_node: &mut HierarchyNode, idx: &mut usize| {
        if let Some((name, start, lines)) = func.take() {
            let content = format!("Line {}: {}\n\n{}", start, name, lines.join("\n"));
            let path = format!("{}/function[{}]", file_name, idx);
            let node = HierarchyNode::new(content, HierarchyLevel::Section, *idx, path);
            root_node.children.push(node);
            *idx += 1;
        }
    };
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Detect function/class definitions
        let is_function = trimmed.starts_with("fn ")
            || trimmed.starts_with("func ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ");
        
        if is_function {
            flush_function(&mut current_function, &mut root, &mut function_index);
            
            let func_name = extract_function_name(trimmed);
            current_function = Some((func_name, line_num + 1, Vec::new()));
        }
        
        if let Some(ref mut func) = current_function {
            func.2.push(line.to_string());
        }
    }
    
    flush_function(&mut current_function, &mut root, &mut function_index);
    
    // If no functions found, treat as plain text
    if root.children.is_empty() {
        let plain_root = parse_plain_text(content, file_name);
        return plain_root;
    }
    
    root
}

/// Extract function/class name from definition line
fn extract_function_name(line: &str) -> String {
    // Try to extract name between common patterns
    let patterns = [
        ("fn ", "("),
        ("func ", "("),
        ("def ", "("),
        ("function ", "("),
        ("class ", " "),
        ("pub fn ", "("),
        ("async fn ", "("),
        ("pub async fn ", "("),
    ];
    
    for (prefix, end) in &patterns {
        if let Some(pos) = line.find(prefix) {
            let after_prefix = &line[pos + prefix.len()..];
            if let Some(end_pos) = after_prefix.find(end) {
                return after_prefix[..end_pos].trim().to_string();
            }
        }
    }
    
    line.to_string()
}

/// Determine file type from extension
pub fn get_file_type(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "md" | "markdown" => "markdown",
        "json" | "jsonl" => "json",
        "html" | "htm" => "html",
        "xml" => "xml",
        "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "go" | "rb" | "php" => "code",
        _ => "text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parsing() {
        let content = r##"# Introduction

This is the intro paragraph.

## Installation

Follow these steps:

1. Install
2. Configure

```bash
npm install
```

## Usage

Use it like this."##;

        let tree = parse_markdown(content, "readme.md");
        
        assert_eq!(tree.level, HierarchyLevel::Document);
        assert!(tree.children.len() >= 2, "Expected at least 2 sections, got {}", tree.children.len()); // At least 2 sections
    }

    #[test]
    fn test_sentence_splitting() {
        let text = "This is sentence one. This is sentence two! How about this?";
        let sentences = split_sentences(text);
        
        assert!(sentences.len() >= 3);
    }

    #[test]
    fn test_code_parsing() {
        let content = r#"fn main() {
    println!("Hello");
}

fn other() {
    do_something();
}"#;

        let tree = parse_code(content, "main.rs");
        
        assert_eq!(tree.level, HierarchyLevel::Document);
        assert!(tree.children.len() >= 2); // At least 2 functions
    }
}
