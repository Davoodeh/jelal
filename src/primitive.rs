//! Holds the primitive aliases and transparent wrappers and their utilities.

// TODO add new methods for everything

use core::cmp::Ordering;

use crate::utility::DidSaturate;

/// Counts consecutive days for addition and subtraction operations.
pub type IDayDiff = i32;

/// Unsigned variant of [`IDayDiff`]. This is to be avoided if the signed variant can be used.
pub type UDayDiff = u32;

/// The unsigned primitive type for counting days of a [`Month`].
pub type UMonthDay = u8;

/// The signed equal day counter type for [`UMonthDay`].
pub type IMonthDay = i8;

/// An alias for a commonly used format of Jalali as a type (Year, Month, Day).
pub type Ymd = (Year, Month, UMonthDay);

/// The primitive underlying types for [`Ymd`].
pub type IntYmd = (IYear, UMonth, UMonthDay);

/// The default primitive that holds all the values for months ([`Month::MIN`] to [`Month::MAX`]).
pub type UMonth = u8;

/// Signed variant of the default primitive [`UMonth`].
pub type IMonth = i8;

/// The default primitive that holds all the ordinals ([`Ordinal::MIN`] to [`Ordinal::MAX`]).
pub type UOrdinal = u16;

/// Signed variant of the default primitive [`UOrdinal`].
pub type IOrdinal = i16;

/// The default primitive that holds all the years ([`Year::MIN`] to [`Year::MAX`]).
///
/// There is no unsigned equivalent for this type like the others.
pub type IYear = i32;

/// Holds valid months count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstFieldOrder)]
#[repr(transparent)]
pub struct Month(pub(crate) UMonth);

int_wrapper! {
    ident: Month,
    signed: IMonth,
    unsigned: UMonth,
}

impl Month {
    /// Unix Epoch in this format (equivalent to Gregorian January (1st) in 1970, [`Year::EPOCH`]).
    pub const EPOCH: Self = Self(10);

    /// The first month of the Jalali year; 1: Farvardin.
    pub const MIN: Self = Self(1);

    /// The start of the second half of the year in months.
    pub const MID: Self = Self(7);

    /// The last month of the Jalali year; 12: Esfand.
    pub const MAX: Self = Self(12);

    /// Convert a valid month to ordinal assuming 0th day of the month (-1) if month is valid.
    pub const fn to_ordinal_assume_zero(&self) -> Ordinal {
        let zm = self.0 as UOrdinal - 1;
        Ordinal(if zm <= 6 {
            zm * 31
        } else {
            (zm - 6) * 30 + 186
        })
    }

    /// Create a new instance and limit it to [`Self::MIN`] and [`Self::MAX`].
    pub const fn new(value: UMonth) -> Self {
        if value < Self::MIN.0 {
            Self::MIN
        } else if value > Self::MAX.0 {
            Self::MAX
        } else {
            Self(value)
        }
    }

    /// Return the owned types of this value.
    pub const fn get(&self) -> UMonth {
        self.0
    }
}

impl Ord for Month {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}

impl From<Month> for Ordinal {
    fn from(value: Month) -> Self {
        value.to_ordinal_assume_zero()
    }
}

/// A value representing a day of a year in a leap year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstFieldOrder)]
#[repr(transparent)]
pub struct Ordinal(pub(crate) UOrdinal);

int_wrapper! {
    ident: Ordinal,
    signed: IOrdinal,
    unsigned: UOrdinal,
}

impl Ordinal {
    /// Unix Epoch in this format (equivalent to Gregorian 1st of January, 1970, [`Year::EPOCH`]).
    pub const EPOCH: Self = Self(287);

    /// Marks the first day of the year for a valid calendar year (this struct starts from 1).
    pub const MIN: Self = Self(1);

    /// The first day after the sixth month of the year (first day of [`Month::MID`]).
    pub const MID: Self = Self(187);

    /// The absolute maximum day count for any year (leap, 366).
    pub const MAX: Self = Self(366);

    /// The maximum day count for a non-leap year (365).
    pub const MAX_NON_LEAP: Self = Self::MAX.add_strict(-1).result;

    /// Create a new instance and limit it to [`Self::MIN`] and [`Self::MAX`].
    pub const fn new(value: UOrdinal) -> Self {
        if value < Self::MIN.0 {
            Self::MIN
        } else if value > Self::MAX.0 {
            Self::MAX
        } else {
            Self(value)
        }
    }

    /// Return the owned types of this value.
    pub const fn get(&self) -> UOrdinal {
        self.0
    }
}

impl Ord for Ordinal {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}

// TODO rename impl_new to new_strict and implement new off of it.

/// The base year counter type for Jalali calendar (no 0 variant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstFieldOrder)]
#[repr(transparent)]
pub struct Year(pub(crate) IYear);

int_wrapper!(
    ident: Year,
    signed: IYear,
    skip_i32_helpers: true,
);

impl Year {
    /// Unix Epoch in this format (equivalent to Gregorian 1970).
    pub const EPOCH: Self = Self(1348);

    /// The furthest year in the past possible for this struct.
    pub const MIN: Self = Self(IYear::MIN);

    /// The furthest year in the future possible for this struct.
    pub const MAX: Self = Self(IYear::MAX);

    /// The source of truth for the zero replacement value (-1 is before year 1, skipping 0).
    pub const ZERO_REPLACEMENT: Self = Self(-1);

    /// Create a valid year and if 0, replace it with -1 ([`Self::ZERO_REPLACEMENT`] in effect).
    pub const fn new(value: IYear) -> Self {
        if value == 0 {
            Self::ZERO_REPLACEMENT
        } else {
            Self(value)
        }
    }

    /// Persian Wikipedia's list of leap years pre-calculated.
    ///
    /// NOTE Do not rely on this.
    pub const LEAPS_1210_TO_1500: [Self; 71] = unsafe {
        core::mem::transmute([
            1210, 1214, 1218, 1222, 1226, 1230, 1234, 1238, 1243, 1247, 1251, 1255, 1259, 1263,
            1267, 1271, 1276, 1280, 1284, 1288, 1292, 1296, 1300, 1304, 1309, 1313, 1317, 1321,
            1325, 1329, 1333, 1337, 1342, 1346, 1350, 1354, 1358, 1362, 1366, 1370, 1375, 1379,
            1383, 1387, 1391, 1395, 1399, 1403, 1408, 1412, 1416, 1420, 1424, 1428, 1432, 1436,
            1441, 1445, 1449, 1453, 1457, 1461, 1465, 1469, 1474, 1478, 1482, 1486, 1490, 1494,
            1498,
        ])
    };

    /// Years that are not leap while 33-year rule marks them as leap.
    ///
    /// "All these years are not leap, while they are considered leap by the 33-year
    /// rule. The year following each of them is leap, but it's considered non-leap
    /// by the 33-year rule. This table has been tested to match the modified
    /// astronomical algorithm based on the 52.5 degrees east meridian from 1178 AP
    /// (an arbitrary date before the Persian calendar was adopted in 1304 AP) to
    /// 3000 AP (an arbitrary date far into the future)."
    ///
    /// Taken from
    /// <https://github.com/unicode-org/icu4x/blob/3e3da0a0a34bfe3056d0f89183270ea683f4a23c/utils/calendrical_calculations/src/persian.rs#L23>
    // TODO make a generalized algorithmic implementation
    // TODO fix cbindgen ignoring this
    // keep it semi-clean
    pub const NON_LEAP_CORRECTION: [Self; 78] = unsafe {
        core::mem::transmute([
            1502, 1601, 1634, 1667, 1700, 1733, 1766, 1799, 1832, 1865, 1898, 1931, 1964, 1997,
            2030, 2059, 2063, 2096, 2129, 2158, 2162, 2191, 2195, 2224, 2228, 2257, 2261, 2290,
            2294, 2323, 2327, 2356, 2360, 2389, 2393, 2422, 2426, 2455, 2459, 2488, 2492, 2521,
            2525, 2554, 2558, 2587, 2591, 2620, 2624, 2653, 2657, 2686, 2690, 2719, 2723, 2748,
            2752, 2756, 2781, 2785, 2789, 2818, 2822, 2847, 2851, 2855, 2880, 2884, 2888, 2913,
            2917, 2921, 2946, 2950, 2954, 2979, 2983, 2987,
        ])
    };

    /// A search into [`Self::NON_LEAP_CORRECTION`].
    pub const fn is_no_leap_correction(&self) -> bool {
        // HACK(const): binary_search not in const: NON_LEAP_CORRECTION.binary_search(&year).is_ok()
        if self.cmp(&Self::NON_LEAP_CORRECTION[0]).is_lt()
            || self
                .cmp(&Self::NON_LEAP_CORRECTION[Self::NON_LEAP_CORRECTION.len() - 1])
                .is_gt()
        {
            return false;
        }

        let mut i = 0;
        while i < Self::NON_LEAP_CORRECTION.len() {
            if self.cmp(&Self::NON_LEAP_CORRECTION[i]).is_eq() {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Is this year a leap year (366 days instead of 365).
    ///
    /// Calculated using the 33-year rule. Taken from
    /// <https://github.com/unicode-org/icu4x/blob/3e3da0a0a34bfe3056d0f89183270ea683f4a23c/utils/calendrical_calculations/src/persian.rs#L161C1-L173C2>
    pub const fn is_leap(&self) -> bool {
        if self.cmp(&Self::NON_LEAP_CORRECTION[0]).is_ge() && self.is_no_leap_correction() {
            return false;
        }

        let prev = self.add_strict(-1);
        // no previous year so assume no
        if prev.did_saturate {
            return false;
        }

        if self.cmp(&Self::NON_LEAP_CORRECTION[0]).is_gt() && prev.result.is_no_leap_correction() {
            return true;
        }

        (25 * self.0 as i64 + 11).rem_euclid(33) < 8
    }

    /// Return the number of the maximum consecutive day of the year (365 or 366 for leaps).
    pub const fn max_ordinal(&self) -> Ordinal {
        if self.is_leap() {
            Ordinal::MAX
        } else {
            Ordinal::MAX_NON_LEAP
        }
    }

    /// Return the owned types of this value.
    pub const fn get(&self) -> IYear {
        self.0
    }
}

impl Ord for Year {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::cmp(self, other)
    }
}
