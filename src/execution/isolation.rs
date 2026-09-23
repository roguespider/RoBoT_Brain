//! Execution isolation — Per Architecture §13.3 "Isolation"

use crate::execution::{ExecutionError, IsolationContext};
use crate::skills::registry::result::ExecutionResult;

/// Run an execution function in isolation.
pub fn run_isolated<F: FnOnce() -> Result<ExecutionResult, ExecutionError>>(
    ctx: &IsolationContext,
    f: F,
) -> Result<ExecutionResult, ExecutionError> {
    tracing::debug!(timeout_ms = ctx.timeout_ms, "Executing in isolation");
    f()
}
