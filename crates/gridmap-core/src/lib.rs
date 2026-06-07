pub mod candidates;
pub mod detection;
pub mod features;
pub mod regions;
pub mod spatial;
pub mod store;
pub mod types;

/// Returns the crate version string.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
