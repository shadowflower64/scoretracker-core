/// Common directories used by scoretracker.
pub mod dirs;

/// Extra file input/output functions.
pub mod file_ex;

/// File lock handling.
pub mod lockfile;

/// Module with simple logging macros.
pub mod log;

/// Terminal ANSI formatting and color code constants.
pub mod terminal_colors;

/// Module for nanosecond timestamp struct: [`timestamp::NsTimestamp`].
pub mod timestamp;

/// Module for [`percentage::Percentage`] struct, for handling percentage values without floating point errors.
pub mod percentage;

/// Module for [`uuid::UuidString`], a (de)serializable wrapper for [`::uuid::Uuid`].
pub mod uuid;
