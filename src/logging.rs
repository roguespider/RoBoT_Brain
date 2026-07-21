pub fn init_logging() {
    // Initialize tracing that outputs to stderr only
    // This prevents polluting stdout which is used for MCP stdio transport
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stderr)
        .init();
}
