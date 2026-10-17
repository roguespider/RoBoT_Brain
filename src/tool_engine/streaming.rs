use serde_json::Value;

/// Streaming support for incremental tool output.
/// Per Architecture §13.27 (Tool Streaming): incremental output for long-running tools.
pub struct ToolStreamer {
    /// Buffer for streaming output.
    buffer: Vec<Value>,
}

impl ToolStreamer {
    /// Create a new streamer.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Emit a streaming chunk.
    pub fn emit(&mut self, chunk: Value) {
        self.buffer.push(chunk);
    }

    /// Get all buffered chunks.
    pub fn chunks(&self) -> &[Value] {
        &self.buffer
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get chunk count.
    pub fn chunk_count(&self) -> usize {
        self.buffer.len()
    }
}

/// Stream a tool result incrementally.
/// Per Architecture §13.27: streaming support for incremental output.
pub fn stream_result(streamer: &mut ToolStreamer, chunk: Value) -> Vec<Value> {
    streamer.emit(chunk);
    streamer.chunks().to_vec()
}
