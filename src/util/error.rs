use std::{num::TryFromIntError, time::SystemTimeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemTimeConversionError {
    #[error("system time error: {0}")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("try from int error: {0}")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("out of range")]
    OutOfRange,
}

pub const UNSUPPORTED_TIMESTAMP_MESSAGE: &str = "cannot handle timestamps earlier than 1970-01-01";
