//! Holds traits and their implementation for Rust usage.

use crate::{Date, Day};

/// Convert a time difference in compare to the Epoch to a valid date.
///
/// When the given time can be negative, its absolute will be the base of measurements. Meaning if
/// 5 days before_epoch is the same whether -5 days or +5.
///
/// When the given time has a timezone, it is ignored (the base Epoch always assumes the same
/// timezone as the given value).
pub trait FromEpochDelta {
    /// This struct's time difference as days (86400 seconds).
    ///
    /// This must be saturating.
    fn saturating_d_diff(&self) -> Day;

    /// Convert this many days before the epoch as a Jalali [`Date`].
    ///
    /// The calculations are done in [`Self::saturating_d_diff`] and should not overflow past max.
    fn before_epoch(&self) -> Date {
        Date::from_d_before_epoch(self.saturating_d_diff())
    }

    /// Convert this many days before the epoch as a Jalali [`Date`].
    ///
    /// The calculations are done in [`Self::saturating_d_diff`] and should not underflow past min.
    fn past_epoch(&self) -> Date {
        Date::from_d_past_epoch(self.saturating_d_diff())
    }
}

impl FromEpochDelta for core::time::Duration {
    fn saturating_d_diff(&self) -> Day {
        (self.as_secs() / 86400).min(Day::MAX as u64) as Day
    }
}

#[cfg(feature = "std")]
impl FromEpochDelta for std::time::SystemTime {
    fn saturating_d_diff(&self) -> Day {
        match self.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.saturating_d_diff(),
            Err(e) => e.duration().saturating_d_diff(),
        }
    }
}

#[cfg(feature = "time")]
impl FromEpochDelta for time::Date {
    fn saturating_d_diff(&self) -> Day {
        time::UtcDateTime::new(self.clone(), time::Time::MIDNIGHT).saturating_d_diff()
    }
}

#[cfg(feature = "time")]
impl FromEpochDelta for time::Duration {
    fn saturating_d_diff(&self) -> Day {
        let d: i64 = self.whole_days().saturating_abs(); // must return i64
        d as Day
    }
}

#[cfg(feature = "time")]
impl FromEpochDelta for time::OffsetDateTime {
    fn saturating_d_diff(&self) -> Day {
        let sec: i64 = self.unix_timestamp().saturating_abs(); // must return i64
        sec as Day / 86400
    }
}

#[cfg(feature = "time")]
impl FromEpochDelta for time::PrimitiveDateTime {
    fn saturating_d_diff(&self) -> Day {
        self.as_utc().saturating_d_diff()
    }
}

#[cfg(feature = "time")]
impl FromEpochDelta for time::UtcDateTime {
    fn saturating_d_diff(&self) -> Day {
        let sec: i64 = self.unix_timestamp().saturating_abs(); // must return i64
        sec as Day / 86400
    }
}
