//! Observability — Per Architecture Chapter 27 (Cognitive Monitoring and Observability).
//!
//! Provides visibility into the internal operation of RoBoT, including
//! cognitive tracing, metrics, events, timelines, and visualization.

pub mod analyzer;
pub mod events;
pub mod exporter;
pub mod metrics;
pub mod timeline;
pub mod tracing;
pub mod visualization;

pub use analyzer::Analyzer;
pub use events::{CategorizedEvent, CognitiveEvent, EventCategory};
pub use exporter::Exporter;
pub use metrics::{Metric, Metrics};
pub use timeline::{CognitiveTimeline, TimelineEntry};
pub use tracing::{CognitiveTrace, TraceSpan};
pub use visualization::{VizEdge, VizGraph, VizNode};
