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
        // Images are handled separately - this should not be called for images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "ico" | "tiff" | "svg" | "svgz" => {
            anyhow::bail!("Images should use extract_image_metadata(), not extract_text()")
        }
        _ => {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        }
    }
}

/// Extract metadata from an image file
/// Returns a structured description of the image for memory storage
pub fn extract_image_metadata(path: &Path) -> Result<ImageMetadata> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();
    
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // For SVG, we can try to extract some information
    let svg_info = if extension == "svg" || extension == "svgz" {
        extract_svg_info(path)?
    } else {
        None
    };
    
    Ok(ImageMetadata {
        filename,
        path: path.to_string_lossy().to_string(),
        format: extension,
        file_size_bytes: file_size,
        file_size_human: format_file_size(file_size),
        has_svg_content: svg_info.is_some(),
        svg_content_preview: svg_info,
        note: "This is an image file. Only metadata is stored, not the image content.".to_string(),
    })
}

/// Extract information from SVG files
fn extract_svg_info(path: &Path) -> Result<Option<String>> {
    let content = std::fs::read_to_string(path)?;
    
    // Check if SVG is valid XML
    if !content.trim().starts_with("<?xml") && !content.trim().starts_with("<svg") {
        return Ok(None);
    }
    
    // Try to extract dimensions
    let mut info = String::new();
    
    // Look for width/height
    if let Some(width) = extract_svg_attr(&content, "width") {
        info.push_str(&format!("Width: {}\n", width));
    }
    if let Some(height) = extract_svg_attr(&content, "height") {
        info.push_str(&format!("Height: {}\n", height));
    }
    if let Some(viewbox) = extract_svg_attr(&content, "viewBox") {
        info.push_str(&format!("ViewBox: {}\n", viewbox));
    }
    
    // Count elements
    let rect_count = content.matches("<rect").count();
    let circle_count = content.matches("<circle").count();
    let path_count = content.matches("<path").count();
    let text_count = content.matches("<text").count();
    
    if rect_count > 0 || circle_count > 0 || path_count > 0 || text_count > 0 {
        info.push_str(&format!("Contains: {} shapes, {} text elements\n", path_count + rect_count + circle_count, text_count));
    }
    
    if info.is_empty() {
        Ok(None)
    } else {
        Ok(Some(info))
    }
}

/// Extract an attribute value from SVG content
fn extract_svg_attr(content: &str, attr: &str) -> Option<String> {
    // Try quoted attribute
    let pattern = format!("{}=\"", attr);
    if let Some(pos) = content.find(&pattern) {
        let start = pos + pattern.len();
        if let Some(end) = content[start..].find('"') {
            return Some(content[start..start + end].to_string());
        }
    }
    
    // Try single quoted attribute
    let pattern = format!("{}='", attr);
    if let Some(pos) = content.find(&pattern) {
        let start = pos + pattern.len();
        if let Some(end) = content[start..].find('\'') {
            return Some(content[start..start + end].to_string());
        }
    }
    
    None
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Image metadata structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageMetadata {
    pub filename: String,
    pub path: String,
    pub format: String,
    pub file_size_bytes: u64,
    pub file_size_human: String,
    pub has_svg_content: bool,
    pub svg_content_preview: Option<String>,
    pub note: String,
}

impl ImageMetadata {
    /// Convert to memory-friendly text
    pub fn to_memory_content(&self) -> String {
        let mut content = format!(
            "IMAGE FILE\n\
             Filename: {}\n\
             Format: {}\n\
             Size: {}\n",
            self.filename, self.format, self.file_size_human
        );
        
        if let Some(ref svg_info) = self.svg_content_preview {
            content.push_str(&format!("\nSVG Details:\n{}", svg_info));
        }
        
        content.push_str(&format!("\nNote: {}\n", self.note));
        
        content
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
    
    // Check for embedded images (diagrams)
    let has_images = detect_pdf_images(&bytes);
    
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
    
    // Add note if PDF has embedded images
    if has_images && !result.trim().is_empty() {
        result.push_str("\n\n---\n");
        result.push_str("NOTE: This PDF contains embedded images/diagrams.\n");
        result.push_str("The images are not extracted as text, but you can see them in the original PDF.\n");
        result.push_str("If you need information from diagrams, please view the original PDF file.\n");
    }
    
    if result.trim().is_empty() {
        // PDF has no extractable text but might have images
        if has_images {
            anyhow::bail!("This PDF contains only images/diagrams with no extractable text. Please view the original PDF file for diagram content.");
        } else {
            anyhow::bail!("Could not extract text from PDF - file may be scanned/image-based");
        }
    }
    
    Ok(result)
}

/// Detect if PDF contains embedded images
fn detect_pdf_images(bytes: &[u8]) -> bool {
    // Look for image XObject markers in PDF
    // PDF images are typically stored as /Subtype /Image or /XObject /Image
    let content = String::from_utf8_lossy(bytes);
    
    // Common PDF image patterns
    let patterns = [
        "/Subtype /Image",
        "/XObject << /Image",
        "stream\nI I\n",  // FlateDecode image stream
        "/DCTDecode",     // JPEG images
        "/CCITTFaxDecode", // Fax images
        "/JBIG2Decode",  // JBIG2 images
    ];
    
    for pattern in &patterns {
        if content.contains(pattern) {
            return true;
        }
    }
    
    // Check for inline images
    if content.contains("BI") && content.contains("ID") && content.contains("EI") {
        return true;
    }
    
    false
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

/// Check if content looks like readable text (not binary garbage)
/// Returns (is_valid, reason) where is_valid is true if content is readable
pub fn validate_text_quality(content: &str) -> (bool, String) {
    // Empty content is not valid
    if content.trim().is_empty() {
        return (false, "Content is empty or whitespace only".to_string());
    }
    
    let bytes = content.as_bytes();
    let len = bytes.len();
    
    // Count printable/valid text characters
    let mut printable_count = 0;
    let mut null_count = 0;
    let mut control_count = 0;
    
    for &byte in bytes {
        if byte == 0 {
            null_count += 1;
        } else if byte < 32 && byte != 9 && byte != 10 && byte != 13 {
            // Allow tab, newline, carriage return
            control_count += 1;
        } else if byte >= 32 && byte < 127 || byte >= 128 {
            printable_count += 1;
        }
    }
    
    let printable_ratio = printable_count as f64 / len as f64;
    
    // Check for null bytes (strong indicator of binary)
    if null_count > 0 {
        return (false, format!("Contains {} null bytes (binary content)", null_count));
    }
    
    // Check printable ratio - if less than 70% printable, likely binary
    if printable_ratio < 0.7 {
        return (false, format!("Only {:.0}% printable characters (likely binary)", printable_ratio * 100.0));
    }
    
    // Check for too many control characters
    if control_count > len / 10 {
        return (false, format!("Too many control characters ({})", control_count));
    }
    
    // Check for replacement characters (UTF-8 decoding failures)
    let replacement_count = content.chars().filter(|c| *c == '\u{FFFD}').count();
    if replacement_count > len / 100 {
        return (false, format!("Contains {} replacement characters (encoding errors)", replacement_count));
    }
    
    // Content looks valid
    (true, "Valid text content".to_string())
}
