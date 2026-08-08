//! Code analyzer for detecting stub patterns

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::patterns::CodePatterns;
use super::types::{CodeIssue, IssueType};

/// Analyzes the source code for stub patterns and partial implementations
pub struct CodeAnalyzer {
    /// Base path to the source code
    source_path: PathBuf,
    /// Pre-compiled regex patterns
    patterns: CodePatterns,
}

impl CodeAnalyzer {
    /// Create a new code analyzer
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

            // Check for panic!()
            if let Some(issue) = self.check_panic_stub(line, file_path, line_number) {
                issues.push(issue);
            }

            // Check for underscore-prefixed variables
            if let Some(issue) = self.check_underscore_prefix(line, file_path, line_number) {
                issues.push(issue);
            }
        }

        // Analyze for stub functions
        let stub_issues = self.analyze_stub_functions(&content, file_path);
        issues.extend(stub_issues);

        issues
    }

    fn check_allow_annotation(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.allow_annotation.is_match(line) {
            // But skip if it's the module-level declaration
            if line.trim().starts_with("mod ") || line.trim().starts_with("pub mod ") {
                return None;
            }
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::AllowAnnotation,
                description: "#[allow(*)] annotation found - silences compiler warnings".to_string(),
            })
        } else {
            None
        }
    }

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
                description: "unimplemented!() macro found - function not implemented".to_string(),
            })
        } else {
            None
        }
    }

    fn check_todo(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        if self.patterns.todo.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Todo,
                description: "todo!() macro found - incomplete code".to_string(),
            })
        } else {
            None
        }
    }

    fn check_panic_stub(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.panic.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Panic,
                description: "panic!() macro found - potential early exit".to_string(),
            })
        } else {
            None
        }
    }

    fn check_underscore_prefix(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        // Skip comments and strings
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('"') {
            return None;
        }

        if self.patterns.underscore_prefix.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::UnderscorePrefix,
                description: "Underscore-prefixed identifier found - potentially unused".to_string(),
            })
        } else {
            None
        }
    }

    fn check_dead_code_allow(
        &self,
        line: &str,
        file_path: &Path,
        line_number: usize,
    ) -> Option<CodeIssue> {
        if self.patterns.dead_code_allow.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::DeadCodeAllow,
                description: "#[allow(dead_code)] found - dead code suppressed".to_string(),
            })
        } else {
            None
        }
    }

    fn check_unused_import(
        &self,
        _line: &str,
        _file_path: &Path,
        _line_number: usize,
    ) -> Option<CodeIssue> {
        // This would require more sophisticated AST analysis
        // For now, return None - Rust's unused_imports lint handles this
        None
    }

    fn analyze_stub_functions(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut in_function = false;
        let mut function_start = 0;
        let mut function_name = String::new();
        let mut brace_count = 0;
        let mut has_real_work = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Function detection
            if !in_function
                && (trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("pub async fn ")
                    || trimmed.starts_with("async fn "))
            {
                in_function = true;
                function_start = i;
                has_real_work = false;
                brace_count = 0;

                // Extract function name
                function_name = trimmed
                    .split(|c: char| c == '(' || c == '{')
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();
            }

            if in_function {
                // Count braces
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                // Check for actual work
                if trimmed.ends_with(';')
                    && !trimmed.contains("//")
                    && !trimmed.contains("return")
                    && !trimmed.is_empty()
                {
                    has_real_work = true;
                }

                // Function ends when braces are balanced
                if brace_count == 0 && i > function_start {
                    // Check for stub patterns
                    let func_content: String = lines[function_start..=i].join("\n");

                    // Check for immediate return with placeholder
                    if func_content.contains("->") {
                        let return_match = lines[function_start..=i]
                            .iter()
                            .find(|l| l.contains("return") || l.contains("->"));

                        if let Some(ret_line) = return_match {
                            if ret_line.contains("None")
                                || ret_line.contains("todo!")
                                || ret_line.contains("unimplemented")
                                || ret_line.contains("Default")
                            {
                                issues.push(CodeIssue {
                                    file_path: file_path.to_path_buf(),
                                    line_number: function_start + 1,
                                    issue_type: IssueType::PlaceholderReturn,
                                    description: format!(
                                        "Function '{}' returns placeholder value",
                                        function_name
                                    ),
                                });
                            }
                        }
                    }

                    // This is a heuristic: if a pub fn has no calls to it within 50 lines, flag it
                    // (very rough heuristic - could have false positives)
                    in_function = false;
                }
            }
        }

        issues
    }

    /// Get analysis summary
    pub fn get_summary(&self, issues: &[CodeIssue]) -> AnalysisSummary {
        AnalysisSummary::new(issues, &self.source_path)
    }
}

/// Summary of code analysis
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    pub total_issues: usize,
    pub issues_by_type: HashMap<String, usize>,
    pub issues_by_file: HashMap<String, usize>,
    pub source_path: PathBuf,
}

impl AnalysisSummary {
    /// Create new summary from issues
    pub fn new(issues: &[CodeIssue], source_path: &Path) -> Self {
        let mut summary = Self {
            total_issues: issues.len(),
            source_path: source_path.to_path_buf(),
            ..Default::default()
        };

        for issue in issues {
            let type_key = issue.issue_type.to_string();
            *summary.issues_by_type.entry(type_key).or_insert(0) += 1;

            let file_key = if let Ok(stripped) =
                issue.file_path.strip_prefix(source_path)
            {
                stripped.to_string_lossy().to_string()
            } else {
                issue.file_path.to_string_lossy().to_string()
            };
            *summary.issues_by_file.entry(file_key).or_insert(0) += 1;
        }

        summary
    }

    /// Print analysis summary table
    pub fn print_table(&self) {
        crate::teeprintln!("\n{}", "═".repeat(100));
        crate::teeprintln!("  CODE ANALYSIS SUMMARY");
        crate::teeprintln!("{}", "═".repeat(100));
        crate::teeprintln!("");
        crate::teeprintln!(
            "  {:<30} {:>10}",
            "Issue Type",
            "Count"
        );
        crate::teeprintln!("  {}", "─".repeat(42));

        let mut types: Vec<_> = self.issues_by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));

        for (issue_type, count) in types {
            crate::teeprintln!("  {:<30} {:>10}", issue_type, count);
        }

        crate::teeprintln!("  {}", "─".repeat(42));
        crate::teeprintln!("  {:<30} {:>10}", "TOTAL", self.total_issues);
        crate::teeprintln!("");
    }
}
