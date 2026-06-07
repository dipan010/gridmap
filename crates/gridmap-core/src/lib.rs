pub mod candidates;
pub mod features;
pub mod spatial;
pub mod store;
pub mod types;

/// Returns the crate version string.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
