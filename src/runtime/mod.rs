//! Runtime — Per Architecture Chapter 3X (AI Runtime and Model Execution).
//!
//! The AI Runtime abstracts model execution, providing a replaceable backend
//! for inference. Currently supports Candle as the primary backend.

pub mod ai_runtime;
pub mod cache_manager;
pub mod device_manager;
pub mod diagnostics;
pub mod execution_scheduler;
pub mod model_manager;
pub mod model_registry;
pub mod resource_manager;
pub mod streaming;
pub mod tokenizer_manager;

// Re-export runtime types
pub use ai_runtime::AiRuntime;
pub use cache_manager::{CacheManager, CacheStats};
pub use device_manager::{DeviceManager, DeviceProperties, DeviceType};
pub use diagnostics::{DiagnosticReport, Diagnostics};
pub use execution_scheduler::ExecutionScheduler;
pub use model_manager::ModelManager;
pub use model_registry::ModelRegistry;
pub use resource_manager::ResourceManager;
pub use streaming::Streaming;
pub use tokenizer_manager::TokenizerManager;
