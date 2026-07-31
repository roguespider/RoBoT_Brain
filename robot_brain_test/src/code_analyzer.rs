//! Code Analyzer Module
//! 
//! Analyzes source code to detect:
//! - #[allow(*)] annotations
//! - unimplemented!() macros
//! - todo!() macros  
//! - panic!() with stub-like messages
//! - Partial/stub function implementations
//! - Functions that return early without doing work

use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;

/// Represents a code issue found during analysis
#[derive(Debug, Clone)]
pub struct CodeIssue {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub issue_type: IssueType,
    pub description: String,
    pub code_snippet: String,
}

/// Types of issues that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    AllowAnnotation,
    Unimplemented,
    Todo,
    Panic,
    EarlyReturnStub,
    PlaceholderReturn,
    StubPattern,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::AllowAnnotation => write!(f, "#[allow(*)]"),
            IssueType::Unimplemented => write!(f, "unimplemented!()"),
            IssueType::Todo => write!(f, "todo!()"),
            IssueType::Panic => write!(f, "panic!()"),
            IssueType::EarlyReturnStub => write!(f, "Early Return Stub"),
            IssueType::PlaceholderReturn => write!(f, "Placeholder Return"),
            IssueType::StubPattern => write!(f, "Stub Pattern"),
        }
    }
}

/// Analyzes the source code for stub patterns and partial implementations
pub struct CodeAnalyzer {
    /// Base path to the source code
    source_path: PathBuf,
}

impl CodeAnalyzer {
    pub fn new(source_path: PathBuf) -> Self {
        Self { source_path }
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
                    if name.as_ref().map(|n| !n.starts_with('.') && n != "target" && n != "tests" && n != "benches").unwrap_or(false) {
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
        }
        
        // Check for stub function patterns (entire function body analysis)
        issues.extend(self.analyze_stub_functions(&content, file_path));
        
        issues
    }

    /// Check for #[allow(*)] annotations
    fn check_allow_annotation(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        let allow_regex = Regex::new(r#"#\s*\[\s*allow\s*\([^)]*\)"#).ok()?;
        
        if allow_regex.is_match(line) {
            // Extract the allow annotation
            let captures = allow_regex.captures(line)?;
            let matched = captures.get(0)?.as_str();
            
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::AllowAnnotation,
                description: format!("Found #[allow(...)] annotation which suppresses warnings: {}", matched),
                code_snippet: line.trim().to_string(),
            })
        } else {
            None
        }
    }

    /// Check for unimplemented!() macro
    fn check_unimplemented(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        let unimplemented_regex = Regex::new(r"unimplemented!\s*\(").ok()?;
        
        if unimplemented_regex.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Unimplemented,
                description: "Found unimplemented!() macro - function is not implemented".to_string(),
                code_snippet: line.trim().to_string(),
            })
        } else {
            None
        }
    }

    /// Check for todo!() macro
    fn check_todo(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        let todo_regex = Regex::new(r"todo!\s*\(").ok()?;
        
        if todo_regex.is_match(line) {
            Some(CodeIssue {
                file_path: file_path.to_path_buf(),
                line_number,
                issue_type: IssueType::Todo,
                description: "Found todo!() macro - function implementation incomplete".to_string(),
                code_snippet: line.trim().to_string(),
            })
        } else {
            None
        }
    }

    /// Check for panic! with stub-like messages
    fn check_panic_stub(&self, line: &str, file_path: &Path, line_number: usize) -> Option<CodeIssue> {
        let panic_regex = Regex::new(r#"panic!\s*\(["#).ok()?;
        
        if panic_regex.is_match(line) {
            let lower_line = line.to_lowercase();
            let stub_indicators = [
                "stub", "not implemented", "todo", "wip", "placeholder",
                "not yet", "coming soon", "tbd", "xxx", "fixme"
            ];
            
            for indicator in stub_indicators {
                if lower_line.contains(indicator) {
                    return Some(CodeIssue {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        issue_type: IssueType::Panic,
                        description: format!("Found panic!() with stub indicator '{}' - function is not fully implemented", indicator),
                        code_snippet: line.trim().to_string(),
                    });
                }
            }
        }
        
        None
    }

    /// Analyze function bodies for stub patterns
    fn analyze_stub_functions(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        
        // Pattern: Function that only returns Ok() or Err() immediately
        let _stub_return_regex = match Regex::new(
            r"pub\s+async\s+fn\s+(\w+).*?\{[^}]*(Ok\(|Err\().*\}[^}]*$"
        ) {
            Ok(re) => Some(re),
            Err(_) => None,
        };
        
        // Pattern: Function that just returns default/unimplemented values
        let _placeholder_regex = match Regex::new(
            r"(Vec::new\(\)|HashMap::new\(\)|None|Default::default\(\)|\[\].*to_vec\(\))"
        ) {
            Ok(re) => Some(re),
            Err(_) => None,
        };
        
        // Check for functions that are just stubs returning empty/default values
        let _stub_fn_regex = match Regex::new(
            r"pub\s+(async\s+)?fn\s+(\w+).*?\{(\s*(//[^\n]*\n)?\s*)*(Ok\(|Err\(|return)"
        ) {
            Ok(re) => Some(re),
            Err(_) => None,
        };
        
        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;
            
            // Check for early returns that might indicate stubs
            let early_return_patterns = [
                (r"^\s*return\s+Ok\(ToolOutput::", "Returns Ok() immediately"),
                (r"^\s*return\s+Err\(", "Returns Err() immediately"),
                (r"^\s*Ok\(ToolOutput::", "Returns Ok() immediately"),
            ];
            
            for (pattern, desc) in early_return_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(line) {
                        // Check if this is the only statement in the function
                        let context_start = line_num.saturating_sub(5);
                        let context_lines: Vec<&str> = content.lines()
                            .skip(context_start)
                            .take(10)
                            .collect();
                        let context = context_lines.join("\n");
                        
                        // If function is just one or two lines, it's likely a stub
                        if context.matches('{').count() <= 2 && context.matches("Ok(").count() <= 2 {
                            issues.push(CodeIssue {
                                file_path: file_path.to_path_buf(),
                                line_number,
                                issue_type: IssueType::EarlyReturnStub,
                                description: desc.to_string(),
                                code_snippet: line.trim().to_string(),
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
                IssueType::Unimplemented => summary.unimplemented += 1,
                IssueType::Todo => summary.todos += 1,
                IssueType::Panic => summary.panics += 1,
                IssueType::EarlyReturnStub => summary.early_returns += 1,
                IssueType::PlaceholderReturn => summary.placeholder_returns += 1,
                IssueType::StubPattern => summary.stub_patterns += 1,
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
    pub unimplemented: usize,
    pub todos: usize,
    pub panics: usize,
    pub early_returns: usize,
    pub placeholder_returns: usize,
    pub stub_patterns: usize,
    pub issues_by_file: std::collections::HashMap<String, usize>,
}

impl AnalysisSummary {
    /// Check if there are any critical issues
    #[allow(dead_code)]
    pub fn has_critical_issues(&self) -> bool {
        self.unimplemented > 0 || self.todos > 0 || self.stub_patterns > 0
    }
    
    /// Print summary in table format
    pub fn print_table(&self) {
        println!("\n{}", "═".repeat(80));
        println!("  CODE ANALYSIS SUMMARY");
        println!("{}", "═".repeat(80));
        println!();
        println!("  {:<35} {:>10}", "Issue Type", "Count");
        println!("  {}", "─".repeat(48));
        println!("  {:<35} {:>10}", "#[allow(*)] annotations", self.allow_annotations);
        println!("  {:<35} {:>10}", "unimplemented!() macros", self.unimplemented);
        println!("  {:<35} {:>10}", "todo!() macros", self.todos);
        println!("  {:<35} {:>10}", "panic!() stubs", self.panics);
        println!("  {:<35} {:>10}", "Early return stubs", self.early_returns);
        println!("  {:<35} {:>10}", "Placeholder returns", self.placeholder_returns);
        println!("  {:<35} {:>10}", "Stub patterns", self.stub_patterns);
        println!("  {}", "─".repeat(48));
        println!("  {:<35} {:>10}", "TOTAL ISSUES", self.total_issues);
        println!();
        
        if !self.issues_by_file.is_empty() {
            println!("  Issues by File:");
            println!("  {}", "─".repeat(48));
            for (file, count) in &self.issues_by_file {
                // Extract relative path
                let relative = file.split("src/").last().unwrap_or(file);
                println!("    {:<40} {:>6}", relative, count);
            }
        }
        
        println!("{}", "═".repeat(80));
    }
}
