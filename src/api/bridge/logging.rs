pub fn init_logging() {
    // For MCP stdio transport, we must NOT log to stdout.
    // In `diagnose` mode we log to stderr so the diagnostic output is visible;
    // otherwise logs go to a null writer (stdio protocol owns stdout).
    let diagnose_mode = std::env::args().nth(1).is_some_and(|arg| arg == "diagnose");
    if diagnose_mode {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .try_init()
            .ok();
        return;
    }
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::sink) // Discard tracing logs
        .with_ansi(false)
        .try_init() // Use try_init to avoid panic if rmcp/handler.rs already set one
        .ok(); // Silently ignore if subscriber is already configured

    // Print startup message to stderr so users know the server started
    eprintln!("RoBoT Brain MCP server starting...");
}
