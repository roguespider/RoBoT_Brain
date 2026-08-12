//! Types Module
//!
//! Defines the core data structures for test requirements.

use serde::{Deserialize, Serialize};

/// Represents a test requirement for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Unique identifier for this test
    pub id: String,
    /// Name of the function/tool
    pub function_name: String,
    /// Category this function belongs to
    pub category: String,
    /// Whether this function requires the workflow to be initialized first
    pub requires_workflow: bool,
    /// Whether this function requires specific data to exist first
    pub requires_data: Option<DataRequirement>,
    /// Expected behavior description
    pub expected_behavior: String,
    /// Test validation checks (what to verify in the result)
    pub validation: Vec<ValidationCheck>,
    /// Priority level (1 = critical, 2 = important, 3 = nice to have)
    pub priority: u8,
}

/// Data that needs to be created before testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRequirement {
    pub data_type: String,
    pub creation_tool: String,
    pub min_count: usize,
}

/// Validation checks to perform on the result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub check_type: CheckType,
    pub field: String,
    pub expected_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckType {
    HasField,
    IsNonEmpty,
    IsSuccess,
    MatchesPattern,
    GreaterThan,
    LessThan,
}
