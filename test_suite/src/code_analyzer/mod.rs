//! Code Analyzer Module
//!
//! Analyzes source code to detect:
//! - #[allow(*)] annotations
//! - unimplemented!() macros
//! - todo!() macros
//! - panic!() with stub-like messages
//! - Partial/stub function implementations
//! - Functions that return early without doing work
//! - Underscore-prefixed identifiers (_unused, _ignored, etc.)

pub mod analyzer;
pub mod lint;
pub mod patterns;
pub mod types;

// Re-export for convenience
pub use analyzer::CodeAnalyzer;
pub use lint::{LintAnalyzer, LintSummary};
pub use types::{CodeIssue, LintIssue, LintLevel};
