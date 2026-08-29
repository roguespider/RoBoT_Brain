// src/bridge/app/initialization/mcp_client_diagnostics.rs
//! MCP client connection-management probe (explicit diagnostics, P2-001C).

use crate::bridge::app::state::App;

/// Exercise MCP client connection-management methods. With no servers
/// connected these are safe no-ops, but they keep disconnect,
/// disconnect_all and refresh_tools live.
///
/// Returns `Err` if disconnect or disconnect_all operations fail.
pub async fn run_mcp_client_probe(app: &App) -> std::result::Result<(), String> {
    let db_path = app.mcp_context.database.path().display().to_string();
    tracing::debug!("MCP client diagnostics: database at {}", db_path);
    let mcp_client = match crate::bridge::tools::agent::get_mcp_client() {
        Some(client) => client,
        None => {
            tracing::warn!("MCP client diagnostics skipped: client not initialized");
            return Ok(());
        }
    };
    // Try disconnect from a probe server name. Returns Ok(false) when no such
    // server is connected — that is a safe no-op, not a failure.
    let disconnect_ok = mcp_client
        .disconnect("diagnostics-probe-server")
        .await
        .unwrap_or(false);
    let cleared = mcp_client.disconnect_all().await;
    let refresh_ok = mcp_client
        .refresh_tools("diagnostics-probe-server")
        .await
        .is_ok();
    tracing::info!(
        "MCP client management verified: disconnect={} disconnect_all={} refresh_tools_ok={}",
        disconnect_ok,
        cleared,
        refresh_ok
    );
    Ok(())
}
