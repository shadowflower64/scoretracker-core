use std::{num::TryFromIntError, time::SystemTimeError};

#[derive(Debug)]
pub enum SystemTimeConversionError {
    SystemTimeError(SystemTimeError),
    TryFromIntError(TryFromIntError),
    OutOfRange,
}

impl From<SystemTimeError> for SystemTimeConversionError {
    fn from(value: SystemTimeError) -> Self {
        Self::SystemTimeError(value)
    }
}

impl From<TryFromIntError> for SystemTimeConversionError {
    fn from(value: TryFromIntError) -> Self {
        Self::TryFromIntError(value)
    }
}

pub const UNSUPPORTED_TIMESTAMP_MESSAGE: &str = "cannot handle timestamps earlier than 1970-01-01";
