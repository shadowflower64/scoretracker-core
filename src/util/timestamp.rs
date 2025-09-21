use crate::util::error::SystemTimeConversionError;
use chrono::{DateTime, Local, SecondsFormat, TimeZone, Utc};
use serde::{Deserialize, Serialize, de::Visitor};
use std::fmt;
use std::num::TryFromIntError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Timestamp expressed as nanoseconds since [`UNIX_EPOCH`].
///
/// This type uses [`i128`] internally - it allows for both negative and positive numbers.
/// However, most of Rust's timestamp structures seem to be incapable of storing timestamps before [`UNIX_EPOCH`],
/// so many of the conversion methods between this type and Rust's types may fail.
///
/// # Serialization and deserialization
/// This type is serialized as [`i128`] when used with serde.
///
/// This type can be deserialized from any integer value, although if the integer is larger than [`i128::MAX`] then the conversion will fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NsTimestamp(i128);

impl NsTimestamp {
    /// Create a new timestamp based on [`SystemTime::now`].
    pub fn now() -> Self {
        SystemTime::now().try_into().unwrap()
    }

    /// Get the amount of seconds since [`UNIX_EPOCH`].
    pub fn as_secs(self) -> i128 {
        self.0 / 1_000_000_000i128
    }

    /// Get the amount of milliseconds since [`UNIX_EPOCH`].
    pub fn as_millis(self) -> i128 {
        self.0 / 1_000_000i128
    }

    /// Get the amount of microseconds since [`UNIX_EPOCH`].
    pub fn as_micros(self) -> i128 {
        self.0 / 1_000i128
    }

    /// Get the amount of nanoseconds since [`UNIX_EPOCH`].
    pub fn as_nanos(self) -> i128 {
        self.0
    }

    /// Get the duration of time that has passed since [`UNIX_EPOCH`].
    ///
    /// # Panics
    /// This function will panic if the conversion fails.
    /// See the implementation of [`NsTimestamp::try_into<Duration>`] for details.
    pub fn as_duration(self) -> Duration {
        self.try_into().unwrap()
    }

    /// Convert the timestamp as a [`SystemTime`] struct.
    ///
    /// # Panics
    /// This function will panic if the conversion fails.
    /// See the implementation of [`NsTimestamp::try_into<SystemTime>`] for details.
    pub fn as_system_time(self) -> SystemTime {
        self.try_into().unwrap()
    }

    /// Convert the timestamp as a [`DateTime`] struct.
    ///
    /// # Panics
    /// This function will panic if the conversion to [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::as_system_time`] for details.
    pub fn as_date_time<Tz: TimeZone>(self) -> DateTime<Tz>
    where
        DateTime<Tz>: From<SystemTime>,
    {
        self.as_system_time().into()
    }

    /// Convert the timestamp as a [`DateTime<Utc>`] struct.
    ///
    /// # Panics
    /// This function will panic if the conversion to [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::as_system_time`] for details.
    pub fn as_date_time_utc(self) -> DateTime<Utc> {
        self.as_date_time()
    }

    /// Convert the timestamp as a [`DateTime<Local>`] struct.
    ///
    /// # Panics
    /// This function will panic if the conversion to [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::as_system_time`] for details.
    pub fn as_date_time_local(self) -> DateTime<Local> {
        self.as_date_time()
    }

    /// Create a [`NsTimestamp`] from the amount of seconds since [`UNIX_EPOCH`].
    pub fn from_secs(secs: i64) -> Self {
        Self((secs as i128) * 1_000_000_000i128)
    }

    /// Create a [`NsTimestamp`] from the amount of milliseconds since [`UNIX_EPOCH`].
    pub fn from_millis(millis: i64) -> Self {
        Self((millis as i128) * 1_000_000i128)
    }

    /// Create a [`NsTimestamp`] from the amount of microseconds since [`UNIX_EPOCH`].
    pub fn from_micros(micros: i64) -> Self {
        Self((micros as i128) * 1_000i128)
    }

    /// Create a [`NsTimestamp`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    pub fn from_nanos(nanos: i128) -> Self {
        Self(nanos)
    }

    /// Create a [`NsTimestamp`] from the duration of time since [`UNIX_EPOCH`].
    ///
    /// # Panics
    /// This function will panic if the conversion to [`Duration`] fails.
    /// See the implementation of [`NsTimestamp::try_from<Duration>`] for details.
    pub fn from_duration(duration: Duration) -> Self {
        duration.try_into().unwrap()
    }

    /// Create a [`NsTimestamp`] from [`SystemTime`].
    ///
    /// # Panics
    /// This function will panic if the conversion from [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::try_from<SystemTime>`] for details.
    pub fn from_system_time(system_time: SystemTime) -> Self {
        system_time.try_into().unwrap()
    }

    /// Create a [`NsTimestamp`] from [`DateTime`].
    ///
    /// # Panics
    /// This function will panic if the conversion from [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::from_system_time`] for details.
    pub fn from_date_time<Tz: TimeZone>(date_time: DateTime<Tz>) -> Self
    where
        DateTime<Tz>: Into<SystemTime>,
    {
        let system_time = date_time.into();
        Self::from_system_time(system_time)
    }

    /// Create a [`NsTimestamp`] from [`DateTime<Utc>`].
    ///
    /// # Panics
    /// This function will panic if the conversion from [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::from_date_time`] for details.
    pub fn from_date_time_utc(date_time: DateTime<Utc>) -> Self {
        Self::from_date_time(date_time)
    }

    /// Create a [`NsTimestamp`] from [`DateTime<Local>`].
    ///
    /// # Panics
    /// This function will panic if the conversion from [`SystemTime`] fails.
    /// See the implementation of [`NsTimestamp::from_date_time`] for details.
    pub fn from_date_time_local(date_time: DateTime<Local>) -> Self {
        Self::from_date_time(date_time)
    }

    pub fn to_date_time_string_utc(self) -> String {
        self.as_date_time_utc().to_rfc3339_opts(SecondsFormat::Nanos, true)
    }

    pub fn to_date_time_string_local(self) -> String {
        self.as_date_time_local().to_rfc3339_opts(SecondsFormat::Nanos, false)
    }
}

impl fmt::Display for NsTimestamp {
    /// Display a [`NsTimestamp`] as a UTC datetime string, and the amount of nanoseconds since [`UNIX_EPOCH`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_date_time_string_local(), self.0)
    }
}

impl From<i128> for NsTimestamp {
    /// Create a [`NsTimestamp`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    fn from(value: i128) -> Self {
        NsTimestamp(value)
    }
}

impl TryFrom<u128> for NsTimestamp {
    type Error = TryFromIntError;

    /// Try to convert a [`u128`] into a [`NsTimestamp`].
    ///
    /// # Errors
    /// This function will return a [`TryFromIntError`] if the the duration of time since the [`UNIX_EPOCH`] in nanoseconds is larger than [`i128::MAX`].
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let signed: i128 = value.try_into()?;
        Ok(signed.into())
    }
}

impl TryFrom<Duration> for NsTimestamp {
    type Error = TryFromIntError;

    /// Try to convert a [`Duration`] into a [`NsTimestamp`].
    ///
    /// # Errors
    /// This function will return a [`TryFromIntError`] if the the duration of time since the [`UNIX_EPOCH`] in nanoseconds is larger than [`i128::MAX`].
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        value.as_nanos().try_into()
    }
}

impl TryFrom<SystemTime> for NsTimestamp {
    type Error = SystemTimeConversionError;

    /// Try to convert a [`SystemTime`] into a [`NsTimestamp`].
    ///
    /// # Errors
    /// This function will return a [`SystemTimeConversionError`] if either:
    /// - the [`SystemTime`] represents a value that is earlier than the [`UNIX_EPOCH`], or
    /// - the duration of time since the [`UNIX_EPOCH`] in nanoseconds is larger than [`i128::MAX`].
    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        Ok(value.duration_since(UNIX_EPOCH)?.try_into()?)
    }
}

impl TryInto<Duration> for NsTimestamp {
    type Error = TryFromIntError;

    /// Try to convert a [`NsTimestamp`] into a [`Duration`].
    ///
    /// # Errors
    /// This function will return a [`TryFromIntError`] if either:
    /// - the amount of nanoseconds in [`self`] is negative, or
    /// - the amount of seconds in [`self`] is over [`u64::MAX`].
    fn try_into(self) -> Result<Duration, Self::Error> {
        let secs = self.0.checked_div(1_000_000_000i128).expect("this value should never overflow");
        let nanos = self.0.checked_rem(1_000_000_000i128).expect("this value should never overflow");
        let duration = Duration::new(
            secs.try_into()?,  // This fails for insanely large values of `seconds` (over [`u64::MAX`]).
            nanos.try_into()?, // TODO: This fails for negative values of `nanos`, which occur as a result of negative values of `self.0`.
        );
        Ok(duration)
    }
}

impl TryInto<SystemTime> for NsTimestamp {
    type Error = SystemTimeConversionError;

    /// Try to convert a [`NsTimestamp`] into a [`SystemTime`].
    ///
    /// # Errors
    /// This function will return a [`SystemTimeConversionError`] if:
    /// - the amount of nanoseconds in [`self`] is negative,
    /// - the amount of seconds in [`self`] is over [`u64::MAX`], or
    /// - the timestamp is out of range of [`SystemTime`] (when `SystemTime::UNIX_EPOCH.checked_add(duration)` fails.)
    fn try_into(self) -> Result<SystemTime, Self::Error> {
        let duration = self.try_into()?;
        let system_time = SystemTime::UNIX_EPOCH
            .checked_add(duration)
            .ok_or(SystemTimeConversionError::OutOfRange)?;
        Ok(system_time)
    }
}

impl Serialize for NsTimestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i128(self.0)
    }
}

struct NanosecondTimestampVisitor;

impl<'de> Visitor<'de> for NanosecondTimestampVisitor {
    type Value = NsTimestamp;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a number of nanoseconds since 1970-01-01 (can be negative)")
    }

    fn visit_i8<E: serde::de::Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i16<E: serde::de::Error>(self, v: i16) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        self.visit_i128(v as i128)
    }

    fn visit_i128<E: serde::de::Error>(self, v: i128) -> Result<Self::Value, E> {
        Ok(NsTimestamp(v))
    }

    fn visit_u8<E: serde::de::Error>(self, v: u8) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        self.visit_u128(v as u128)
    }

    fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Self::Value, E> {
        Ok(NsTimestamp(
            v.try_into().map_err(|e| E::custom(format!("u128 does not fit in i128: {e:?}")))?,
        ))
    }
}

impl<'de> Deserialize<'de> for NsTimestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_i128(NanosecondTimestampVisitor)
    }
}
