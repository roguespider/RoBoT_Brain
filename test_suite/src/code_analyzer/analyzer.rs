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

        // Per-file analysis (reads each file and analyzes its content)
        for file_path in &rust_files {
            let file_issues = self.analyze_file(file_path);
            issues.extend(file_issues);
        }

        // Collect all file contents for cross-file analysis
        // (files are re-read here to avoid storing contents during per-file pass)
        let mut file_contents: Vec<(PathBuf, String)> = Vec::new();
        for file_path in &rust_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                file_contents.push((file_path.clone(), content));
            }
        }

        // Cross-file analysis: detect pub fn declarations never called anywhere
        // Architecture requirement (30-Testing-and-Validation-Architecture.md §30.7):
        // "Validation must confirm: it is called"
        let never_called_issues = self.analyze_never_called(&file_contents);
        issues.extend(never_called_issues);

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

    /// Analyze a single file for issues (reads file from disk)
    pub fn analyze_file(&self, file_path: &Path) -> Vec<CodeIssue> {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        self.analyze_file_content(file_path, &content)
    }

    /// Analyze a single file's content for issues
    fn analyze_file_content(&self, file_path: &Path, content: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

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
        let stub_issues = self.analyze_stub_functions(content, file_path);
        issues.extend(stub_issues);

        // Analyze for unused imports (architecture: "no unused abstractions")
        let import_issues = self.analyze_unused_imports(content, file_path);
        issues.extend(import_issues);

        // Analyze for early return stubs
        let early_return_issues = self.analyze_early_returns(content, file_path);
        issues.extend(early_return_issues);

        // Analyze for functions that always return Err
        let always_err_issues = self.analyze_always_err(content, file_path);
        issues.extend(always_err_issues);

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

    /// Detect `use` imports whose imported names are never referenced elsewhere in the file.
    ///
    /// Architecture requirement (30-Testing-and-Validation-Architecture.md §30.6):
    /// "no unused abstractions". This cross-references each imported symbol against
    /// the rest of the file content.
    ///
    /// NOTE: `pub use` re-exports are NOT checked — they are public API exports
    /// consumed by other modules, not local imports. Only private `use` statements
    /// are checked for local usage.
    fn analyze_unused_imports(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip pub use re-exports — they are public API, consumed externally
            if trimmed.starts_with("pub use ") {
                continue;
            }

            // Only process private use statements
            if !trimmed.starts_with("use ") {
                continue;
            }

            // Extract the imported name(s) from the use statement
            // Handles: use foo::bar;  use foo::{bar, baz};  use foo::*;  use foo as bar;
            let import_names = extract_imported_names(trimmed);
            if import_names.is_empty() {
                continue;
            }

            // For each imported name, check if it appears elsewhere in the file
            // (excluding the import line itself and comments)
            for name in &import_names {
                // Skip glob imports (use foo::*), self, and crate references
                // These can't be verified statically without knowing module exports
                if name.ends_with('*') || *name == "self" || *name == "crate" {
                    continue;
                }

                // For names containing "::" (e.g., path-style imports in groups),
                // extract the last segment as the local name
                let check_name = name.rsplit("::").next().unwrap_or(name);

                // Skip known trait imports — they're used via method syntax (.context(),
                // .read_to_string(), etc.) not by name. Check for their method patterns.
                if is_trait_used_via_methods(check_name, &lines, line_num) {
                    continue;
                }

                let is_used = lines
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != line_num)
                    .any(|(_, l)| {
                        let l_trimmed = l.trim();
                        // Skip comment lines
                        if l_trimmed.starts_with("//") {
                            return false;
                        }
                        // Check if the name appears as a word boundary match
                        contains_word(l, check_name)
                    });

                if !is_used {
                    issues.push(CodeIssue {
                        file_path: file_path.to_path_buf(),
                        line_number: line_num + 1,
                        issue_type: IssueType::UnusedImport,
                        description: format!(
                            "Import '{}' is never used in this file",
                            check_name
                        ),
                    });
                }
            }
        }

        issues
    }

    /// Detect functions that return immediately (early return stub) before doing real work.
    ///
    /// An early return stub is a function whose body begins with a `return` statement
    /// on the first meaningful line, bypassing any real computation.
    fn analyze_early_returns(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut in_function = false;
        let mut function_start = 0;
        let mut brace_count = 0;
        let mut function_name = String::new();
        let mut body_started = false;
        let mut first_body_line: Option<usize> = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if !in_function
                && (trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("pub async fn ")
                    || trimmed.starts_with("async fn "))
            {
                in_function = true;
                function_start = i;
                brace_count = 0;
                body_started = false;
                first_body_line = None;
                function_name = trimmed
                    .split(['(', '{'])
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();
            }

            if in_function {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                // Track when the function body starts (first line after opening brace)
                if !body_started && brace_count > 0 {
                    body_started = true;
                }
                if body_started && first_body_line.is_none() && !trimmed.is_empty() {
                    // Skip lines that are just the opening brace or doc comments
                    if trimmed != "{" && !trimmed.starts_with("///") && !trimmed.starts_with("//") {
                        first_body_line = Some(i);
                    }
                }

                // Function ends
                if brace_count == 0 && i > function_start {
                    // Check if the first body line is an immediate return
                    if let Some(first_idx) = first_body_line {
                        let first_line = lines[first_idx].trim();
                        if first_line.starts_with("return ")
                            || first_line.starts_with("return;")
                        {
                            // Only flag if it's not a conditional return (if/return patterns are OK)
                            if !first_line.starts_with("return Err")
                                && !first_line.contains("if ")
                            {
                                issues.push(CodeIssue {
                                    file_path: file_path.to_path_buf(),
                                    line_number: function_start + 1,
                                    issue_type: IssueType::EarlyReturnStub,
                                    description: format!(
                                        "Function '{}' returns immediately without doing work",
                                        function_name
                                    ),
                                });
                            }
                        }
                    }

                    in_function = false;
                }
            }
        }

        issues
    }

    /// Detect functions whose body only returns `Err(...)`, regardless of input.
    ///
    /// Such functions always fail and represent incomplete or stub implementations.
    ///
    /// To avoid false positives, this only flags functions where:
    /// - The body contains `return Err(` or a trailing `Err(` expression
    /// - There is NO `return Ok(` or trailing `Ok(` expression
    /// - There is NO `match` statement (match arms can produce different results)
    /// - There is NO `.await` call (async results can succeed)
    /// - There is NO `if`/`else` branching that could produce Ok
    fn analyze_always_err(&self, content: &str, file_path: &Path) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut in_function = false;
        let mut function_start = 0;
        let mut brace_count = 0;
        let mut function_name = String::new();
        let mut function_body: Vec<&str> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if !in_function
                && (trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("pub async fn ")
                    || trimmed.starts_with("async fn "))
            {
                in_function = true;
                function_start = i;
                brace_count = 0;
                function_body.clear();
                function_name = trimmed
                    .split(['(', '{'])
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();
            }

            if in_function {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                // Collect non-comment body lines
                if !trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.is_empty() {
                    function_body.push(trimmed);
                }

                // Function ends
                if brace_count == 0 && i > function_start {
                    // Check for explicit Ok/Err returns and trailing expressions
                    let has_return_ok = function_body.iter().any(|l| l.contains("return Ok"));
                    let has_return_err = function_body.iter().any(|l| l.contains("return Err"));
                    let has_trailing_ok = function_body.iter().any(|l| {
                        l.starts_with("Ok(") || l.starts_with("Ok (")
                    });
                    let has_trailing_err = function_body.iter().any(|l| {
                        l.starts_with("Err(") || l.starts_with("Err (")
                    });

                    // Exclude functions with complex logic that could produce Ok
                    let has_match = function_body.iter().any(|l| {
                        l.contains("match ") || l.starts_with("match")
                    });
                    let has_await = function_body.iter().any(|l| l.contains(".await"));
                    let has_if_branch = function_body.iter().any(|l| {
                        l.starts_with("if ") || l.starts_with("if(")
                    });

                    let has_any_ok = has_return_ok || has_trailing_ok;
                    let has_any_err = has_return_err || has_trailing_err;

                    // Only flag if: has Err but no Ok, and no complex logic
                    let is_always_err = has_any_err
                        && !has_any_ok
                        && !has_match
                        && !has_await
                        && !has_if_branch;

                    if is_always_err {
                        issues.push(CodeIssue {
                            file_path: file_path.to_path_buf(),
                            line_number: function_start + 1,
                            issue_type: IssueType::AlwaysErr,
                            description: format!(
                                "Function '{}' only returns Err - always fails",
                                function_name
                            ),
                        });
                    }

                    in_function = false;
                }
            }
        }

        issues
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
                    .split(['(', '{'])
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

                    // Check for immediate return with placeholder (only if no real work was done)
                    if !has_real_work && func_content.contains("->") {
                        let return_match = lines[function_start..=i]
                            .iter()
                            .find(|l| l.contains("return") || l.contains("->"));

                        if let Some(ret_line) = return_match
                            && (ret_line.contains("None")
                                || ret_line.contains("todo!")
                                || ret_line.contains("unimplemented")
                                || ret_line.contains("Default"))
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

    /// Cross-file analysis: detect `pub fn` declarations that are never called anywhere
    /// in the codebase.
    ///
    /// Architecture requirement (30-Testing-and-Validation-Architecture.md §30.7):
    /// "Validation must confirm: it is called".
    ///
    /// This collects all public function names, then searches all file contents for
    /// call sites. A pub fn with zero call sites (excluding its own definition) is flagged.
    fn analyze_never_called(
        &self,
        file_contents: &[(PathBuf, String)],
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Phase 1: Collect all pub fn names and their locations
        let mut pub_fns: Vec<(PathBuf, usize, String)> = Vec::new();
        for (file_path, content) in file_contents {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("pub async fn ")
                {
                    let name = trimmed
                        .split(['(', '{', '<'])
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() && name != "fn" {
                        pub_fns.push((file_path.clone(), i + 1, name));
                    }
                }
            }
        }

        // Phase 2: For each pub fn, search all files for call sites
        // A call site is `name(` appearing in a non-definition context
        for (def_path, def_line, fn_name) in &pub_fns {
            let call_pattern = format!("{}(", fn_name);
            let method_pattern = format!(".{}(", fn_name);

            let mut call_count = 0;
            for (file_path, content) in file_contents {
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    // Skip the definition line itself
                    if file_path == def_path && i + 1 == *def_line {
                        continue;
                    }
                    // Skip comments
                    if trimmed.starts_with("//") || trimmed.starts_with("///") {
                        continue;
                    }
                    // Count direct calls (name() or name(args)) and method calls (.name())
                    if contains_word(line, fn_name) {
                        // Check if it's actually a call (followed by '(' or part of .name()
                        if line.contains(&call_pattern) || line.contains(&method_pattern) {
                            call_count += 1;
                        }
                    }
                }
            }

            if call_count == 0 {
                issues.push(CodeIssue {
                    file_path: def_path.clone(),
                    line_number: *def_line,
                    issue_type: IssueType::PublicNeverCalled,
                    description: format!(
                        "Public function '{}' is never called anywhere in the codebase",
                        fn_name
                    ),
                });
            }
        }

        issues
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
        crate::teeprintln!("  Source: {}", self.source_path.display());
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

/// Extract the imported name(s) from a `use` statement.
///
/// Handles common forms:
/// - `use foo::bar;` → `["bar"]`
/// - `use foo::{bar, baz};` → `["bar", "baz"]`
/// - `use foo::*;` → `["*"]`
/// - `use foo as bar;` → `["bar"]`
/// - `use foo::bar as baz;` → `["baz"]`
fn extract_imported_names(use_stmt: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Remove the leading "use " or "pub use " and trailing semicolon
    let stmt = use_stmt.trim();
    let stmt = stmt.strip_prefix("pub ").unwrap_or(stmt);
    let stmt = stmt.strip_prefix("use ").unwrap_or(stmt);
    let stmt = stmt.trim_end_matches(';').trim();

    // Handle grouped imports FIRST: `foo::{bar, baz as qux}`
    // This must be checked before the top-level `as` handler, because
    // `as` can appear inside a group.
    if let Some(brace_pos) = stmt.rfind('{') {
        let group = &stmt[brace_pos + 1..];
        let group = group.trim_end_matches('}');
        for part in group.split(',') {
            let part = part.trim();
            // Handle `as` within groups
            if let Some(as_pos) = part.rfind(" as ") {
                let alias = part[as_pos + 4..].trim();
                if !alias.is_empty() {
                    names.push(alias.to_string());
                }
            } else if !part.is_empty() {
                names.push(part.to_string());
            }
        }
        return names;
    }

    // Handle top-level `as` aliases: `use foo as bar;`
    if let Some(as_pos) = stmt.rfind(" as ") {
        let alias = stmt[as_pos + 4..].trim();
        if !alias.is_empty() {
            names.push(alias.to_string());
        }
        return names;
    }

    // Simple import: take the last segment after `::`
    let last_segment = stmt.rsplit("::").next().unwrap_or(stmt);
    let last_segment = last_segment.trim();
    if !last_segment.is_empty() {
        names.push(last_segment.to_string());
    }

    names
}

/// Check if a line contains a word-boundary match of the given name.
///
/// This avoids false positives where a name like "map" matches inside "mapping".
fn contains_word(line: &str, name: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = line[start..].find(name) {
        let abs_pos = start + pos;
        let before = abs_pos.checked_sub(1);
        let after_idx = abs_pos + name.len();
        let after_char = line.get(after_idx..after_idx + 1).and_then(|s| s.chars().next());

        let is_word_boundary_before = match before {
            None => true,
            Some(idx) => {
                let c = line.chars().nth(idx).unwrap_or(' ');
                !c.is_alphanumeric() && c != '_'
            }
        };
        let is_word_boundary_after = match after_char {
            None => true,
            Some(c) => !c.is_alphanumeric() && c != '_',
        };

        if is_word_boundary_before && is_word_boundary_after {
            return true;
        }

        start = abs_pos + name.len();
        if start >= line.len() {
            break;
        }
    }
    false
}

/// Check if an imported name is a known trait that's used via method syntax.
///
/// Trait imports (like `anyhow::Context`, `std::io::Read`) don't appear by name
/// in the file — they provide methods called via dot syntax (`.context()`,
/// `.read_to_string()`). This function maps known trait names to their method
/// patterns and checks if any appear in the file (excluding the import line).
fn is_trait_used_via_methods(name: &str, lines: &[&str], import_line: usize) -> bool {
    // Map trait names to method patterns they provide
    let methods: &[&str] = match name {
        // anyhow::Context
        "Context" => &[".context(", ".with_context(", ".context_err("],
        // std::io::Read
        "Read" => &[".read(", ".read_to_string(", ".read_to_end(", ".read_exact(", ".read_line(", ".bytes(", ".chars(", ".take("],
        // std::io::Write
        "Write" => &[".write(", ".write_all(", ".write_fmt(", ".write_str(", ".flush(", ".writeln("],
        // std::io::Seek
        "Seek" => &[".seek(", ".stream_position(", ".rewind("],
        // std::io::BufRead
        "BufRead" => &[".fill_buf(", ".consume(", ".read_line(", ".lines(", ".read_until("],
        // std::convert::TryFrom / TryInto
        "TryFrom" | "TryInto" => &[".try_into(", ".try_from("],
        // std::iter::FromIterator
        "FromIterator" => &[".from_iter(", "FromIterator::"],
        // std::iter::Extend
        "Extend" => &[".extend("],
        // chrono::Datelike
        "Datelike" => &[".year(", ".month(", ".day(", ".weekday(", ".iso_week(", ".num_days_from_", ".ordinal(", ".with_"],
        // chrono::Timelike
        "Timelike" => &[".hour(", ".minute(", ".second(", ".nanosecond(", ".with_hour(", ".with_minute("],
        // tokio::io::AsyncReadExt
        "AsyncReadExt" => &[".read(", ".read_buf(", ".read_exact(", ".read_to_end(", ".readable("],
        // tokio::io::AsyncWriteExt
        "AsyncWriteExt" => &[".write(", ".write_all(", ".write_buf(", ".flush(", ".writable("],
        // tokio::io::AsyncSeekExt
        "AsyncSeekExt" => &[".seek(", ".stream_position("],
        // tokio::io::AsyncBufReadExt
        "AsyncBufReadExt" => &[".read_line(", ".read_until(", ".fill_buf(", ".consume("],
        // candle_core::IndexOp (tensor indexing via .i())
        "IndexOp" => &[".i(", ".n(", ".r#", ".squeeze("],
        // serde::Deserialize (used in derive macros — checked separately)
        // These are handled by the contains_word check since they appear in #[derive(...)]
        _ => return false,
    };

    // Check if any method pattern appears in the file (excluding the import line)
    lines
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != import_line)
        .any(|(_, line)| {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                return false;
            }
            methods.iter().any(|m| line.contains(m))
        })
}
