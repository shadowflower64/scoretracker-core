use chrono::{DateTime, Local, SecondsFormat, TimeZone, Utc};
use serde::{Deserialize, Serialize, de::Visitor};
use std::fmt;
use std::ops::{Add, Sub};
use std::time::SystemTimeError;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("system time conversion error: {0}")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("out of range of SystemTime type")]
    OutOfSystemTimeRange,
    #[error("out of range of Duration type")]
    OutOfDurationRange,
    #[error("out of range")]
    OutOfRange,
}

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
    pub const UNIX_EPOCH: NsTimestamp = NsTimestamp(0);

    /// Create a new timestamp based on [`SystemTime::now`].
    ///
    /// # Example
    /// ```
    /// use std::time::{SystemTime, Duration};
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// let now_system_time = SystemTime::now();
    /// let now_ns = NsTimestamp::now();
    /// let now_ns_as_system_time: SystemTime = now_ns.try_into().unwrap();
    /// let difference = now_ns_as_system_time.duration_since(now_system_time).unwrap();
    /// assert!(difference < Duration::from_secs(1));
    /// ```
    pub fn now() -> Self {
        SystemTime::now().into()
    }

    /// Get the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000_000_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" second is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the second that *has already passed or is currently passing*,
    /// and will never give you a second that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range                 | `.as_secs()` result |
    /// | -------------------------------- | ------------------- |
    /// | `-3_000_000_000..-2_000_000_001` |  `-3`               |
    /// | `-2_000_000_000..-1_000_000_001` |  `-2`               |
    /// | `-1_000_000_000..-1`             |  `-1`               |
    /// | `0..999_999_999`                 |  `0`                |
    /// | `1_000_000_000..1_999_999_999`   |  `1`                |
    /// | `2_000_000_000..2_999_999_999`   |  `2`                |
    /// | `3_000_000_000..3_999_999_999`   |  `3`                |
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// assert_eq!(NsTimestamp::from_nanos(-3_000_000_000).as_secs(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_999_999_999).as_secs(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_000_000_001).as_secs(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_000_000_000).as_secs(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_999_999_999).as_secs(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_000_000_001).as_secs(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_000_000_000).as_secs(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-999_999_999).as_secs(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-1).as_secs(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(0).as_secs(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1).as_secs(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(999_999_999).as_secs(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1_000_000_000).as_secs(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_000_000_001).as_secs(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_999_999_999).as_secs(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(2_000_000_000).as_secs(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_000_000_001).as_secs(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_999_999_999).as_secs(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(3_000_000_000).as_secs(), 3);
    /// ```
    pub fn as_secs(self) -> i128 {
        self.0.div_euclid(1_000_000_000i128)
    }

    /// Get the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" millisecond is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the millisecond that *has already passed or is currently passing*,
    /// and will never give you a millisecond that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range         | `.as_millis()` result |
    /// | -------------------------| --------------------- |
    /// | `-3_000_000..-2_000_001` |  `-3`                 |
    /// | `-2_000_000..-1_000_001` |  `-2`                 |
    /// | `-1_000_000..-1`         |  `-1`                 |
    /// | `0..999_999`             |  `0`                  |
    /// | `1_000_000..1_999_999`   |  `1`                  |
    /// | `2_000_000..2_999_999`   |  `2`                  |
    /// | `3_000_000..3_999_999`   |  `3`                  |
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// assert_eq!(NsTimestamp::from_nanos(-3_000_000).as_millis(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_999_999).as_millis(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_000_001).as_millis(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_000_000).as_millis(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_999_999).as_millis(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_000_001).as_millis(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_000_000).as_millis(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-999_999).as_millis(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-1).as_millis(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(0).as_millis(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1).as_millis(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(999_999).as_millis(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1_000_000).as_millis(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_000_001).as_millis(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_999_999).as_millis(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(2_000_000).as_millis(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_000_001).as_millis(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_999_999).as_millis(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(3_000_000).as_millis(), 3);
    /// ```
    pub fn as_millis(self) -> i128 {
        self.0.div_euclid(1_000_000i128)
    }

    /// Get the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// This uses [`i128::div_euclid`] to divide the number of nanoseconds by `1_000i128`, which means it will always round down, towards `-Infinity`.
    /// It doesn't use the default dividing method, which rounds towards zero, because that would mean that the "zeroth" microsecond is twice as long as all other ones.
    ///
    /// In short, this means that this function gives you the index of the microsecond that *has already passed or is currently passing*,
    /// and will never give you a microsecond that is in the future.
    ///
    /// # Example values
    /// | Nanosecond range | `.as_micros()` result |
    /// | -----------------| --------------------- |
    /// | `-3_000..-2_001` |  `-3`                 |
    /// | `-2_000..-1_001` |  `-2`                 |
    /// | `-1_000..-1`     |  `-1`                 |
    /// | `0..999`         |  `0`                  |
    /// | `1_000..1_999`   |  `1`                  |
    /// | `2_000..2_999`   |  `2`                  |
    /// | `3_000..3_999`   |  `3`                  |
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// assert_eq!(NsTimestamp::from_nanos(-3_000).as_micros(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_999).as_micros(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_001).as_micros(), -3);
    /// assert_eq!(NsTimestamp::from_nanos(-2_000).as_micros(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_999).as_micros(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_001).as_micros(), -2);
    /// assert_eq!(NsTimestamp::from_nanos(-1_000).as_micros(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-999).as_micros(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(-1).as_micros(), -1);
    /// assert_eq!(NsTimestamp::from_nanos(0).as_micros(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1).as_micros(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(999).as_micros(), 0);
    /// assert_eq!(NsTimestamp::from_nanos(1_000).as_micros(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_001).as_micros(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(1_999).as_micros(), 1);
    /// assert_eq!(NsTimestamp::from_nanos(2_000).as_micros(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_001).as_micros(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(2_999).as_micros(), 2);
    /// assert_eq!(NsTimestamp::from_nanos(3_000).as_micros(), 3);
    /// ```
    pub fn as_micros(self) -> i128 {
        self.0.div_euclid(1_000i128)
    }

    /// Get the amount of nanoseconds since [`UNIX_EPOCH`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// assert_eq!(NsTimestamp::from_nanos(1234).as_nanos(), 1234);
    /// ```
    pub fn as_nanos(self) -> i128 {
        self.0
    }

    /// Create a [`NsTimestamp`] from the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given second.
    /// For example, for the "zeroth" second, the resulting timestamp is `0` nanoseconds,
    /// for the "first" second, the resulting timestamp is `1_000_000_000` nanoseconds,
    /// for the "negative first" (-1st) second, the resulting timestamp is `-1_000_000_000` nanoseconds.
    ///
    /// Since this function takes the number of seconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    pub fn from_secs(secs: i64) -> Self {
        Self((secs as i128) * 1_000_000_000i128)
    }

    /// Try to create a [`NsTimestamp`] from the amount of seconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given second.
    /// For example, for the "zeroth" second, the resulting timestamp is `0` nanoseconds,
    /// for the "first" second, the resulting timestamp is `1_000_000_000` nanoseconds,
    /// for the "negative first" (-1st) second, the resulting timestamp is `-1_000_000_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `secs` by `1_000_000_000` overflows [`i128`].
    pub fn try_from_secs(secs: i128) -> Result<Self, Error> {
        Ok(Self(secs.checked_mul(1_000_000_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create a [`NsTimestamp`] from the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given millisecond.
    /// For example, for the "zeroth" millisecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" millisecond, the resulting timestamp is `1_000_000` nanoseconds,
    /// for the "negative first" (-1st) millisecond, the resulting timestamp is `-1_000_000` nanoseconds.
    ///
    /// Since this function takes the number of milliseconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    pub fn from_millis(millis: i64) -> Self {
        Self((millis as i128) * 1_000_000i128)
    }

    /// Try to create a [`NsTimestamp`] from the amount of milliseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given millisecond.
    /// For example, for the "zeroth" millisecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" millisecond, the resulting timestamp is `1_000_000` nanoseconds,
    /// for the "negative first" (-1st) millisecond, the resulting timestamp is `-1_000_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `millis` by `1_000_000` overflows [`i128`].
    pub fn try_from_millis(millis: i128) -> Result<Self, Error> {
        Ok(Self(millis.checked_mul(1_000_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create a [`NsTimestamp`] from the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given microsecond.
    /// For example, for the "zeroth" microsecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" microsecond, the resulting timestamp is `1_000` nanoseconds,
    /// for the "negative first" (-1st) microsecond, the resulting timestamp is `-1_000` nanoseconds.
    ///
    /// Since this function takes the number of microseconds as an [`i64`], this function will never fail,
    /// as the result of the multiplication always fits within a [`i128`].
    pub fn from_micros(micros: i64) -> Self {
        Self((micros as i128) * 1_000i128)
    }

    /// Try to create a [`NsTimestamp`] from the amount of microseconds since [`UNIX_EPOCH`].
    ///
    /// The timestamp points to the beginning of the given microsecond.
    /// For example, for the "zeroth" microsecond, the resulting timestamp is `0` nanoseconds,
    /// for the "first" microsecond, the resulting timestamp is `1_000` nanoseconds,
    /// for the "negative first" (-1st) microsecond, the resulting timestamp is `-1_000` nanoseconds.
    ///
    /// # Errors
    /// This function will return an [`Error::OutOfRange`], if the result of multiplying `micros` by `1_000` overflows [`i128`].
    pub fn try_from_micros(micros: i128) -> Result<Self, Error> {
        Ok(Self(micros.checked_mul(1_000i128).ok_or(Error::OutOfRange)?))
    }

    /// Create a [`NsTimestamp`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    pub fn from_nanos(nanos: i128) -> Self {
        Self(nanos)
    }

    pub fn to_date_time_string_utc(self) -> String {
        let date_time: DateTime<Utc> = self.try_into().unwrap();
        date_time.to_rfc3339_opts(SecondsFormat::Nanos, true)
    }

    pub fn to_date_time_string_local(self) -> String {
        let date_time: DateTime<Local> = self.try_into().unwrap();
        date_time.to_rfc3339_opts(SecondsFormat::Nanos, false)
    }

    /// Returns the timestamp opposite to the provided origin.
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    ///
    /// let origin = NsTimestamp::from_secs(5);
    /// let two_seconds_later = origin + 2_000_000_000;
    /// let two_seconds_earlier = two_seconds_later.invert_with_origin(origin);
    /// assert_eq!(two_seconds_earlier, NsTimestamp::from_secs(3));
    ///
    /// let twelfth_second_since_epoch = NsTimestamp::from_secs(12);
    /// let twelfth_second_before_epoch = twelfth_second_since_epoch.invert_with_origin(NsTimestamp::UNIX_EPOCH);
    /// assert_eq!(twelfth_second_before_epoch, NsTimestamp::from_secs(-12));
    /// ```
    pub fn invert_with_origin(self, origin: Self) -> Self {
        let duration_since_origin = self.0 - origin.0;
        Self(origin.0 - duration_since_origin)
    }
}

impl fmt::Display for NsTimestamp {
    /// Display a [`NsTimestamp`] as a UTC datetime string, and the amount of nanoseconds since [`UNIX_EPOCH`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_date_time_string_local(), self.0)
    }
}

impl Add<i128> for NsTimestamp {
    type Output = Self;
    fn add(self, rhs: i128) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl Sub<i128> for NsTimestamp {
    type Output = Self;
    fn sub(self, rhs: i128) -> Self::Output {
        Self(self.0.sub(rhs))
    }
}

impl Sub for NsTimestamp {
    type Output = i128;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0.sub(rhs.0)
    }
}

impl From<i128> for NsTimestamp {
    /// Create a [`NsTimestamp`] from the amount of nanoseconds since [`UNIX_EPOCH`].
    fn from(value: i128) -> Self {
        NsTimestamp(value)
    }
}

impl TryFrom<u128> for NsTimestamp {
    type Error = Error;

    /// Try to convert a [`u128`] into a [`NsTimestamp`].
    ///
    /// # Errors
    /// This function will return a [`TryFromIntError`] if the the duration of time since the [`UNIX_EPOCH`] in nanoseconds is larger than [`i128::MAX`].
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let signed: i128 = value.try_into().ok().ok_or(Error::OutOfRange)?;
        Ok(signed.into())
    }
}

impl From<Duration> for NsTimestamp {
    /// Convert a [`Duration`] into a [`NsTimestamp`].
    fn from(value: Duration) -> Self {
        value.as_nanos().try_into().expect("duration does not fit in i128")
    }
}

impl From<SystemTime> for NsTimestamp {
    /// Convert a [`SystemTime`] into a [`NsTimestamp`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::NsTimestamp;
    /// use std::time::{SystemTime, UNIX_EPOCH, Duration};
    ///
    /// let system_time = SystemTime::now();
    /// let ns_timestamp = NsTimestamp::from(system_time);
    /// let duration = system_time.duration_since(UNIX_EPOCH).unwrap();
    /// assert_eq!(duration.as_nanos() as i128, ns_timestamp.as_nanos());
    ///
    /// let system_time = UNIX_EPOCH;
    /// let ns_timestamp = NsTimestamp::from(system_time);
    /// let duration = system_time.duration_since(UNIX_EPOCH).unwrap();
    /// assert_eq!(duration.as_nanos() as i128, ns_timestamp.as_nanos());
    /// assert_eq!(0, ns_timestamp.as_nanos());
    ///
    /// let system_time = UNIX_EPOCH - Duration::from_secs(5);
    /// let ns_timestamp = NsTimestamp::from(system_time);
    /// let negative_duration = UNIX_EPOCH.duration_since(system_time).unwrap();
    /// assert_eq!(-(negative_duration.as_nanos() as i128), ns_timestamp.as_nanos());
    /// ```
    fn from(value: SystemTime) -> Self {
        match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                // Provided SystemTime is later than or equal to UNIX_EPOCH
                duration.into()
            }
            Err(e) => {
                // Provided SystemTime is earlier than UNIX_EPOCH
                let negative_duration = e.duration();
                NsTimestamp::from(negative_duration).invert_with_origin(NsTimestamp::UNIX_EPOCH)
            }
        }
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for NsTimestamp {
    /// Convert a [`DateTime<Tz>`] into a [`NsTimestamp`].
    fn from(value: DateTime<Tz>) -> Self {
        let system_time = SystemTime::from(value);
        system_time.into()
    }
}

impl TryFrom<NsTimestamp> for Duration {
    type Error = Error;

    /// Try to convert a [`NsTimestamp`] into a [`Duration`].
    ///
    /// # Errors
    /// This function will return an error if the value is out of range of the [`Duration`] structure. This happens in two cases:
    /// - if the amount of nanoseconds in [`self`] is negative,
    /// - if the amount of seconds in [`self`] is over [`u64::MAX`].
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::{NsTimestamp, Error};
    /// use std::time::Duration;
    ///
    /// let ns_timestamp = NsTimestamp::now();
    /// let duration = Duration::try_from(ns_timestamp);
    /// assert!(matches!(duration, Ok(_)));
    ///
    /// let ns_timestamp = NsTimestamp::from_nanos(-1);
    /// let duration = Duration::try_from(ns_timestamp);
    /// assert!(matches!(duration, Err(Error::OutOfDurationRange)));
    ///
    /// let ns_timestamp = NsTimestamp::try_from_secs(u64::MAX as i128).unwrap();
    /// let duration = Duration::try_from(ns_timestamp);
    /// assert!(matches!(duration, Ok(_)));
    ///
    /// let ns_timestamp = NsTimestamp::try_from_secs(u64::MAX as i128 + 1).unwrap();
    /// let duration = Duration::try_from(ns_timestamp);
    /// assert!(matches!(duration, Err(Error::OutOfDurationRange)));
    /// ```
    fn try_from(value: NsTimestamp) -> Result<Self, Self::Error> {
        let nanos = value.0.rem_euclid(1_000_000_000i128) as u32; // this never fails
        let secs = value
            .0
            .div_euclid(1_000_000_000i128)
            .try_into()
            .ok()
            .ok_or(Error::OutOfDurationRange)?;
        let duration = Duration::new(secs, nanos);
        Ok(duration)
    }
}

impl TryFrom<NsTimestamp> for SystemTime {
    type Error = Error;

    /// Try to convert a [`NsTimestamp`] into a [`SystemTime`].
    ///
    /// # Errors
    /// This function will return a [`SystemTimeConversionError`] if:
    /// - the amount of nanoseconds in [`self`] is negative,
    /// - the amount of seconds in [`self`] is over [`u64::MAX`], or
    /// - the timestamp is out of range of [`SystemTime`] (when `SystemTime::UNIX_EPOCH.checked_add(duration)` fails.)
    ///
    /// # Examples
    /// ```
    /// use scoretracker_core::util::timestamp::{NsTimestamp, Error};
    /// use std::time::SystemTime;
    /// let timestamp = NsTimestamp::from_nanos(1);
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Ok(_)));
    ///
    /// let timestamp = NsTimestamp::from_nanos(-1);
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// assert!(matches!(system_time, Err(Error::OutOfDurationRange)));
    ///
    /// let timestamp = NsTimestamp::from_nanos(u64::MAX as i128 * 1_000_000_000 + 1);
    /// let system_time: Result<SystemTime, _> = timestamp.try_into();
    /// dbg!(&system_time);
    /// assert!(matches!(system_time, Err(Error::OutOfSystemTimeRange)));
    /// ```
    fn try_from(value: NsTimestamp) -> Result<Self, Self::Error> {
        let duration = value.try_into()?;
        let system_time = SystemTime::UNIX_EPOCH.checked_add(duration).ok_or(Error::OutOfSystemTimeRange)?;
        Ok(system_time)
    }
}

impl<Tz: TimeZone> TryFrom<NsTimestamp> for DateTime<Tz>
where
    DateTime<Tz>: From<SystemTime>,
{
    /// Try to convert a [`NsTimestamp`] into a [`DateTime<Tz>`].
    ///
    /// This function uses [`SystemTime`] under the hood, check out the documentation for [`SystemTime::TryFrom<NsTimestamp>`] for more information.
    type Error = Error;
    fn try_from(value: NsTimestamp) -> Result<Self, Self::Error> {
        let system_time: SystemTime = value.try_into()?;
        Ok(DateTime::from(system_time))
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
