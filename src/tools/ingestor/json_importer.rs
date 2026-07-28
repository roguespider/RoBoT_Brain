// src/tools/ingestor/json_importer.rs
#![allow(dead_code)]

// Smart JSON importer that extracts structured data into memories

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

/// Configuration for JSON import
#[derive(Debug, Clone)]
pub struct JsonImportConfig {
    /// Include metadata fields (IDs, timestamps, etc.) as context
    pub include_metadata: bool,
    /// Maximum depth to recurse into nested objects
    pub max_depth: usize,
    /// Minimum text length to store (skip very short values)
    pub min_text_length: usize,
    /// Extract array items as individual memories
    pub explode_arrays: bool,
}

impl Default for JsonImportConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            max_depth: 10,
            min_text_length: 1,
            explode_arrays: true,
        }
    }
}

/// A single piece of extracted JSON data
#[derive(Debug, Clone)]
pub struct ExtractedJsonData {
    /// The actual text content
    pub content: String,
    /// JSON path in the original file (e.g., "messages[0].content")
    pub json_path: String,
    /// The field name (e.g., "content", "title")
    pub field_name: String,
    /// Context from sibling fields (e.g., "role: user, id: 123")
    pub sibling_context: String,
    /// The type of data (conversation, data, metadata, etc.)
    pub data_type: String,
    /// Raw JSON value for reference
    pub raw_value: Value,
}

impl ExtractedJsonData {
    /// Convert to memory content string
    pub fn to_memory_content(&self) -> String {
        let mut content = self.content.clone();
        
        if !self.sibling_context.is_empty() {
            content.push_str("\n\n[Context: ");
            content.push_str(&self.sibling_context);
            content.push_str("]");
        }
        
        content.push_str("\n\n[Source: ");
        content.push_str(&self.json_path);
        content.push_str("]");
        
        content
    }
}

/// Result of importing a JSON file
#[derive(Debug)]
pub struct JsonImportResult {
    /// All extracted data pieces
    pub items: Vec<ExtractedJsonData>,
    /// Detected type of JSON file
    pub json_type: JsonFileType,
    /// Summary statistics
    pub stats: ImportStats,
    /// Any warnings
    pub warnings: Vec<String>,
}

/// Statistics about the import
#[derive(Debug, Default)]
pub struct ImportStats {
    pub total_items: usize,
    pub text_items: usize,
    pub metadata_items: usize,
    pub nested_items: usize,
    pub skipped_items: usize,
}

/// Types of JSON files we can detect
#[derive(Debug, PartialEq)]
pub enum JsonFileType {
    /// Array of chat messages
    Conversation,
    /// Array of similar objects (products, users, etc.)
    DataArray,
    /// Single object with mixed fields
    MixedObject,
    /// Key-value pairs / configuration
    Config,
    /// Chroma/LangChain export
    EmbeddingsExport,
    /// Unknown structure
    Unknown,
}

/// Import a JSON file and extract structured data
pub fn import_json_file(path: &Path, config: Option<JsonImportConfig>) -> Result<JsonImportResult> {
    let config = config.unwrap_or_default();
    
    // Read and parse JSON
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read JSON file: {}", path.display()))?;
    
    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON file: {}", path.display()))?;
    
    // Detect JSON type
    let json_type = detect_json_type(&value);
    
    // Extract data based on type
    let mut items = Vec::new();
    let mut warnings = Vec::new();
    
    match json_type {
        JsonFileType::Conversation => {
            extract_conversation(&value, "", &config, &mut items, &mut warnings)?;
        }
        JsonFileType::DataArray => {
            extract_data_array(&value, "", &config, &mut items, &mut warnings)?;
        }
        JsonFileType::EmbeddingsExport => {
            extract_embeddings_export(&value, "", &config, &mut items, &mut warnings)?;
        }
        _ => {
            extract_generic_json(&value, "", &config, &mut items, &mut warnings, 0)?;
        }
    }
    
    // Calculate stats
    let stats = ImportStats {
        total_items: items.len(),
        text_items: items.iter().filter(|i| i.data_type == "text").count(),
        metadata_items: items.iter().filter(|i| i.data_type == "metadata").count(),
        nested_items: items.iter().filter(|i| i.data_type == "nested").count(),
        skipped_items: 0,
    };
    
    Ok(JsonImportResult {
        items,
        json_type,
        stats,
        warnings,
    })
}

/// Detect the type of JSON file
fn detect_json_type(value: &Value) -> JsonFileType {
    match value {
        // Check for conversation/chat format
        Value::Object(obj) => {
            // Check for Chroma/LangChain export format
            if obj.contains_key("documents") || obj.contains_key("embeddings") {
                return JsonFileType::EmbeddingsExport;
            }
            
            // Check for conversation format
            if obj.contains_key("messages") || obj.contains_key("conversation") || obj.contains_key("chat") {
                return JsonFileType::Conversation;
            }
            
            // Check for array of objects (data)
            if let Some(arr) = obj.values().find_map(|v| v.as_array()) {
                if !arr.is_empty() && arr.iter().all(|v| v.is_object()) {
                    return JsonFileType::DataArray;
                }
            }
            
            // Mixed object
            JsonFileType::MixedObject
        }
        // Check for array of messages
        Value::Array(arr) => {
            if !arr.is_empty() {
                // Check if it's an array of messages (objects with role/content)
                if arr.iter().all(|v| {
                    v.is_object() && (
                        v.get("role").is_some() || 
                        v.get("content").is_some() ||
                        v.get("message").is_some()
                    )
                }) {
                    return JsonFileType::Conversation;
                }
                
                // Check if it's an array of similar objects
                if arr.iter().all(|v| v.is_object()) {
                    return JsonFileType::DataArray;
                }
            }
            JsonFileType::Unknown
        }
        _ => JsonFileType::Unknown,
    }
}

/// Extract conversation/chat format
fn extract_conversation(
    value: &Value,
    path: &str,
    config: &JsonImportConfig,
    items: &mut Vec<ExtractedJsonData>,
    _warnings: &mut Vec<String>,
) -> Result<()> {
    // Handle array of messages at root
    if let Value::Array(arr) = value {
        for (idx, item) in arr.iter().enumerate() {
            let item_path = format!("[{}]", idx);
            extract_message_item(item, &item_path, config, items);
        }
        return Ok(());
    }
    
    // Handle object with messages/conversation field
    let obj = match value {
        Value::Object(o) => o,
        _ => return Ok(()),
    };
    
    // Extract metadata (id, title, etc.)
    let metadata_fields = ["id", "_id", "title", "name", "created_at", "updated_at", "conversation_id"];
    let mut metadata_pairs = Vec::new();
    
    for key in metadata_fields {
        if let Some(val) = obj.get(key) {
            if let Some(s) = val.as_str() {
                metadata_pairs.push(format!("{}: {}", key, s));
            } else if let Some(n) = val.as_i64() {
                metadata_pairs.push(format!("{}: {}", key, n));
            } else if let Some(n) = val.as_f64() {
                metadata_pairs.push(format!("{}: {}", key, n));
            }
        }
    }
    
    // Store metadata as single item if we have any
    if !metadata_pairs.is_empty() && config.include_metadata {
        items.push(ExtractedJsonData {
            content: metadata_pairs.join(", "),
            json_path: path.to_string(),
            field_name: "_metadata".to_string(),
            sibling_context: String::new(),
            data_type: "metadata".to_string(),
            raw_value: Value::Null,
        });
    }
    
    // Find messages/conversation arrays
    let message_keys = ["messages", "conversation", "chat", "entries", "history"];
    
    for key in message_keys {
        if let Some(messages) = obj.get(key) {
            if let Value::Array(arr) = messages {
                for (idx, msg) in arr.iter().enumerate() {
                    let msg_path = if path.is_empty() {
                        format!("{}[{}]", key, idx)
                    } else {
                        format!("{}.{}[{}]", path, key, idx)
                    };
                    extract_message_item(msg, &msg_path, config, items);
                }
            }
        }
    }
    
    Ok(())
}

/// Extract a single message item
fn extract_message_item(item: &Value, path: &str, config: &JsonImportConfig, items: &mut Vec<ExtractedJsonData>) {
    let obj = match item {
        Value::Object(o) => o,
        _ => return,
    };
    
    // Collect sibling context (role, timestamp, etc.)
    let mut sibling_context = Vec::new();
    let context_fields = ["role", "speaker", "author", "timestamp", "date", "time", "sender", "id", "_id"];
    
    for key in context_fields {
        if let Some(val) = obj.get(key) {
            if let Some(s) = val.as_str() {
                sibling_context.push(format!("{}: {}", key, s));
            }
        }
    }
    
    let sibling_context_str = sibling_context.join(", ");
    
    // Extract content
    let content_fields = ["content", "text", "message", "body", "description", "value"];
    
    for field in content_fields {
        if let Some(content_val) = obj.get(field) {
            if let Some(text) = content_val.as_str() {
                if text.len() >= config.min_text_length {
                    items.push(ExtractedJsonData {
                        content: text.to_string(),
                        json_path: format!("{}.{}", path, field),
                        field_name: field.to_string(),
                        sibling_context: sibling_context_str.clone(),
                        data_type: "conversation.message".to_string(),
                        raw_value: content_val.clone(),
                    });
                    return; // Found content, done
                }
            }
        }
    }
    
    // If no content field found, extract all string fields
    for (key, val) in obj {
        if let Some(text) = val.as_str() {
            if text.len() >= config.min_text_length {
                items.push(ExtractedJsonData {
                    content: text.to_string(),
                    json_path: format!("{}.{}", path, key),
                    field_name: key.clone(),
                    sibling_context: sibling_context_str.clone(),
                    data_type: "conversation.other".to_string(),
                    raw_value: val.clone(),
                });
            }
        }
    }
}

/// Extract data array format (array of objects)
fn extract_data_array(
    value: &Value,
    path: &str,
    config: &JsonImportConfig,
    items: &mut Vec<ExtractedJsonData>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let arr = match value {
        Value::Array(a) => a,
        Value::Object(obj) => {
            // Find the array in the object
            if let Some(arr) = obj.values().find_map(|v| v.as_array()) {
                arr
            } else {
                // Treat entire object as a single item
                extract_generic_json(value, path, config, items, warnings, 0)?;
                return Ok(());
            }
        }
        _ => return Ok(()),
    };
    
    // Group items by structure to determine extraction strategy
    if arr.is_empty() {
        return Ok(());
    }
    
    // Check if all items have similar structure
    let _first_keys: std::collections::HashSet<_> = if let Some(Value::Object(o)) = arr.first() {
        o.keys().collect()
    } else {
        std::collections::HashSet::new()
    };
    
    // Determine if it's a key-value list or record list
    let is_key_value = arr.iter().all(|item| {
        if let Value::Object(o) = item {
            o.len() == 2 && o.contains_key("key") && o.contains_key("value")
        } else {
            false
        }
    });
    
    if is_key_value {
        // Extract as key-value pairs
        for (idx, item) in arr.iter().enumerate() {
            if let Value::Object(o) = item {
                let key = o.get("key").and_then(|v| v.as_str()).unwrap_or("unknown");
                let val = o.get("value")
                    .map(|v| {
                        if let Some(s) = v.as_str() {
                            s.to_string()
                        } else {
                            serde_json::to_string(v).unwrap_or_default()
                        }
                    })
                    .unwrap_or_default();
                
                items.push(ExtractedJsonData {
                    content: format!("{}: {}", key, val),
                    json_path: format!("{}[{}]", path, idx),
                    field_name: key.to_string(),
                    sibling_context: String::new(),
                    data_type: "config".to_string(),
                    raw_value: item.clone(),
                });
            }
        }
    } else {
        // Extract each object as a record
        for (idx, item) in arr.iter().enumerate() {
            let item_path = format!("{}[{}]", path, idx);
            extract_object_as_record(item, &item_path, config, items)?;
        }
    }
    
    Ok(())
}

/// Extract an object as a single record (for data arrays)
fn extract_object_as_record(
    value: &Value,
    path: &str,
    config: &JsonImportConfig,
    items: &mut Vec<ExtractedJsonData>,
) -> Result<()> {
    let obj = match value {
        Value::Object(o) => o,
        _ => return Ok(()),
    };
    
    // Identify the "main" content field
    let content_fields = ["name", "title", "description", "content", "text", "summary", "body", "message"];
    let mut main_content = None;
    let mut main_field = None;
    
    for field in content_fields {
        if let Some(val) = obj.get(field) {
            if let Some(s) = val.as_str() {
                if s.len() >= config.min_text_length {
                    main_content = Some(s.to_string());
                    main_field = Some(field.to_string());
                    break;
                }
            }
        }
    }
    
    // Collect sibling context (everything except main content)
    let sibling_context = obj.iter()
        .filter(|(k, _v)| Some(&k[..]) != main_field.as_deref())
        .filter_map(|(k, v)| {
            let val_str = if let Some(s) = v.as_str() {
                s.to_string()
            } else if v.is_number() {
                v.to_string()
            } else if v.is_boolean() {
                v.to_string()
            } else {
                return None;
            };
            Some(format!("{}: {}", k, val_str))
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    if let Some((content, field)) = main_content.zip(main_field) {
        items.push(ExtractedJsonData {
            content,
            json_path: path.to_string(),
            field_name: field,
            sibling_context,
            data_type: "data.record".to_string(),
            raw_value: value.clone(),
        });
    } else {
        // No main content field - extract all string fields
        let mut all_text = Vec::new();
        for (key, val) in obj {
            if let Some(text) = val.as_str() {
                if text.len() >= config.min_text_length {
                    all_text.push(format!("{}: {}", key, text));
                }
            }
        }
        
        if !all_text.is_empty() {
            items.push(ExtractedJsonData {
                content: all_text.join("\n"),
                json_path: path.to_string(),
                field_name: "_record".to_string(),
                sibling_context: String::new(),
                data_type: "data.record".to_string(),
                raw_value: value.clone(),
            });
        }
    }
    
    Ok(())
}

/// Extract embeddings export (Chroma, LangChain, etc.)
fn extract_embeddings_export(
    value: &Value,
    path: &str,
    config: &JsonImportConfig,
    items: &mut Vec<ExtractedJsonData>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let obj = match value {
        Value::Object(o) => o,
        _ => return Ok(()),
    };
    
    // Extract documents from various formats
    let doc_keys = ["documents", "texts", "content", "chunks", "passages"];
    
    for key in doc_keys {
        if let Some(docs) = obj.get(key) {
            let (docs_array, ids_array, metadatas_array) = if let Value::Array(arr) = docs {
                // Documents might be nested in arrays (Chroma format: [[doc1, doc2]])
                if let Some(Value::Array(inner)) = arr.first() {
                    let ids = obj.get("ids").and_then(|v| v.as_array()).map(|a| a.clone());
                    let metadatas = obj.get("metadatas").and_then(|v| v.as_array()).map(|a| a.clone());
                    (inner.clone(), ids, metadatas)
                } else {
                    // Direct array of documents
                    let ids = obj.get("ids").and_then(|v| v.as_array()).map(|a| a.clone());
                    let metadatas = obj.get("metadatas").and_then(|v| v.as_array()).map(|a| a.clone());
                    (arr.clone(), ids, metadatas)
                }
            } else {
                continue;
            };
            
            warnings.push("Extracting documents from embeddings export. Embeddings themselves are not stored.".to_string());
            
            for (idx, doc) in docs_array.iter().enumerate() {
                let content = doc.as_str().unwrap_or("");
                
                if content.len() < config.min_text_length {
                    continue;
                }
                
                // Get ID if available
                let id_str = ids_array
                    .as_ref()
                    .and_then(|a| a.get(idx))
                    .and_then(|v| v.as_str())
                    .map(|s| format!("id: {}", s))
                    .unwrap_or_default();
                
                // Get metadata if available
                let metadata_str = metadatas_array
                    .as_ref()
                    .and_then(|a| a.get(idx))
                    .and_then(|v| {
                        if let Value::Object(o) = v {
                            let pairs: Vec<_> = o.iter()
                                .filter_map(|(k, val)| {
                                    let s = if let Some(str_val) = val.as_str() {
                                        str_val.to_string()
                                    } else {
                                        val.to_string()
                                    };
                                    Some(format!("{}: {}", k, s))
                                })
                                .collect();
                            Some(pairs.join(", "))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                
                let sibling_context = [id_str, metadata_str].iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                
                items.push(ExtractedJsonData {
                    content: content.to_string(),
                    json_path: format!("{}.{}[{}]", path, key, idx),
                    field_name: "document".to_string(),
                    sibling_context,
                    data_type: "embeddings.document".to_string(),
                    raw_value: doc.clone(),
                });
            }
        }
    }
    
    Ok(())
}

/// Extract generic JSON (fallback for unknown structures)
/// IMPORTANT: This extracts ALL content types including numbers, booleans, and null
/// to ensure no data is lost from simple JSON files.
fn extract_generic_json(
    value: &Value,
    path: &str,
    config: &JsonImportConfig,
    items: &mut Vec<ExtractedJsonData>,
    warnings: &mut Vec<String>,
    depth: usize,
) -> Result<()> {
    if depth > config.max_depth {
        warnings.push(format!("Max depth {} exceeded at {}", config.max_depth, path));
        return Ok(());
    }
    
    match value {
        Value::Null => {
            // Store null as a marker item - helps preserve structure awareness
            if depth > 0 { // Don't add at root level
                items.push(ExtractedJsonData {
                    content: "null".to_string(),
                    json_path: path.to_string(),
                    field_name: "_null".to_string(),
                    sibling_context: String::new(),
                    data_type: "null".to_string(),
                    raw_value: Value::Null,
                });
            }
        }
        Value::Bool(b) => {
            // Store booleans - they may represent important state
            items.push(ExtractedJsonData {
                content: b.to_string(),
                json_path: path.to_string(),
                field_name: "_boolean".to_string(),
                sibling_context: String::new(),
                data_type: "boolean".to_string(),
                raw_value: Value::Bool(*b),
            });
        }
        Value::Number(n) => {
            // Store numbers - they may be important values
            items.push(ExtractedJsonData {
                content: n.to_string(),
                json_path: path.to_string(),
                field_name: "_number".to_string(),
                sibling_context: String::new(),
                data_type: "number".to_string(),
                raw_value: value.clone(),
            });
        }
        Value::String(s) => {
            if s.len() >= config.min_text_length {
                items.push(ExtractedJsonData {
                    content: s.clone(),
                    json_path: path.to_string(),
                    field_name: "_value".to_string(),
                    sibling_context: String::new(),
                    data_type: "text".to_string(),
                    raw_value: value.clone(),
                });
            }
        }
        Value::Array(arr) => {
            if config.explode_arrays && !arr.is_empty() {
                // Check if array contains objects (process each)
                if arr.iter().all(|v| v.is_object()) {
                    for (idx, item) in arr.iter().enumerate() {
                        let item_path = format!("{}[{}]", path, idx);
                        extract_object_as_record(item, &item_path, config, items)?;
                    }
                } else {
                    // Array of primitives - join them
                    let joined = arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    if !joined.is_empty() {
                        items.push(ExtractedJsonData {
                            content: joined,
                            json_path: path.to_string(),
                            field_name: "_array".to_string(),
                            sibling_context: format!("{} items", arr.len()),
                            data_type: "text".to_string(),
                            raw_value: value.clone(),
                        });
                    }
                }
            }
        }
        Value::Object(obj) => {
            for (key, val) in obj {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                
                // Recursively extract
                extract_generic_json(val, &field_path, config, items, warnings, depth + 1)?;
            }
        }
    }
    
    Ok(())
}

/// Format import result for display
pub fn format_import_summary(result: &JsonImportResult) -> String {
    let mut summary = String::new();
    
    summary.push_str(&format!("JSON Type: {:?}\n", result.json_type));
    summary.push_str(&format!("Total items extracted: {}\n", result.stats.total_items));
    
    if result.stats.text_items > 0 {
        summary.push_str(&format!("  - Text items: {}\n", result.stats.text_items));
    }
    if result.stats.metadata_items > 0 {
        summary.push_str(&format!("  - Metadata items: {}\n", result.stats.metadata_items));
    }
    if result.stats.nested_items > 0 {
        summary.push_str(&format!("  - Nested items: {}\n", result.stats.nested_items));
    }
    
    if !result.warnings.is_empty() {
        summary.push_str("\nWarnings:\n");
        for warning in &result.warnings {
            summary.push_str(&format!("  - {}\n", warning));
        }
    }
    
    summary
}
