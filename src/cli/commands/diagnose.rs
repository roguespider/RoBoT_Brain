// src/cli/commands/diagnose.rs
//! Explicit subsystem diagnostics command (P2-001C).
//!
//! Production startup never runs these probes; they are available only
//! through the explicit `robot diagnose` entry point, which is dispatched
//! from `main.rs` inside the existing tokio runtime. This module documents
//! the command and prints usage when invoked through the sync CLI path.

use crate::cli::output;
use anyhow::Result;

pub fn run() -> Result<()> {
    output::section_header("RoBoT Subsystem Diagnostics");
    output::info_msg(
        "Diagnostics must be invoked as `robot diagnose` (async entry point in main.rs).",
    );
    Ok(())
}
