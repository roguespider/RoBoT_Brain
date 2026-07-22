// src/tools/ingestor/workflow.rs
// Workflow operations: list/delete imported files

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::tools::ToolOutput;
use crate::tools::ingestor::file_collector::{collect_importable_files, get_import_folder};

use super::ListImportableInput;
use super::ListIngestedFilesInput;
use super::DeleteIngestedFilesInput;

// ============================================================================
// LIST IMPORTABLE FILES
// ============================================================================

pub async fn execute_list_importable(
    input: ListImportableInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(5);
    
    if !folder.exists() {
        return Ok(ToolOutput::success(serde_json::json!({
            "files": [],
            "folder": folder.to_string_lossy(),
            "message": "Folder does not exist"
        })));
    }
    
    let files = collect_importable_files(&folder)?;
    let files: Vec<_> = files.into_iter().take(limit).collect();
    
    let total = collect_importable_files(&folder)?.len();
    
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "folder": folder.to_string_lossy(),
        "count": files.len(),
        "total": total,
        "message": if files.is_empty() {
            "No importable files found".to_string()
        } else {
            format!("Found {} files (showing {})", total, files.len())
        }
    })))
}

// ============================================================================
// LIST/DELETE INGESTED FILES
// ============================================================================

pub async fn execute_list_ingested_files(
    input: ListIngestedFilesInput,
) -> Result<ToolOutput> {
    let folder = get_import_folder(input.folder.as_deref());
    let limit = input.limit.unwrap_or(50);
    
    let files = collect_importable_files(&folder)?;
    let files: Vec<_> = files.into_iter().take(limit).collect();
    
    Ok(ToolOutput::success(serde_json::json!({
        "files": files,
        "count": files.len(),
        "warning": "These files have been ingested into memory. Delete originals if no longer needed."
    })))
}

pub async fn execute_delete_ingested_files(
    input: DeleteIngestedFilesInput,
) -> Result<ToolOutput> {
    // Verify confirmation
    if input.confirmation.to_lowercase() != "yes" && input.confirmation.to_lowercase() != "confirm" {
        return Ok(ToolOutput::error(
            "Deletion cancelled. Must confirm with 'yes' or 'confirm'.".to_string()
        ));
    }
    
    let mut deleted = Vec::new();
    let mut failed = Vec::new();
    
    for file_path in &input.files {
        let path = Path::new(file_path);
        
        if !path.exists() {
            failed.push(serde_json::json!({
                "path": file_path,
                "error": "File not found"
            }));
            continue;
        }
        
        match fs::remove_file(path) {
            Ok(()) => {
                tracing::info!("Deleted file: {:?}", path);
                deleted.push(file_path.clone());
            }
            Err(e) => {
                tracing::warn!("Failed to delete {:?}: {}", path, e);
                failed.push(serde_json::json!({
                    "path": file_path,
                    "error": e.to_string()
                }));
            }
        }
    }
    
    Ok(ToolOutput::success(serde_json::json!({
        "deleted": deleted,
        "deleted_count": deleted.len(),
        "failed": failed,
        "failed_count": failed.len(),
        "message": if deleted.is_empty() {
            "No files were deleted".to_string()
        } else {
            format!("Successfully deleted {} file(s)", deleted.len())
        }
    })))
}
