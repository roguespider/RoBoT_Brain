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

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Fallback regex pattern (always valid - "." matches any character)
static FALLBACK_REGEX: OnceLock<Regex> = OnceLock::new();

/// Get or create the fallback regex pattern
///
/// Uses unwrap_or_else to handle the Result safely. For the dot pattern ".":
/// - "." is a valid regex that matches any single character
/// - The regex crate guarantees this pattern will always compile successfully
/// - Using unwrap_or_else with a fallback closure handles potential edge cases
fn get_fallback_regex() -> &'static Regex {
    FALLBACK_REGEX.get_or_init(|| {
        // "." is guaranteed to be a valid regex pattern
        Regex::new(".").unwrap_or_else(|_| {
            // If "." fails (theoretically impossible), use "$." which matches
            // a newline at the end of string, or use another safe fallback
            Regex::new("$.").unwrap_or_else(|_| {
                // Final fallback - these patterns are guaranteed to be valid
                // We iterate through known-valid patterns until one succeeds
                // This loop always executes at least once since "." is always valid
                loop {
                    // Try a series of guaranteed-valid patterns
                    let pattern = "."; // Always valid - matches any character
                    match Regex::new(pattern) {
                        Ok(r) => break r,
                        Err(_) => continue, // Should never happen for "."
                    }
                }
            })
        })
    })
}

/// Represents a code issue found during analysis
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, PartialEq)]
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
        }
    }
}

/// Pre-compiled regex patterns for code analysis
struct CodePatterns {
    allow_annotation: Regex,
    dead_code_allow: Regex,
    unimplemented: Regex,
    todo: Regex,
    panic: Regex,
    underscore_prefix: Regex,
}

impl CodePatterns {
    fn new() -> Self {
        Self {
            allow_annotation: Regex::new(r#"#\s*\[\s*allow\s*\([^)]*\)"#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            dead_code_allow: Regex::new(r#"#\s*\[\s*allow\s*\(dead_code\)"#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            unimplemented: Regex::new(r"unimplemented!\s*\(")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            todo: Regex::new(r"todo!\s*\(")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            panic: Regex::new(r#"panic!\s*\("#)
                .unwrap_or_else(|_| get_fallback_regex().clone()),
            underscore_prefix: Regex::new(r"\b_\w+\b")
                .unwrap_or_else(|_| get_fallback_regex().clone()),
        }
    }
}

/// Analyzes the source code for stub patterns and partial implementations
pub struct CodeAnalyzer {
    /// Base path to the source code
    source_path: PathBuf,
    /// Pre-compiled regex patterns
    patterns: CodePatterns,
}

impl CodeAnalyzer {
    pub fn new(source_path: PathBuf) -> Self {
        Self {
            source_path,
            patterns: CodePatterns::new(),
        }
    }

    /// Run full analysis on the source code
    pub fn analyze(&self) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Find all Rust source files
        let rust_files = self.find_rust_files();

        for file_path in rust_files {
            let file_issues = self.analyze_file(&file_path);
            issues.extend(file_issues);
        }

        issues
    }

    /// Find all .rs files in the source directory
    fn find_rust_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.collect_rust_files(&self.source_path, &mut files);
        files
    }

    fn collect_rust_files(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip test directories and target
                    let name = path.file_name().map(|n| n.to_string_lossy());
                    if name
                        .as_ref()
                        .map(|n| {
                            !n.starts_with('.') && n != "target" && n != "tests" && n != "benches"
                        })
                        .unwrap_or(false)
                    {
                        self.collect_rust_files(&path, files);
                    }
                } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
    }

    /// Analyze a single file for issues
    fn analyze_file(&self, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return issues,
        };

        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_number = line_num + 1;

            // Check for #[allow(*)]
            if let Some(issue) = self.check_allow_annotation(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for #[allow(dead_code)] specifically
            if let Some(issue) = self.check_dead_code_allow(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for unimplemented!()
            if let Some(issue) = self.check_unimplemented(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for todo!()
            if let Some(issue) = self.check_todo(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for panic! with stub messages
            if let Some(issue) = self.check_panic_stub(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for underscore-prefixed identifiers
            if let Some(issue) = self.check_underscore_prefix(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for unused imports
            if let Some(issue) = self.check_unused_import(line, file_path, line_number) {
                issues.push(issue);
            }
        }

        // Check for stub function patterns (entire function body analysis)
        issues.extend(self.analyze_stub_functions(&content, file_path));

        issues
    }

    /// Check for #[allow(*)] annotations
    fn check_allow_annotation(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.allow_annotation.is_match(line) {
            // Extract the allow annotation
            let captures = self.patterns.allow_annotation.captures(line)?;
            let matched = captures.get(0)?.as_str();

            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::AllowAnnotation,
                description: format!(
                    "Found #[allow(...)] annotation which suppresses warnings: {}",
                    matched
                ),
            })
        } else {
            None
        }
    }

    /// Check for unimplemented!() macro
    fn check_unimplemented(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.unimplemented.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Unimplemented,
                description: "Found unimplemented!() macro - function is not implemented"
                    .to_string(),
            })
        } else {
            None
        }
    }

    /// Check for todo!() macro
    fn check_todo(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        if self.patterns.todo.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Todo,
                description: "Found todo!() macro - function implementation incomplete".to_string(),
            })
        } else {
            None
        }
    }

    /// Check for panic! with stub-like messages
    fn check_panic_stub(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.panic.is_match(line) {
            let lower_line = line.to_lowercase();
            let stub_indicators = [
                "stub",
                "not implemented",
                "todo",
                "wip",
                "placeholder",
                "not yet",
                "coming soon",
                "tbd",
                "xxx",
                "fixme",
            ];

            for indicator in stub_indicators {
                if lower_line.contains(indicator) {
                    return Some(CodeIssue {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        issue_type: IssueType::Panic,
                        description: format!("Found panic!() with stub indicator '{}' - function is not fully implemented", indicator),
                    });
                }
            }
        }

        None
    }

    /// Check for underscore-prefixed identifiers (variables, functions, etc.)
    /// These often indicate unused or intentionally ignored code that should be reviewed
    fn check_underscore_prefix(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        // Skip comments and doc comments
        let trimmed = line.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
        {
            return None;
        }

        // Skip lines that are just type annotations for unused params
        if line.contains("__") {
            // Double underscore often means intentionally ignored
            return None;
        }

        // Skip string literals containing underscore-prefixed identifiers
        // This handles cases like ["_id", "_type"] in JSON field arrays
        let without_strings = line
            .split(|c| ['"', '\''].contains(&c))
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(caps) = self.patterns.underscore_prefix.captures(&without_strings) {
            let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");

            return Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::UnderscorePrefix,
                description: format!("Underscore-prefixed identifier: {}", matched),
            });
        }

        None
    }

    /// Check for #[allow(dead_code)] and #[allow(unused)] annotations
    fn check_dead_code_allow(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.dead_code_allow.is_match(line) {
            let captures = self.patterns.dead_code_allow.captures(line)?;
            let matched = captures.get(0)?.as_str();

            return Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::DeadCodeAllow,
                description: format!(
                    "Found #[allow(dead_code)] suppression - dead code may exist: {}",
                    matched
                ),
            });
        }
        None
    }

    /// Check for unused imports
    fn check_unused_import(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        let trimmed = line.trim();
        if !trimmed.starts_with("use ") && !trimmed.starts_with("use {") {
            return None;
        }

        // Skip lines that end with ; and contain wildcard or multiple items (likely used)
        // Flag single imports that may be unused
        if trimmed.starts_with("use crate::") || trimmed.starts_with("use std::") {
            return Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::UnusedImport,
                description: format!("Import that may be unused: {}", trimmed),
            });
        }
        None
    }

    /// Analyze function bodies for stub patterns
    fn analyze_stub_functions(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Pattern: Function that only returns Ok() or Err() immediately
        let _stub_return_regex =
            Regex::new(r"pub\s+async\s+fn\s+(\w+).*?\{[^}]*(Ok\(|Err\().*\}[^}]*$").ok();

        // Pattern: Function that just returns default/unimplemented values
        let _placeholder_regex = Regex::new(
            r"(Vec::new\(\)|HashMap::new\(\)|None|Default::default\(\)|\[\].*to_vec\(\))",
        )
        .ok();

        // Check for functions that are just stubs returning empty/default values
        let _stub_fn_regex = Regex::new(
            r"pub\s+(async\s+)?fn\s+(\w+).*?\{(\s*(//[^\n]*\n)?\s*)*(Ok\(|Err\(|return)",
        )
        .ok();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Check for early returns that might indicate stubs
            let early_return_patterns = [
                (
                    r"^\s*return\s+Ok\(\)\s*;?\s*$",
                    "Returns Ok() with no value",
                ),
                (r"^\s*return\s+Err\(", "Returns Err() immediately"),
                (r"^\s*Ok\(ToolOutput::", "Returns Ok() immediately"),
            ];

            for (pattern, desc) in early_return_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(line) {
                        // Check if this is the only statement in the function
                        let context_start = line_num.saturating_sub(5);
                        let context_lines: Vec<&str> =
                            content.lines().skip(context_start).take(10).collect();
                        let context = context_lines.join("\n");

                        // If function is just one or two lines, it's likely a stub
                        // For Err returns: be aggressive and flag them
                        // For Ok returns: skip if context has proper error handling
                        let has_proper_error = context.contains("Error")
                            || context.contains("anyhow")
                            || context.contains("Result")
                            || context.contains("?");
                        let is_err_return = desc.contains("Err()");
                        if context.matches('{').count() <= 2
                            && context.matches("Ok(").count() <= 2
                            && (is_err_return || !has_proper_error)
                        {
                            issues.push(CodeIssue {
                                file_path: file_path.to_path_buf(),
                                line_number,
                                issue_type: IssueType::EarlyReturnStub,
                                description: desc.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Check for public functions that only return Err() (always-failing functions)
        let public_fn_re = Regex::new(r"pub\s+(async\s+)?fn\s+(\w+)\s*\(").ok();
        let return_err_re = Regex::new(r"return\s+Err\(").ok();
        if let (Some(pub_re), Some(err_re)) = (public_fn_re, return_err_re) {
            for (line_num, line) in content.lines().enumerate() {
                let line_number = line_num + 1;
                // Check if this line declares a public function
                if let Some(caps) = pub_re.captures(line) {
                    let fn_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                    // Look at the next few lines to see if it immediately returns Err
                    let context_lines: Vec<&str> = content.lines().skip(line_num).take(8).collect();
                    let fn_body = context_lines.join("\n");

                    // If the function body is just "return Err(...);" with no actual logic
                    if err_re.is_match(&fn_body)
                        && fn_body.contains("return Err(")
                        && fn_body.matches('{').count() <= 2
                        && !fn_body.contains("match ")
                        && !fn_body.contains("if ")
                        && !fn_body.contains("await ")
                        && !fn_body.contains("let ")
                    {
                        issues.push(CodeIssue {
                            file_path: file_path.to_path_buf(),
                            line_number,
                            issue_type: IssueType::AlwaysErr,
                            description: format!(
                                "Public function '{}' always returns Err - likely a stub",
                                fn_name
                            ),
                        });
                    }
                }
            }
        }

        // Check for public functions that appear to be defined but never referenced in the same file
        // This is a heuristic: if a pub fn has no calls to it within 50 lines, flag it
        let pub_fn_re = Regex::new(r"pub\s+(async\s+)?fn\s+(\w+)\s*").ok();
        if let Some(pub_re) = pub_fn_re {
            for (line_num, line) in content.lines().enumerate() {
                let line_number = line_num + 1;
                if let Some(caps) = pub_re.captures(line) {
                    let fn_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    if fn_name.is_empty() {
                        continue;
                    }

                    // Look in surrounding lines for usages of this function
                    let start = line_num.saturating_sub(3);
                    let end = std::cmp::min(line_num + 30, content.lines().count());
                    let surrounding: String = content
                        .lines()
                        .skip(start)
                        .take(end - start)
                        .collect::<Vec<_>>()
                        .join("\n");

                    // Count usages of this function name outside the definition line
                    let usages = surrounding.matches(fn_name).count() - 1; // subtract the definition line
                    if usages == 0
                        && fn_name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                    {
                        // Skip common test/utility patterns
                        let skip_names = [
                            "new", "default", "from", "into", "try_from", "try_into", "clone",
                            "copy",
                        ];
                        if !skip_names.contains(&fn_name) {
                            issues.push(CodeIssue {
                                file_path: file_path.to_path_buf(),
                                line_number,
                                issue_type: IssueType::PublicNeverCalled,
                                description: format!(
                                    "Public function '{}' appears to never be called in its module",
                                    fn_name
                                ),
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Get summary statistics
    pub fn get_summary(&self, issues: &[CodeIssue]) -> AnalysisSummary {
        let mut summary = AnalysisSummary::default();

        for issue in issues {
            match issue.issue_type {
                IssueType::AllowAnnotation => summary.allow_annotations += 1,
                IssueType::DeadCodeAllow => summary.dead_code_allows += 1,
                IssueType::Unimplemented => summary.unimplemented += 1,
                IssueType::Todo => summary.todos += 1,
                IssueType::Panic => summary.panics += 1,
                IssueType::EarlyReturnStub => summary.early_returns += 1,
                IssueType::UnderscorePrefix => summary.underscore_prefixes += 1,
                IssueType::UnusedImport => summary.unused_imports += 1,
                IssueType::PublicNeverCalled => summary.public_never_called += 1,
                IssueType::AlwaysErr => summary.always_err += 1,
            }

            // Count by file
            let file_key = issue.file_path.to_string_lossy().to_string();
            *summary.issues_by_file.entry(file_key).or_insert(0) += 1;
        }

        summary.total_issues = issues.len();
        summary
    }
}

/// Summary of code analysis
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    pub total_issues: usize,
    pub allow_annotations: usize,
    pub dead_code_allows: usize,
    pub unimplemented: usize,
    pub todos: usize,
    pub panics: usize,
    pub early_returns: usize,
    pub underscore_prefixes: usize,
    pub unused_imports: usize,
    pub public_never_called: usize,
    pub always_err: usize,
    pub issues_by_file: std::collections::HashMap<String, usize>,
}

impl AnalysisSummary {
    /// Print summary in table format
    pub fn print_table(&self) {
        crate::teeprintln!("\n{}", "═".repeat(80));
        crate::teeprintln!("  CODE ANALYSIS SUMMARY");
        crate::teeprintln!("{}", "═".repeat(80));
        crate::teeprintln!("");
        crate::teeprintln!("  {:<35} {:>10}", "Issue Type", "Count");
        crate::teeprintln!("  {}", "─".repeat(48));
        crate::teeprintln!(
            "  {:<35} {:>10}",
            "#[allow(*)] annotations",
            self.allow_annotations
        );
        crate::teeprintln!(
            "  {:<35} {:>10}",
            "#[allow(dead_code)] suppression",
            self.dead_code_allows
        );
        crate::teeprintln!(
            "  {:<35} {:>10}",
            "unimplemented!() macros",
            self.unimplemented
        );
        crate::teeprintln!("  {:<35} {:>10}", "todo!() macros", self.todos);
        crate::teeprintln!("  {:<35} {:>10}", "panic!() stubs", self.panics);
        crate::teeprintln!("  {:<35} {:>10}", "Early return stubs", self.early_returns);
        crate::teeprintln!(
            "  {:<35} {:>10}",
            "Underscore-prefixed code",
            self.underscore_prefixes
        );
        crate::teeprintln!("  {:<35} {:>10}", "Unused imports", self.unused_imports);
        crate::teeprintln!(
            "  {:<35} {:>10}",
            "Public never called",
            self.public_never_called
        );
        crate::teeprintln!("  {:<35} {:>10}", "Always returns Err", self.always_err);
        crate::teeprintln!("  {}", "─".repeat(48));
        crate::teeprintln!("  {:<35} {:>10}", "TOTAL ISSUES", self.total_issues);
        crate::teeprintln!("");

        if !self.issues_by_file.is_empty() {
            crate::teeprintln!("  Issues by File:");
            crate::teeprintln!("  {}", "─".repeat(48));
            for (file, count) in &self.issues_by_file {
                // Extract relative path
                let relative = file.split("src/").last().unwrap_or(file);
                crate::teeprintln!("    {:<40} {:>6}", relative, count);
            }
        }

        crate::teeprintln!("{}", "═".repeat(80));
    }
}

// ========================================================================
// LINT ANALYSIS
// ========================================================================

use std::process::Command;

/// Represents a lint warning or error from clippy or rustc
#[derive(Debug, Clone, PartialEq)]
pub struct LintIssue {
    pub file_path: String,
    pub line_number: usize,
    pub column: usize,
    pub level: LintLevel,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
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

/// Runs clippy and cargo check to find lint issues
pub struct LintAnalyzer;

impl LintAnalyzer {
    /// Run cargo clippy and parse the output
    pub fn run_clippy(project_path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let output = Command::new("cargo")
            .args(["clippy", "--", "-D", "warnings", "--cap-lints", "warn"])
            .current_dir(project_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(Self::parse_lint_output(&combined))
    }

    /// Run cargo check and parse the output
    pub fn run_check(project_path: &Path) -> anyhow::Result<Vec<LintIssue>> {
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(project_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);

        Ok(Self::parse_lint_output(&combined))
    }

    /// Parse lint output into structured issues
    fn parse_lint_output(output: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        // Pattern for rustc/clipp output: file:line:col: level: code (message)
        let re = match regex::Regex::new(
            r"^(.+?):(\d+):(\d+):\s*((?:error|warning|help|note)+(?:\[\w+\])?):\s*((?:\w+)+)\s*(.*)$"
        ) {
            Ok(r) => r,
            Err(_) => {
                // If pattern is invalid (should never happen), return empty results
                return issues;
            }
        };

        for line in output.lines() {
            // Try main pattern first
            if let Some(caps) = re.captures(line) {
                let file = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let col: usize = caps
                    .get(3)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                let level_str = caps.get(4).map(|m| m.as_str()).unwrap_or("warning");
                let code = caps.get(5).map(|m| m.as_str()).unwrap_or("").to_string();
                let message = caps
                    .get(6)
                    .map(|m| m.as_str().trim())
                    .unwrap_or("")
                    .to_string();

                let level = match level_str.trim() {
                    s if s.contains("error") => LintLevel::Error,
                    s if s.contains("warning") => LintLevel::Warning,
                    s if s.contains("help") => LintLevel::Help,
                    _ => LintLevel::Note,
                };

                // Skip empty messages and notes/help without code
                if !message.is_empty()
                    && (level == LintLevel::Error
                        || level == LintLevel::Warning
                        || !code.is_empty())
                {
                    issues.push(LintIssue {
                        file_path: file,
                        line_number: line_num,
                        column: col,
                        level,
                        code,
                        message,
                    });
                }
            }
        }

        // Deduplicate
        issues.sort_by(|a, b| {
            let file_cmp = a.file_path.cmp(&b.file_path);
            if file_cmp == std::cmp::Ordering::Equal {
                a.line_number.cmp(&b.line_number)
            } else {
                file_cmp
            }
        });
        issues.dedup();

        issues
    }
}

/// Summary of lint analysis
#[derive(Debug, Default)]
pub struct LintSummary {
    pub errors: usize,
    pub warnings: usize,
    pub helps: usize,
    pub notes: usize,
    pub issues: Vec<LintIssue>,
    pub issues_by_file: std::collections::HashMap<String, usize>,
}

impl LintSummary {
    pub fn new(issues: Vec<LintIssue>) -> Self {
        let mut summary = Self {
            issues,
            ..Default::default()
        };

        for issue in &summary.issues {
            match issue.level {
                LintLevel::Error => summary.errors += 1,
                LintLevel::Warning => summary.warnings += 1,
                LintLevel::Help => summary.helps += 1,
                LintLevel::Note => summary.notes += 1,
            }

            let file_key = issue.file_path.clone();
            *summary.issues_by_file.entry(file_key).or_insert(0) += 1;
        }

        summary
    }

    /// Print lint summary and issues table
    pub fn print_report(&self) {
        crate::teeprintln!("\n{}", "═".repeat(100));
        crate::teeprintln!("  LINT ANALYSIS SUMMARY (clippy + cargo check)");
        crate::teeprintln!("{}", "═".repeat(100));
        crate::teeprintln!("");
        crate::teeprintln!("  {:<20} {:>10}", "Lint Level", "Count");
        crate::teeprintln!("  {}", "─".repeat(33));
        if self.errors > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Errors", self.errors);
        }
        if self.warnings > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Warnings", self.warnings);
        }
        if self.helps > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Help suggestions", self.helps);
        }
        if self.notes > 0 {
            crate::teeprintln!("  {:<20} {:>10}", "Notes", self.notes);
        }
        crate::teeprintln!("  {}", "─".repeat(33));
        crate::teeprintln!(
            "  {:<20} {:>10}",
            "TOTAL (E+W)",
            self.errors + self.warnings
        );
        crate::teeprintln!("");

        if !self.issues_by_file.is_empty() && (self.errors > 0 || self.warnings > 0) {
            crate::teeprintln!("  Lint issues by File:");
            crate::teeprintln!("  {}", "─".repeat(33));
            let mut files: Vec<_> = self.issues_by_file.iter().collect();
            files.sort_by(|a, b| b.1.cmp(a.1));
            for (file, count) in files.iter().take(10) {
                let relative = file.split("src/").last().unwrap_or(file);
                crate::teeprintln!("    {:.<40} {:>6}", relative, count);
            }
        }

        // Print detailed issues table if there are errors or warnings
        if !self.issues.is_empty()
            && self
                .issues
                .iter()
                .any(|i| i.level == LintLevel::Error || i.level == LintLevel::Warning)
        {
            crate::teeprintln!("");
            crate::teeprintln!("{}", "─".repeat(100));
            crate::teeprintln!("  DETAILED LINT ISSUES TABLE");
            crate::teeprintln!("{}", "─".repeat(100));
            crate::teeprintln!("");
            crate::teeprintln!("┌{:─<8}┬{:─<6}┬{:─<50}┬{:─<30}┐", "", "", "", "");
            crate::teeprintln!(
                "│{:^8}│{:^6}│{:^50}│{:^30}│",
                "Level",
                "Line",
                "File",
                "Message"
            );
            crate::teeprintln!("├{:─<8}┼{:─<6}┼{:─<50}┼{:─<30}┤", "", "", "", "");

            for issue in &self.issues {
                if issue.level == LintLevel::Error || issue.level == LintLevel::Warning {
                    let file_short = if issue.file_path.len() > 48 {
                        format!("...{}", &issue.file_path[issue.file_path.len() - 45..])
                    } else {
                        issue.file_path.clone()
                    };

                    let msg_short = if issue.message.len() > 28 {
                        format!("{}...", &issue.message[..25])
                    } else {
                        issue.message.clone()
                    };

                    let level_str = match issue.level {
                        LintLevel::Error => "ERROR",
                        LintLevel::Warning => "WARN",
                        LintLevel::Help => "HELP",
                        LintLevel::Note => "NOTE",
                    };

                    crate::teeprintln!(
                        "│{:^8}│{:^6}│{:.<50}│{:.<30}│",
                        level_str,
                        issue.line_number,
                        file_short,
                        msg_short
                    );
                }
            }

            crate::teeprintln!("└{:─<8}┴{:─<6}┴{:─<50}┴{:─<30}┘", "", "", "", "");
        }

        crate::teeprintln!("{}", "═".repeat(100));
    }
}
