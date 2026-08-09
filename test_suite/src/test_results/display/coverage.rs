//! Tool coverage display.
//! Renders the server-tools-vs-tested-tools diff so untested tools are
//! immediately visible in the report.

use super::super::TestReport;

impl TestReport {
    /// Print the tool coverage section (server tools vs tested tools).
    pub fn print_coverage(&self) {
        crate::teeprintln!("\n┌{:─<98}┐", "");
        crate::teeprintln!(
            "│ {:^96} │",
            "🔎 TOOL COVERAGE (server tools/list vs test registry)"
        );
        crate::teeprintln!("├{:─<98}┤", "");

        crate::teeprintln!(
            "│  Server tools exposed:  {:>68} │",
            self.coverage.server_tool_count()
        );
        crate::teeprintln!(
            "│  Registry tools tested: {:>68} │",
            self.coverage.tested_tool_count()
        );
        crate::teeprintln!(
            "│  Coverage:              {:>67.1}% │",
            self.coverage.coverage_percent()
        );
        crate::teeprintln!(
            "│  Untested server tools: {:>68} │",
            self.coverage.untested_count()
        );
        crate::teeprintln!(
            "│  Phantom (tested, not exposed): {:>49} │",
            self.coverage.phantom_count()
        );

        if self.coverage.has_gap() {
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│  ⚠️  UNTESTED SERVER TOOLS (no test requirement):");
            // Render in columns for readability.
            let joined = self.coverage.untested_tools.join(", ");
            for chunk in chunks(&joined, 92) {
                crate::teeprintln!("│    {}", chunk);
            }
        }

        if !self.coverage.phantom_tools.is_empty() {
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│  ℹ️  PHANTOM TESTS (registry entries with no matching server tool):");
            let joined = self.coverage.phantom_tools.join(", ");
            for chunk in chunks(&joined, 92) {
                crate::teeprintln!("│    {}", chunk);
            }
        }

        if !self.coverage.has_gap() && self.coverage.phantom_tools.is_empty() {
            crate::teeprintln!("├{:─<98}┤", "");
            crate::teeprintln!("│  ✅ Every server tool is covered by a test requirement");
        }

        crate::teeprintln!("└{:─<98}┘", "");
    }
}

/// Split `s` into lines of at most `max` chars, breaking on word boundaries.
fn chunks(s: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max {
            current.push(' ');
            current.push_str(word);
        } else {
            out.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}
