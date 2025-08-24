#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_main, no_std)]

#[cfg(not(any(test, feature = "std")))] // suppress duplicate error
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(feature = "std")]
extern crate std;

use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

#[macro_use]
mod r#macro;

mod primitive;
mod utility;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "c")]
use ffi::tm;

pub use primitive::*;

pub use crate::utility::DidSaturate;

/// The day of the month and its related month in a leap year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthDay {
    /// The month of the year.
    pub(crate) month: Month,
    /// The day of the associated month.
    pub(crate) day: UMonthDay,
}

impl MonthDay {
    /// The minimum possible day, the start of every month.
    pub const MIN_DAY: UMonthDay = 1;

    /// The maximum day count of the year (for months prior to [`Month::MID`] or start of fall).
    pub const MAX_DAY: UMonthDay = 31;

    /// The maximum number of days in a month post [`Month::MID`].
    pub const POST_MID_MAX_DAY: UMonthDay = Self::MAX_DAY - 1;

    /// The maximum of the last month in a non-leap year.
    pub const NON_LEAP_LAST_MAX_DAY: UMonthDay = 29;

    /// The maximum of the last month in a non-leap year.
    #[deprecated(since = "0.4.1", note = "use [`Self::NON_LEAP_LAST_MAX_DAY`] instead")]
    pub const NON_LEAP_LAST_MONTH_DAY_MAX: UMonthDay = Self::NON_LEAP_LAST_MAX_DAY;

    /// The maximum of the last month in a leap year.
    pub const LEAP_LAST_MAX_DAY: UMonthDay = Self::NON_LEAP_LAST_MAX_DAY + 1;

    /// The maximum of the last month in a leap year.
    #[deprecated(since = "0.4.1", note = "use [`Self::LEAP_LAST_MAX_DAY`] instead")]
    pub const LEAP_LAST_MONTH_DAY_MAX: UMonthDay = Self::LEAP_LAST_MAX_DAY;

    /// The day of month in Jalali for Unix Epoch.
    pub const EPOCH_DAY: UMonthDay = 11;

    /// The minimum valid this inner type, everything saturates to this if less.
    pub const MIN: Self = Self {
        month: Month::MIN,
        day: Self::MIN_DAY,
    };

    /// The maxmium valid this inner type, everything saturates to this if greater.
    pub const MAX: Self = Self {
        month: Month::MAX,
        day: Self::LEAP_LAST_MAX_DAY,
    };

    /// Unix Epoch in this format.
    pub const EPOCH: Self = Self {
        month: Month::EPOCH,
        day: Self::EPOCH_DAY,
    };

    /// Create a new valid instance and slightly saturate and modify to fit a valid instance.
    pub const fn new(month: Month, day: UMonthDay) -> Self {
        let month_mid_cmp = month.cmp(&Month::MID);
        Self {
            month,
            day: if day < Self::MIN_DAY {
                Self::MIN_DAY
            } else if month_mid_cmp.is_lt() && day > Self::MAX_DAY {
                Self::MAX_DAY
            } else if month_mid_cmp.is_ge() && day > Self::POST_MID_MAX_DAY {
                Self::POST_MID_MAX_DAY
            } else {
                day
            },
        }
    }

    /// Return the ordinal (day of the year) for this month and its day.
    pub const fn to_ordinal(&self) -> Ordinal {
        self.month
            .to_ordinal_assume_zero()
            .add_strict(self.day as IOrdinal)
            .result
    }

    /// Add or sub a value to this month and saturate to the limits.
    ///
    /// This is exactly as [`Self::add_month_strict`] but returns the value only.
    pub const fn add_month(self, month: IMonth) -> Self {
        self.add_month_strict(month).result
    }

    /// Add or sub a value to the day of this and saturate to the limits.
    ///
    /// This is exactly as [`Self::add_day_strict`] but returns the value only.
    pub const fn add_day(self, day: IMonthDay) -> Self {
        self.add_day_strict(day).result
    }

    /// Create a valid month and day (in order) from a valid day of the year.
    pub const fn from_ordinal(value: Ordinal) -> Self {
        /// Count how many days are in a month if all the months are the same length.
        const fn same_length_month_counter<const DAYS_IN_A_MONTH: UMonthDay>(
            days: UOrdinal,
        ) -> (UMonth, UMonthDay) {
            let dom = DAYS_IN_A_MONTH as UOrdinal;
            // there are two main ways that the month and day may not be valid fails:
            // 1. Month is 0 meaning doy is below or equal 31 this is handled by the saturating_sub
            // 2. Day remainder by months is 0 zero (rough example doy%30): in that case this, the
            //    last month must be decreased by one and the last day of the removed month assumed

            // first month of the year is 1 so if any days are in the new month, add (1+).
            // in other words months start from 1 and the div results make this a necessary
            let div = 1 + (days / dom);

            // if no days are left means we are in the last day of month so its not a full div yet
            match days % dom {
                0 => (div.saturating_sub(1) as UMonth, dom as UMonthDay),
                rem => (div as UMonth, rem as UMonthDay),
            }
        }

        const MID: UOrdinal = Ordinal::MID.0;
        let ordinal = value.0;
        let (month, day) = if ordinal < MID {
            same_length_month_counter::<{ Self::MAX_DAY }>(ordinal)
        } else {
            let (div, rem) =
                same_length_month_counter::<{ Self::POST_MID_MAX_DAY }>(ordinal - (MID - 1));
            (div + 6, rem)
        };

        Self {
            month: Month(month),
            day,
        }
    }

    /// Add or sub a value to the month of this and return if modifications to output was required.
    ///
    /// This functions returns a boolean which if true, signals that the results of the raw
    /// calculations would overflow or underflow and saturation occured.
    pub const fn add_month_strict(self, month: IMonth) -> DidSaturate<Self> {
        let month = self.month.add_strict(month);
        let result = Self::new(month.result, self.day);
        DidSaturate::new(month.did_saturate || self.cmp(&result).is_ne(), result)
    }

    /// Add or sub a value to the day of this and return if modifications to output was required.
    ///
    /// This functions returns a boolean which if true, signals that the results of the raw
    /// calculations would overflow or underflow and saturation occured.
    pub const fn add_day_strict(self, day: IMonthDay) -> DidSaturate<Self> {
        match self.day.checked_add_signed(day) {
            Some(day) => {
                let result = Self::new(self.month, day);
                DidSaturate::new(self.cmp(&result).is_ne(), result)
            }
            None => DidSaturate::saturated(Self::new(
                self.month,
                if day.is_negative() {
                    Self::MIN_DAY
                } else {
                    Self::MAX_DAY
                },
            )),
        }
    }

    /// Return the owned types of this value.
    pub const fn get(&self) -> (Month, UMonthDay) {
        (self.month, self.day)
    }

    /// Return the value of inner `Self::month` for this instance.
    pub const fn month(&self) -> Month {
        self.month
    }

    /// Return the value of inner `Self::day` for this instance.
    pub const fn day(&self) -> UMonthDay {
        self.day
    }

    /// Const-context definition of [`Ord::cmp`].
    pub const fn cmp(&self, other: &Self) -> Ordering {
        self.month.cmp(&other.month).then(cmp!(self.day, other.day))
    }
}

impl PartialOrd for MonthDay {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for MonthDay {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}

impl<M, D> From<MonthDay> for (M, D)
where
    M: From<Month>,
    D: From<UMonthDay>,
{
    fn from(value: MonthDay) -> Self {
        (value.month.into(), value.day.into())
    }
}

impl From<MonthDay> for Month {
    fn from(value: MonthDay) -> Self {
        value.month
    }
}

impl From<MonthDay> for UMonthDay {
    fn from(value: MonthDay) -> Self {
        value.day
    }
}

impl From<MonthDay> for IMonthDay {
    fn from(value: MonthDay) -> Self {
        value.day as i8
    }
}

impl From<MonthDay> for Ordinal {
    fn from(value: MonthDay) -> Self {
        value.to_ordinal()
    }
}

impl From<Ordinal> for Month {
    fn from(value: Ordinal) -> Self {
        MonthDay::from(value).month()
    }
}

impl From<Ordinal> for MonthDay {
    fn from(value: Ordinal) -> Self {
        MonthDay::from_ordinal(value)
    }
}

impl From<Date> for MonthDay {
    fn from(value: Date) -> Self {
        MonthDay::from_ordinal(value.ordinal())
    }
}

impl<M, D> From<(M, D)> for MonthDay
where
    M: Into<Month>,
    D: Into<UMonthDay>,
{
    fn from(value: (M, D)) -> Self {
        MonthDay::new(value.0.into(), value.1.into())
    }
}

impl<M, D> From<(M, D)> for Ordinal
where
    (M, D): Into<MonthDay>,
{
    fn from(value: (M, D)) -> Self {
        <(M, D) as Into<MonthDay>>::into(value).into()
    }
}

/// A Jalali valid date.
///
/// See [`Year`] for more information about year count. [`Self::MIN`] to [`Self::MAX`] is the
/// representable range (not necessarily all correct in leap calculation or conversion). Year 0 is
/// not a valid year (see [`Year::ZERO_REPLACEMENT`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    /// The year of this date.
    pub(crate) year: Year,
    /// The number of days passed since the start of the year.
    pub(crate) ordinal: Ordinal,
}

impl Date {
    /// The furthest in the past that can be represented with this struct.
    pub const MIN: Self = Self {
        year: Year::MIN,
        ordinal: Ordinal::MIN,
    };

    /// The furthest in the future that can be represented with this struct.
    pub const MAX: Self = Self {
        year: Year::MAX,
        ordinal: Ordinal::MAX,
    };

    /// Unix Epoch in this format (equivalent to Gregorian 1st of January [`MonthDay`], 1970).
    pub const EPOCH: Self = Self {
        year: Year::EPOCH,
        ordinal: Ordinal::EPOCH,
    };

    /// Create a new Jalali date or slightly change values to be valid.
    pub const fn new(year: Year, ordinal: Ordinal) -> Self {
        Self {
            year,
            ordinal: if year.max_ordinal().cmp(&ordinal).is_lt() {
                Ordinal::MAX_NON_LEAP
            } else {
                ordinal
            },
        }
    }

    /// Add a year to this date and saturate the results at limits.
    ///
    /// This is exactly as [`Self::add_year_strict`] but returns the value only.
    pub const fn add_year(self, year: IYear) -> Self {
        self.add_year_strict(year).result
    }

    /// Add a ordinal to this date and saturate the results at limits.
    ///
    /// This is exactly as [`Self::add_ordinal_strict`] but returns the value only.
    pub const fn add_ordinal(self, ordinal: IOrdinal) -> Self {
        self.add_ordinal_strict(ordinal).result
    }

    /// Add a month count to this date and saturate the results at limits.
    ///
    /// This is exactly as [`Self::add_month_strict`] but returns the value only.
    pub const fn add_month(self, month: IMonth) -> Self {
        self.add_month_strict(month).result
    }

    /// Add this many consecutive months to this date.
    ///
    /// This is exactly as [`Self::add_months_strict`] but returns the value only.
    pub const fn add_months(self, months: IDayDiff) -> Self {
        self.add_months_strict(months).result
    }

    /// Add or remove the given number of consecutive days to this date.
    ///
    /// This is exactly as [`Self::add_days_strict`] but returns the value only.
    pub const fn add_days(self, days: IDayDiff) -> Self {
        self.add_days_strict(days).result
    }

    /// Return how many days on this date will result to the given destination.
    ///
    /// This is exactly as [`Self::diff_as_days_strict`] but returns the value only.
    pub const fn diff_as_days(&self, other: Self) -> IDayDiff {
        self.diff_as_days_strict(other).result
    }

    /// Return how many days has passed since or is yet to reach [`Self::EPOCH`].
    ///
    /// This is exactly as [`Self::diff_epoch_strict`] but returns the value only.
    pub const fn diff_epoch(&self) -> IDayDiff {
        self.diff_epoch_strict().result
    }

    /// Add a year to this date and return if the values could not be produced normally.
    ///
    /// See the inner [`Year::add_strict`] and [`Ordinal::add_strict`].
    pub const fn add_year_strict(self, year: IYear) -> DidSaturate<Self> {
        let year = self.year.add_strict(year);
        let result = Self::new(year.result, self.ordinal);
        DidSaturate::new(year.did_saturate || self.cmp(&result).is_ne(), result)
    }

    /// Add a ordinal to this date and return if the values could not be produced normally.
    ///
    /// This is the same as adding two ordinals. Adding an ordinal (day of year)  to another will
    /// saturate at year boundaries and do not exceed to the next year. This function will not pass
    /// through year boundaries. Use [`Self::add_days_strict`] to pass into the next or previous
    /// year.
    ///
    /// See the inner [`Year::add_strict`] and [`Ordinal::add_strict`].
    pub const fn add_ordinal_strict(self, ordinal: IOrdinal) -> DidSaturate<Self> {
        let ordinal = self.ordinal.add_strict(ordinal);
        let result = Self::new(self.year, ordinal.result);
        DidSaturate::new(ordinal.did_saturate || self.cmp(&result).is_ne(), result)
    }

    /// Add a month count to this date and return if the values could not be produced normally.
    ///
    /// This will not pass year boundaries. If you are looking for one that goes through year
    /// boundaries use [`Self::add_months_strict`].
    ///
    /// See the inner [`Year::add_strict`] and [`Ordinal::add_strict`].
    pub const fn add_month_strict(self, month: IMonth) -> DidSaturate<Self> {
        let dom = MonthDay::from_ordinal(self.ordinal).add_month_strict(month);
        let result = Self::new(self.year, dom.result.to_ordinal());
        DidSaturate::new(dom.did_saturate || self.cmp(&result).is_ne(), result)
    }

    /// Add or remove a year for each 12 months given returning remainder (leap correct).
    ///
    /// This is saturating meaning won't overflow or underflow the year if the day does not exist in
    /// the new month and will automatically correct.
    const fn add_months_assume_new_year(self, months: IDayDiff) -> DidSaturate<Self> {
        let toward_past = months.is_negative();
        let months: UDayDiff = months.unsigned_abs();

        const MC: UDayDiff = Month::MAX.get() as UDayDiff;

        let (div, rem) = match months % MC {
            0 => ((months / MC).saturating_sub(1), MC),
            v => (months / MC, v),
        };

        // TODO add a test to ensure that (2**(BITS) / 12) < (2**(BITS-1) - 1)
        // ((2**32 / 12) is less than 2**31 so this the cast always works.)

        let year = self.year.add_strict(if toward_past {
            -(div as IDayDiff)
        } else {
            div as IDayDiff
        });

        // will definitely not saturate for % properties
        let ordinal = MonthDay::new(Month::new(rem as UMonth), MonthDay::MIN_DAY).to_ordinal();

        DidSaturate::new(year.did_saturate, Self::new(year.result, ordinal))
    }

    /// Add this many consecutive months to this date.
    ///
    /// This will pass year boundaries. If you are looking for one that stops at year boundaries use
    /// [`Self::add_month_strict`].
    pub const fn add_months_strict(self, months: IDayDiff) -> DidSaturate<Self> {
        let self_month_day = MonthDay::from_ordinal(self.ordinal);
        let (months, did_saturate) =
            match months.checked_add(self_month_day.month().get() as IDayDiff) {
                Some(v) => (v, false),
                None => (
                    if months.is_negative() {
                        IDayDiff::MIN
                    } else {
                        IDayDiff::MAX
                    },
                    true,
                ),
            };
        let v = self.add_months_assume_new_year(months);
        let did_saturate = did_saturate || v.did_saturate;

        let v = v
            .result
            .add_ordinal_strict((self_month_day.day() - 1) as IOrdinal);
        DidSaturate::new(did_saturate || v.did_saturate, v.result)
    }

    /// Add or remove a year for each 365/366 days given returning remainder (leap correct).
    ///
    /// This is saturating meaning won't overflow or underflow the year if excessive days are
    /// removed or added.
    const fn add_days_assume_new_year(mut self, days: IDayDiff) -> DidSaturate<Self> {
        let toward_past = days.is_negative();
        let step_year_diff = if toward_past { -1 } else { 1 };
        let mut days: UDayDiff = days.unsigned_abs();

        loop {
            let max_doy = self.year.max_ordinal();

            if days <= max_doy.get() as UDayDiff {
                self.ordinal = Ordinal::new(days as UOrdinal); // ensure no 0, it might be here
                return DidSaturate::not_saturated(self);
            }
            // won't be zero since it's strictly larger than max_doy or would have returned
            days -= max_doy.get() as u32;

            // add or remove one year in this ugly form until more helpers are added
            let year = self.year.add_strict(step_year_diff);
            self.year = year.result;
            if year.did_saturate {
                self.ordinal = if toward_past { Ordinal::MIN } else { max_doy };
                return DidSaturate::saturated(self);
            }
        }
    }

    /// Add or remove the given number of consecutive days to this date.
    ///
    /// This is not the same as adding ordinals. Adding an ordinal (day of year)  to another will
    /// saturate at year boundaries and do not exceed to the next year. This function will pass
    /// through year boundaries. Use [`Self::add_ordinal_strict`] for the other functionality.
    pub const fn add_days_strict(self, days: IDayDiff) -> DidSaturate<Self> {
        let (days, did_saturate) = match days.checked_add(self.ordinal.0 as IDayDiff) {
            Some(v) => (v, false),
            None => (
                if days.is_negative() {
                    IDayDiff::MIN
                } else {
                    IDayDiff::MAX
                },
                true,
            ),
        };
        let v = self.add_days_assume_new_year(days);
        DidSaturate::new(did_saturate || v.did_saturate, v.result)
    }

    /// Return how many days on this date will result to the given destination.
    pub const fn diff_as_days_strict(&self, mut other: Self) -> DidSaturate<IDayDiff> {
        let toward_past = self.year.cmp(&other.year).is_lt();
        let year_step = if toward_past { -1 } else { 1 };

        // change the delta by that many years until the years have a difference of one.
        let mut year_diff: IDayDiff = 0;
        while self.year.cmp(&other.year).is_ne() {
            year_diff = match year_diff
                .checked_add(year_step * (other.year.max_ordinal().get() as IDayDiff))
            {
                Some(v) => v,
                None => {
                    return DidSaturate::saturated(if toward_past {
                        IDayDiff::MIN
                    } else {
                        IDayDiff::MAX
                    });
                }
            };
            other.year = other.year.add_strict(year_step).result; // to skip over 0
        }

        let ordinal_diff = self.ordinal.get() as IDayDiff - other.ordinal.get() as IDayDiff;
        DidSaturate::not_saturated(year_diff + ordinal_diff)
    }

    /// Return how many days has passed since or is yet to reach [`Self::EPOCH`].
    pub const fn diff_epoch_strict(&self) -> DidSaturate<IDayDiff> {
        self.diff_as_days_strict(Self::EPOCH)
    }

    /// Return the owned types of this value.
    pub const fn get(&self) -> (Year, Ordinal) {
        (self.year, self.ordinal)
    }

    /// Return the value of inner `Self::year` for this instance.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Return the value of inner `Self::ordinal` for this instance.
    pub const fn ordinal(&self) -> Ordinal {
        self.ordinal
    }

    // TODO add functions to calculcate `tm`, `DateTime` and other dates in Gregorian, not only
    //      Shamsi, for example a pair of `update_tm` and `to_tm` should be there to calculate it
    //      That needs a dependency that converts the number of days to its valid gregorian. This
    //      should NOT be implemented here since this is not a gregorian calendar crate.
    //      As of now, the days can be seeked which can subsequently converted to epoch seconds and
    //      used in functions like `localtime`.

    /// Convert this [`Self::to_jtm`] but on the given struct.
    #[cfg(feature = "c")]
    pub const fn update_jtm(&self, jtm: &mut tm) {
        use ffi::c_int;

        let monthday = MonthDay::from_ordinal(self.ordinal);

        jtm.tm_mday = monthday.day as c_int;
        jtm.tm_mon = (monthday.month.get() as c_int) - 1;
        jtm.tm_year = self.year.get();
        jtm.tm_yday = (self.ordinal.get() as c_int) - 1;
    }

    /// Create an [`ffi::tm`] from this date in Jalali.
    ///
    /// If the aim is not to create a new instance and update an already created `tm`, use
    /// [`Self::update_jtm`].
    ///
    /// See its documents for how this struct's values should be interpreted when the date is
    /// assumed to be Jalali. In short, this is exactly as in C but year doesn't have an offset and
    /// only year, month, ordinal and month day are set.
    ///
    /// There are no `from_jtm` equal since there are many ways interprete how this should be done,
    /// (based on ordinal `yday` or `year`, `mon`, `mday` fields to name two).
    ///
    /// To convert this value into a `tm` (Gregorian) use [`Self::diff_epoch`] and then convert that
    /// to seconds to use with `localtime` and `gmtime`.
    #[cfg(feature = "c")]
    pub const fn to_jtm(&self) -> tm {
        let mut jtm = tm::new_zero();
        self.update_jtm(&mut jtm);
        jtm
    }

    /// Const-context definition of [`Ord::cmp`].
    pub const fn cmp(&self, other: &Self) -> Ordering {
        self.year
            .cmp(&other.year)
            .then(self.ordinal.cmp(&other.ordinal))
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}

impl<Y, O> From<Date> for (Y, O)
where
    Y: From<Year>,
    O: From<Ordinal>,
{
    fn from(value: Date) -> Self {
        (value.year.into(), value.ordinal.into())
    }
}

impl From<Date> for Year {
    fn from(value: Date) -> Self {
        value.year
    }
}

impl From<Date> for Ordinal {
    fn from(value: Date) -> Self {
        value.ordinal
    }
}

impl From<Year> for Date {
    fn from(value: Year) -> Self {
        Date::new(value, Ordinal::MIN)
    }
}

impl From<IYear> for Date {
    fn from(value: IYear) -> Self {
        Date::from(Year::from(value))
    }
}

impl<Y, M> From<(Y, M)> for Date
where
    Y: Into<Year>,
    M: Into<Ordinal>,
{
    fn from(value: (Y, M)) -> Self {
        Date::new(value.0.into(), value.1.into())
    }
}

impl<Y, M, D> From<(Y, M, D)> for Date
where
    Y: Into<Year>,
    (M, D): Into<Ordinal>,
{
    fn from(value: (Y, M, D)) -> Self {
        Date::new(value.0.into(), (value.1, value.2).into())
    }
}

impl<Y, M, D> From<Date> for (Y, M, D)
where
    Y: From<Year>,
    (M, D): From<MonthDay>,
{
    fn from(value: Date) -> Self {
        let (year, md): (Year, MonthDay) = value.into();
        let (m, d) = md.into();
        (year.into(), m, d)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (y, m, d) = Ymd::from(self.clone());
        write!(f, "{}/{}/{}", y, m, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_day_max() {
        let from_ordinal: MonthDay = Ordinal::MAX.into();
        assert_eq!(from_ordinal.day(), MonthDay::LEAP_LAST_MAX_DAY);
        assert_eq!(from_ordinal.month(), Month::MAX);
        assert_eq!(from_ordinal, MonthDay::MAX);
    }

    #[test]
    fn test_month_day_min() {
        let from_ordinal: MonthDay = Ordinal::MIN.into();
        assert_eq!(from_ordinal.day(), MonthDay::MIN_DAY);
        assert_eq!(from_ordinal.month(), Month::MIN);
        assert_eq!(from_ordinal, MonthDay::MIN);
    }

    #[test]
    fn test_leap_aligns_with_wikipedia_list_of_33() {
        for i in 1210..=1500 {
            let year = Year::from(i);
            let is_leap = year.is_leap();
            let in_list = Year::LEAPS_1210_TO_1500.binary_search(&year).is_ok();
            assert!(
                if is_leap { in_list } else { !in_list },
                "year {} is miscalculated (guessed as leap: {}, is actually leap: {})",
                i,
                is_leap,
                in_list
            );
        }
    }

    #[test]
    fn test_ordinal_first_day_of_calendar() {
        assert_eq!(Date::from((1, 1, 1)).ordinal(), Ordinal::MIN);
    }

    #[test]
    fn test_ordinal_365_day_of_first_year() {
        assert_eq!(Date::from((1, 12, 29)).ordinal(), Ordinal::MAX_NON_LEAP);
    }

    #[test]
    fn test_ordinal_from_unsuffixed_int() {
        assert_eq!(Ordinal::from(1).get(), 1);
    }

    #[test]
    fn test_month_day_from_ordinal() {
        for m in 1..=6 {
            for d in 1..=31 {
                assert_eq!(
                    Ordinal::from(MonthDay::from((m, d))),
                    Ordinal::from((m - 1) * 31 + d as i32),
                );
            }
        }

        for m in 7..=12 {
            for d in 1..=30 {
                assert_eq!(
                    Ordinal::from(MonthDay::from((m, d))),
                    Ordinal::from((Ordinal::MID - 1i16) + (m - 7) as i16 * 30 + d as i16),
                );
            }
        }
    }

    #[test]
    fn test_add_doy_epoch_1348() {
        let test = |offset: IDayDiff, (y, m, d): IntYmd| {
            let v = Date::EPOCH.add_days_strict(offset).result;
            assert_eq!(v.ordinal().get() as IDayDiff, (287 + offset));
            assert_eq!(v, Date::from((y, m, d)));
        };

        test(0, (1348, 10, 11));
        test(1, (1348, 10, 12));
        test(2, (1348, 10, 13));
        test(4, (1348, 10, 15));
        test(7, (1348, 10, 18));
        test(8, (1348, 10, 19));
        test(9, (1348, 10, 20));
        test(12, (1348, 10, 23));
        test(32, (1348, 11, 13));
        test(62, (1348, 12, 13));
        test(78, (1348, 12, 29));
        // not leap
    }

    #[test]
    fn test_add_doy_epoch_1349() {
        let test = |offset: IDayDiff, (y, m, d): IntYmd| {
            let v = Date::EPOCH.add_days_strict(78 + offset).result;
            assert_eq!(v.ordinal().get() as IDayDiff, offset);
            assert_eq!(v, Date::from((y, m, d)));
        };

        test(1, (1349, 1, 1));
        test(2, (1349, 1, 2));
        test(30, (1349, 1, 30));
        test(31, (1349, 1, 31));
        test(32, (1349, 2, 1));
        test(33, (1349, 2, 2));
        test(43, (1349, 2, 12));
        test(53, (1349, 2, 22));
        test(60, (1349, 2, 29));
        test(61, (1349, 2, 30));
        test(62, (1349, 2, 31));
        test(63, (1349, 3, 1));
        test(64, (1349, 3, 2));
        test(93, (1349, 3, 31));
        test(124, (1349, 4, 31));
        test(155, (1349, 5, 31));
        test(186, (1349, 6, 31));
        test(216, (1349, 7, 30));
        test(246, (1349, 8, 30));
        test(276, (1349, 9, 30));
        test(306, (1349, 10, 30));
        test(336, (1349, 11, 30));
        test(365, (1349, 12, 29));
        // not leap
    }

    #[test]
    fn test_add_ordinal_saturates_while_days_doesnt() {
        let year = Year::from(1350);
        let v = Date::from(year);
        for i in 0..year.max_ordinal().get() {
            // - if the last value is included with the starting day will result in 365+1
            // - small values so the `as` won't do anything unexpected
            assert_eq!(
                v.clone().add_ordinal_strict(i as IOrdinal).result,
                v.clone().add_days_strict(i as IDayDiff).result,
            );
        }

        // stays in this very year
        assert_eq!(
            v.clone().add_ordinal_strict(366).result,
            Date::from((1350, year.max_ordinal())),
        );

        // goes to the next year
        assert_eq!(
            v.clone().add_days_strict(366).result,
            Date::from((1351, 366 - (year.max_ordinal().get() - 1))),
        );
    }

    #[test]
    fn test_add_186_new_year() {
        let v = Date::from(1350);
        assert_eq!(v.ordinal().get(), 1);
        assert_eq!(v.year().get(), 1350);
        assert_eq!(v, Date::from((1350, 1, 1)));

        let v = v.add_ordinal_strict(184).result;
        assert_eq!(v.ordinal().get(), 185);
        assert_eq!(v, Date::from((1350, 185)).into());
        assert_eq!(v, Date::from((1350, 6, 30)));
        assert_eq!(MonthDay::from(v.clone()).day(), 30);
        assert_eq!(MonthDay::from(v.clone()).month().get(), 6);

        let v = v.add_ordinal_strict(1).result;
        assert_eq!(v.ordinal().get(), 186);
        assert_eq!(v, Date::from((1350, 186)));
        assert_eq!(v, Date::from((1350, 186)).into());
        assert_eq!(v, Date::from((1350, 6, 31)));
        assert_eq!(MonthDay::from(v.clone()).day(), 31);
        assert_eq!(MonthDay::from(v.clone()).month().get(), 6);

        let v = v.add_ordinal_strict(1).result;
        assert_eq!(v.ordinal().get(), 187);
        assert_eq!(v, Date::from((1350, 187)));
        assert_eq!(v, Date::from((1350, 187)).into());
        assert_eq!(v, Date::from((1350, 7, 1)));
        assert_eq!(MonthDay::from(v.clone()).day(), 1);
        assert_eq!(MonthDay::from(v.clone()).month().get(), 7);
    }

    #[test]
    fn test_set_doy_leap_for_leap() {
        assert!(Date::from((1403, 366)).year().is_leap());
        assert_eq!(
            Date::from((1403, 365))
                .add_ordinal_strict(1)
                .result
                .ordinal()
                .get(),
            366
        );
        assert_eq!(Date::from((1403, 366)).ordinal().get(), 366);
    }

    #[test]
    fn test_set_doy_leap_for_non_leap() {
        assert!(!Date::from((1404, 366)).year().is_leap());
        assert_eq!(
            Date::from((1404, 365))
                .add_ordinal_strict(1)
                .result
                .ordinal()
                .get(),
            365
        );
        assert_eq!(Date::from((1404, 366)).ordinal().get(), 365); // saturates
    }

    #[test]
    fn test_add_12_month_leap_invalid() {
        let d = Date::from((1403, 12, 30));
        assert_eq!(d.year().get(), 1403);
        assert_eq!(MonthDay::from(d.clone()), MonthDay::from((12, 30)));
        assert_eq!(d.ordinal().get(), 366);

        // keeps at 12 months but the day count is the same
        assert_eq!(
            IntYmd::from(d.add_month_strict(12).result),
            (1403, 12, 30).into()
        );
    }

    #[test]
    fn test_add_12_concecutive_month_leap_invalid() {
        let d = Date::from((1403, 12, 30));

        // `months` variant pushes to the next year but with correct day count.
        assert_eq!(
            IntYmd::from(d.clone().add_months_strict(12).result),
            (1404, 12, 29).into()
        );
        assert_eq!(
            IntYmd::from(d.clone().add_months_strict(13).result),
            (1405, 1, 30).into()
        );
    }

    // Since the library is `cdylib`, Rust doesn't test the snippets in the documentation code, this
    // is a manual copy of the code mentioned in the readme.
    #[test]
    fn test_readme() {
        let fixed_point = Date::from((1404, 2, 13)); // 2025, 5 (May), 3
        assert_eq!(fixed_point.add_days(11), Date::from((1404, 2, 24)));
    }

    #[test]
    fn test_is_leap_year_min_i32() {
        assert!(!Year::from(i32::MIN).is_leap());
    }

    #[test]
    fn test_is_leap_year_1348_pre_and_post_epoch() {
        // this effects the diff epoch tests
        assert!(!(Year::EPOCH - 1).is_leap());
        assert!(!Year::EPOCH.is_leap());
        assert!(!(Year::EPOCH + 1).is_leap());
    }

    #[test]
    fn test_year_zero_and_ones_are_not_leap() {
        // not that it matters but more delicate checks into the code is probably needed if they
        // differ.
        assert!(!Year::from(-1).is_leap());
        // zero untestable in this new typed values assert!(!Year::from(0).is_leap());
        assert!(!Year::from(1).is_leap());
    }

    #[test]
    fn test_d_past_epoch() {
        // past
        assert_eq!(
            Date::from((
                Year::EPOCH,
                MonthDay::EPOCH.month(),
                MonthDay::EPOCH_DAY - 1,
            ))
            .diff_epoch_strict(),
            -1,
        );
        assert_eq!(
            Date::from((
                Year::EPOCH,
                MonthDay::EPOCH.month() - 1,
                MonthDay::EPOCH_DAY,
            ))
            .diff_epoch_strict(),
            -30
        );
        assert_eq!(
            Date::from((
                Year::EPOCH - 1,
                MonthDay::EPOCH.month(),
                MonthDay::EPOCH_DAY,
            ))
            .diff_epoch_strict(),
            -365
        );
        assert_eq!(
            Date::from((
                Year::EPOCH - 1,
                MonthDay::EPOCH.month() - 1,
                MonthDay::EPOCH_DAY - 1,
            ))
            .diff_epoch_strict(),
            -365 - 30 - 1
        );
        // // same
        assert_eq!(Date::EPOCH.diff_epoch_strict(), 0);

        // // future
        assert_eq!(
            Date::from((
                Year::EPOCH,
                MonthDay::EPOCH.month(),
                MonthDay::EPOCH_DAY + 1,
            ))
            .diff_epoch_strict(),
            1,
        );
        assert_eq!(
            Date::from((
                Year::EPOCH,
                MonthDay::EPOCH.month() + 1,
                MonthDay::EPOCH_DAY,
            ))
            .diff_epoch_strict(),
            30
        );
        assert_eq!(
            Date::from((
                Year::EPOCH + 1,
                MonthDay::EPOCH.month(),
                MonthDay::EPOCH_DAY,
            ))
            .diff_epoch_strict(),
            365
        );
        assert_eq!(
            Date::from((
                Year::EPOCH + 1,
                MonthDay::EPOCH.month() + 1,
                MonthDay::EPOCH_DAY + 1,
            ))
            .diff_epoch_strict(),
            365 + 30 + 1
        );
    }
}
