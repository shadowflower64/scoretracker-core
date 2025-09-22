pub mod game;
pub mod hive;
pub mod library;
pub mod play;
pub mod tests;
pub mod util;

/// Current version of scoretracker-core, read from `CARGO_PKG_VERSION`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn example_add(left: u64, right: u64) -> u64 {
    left + right
}
