// src/cli/commands/diagnose.rs
//! Explicit subsystem diagnostics command (P2-001C).
//!
//! Builds the application, runs the full subsystem diagnostic suite, and
//! exits. Production startup never runs these probes; they are available
//! only through this explicit mechanism.

use crate::bridge::app::App;
use crate::cli::output;
use anyhow::Result;

pub fn run() -> Result<()> {
    output::section_header("RoBoT Subsystem Diagnostics");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result: Result<()> = runtime.block_on(async {
        let app = App::new().await?;
        crate::bridge::app::initialization::diagnostics::run_startup_diagnostics(&app).await;
        Ok(())
    });
    result?;

    output::success_msg("Subsystem diagnostics complete");
    Ok(())
}
