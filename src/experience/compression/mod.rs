// src/experience/compression/mod.rs
//! Experience Compression module
//!
//! Compresses multiple similar experiences into a single compressed representation:
//! - Pattern: The common elements across experiences
//! - Confidence: Aggregated confidence level
//! - Exceptions: Cases that don't fit the pattern

pub mod compressor;
pub mod pattern;
pub mod exceptions;

// Allow dead code for public API exports
