/// Shared contract version for all data contracts.
///
/// Every struct in this module implements the `Versioned` trait, which
/// provides a stable version string for serialization compatibility checks.

/// Current contract version string.
pub const CONTRACT_VERSION: &str = "v0.0.2.1";

/// Trait for types that carry a version identifier.
///
/// Implement this trait on any struct that needs version awareness
/// for serialization compatibility.
pub trait Versioned {
    /// Returns the contract version this type conforms to.
    fn version() -> &'static str {
        CONTRACT_VERSION
    }
}
