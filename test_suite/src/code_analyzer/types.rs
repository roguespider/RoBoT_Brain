//! Type definitions for code analysis

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Represents a code issue found during analysis
#[derive(Debug, Clone, Serialize)]
pub struct CodeIssue {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub issue_type: IssueType,
    pub description: String,
}

impl CodeIssue {
    /// Get the relative path from the source directory (e.g., "tools/memory/mod.rs")
    pub fn relative_path(&self, base_path: &Path) -> String {
        if let Ok(stripped) = self.file_path.strip_prefix(base_path) {
            stripped.to_string_lossy().to_string()
        } else {
            self.file_path.to_string_lossy().to_string()
        }
    }
}

/// Types of issues that can be detected
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IssueType {
    AllowAnnotation,
    DeadCodeAllow,
    Unimplemented,
    Todo,
    Panic,
    EarlyReturnStub,
    UnderscorePrefix,
    UnusedImport,
    PublicNeverCalled,
    AlwaysErr,
    PlaceholderReturn,
    /// `#[cfg(test)]` in robot_brain `src/` — tests must live in
    /// `test_suite/`, not in the server's source (AGENTS.md "All tests
    /// live in test_suite (MANDATORY)"). The gate compiles robot_brain in
    /// release so these blocks are invisible to the compiler; this check
    /// surfaces them explicitly.
    CfgTest,
    /// Decorative emoji in code (AGENTS.md "No emoji / plain-text
    /// markers"). Arrows (`->` `|` `v` unicode) are permitted for flow
    /// diagrams; only decorative emoji (check/cross marks, party popper,
    /// clipboard, warning, etc.) are banned. Plain-text markers replace
    /// them: `[OK]` `[FAIL]` `[WARN]` `[INFO]` `[BLOCKED]`.
    Emoji,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::AllowAnnotation => write!(f, "#[allow(*)]"),
            IssueType::DeadCodeAllow => write!(f, "#[allow(dead_code)]"),
            IssueType::Unimplemented => write!(f, "unimplemented!()"),
            IssueType::Todo => write!(f, "todo!()"),
            IssueType::Panic => write!(f, "panic!()"),
            IssueType::EarlyReturnStub => write!(f, "Early Return Stub"),
            IssueType::UnderscorePrefix => write!(f, "_prefix"),
            IssueType::UnusedImport => write!(f, "Unused Import"),
            IssueType::PublicNeverCalled => write!(f, "Public Never Called"),
            IssueType::AlwaysErr => write!(f, "Always Returns Err"),
            IssueType::PlaceholderReturn => write!(f, "Placeholder Return"),
            IssueType::CfgTest => write!(f, "#[cfg(test)]"),
            IssueType::Emoji => write!(f, "Emoji"),
        }
    }
}

/// Issue found by linter (clippy/cargo check)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintIssue {
    pub file_path: String,
    pub line_number: usize,
    pub column: usize,
    pub level: LintLevel,
    pub code: String,
    pub message: String,
}

/// Severity level of a lint issue
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum LintLevel {
    Error,
    Warning,
    Help,
    Note,
}

impl std::fmt::Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevel::Error => write!(f, "error"),
            LintLevel::Warning => write!(f, "warning"),
            LintLevel::Help => write!(f, "help"),
            LintLevel::Note => write!(f, "note"),
        }
    }
}
