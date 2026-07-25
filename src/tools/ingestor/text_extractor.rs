// src/tools/ingestor/text_extractor.rs
// Text extraction from various file formats

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::Result;

/// Extract text from a file based on its extension
pub fn extract_text(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        "pdf" => extract_pdf_text(path),
        "docx" => extract_docx_text(path),
        "epub" => extract_epub_text(path),
        "json" | "jsonl" => {
            // Check if it's a chroma export
            let content = std::fs::read_to_string(path)?;
            if is_chroma_export(&content) {
                extract_chroma_text(&content)
            } else {
                Ok(content)
            }
        }
        _ => {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
    }
}

/// Check if JSON content looks like a chroma export
fn is_chroma_export(content: &str) -> bool {
    // Chroma exports typically have these fields
    let lower = content.to_lowercase();
    lower.contains("\"documents\"") && 
    (lower.contains("\"embeddings\"") || lower.contains("\"metadatas\""))
}

/// Extract text from chroma database export
/// Combines documents with their metadata for proper context
fn extract_chroma_text(content: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(content)?;
    
    let documents = json.get("documents")
        .and_then(|d| d.get(0))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    let metadatas = json.get("metadatas")
        .and_then(|m| m.get(0))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::to_string_pretty(v).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    let ids = json.get("ids")
        .and_then(|i| i.get(0))
        .and_then(|i| i.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        let parts: Vec<_> = arr.iter().filter_map(|x| x.as_str().map(String::from)).collect();
                        Some(parts.join(", "))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    if documents.is_empty() {
        anyhow::bail!("No documents found in chroma export");
    }
    
    let mut result = String::new();
    result.push_str("# Chroma Database Export\n\n");
    
    for (i, doc) in documents.iter().enumerate() {
        result.push_str(&format!("---\n"));
        
        // Add ID if available
        if let Some(id) = ids.get(i) {
            result.push_str(&format!("ID: {}\n", id));
        }
        
        // Add metadata if available
        if let Some(meta) = metadatas.get(i) {
            result.push_str(&format!("Metadata: {}\n", meta));
        }
        
        // Add document content
        result.push_str(&format!("Content:\n{}\n\n", doc));
    }
    
    Ok(result)
}

/// Extract text from PDF
pub fn extract_pdf_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    
    let text = String::from_utf8_lossy(&bytes);
    let mut result = String::new();
    
    for line in text.lines() {
        if line.contains("BT") || line.contains(" Tj") || line.contains(" TJ") {
            let cleaned = line
                .replace("BT", "")
                .replace("ET", "")
                .replace("(", "")
                .replace(")", "")
                .replace("\\n", "\n")
                .replace("\\t", "\t");
            if !cleaned.trim().is_empty() {
                result.push_str(&cleaned);
                result.push('\n');
            }
        }
    }
    
    if result.trim().is_empty() {
        anyhow::bail!("Could not extract text from PDF - file may be scanned/image-based");
    }
    
    Ok(result)
}

/// Extract text from DOCX
pub fn extract_docx_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut content = String::new();
    
    if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
        let mut xml = String::new();
        doc_file.read_to_string(&mut xml)?;
        content = strip_xml_tags(&xml);
    }
    
    if content.trim().is_empty() {
        anyhow::bail!("Could not extract text from DOCX");
    }
    
    Ok(content)
}

/// Extract text from EPUB
pub fn extract_epub_text(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = String::new();
    
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm") || name == "content.opf" {
                let mut html = String::new();
                let mut f = file;
                f.read_to_string(&mut html)?;
                content.push_str(&strip_html_tags(&html));
                content.push_str("\n\n");
            }
        }
    }
    
    if content.trim().is_empty() {
        anyhow::bail!("Could not extract text from EPUB");
    }
    
    Ok(content)
}

/// Strip XML/HTML tags and extract text content
pub fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::new();
    let mut in_content = true;
    
    let mut chars = xml.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            in_content = false;
        } else if c == '>' {
            in_content = true;
        } else if in_content {
            result.push(c);
        }
    }
    
    // Clean up whitespace
    let lines: Vec<&str> = result.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    
    lines.join("\n")
}

/// Strip HTML tags
pub fn strip_html_tags(html: &str) -> String {
    strip_xml_tags(html) // Same logic
}

/// Chunk text into smaller pieces with overlap
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }
    
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;
    
    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        
        if end >= chars.len() {
            break;
        }
        
        start = end - overlap;
    }
    
    chunks
}
