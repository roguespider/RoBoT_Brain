//! Code quality issues display.
//! Contains `print_code_issues` method.

use crate::code_analyzer::CodeIssue;

use super::super::TestReport;

impl TestReport {
    /// Print code issues table
    pub fn print_code_issues(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!("│ {:^96} │", "[WARN]  CODE QUALITY ISSUES DETECTED");
        crate::teeprintln!("├{:─<96}:┤", "─");
        crate::teeprintln!("│  The following issues were found in the source code:");
        crate::teeprintln!("│  These may indicate incomplete implementations or technical debt:");
        crate::teeprintln!("│{:─<97}│", "");

        // Group issues by type
        let mut issues_by_type: std::collections::HashMap<String, Vec<&CodeIssue>> =
            std::collections::HashMap::new();
        for issue in &self.code_issues {
            let key = issue.issue_type.to_string();
            issues_by_type.entry(key).or_default().push(issue);
        }

        for (issue_type, issues) in &issues_by_type {
            crate::teeprintln!("│");
            crate::teeprintln!("│  Issue Type: {}", issue_type);
            crate::teeprintln!("│  Count: {}", issues.len());
            let base_path = self.source_path.as_deref();
            crate::teeprintln!(
                "│  ├── Files affected: {}",
                issues
                    .iter()
                    .map(|i| {
                        
                        base_path
                            .map(|bp| i.relative_path(bp))
                            .unwrap_or_else(|| i.file_path.to_string_lossy().to_string())
                    })
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            );

            // Show ALL issues with full detail (no truncation)
            for (idx, issue) in issues.iter().enumerate() {
                let file_name = base_path
                    .map(|bp| issue.relative_path(bp))
                    .unwrap_or_else(|| issue.file_path.to_string_lossy().to_string());
                let prefix = if idx == issues.len() - 1 {
                    "└──"
                } else {
                    "├──"
                };
                crate::teeprintln!(
                    "│  {} {}:{} — {}",
                    prefix,
                    file_name,
                    issue.line_number,
                    issue.description
                );
            }
        }

        crate::teeprintln!("│{:─<97}│", "");
        crate::teeprintln!("└{:─<97}┘", "");
    }
}
