#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(test, feature = "py")), no_main, no_std)]
#![cfg_attr(feature = "py", allow(unsafe_op_in_unsafe_fn))] // python, staticmethods and unsafe new

use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

#[cfg(not(feature = "wasm"))]
use jelal_proc::fn_attr;

#[cfg(feature = "py")]
use jelal_proc::py_attr;

#[cfg(feature = "py")]
use pyo3::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "py")]
#[pymodule]
fn jelal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_non_leap_correction, m)?)?;
    m.add_function(wrap_pyfunction!(is_leap_year, m)?)?;
    m.add_function(wrap_pyfunction!(max_doy, m)?)?;
    m.add_class::<Md>()?;
    m.add_class::<Date>()?;
    Ok(())
}

#[cfg(all(not(test), not(any(feature = "wasm", feature = "py"))))] // suppress duplicate error
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Counter for years.
pub type Year = i32;

/// Counter for months of a year.
pub type Month = u8;

/// Counter for consecutive days
pub type Day = u32;

/// Counter for days of a week or month.
pub type Dom = u8;

/// Counter for days in a year.
pub type Doy = u16;

/// End of the first half of the year (6th month).
pub const FIRST_HALF_MAX_DOY: Doy = 186;

/// End of the second half of the year in the longest years.
pub const SECOND_HALF_MAX_DOY: Doy = 366;

/// Days of month in months up to and including the sixth (last month before new half).
pub const FIRST_HALF_MAX_DOM: Dom = 31;

/// Days of month in months up to and including the sixth (last month before new half).
pub const SECOND_HALF_MAX_DOM: Dom = 30;

/// The year 1970.
pub const EPOCH_YEAR: Year = 1348;

/// What day of [`EPOCH_YEAR`] is 1970, 1, 1.
pub const EPOCH_DOY: Doy = 287;

/// The month of the first day of 1970.
pub const EPOCH_MONTH: Month = 10;

/// The day of month which corresponds to 1970, 1, 1.
pub const EPOCH_DAY: Day = 11;

/// The equivalent of year zero in this calendar (-1).
///
/// This is a common choice, consult Wikipedia on year 0.
pub const Y0_REPLACEMENT: Year = -1;

/// The start of the calendar, based on Wikipedia and other references, year 1.
pub const Y_START: Year = 1;

/// What is the first day of the year.
pub const DOY_START: Doy = 1;

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
pub const NON_LEAP_CORRECTION: [Year; 78] = [
    1502, 1601, 1634, 1667, 1700, 1733, 1766, 1799, 1832, 1865, 1898, 1931, 1964, 1997, 2030, 2059,
    2063, 2096, 2129, 2158, 2162, 2191, 2195, 2224, 2228, 2257, 2261, 2290, 2294, 2323, 2327, 2356,
    2360, 2389, 2393, 2422, 2426, 2455, 2459, 2488, 2492, 2521, 2525, 2554, 2558, 2587, 2591, 2620,
    2624, 2653, 2657, 2686, 2690, 2719, 2723, 2748, 2752, 2756, 2781, 2785, 2789, 2818, 2822, 2847,
    2851, 2855, 2880, 2884, 2888, 2913, 2917, 2921, 2946, 2950, 2954, 2979, 2983, 2987,
];

/// A search into [`NON_LEAP_CORRECTION`].
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
#[cfg_attr(feature = "py", pyfunction)]
#[cfg_attr(not(feature = "wasm"), fn_attr(const))]
pub fn is_non_leap_correction(year: Year) -> bool {
    #[cfg(feature = "wasm")]
    {
        NON_LEAP_CORRECTION.binary_search(&year).is_ok()
    }
    #[cfg(not(feature = "wasm"))]
    {
        let mut i = 0;
        while i < NON_LEAP_CORRECTION.len() {
            if NON_LEAP_CORRECTION[i] == year {
                return true;
            }
            i += 1;
        }
        false
    }
}

/// Calculated using the 33-year rule
///
/// Taken from <https://github.com/unicode-org/icu4x/blob/3e3da0a0a34bfe3056d0f89183270ea683f4a23c/utils/calendrical_calculations/src/persian.rs#L161C1-L173C2>
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
#[cfg_attr(feature = "py", pyfunction)]
#[cfg_attr(not(feature = "wasm"), fn_attr(const))]
pub fn is_leap_year(year: Year) -> bool {
    if year >= NON_LEAP_CORRECTION[0] && is_non_leap_correction(year) {
        return false;
    }
    if year > NON_LEAP_CORRECTION[0] && is_non_leap_correction(year - 1) {
        return true;
    }

    let year = year as i64; // why?
    (25 * year + 11).rem_euclid(33) < 8
}

/// The number of days in a given year.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
#[cfg_attr(feature = "py", pyfunction)]
#[cfg_attr(not(feature = "wasm"), fn_attr(const))]
pub fn max_doy(y: Year) -> Doy {
    if is_leap_year(y) {
        366
    } else {
        365
    }
}

/// Marks a month and day in Jalali (this is intermediate type for conversions not a date).
///
/// This value is not checked most of the times and is public since how it is used is up to the
/// developers. In other words, there are no "expected" ways that this should work since it's just a
/// tuple buffer for FFIs. In the current usage for [`Date`], it is simply a day and month counter
/// starting from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "c", repr(C))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pyclass(get_all))]
pub struct Md {
    pub m: Month,
    pub d: Dom,
}

impl Md {
    /// Unwrap the month into the last day of the month for a 0 day.
    const fn resolve_zero_d(&mut self) {
        if self.d != 0 || self.m == 0 {
            return;
        }
        self.d = if self.m <= 6 {
            FIRST_HALF_MAX_DOM
        } else {
            SECOND_HALF_MAX_DOM
        };
        self.m -= 1;
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pymethods)]
impl Md {
    /// Tell what day of year is this month and day (reverse of [`Self::from_doy`]).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn to_doy(&self) -> Doy {
        let offset = match self.m as Doy {
            m @ 1..=6 => (m - 1) * FIRST_HALF_MAX_DOM as Doy,
            m @ 7..=12 => (m - 7) * SECOND_HALF_MAX_DOM as Doy + FIRST_HALF_MAX_DOY,
            _ => panic!("month larger than 12 and less than 1"),
        };
        offset + self.d as Doy
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", py_attr(pymethods, staticmethod))]
impl Md {
    /// Count how many months and days is from the start of the year (reverse of [`Self::to_doy`]).
    ///
    /// Add a month for each 30/31 days considering month days from the start of the year.
    ///
    /// Inputs must be from (1..=366) to ensure correct calculation.
    ///
    /// This returns the number of months (0 to strictly under 12) that must be added or removed and
    /// the remaining days (0 to strictly under 31).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn from_doy(doy: Doy) -> Self {
        // TODO I'm sure there is a more elegant way to do this
        let mut candidate = match doy {
            ..=FIRST_HALF_MAX_DOY => Self {
                m: (doy / FIRST_HALF_MAX_DOM as Doy) as Month,
                d: (doy % FIRST_HALF_MAX_DOM as Doy) as Dom,
            },
            _ => Self {
                m: ((doy - FIRST_HALF_MAX_DOY) / SECOND_HALF_MAX_DOM as Doy + 6) as Month,
                d: ((doy - FIRST_HALF_MAX_DOY) % SECOND_HALF_MAX_DOM as Doy) as Dom,
            },
        };

        candidate.resolve_zero_d(); // make sure day is not 0

        // (m+1 hint) first month of the year is 1 so if any days are in the new month, add
        candidate.m += 1;

        candidate
    }
}

/// Jalali equivalent of the Date in whatever measures.
///
/// Since this struct works by measuring days. Does not concern leap seconds or smaller units.
///
/// This calendar does not have a year 0! Skips over 0 to -1.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "c", repr(C))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pyclass(get_all))]
pub struct Date {
    y: Year,
    doy: Doy,
}

impl Date {
    // constructors
    // Option<Self> is generally bad for FFIs

    /// Create a new year if non-zero and leap day is considered.
    ///
    /// Call [`Self::set_doy`] on a newly created [`Self::from_y`].
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn from_y_doy(y: Year, doy: Doy) -> Option<Self> {
        let mut v = Self::from_y(y);
        if !v.set_doy(doy) {
            return None;
        }
        Some(v)
    }

    /// Create a new year if non-zero and leap day is considered.
    ///
    /// Call [`Self::set_doy`] on a newly created [`Self::from_y`].
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn from_ymd(y: Year, m: Month, d: Dom) -> Option<Self> {
        let mut v = Self::from_y(y);
        if !v.set_md(m, d) {
            return None;
        }
        Some(v)
    }

    // modifiers

    /// Ensure year is safe and correct if an operation is done.
    ///
    /// This is private since it should be impossible to create an invalid year.
    const fn ensure_y(y: &mut Year) {
        if *y == 0 {
            *y = Y0_REPLACEMENT
        }
    }

    /// Add or remove a year for each 365/366 days given returning remainder (leap correct).
    ///
    /// Basically add or remove that many days since the start of the given days. In other words,
    /// means that the doy is discarded when this function is called and the given `d` will be the
    /// sole determinator of the new day of the year. In effect, to preserve current day of year, it
    /// needs to be added back once a call is done, either by passing it as a plus to the `d` input
    /// when `toward_past=false` or as a separate call with the previous `doy` in a separate call
    /// with `toward_past=false`. See [`Self::add_d`] and [`Self::sub_d`] for both examples.
    ///
    /// `toward_past` does not only sign the year difference returned, it also effects how leaps are
    /// calculated and is necessary to be set correctly for a correct answer.
    ///
    /// NOTE that this is not simply a [`Self::set_doy`] as that function only edits the day of the
    /// year as the name suggests. This, instead, adds or subtracts past a year and takes any day as
    /// input not just something like 1..=366.
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    fn shift_d_from_start_y(&mut self, mut d: Day, toward_past: bool) {
        loop {
            let max_doy = self.max_doy() as Day;
            if d <= max_doy {
                assert!(self.set_doy(d as Doy));
                break;
            }
            d -= max_doy;
            assert!(if toward_past {
                self.sub_y(1)
            } else {
                self.add_y(1)
            });
        }
    }

    // const trait impls
    //
    // Do not use getters in these codes so they break on change of inner structures which
    // forces the implementor to revise the code

    /// Compare to another, in Rust use [`Ord`] if not in comptime.
    // not to export the ordering type cross boundaries this is not exported
    pub const fn cmp(&self, other: &Self) -> Ordering {
        if self.y < other.y {
            return Ordering::Less;
        }
        if self.y > other.y {
            return Ordering::Greater;
        }

        // year is equal this far so check doy

        if self.doy < other.doy {
            return Ordering::Less;
        }
        if self.doy > other.doy {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", py_attr(pymethods, staticmethod))]
impl Date {
    // unchecked constructors

    /// Create a new year with unchecked arguments (non-zero year, month and day).
    ///
    /// # Safety
    /// - Year must not be 0, if so, replace it with [`Y0_REPLACEMENT`] (-1)
    /// - If 12/30 (366th day of the year, the leap) is selected, the year must be a leap year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub unsafe fn from_ymd_unchecked(y: Year, m: Month, d: Dom) -> Self {
        Self {
            y,
            doy: Md { m, d }.to_doy(),
        }
    }

    /// Create a new year with unchecked arguments (non-zero year, month and day).
    ///
    /// # Safety
    /// - Year must not be 0, if so, replace it with [`Y0_REPLACEMENT`] (-1)
    /// - If 12/30 (366th day of the year, the leap) is selected, the year must be a leap year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub unsafe fn from_y_doy_unchecked(y: Year, doy: Doy) -> Self {
        Self { y, doy }
    }

    /// Return the first day of a given year (year 0 is the same as -1).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn from_y(mut y: Year) -> Self {
        Self::ensure_y(&mut y);
        Self { y, doy: 1 }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pymethods)]
impl Date {
    // getters

    /// Getter for the year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn y(&self) -> Year {
        self.y
    }

    /// Getter for the month.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn m(&self) -> Dom {
        self.md().m
    }

    /// Getter for the day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn d(&self) -> Dom {
        self.md().d
    }

    /// Getter for the day and month.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn md(&self) -> Md {
        Md::from_doy(self.doy())
    }

    /// Getter for the day of the year. What day of year it is (0..=365 for 366 max days).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn doy(&self) -> Doy {
        self.doy
    }

    /// Is this year leap (see [`is_leap_year`]).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    // mangle issue no extern "C"
    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.y)
    }

    /// Return the number of days in this year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    // mangle issue no extern "C"
    pub fn max_doy(&self) -> Doy {
        max_doy(self.y)
    }

    // setters

    /// Set the year of this month and day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn set_y(&mut self, mut y: Year) -> bool {
        Self::ensure_y(&mut y);

        if self.doy == SECOND_HALF_MAX_DOY && !is_leap_year(y) {
            return false;
        }

        self.y = y;

        true
    }

    /// Set the month of year to the given number.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn set_m(&mut self, m: Month) -> bool {
        self.set_md(m, self.d())
    }

    /// Set what day of month it should be.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn set_d(&mut self, d: Dom) -> bool {
        self.set_md(self.m(), d)
    }

    /// Set the month and day of the year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn set_md(&mut self, m: Month, d: Dom) -> bool {
        self.set_doy(Md { m, d }.to_doy())
    }

    /// Set the day of the year if in valid range (1..=366) with respect to leap years.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn set_doy(&mut self, doy: Doy) -> bool {
        if doy > self.max_doy() && doy < 1 {
            return false;
        }

        self.doy = doy;
        true
    }

    /// Add a year to the calendar.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn add_y(&mut self, y: Year) -> bool {
        self.set_y(self.y + y)
    }

    /// Add a year to the calendar.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn sub_y(&mut self, y: Year) -> bool {
        self.set_y(self.y - y)
    }

    /// Set the date to `d` days before this day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn sub_d(&mut self, d: Day) -> bool {
        let doy = self.doy() as Day;

        // since the algorithm is crude and having doy <= d makes an overflow (probably),
        // first we need to add the d and then add back the doy which acts like a carry.
        self.shift_d_from_start_y(d, true);
        self.shift_d_from_start_y(doy, false);
        true
    }

    /// Set the date to `d` days after this day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn add_d(&mut self, d: Day) -> bool {
        let doy = self.doy() as Day;
        self.shift_d_from_start_y(doy + d, false);
        true
    }

    /// Compare two dates and return true if the first is more (later) than the given.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn gt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }

    /// Compare two dates and return true if the first is more (later) than the given or equal.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn gte(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal | Ordering::Greater)
    }

    /// Compare two dates and return true if the first is less (earlier) than the given.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn lt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }

    /// Compare two dates and return true if the first is less (earlier) than the given or equal.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn lte(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less | Ordering::Equal)
    }

    // const trait impls
    //
    // Do not use getters in these codes so they break on change of inner structures which
    // forces the implementor to revise the code

    /// Check equality to another, in Rust use [`Eq`] if not in comptime.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
    #[cfg_attr(not(feature = "wasm"), fn_attr(const))]
    pub fn eq(&self, other: &Self) -> bool {
        self.y == other.y && self.doy == other.doy
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Md { m, d } = self.md();
        let y = self.y();
        write!(f, "{}/{}/{}", y, m, d)
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The not so leap year of 1348, on Dey (10) 11th when Epoch (1970/1/1) starts.
    fn epoch_start() -> Date {
        unsafe { Date::from_ymd_unchecked(1348, 10, 11) }
    }

    #[test]
    fn test_md_max() {
        assert_eq!(Md::from_doy(366).unwrap(), Md { m: 12, d: 30 })
    }

    #[test]
    fn test_md_min() {
        assert_eq!(Md::from_doy(1).unwrap(), Md { m: 1, d: 1 });
    }

    #[test]
    fn test_md_default_min() {
        assert_eq!(Md::from_doy(1).unwrap(), Md { m: 1, d: 1 });
        assert_eq!(Md::default(), Md { m: 1, d: 1 });
    }

    #[test]
    fn test_leap_aligns_with_wikipedia_list_of_33() {
        const LIST: [Year; 71] = [
            1210, 1214, 1218, 1222, 1226, 1230, 1234, 1238, 1243, 1247, 1251, 1255, 1259, 1263,
            1267, 1271, 1276, 1280, 1284, 1288, 1292, 1296, 1300, 1304, 1309, 1313, 1317, 1321,
            1325, 1329, 1333, 1337, 1342, 1346, 1350, 1354, 1358, 1362, 1366, 1370, 1375, 1379,
            1383, 1387, 1391, 1395, 1399, 1403, 1408, 1412, 1416, 1420, 1424, 1428, 1432, 1436,
            1441, 1445, 1449, 1453, 1457, 1461, 1465, 1469, 1474, 1478, 1482, 1486, 1490, 1494,
            1498,
        ];
        for i in 1210..=1500 {
            let is_leap = is_leap_year(i);
            let in_list = LIST.binary_search(&i).is_ok();
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
    fn test_unix_doy_same_as_const() {
        assert_eq!(epoch_start().doy(), 287)
    }

    #[test]
    fn test_doy_first_day_of_year() {
        assert_eq!(unsafe { Date::from_ymd_unchecked(1, 1, 1) }.doy(), 1);
    }

    #[test]
    fn test_doy_365_day_of_year() {
        assert_eq!(unsafe { Date::from_ymd_unchecked(1, 12, 29) }.doy(), 365);
    }

    #[test]
    fn test_add_doy_epoch_1348() {
        let test = |offset: Day, (y, m, d): (Year, Month, Dom)| {
            let mut v = epoch_start();
            v.add_d(offset);
            assert_eq!(v.doy() as Day, (287 + offset));
            assert_eq!(v.doy as Day, (287 + offset));
            assert_eq!(v, unsafe { Date::from_ymd_unchecked(y, m, d) });
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
        let test = |offset: Day, (y, m, d): (Year, Month, Dom)| {
            let mut v = epoch_start();
            v.add_d(78 + offset);
            assert_eq!(v.doy() as Day, offset);
            assert_eq!(v.doy as Day, offset);
            assert_eq!(v, unsafe { Date::from_ymd_unchecked(y, m, d) });
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
    fn test_add_186_new_year() {
        let mut v = Date::from_y(1350);
        assert_eq!(v.doy(), 1);
        assert_eq!(v, unsafe { Date::from_ymd_unchecked(1350, 1, 1) });
        assert_eq!(v, Date { y: 1350, doy: 1 });
        assert_eq!(v.d(), 1);

        v.add_d(184);
        assert_eq!(v.doy(), 185);
        assert_eq!(v, Date { y: 1350, doy: 185 });
        assert_eq!(v, unsafe { Date::from_ymd_unchecked(1350, 6, 30) });
        assert_eq!(v.d(), 30);

        v.add_d(1);
        assert_eq!(v.doy(), 186);
        assert_eq!(v, Date { y: 1350, doy: 186 });
        assert_eq!(v, unsafe { Date::from_ymd_unchecked(1350, 6, 31) });
        assert_eq!(v.d(), 31);

        v.add_d(1);
        assert_eq!(v.doy(), 187);
        assert_eq!(v, Date { y: 1350, doy: 187 });
        assert_eq!(v, unsafe { Date::from_ymd_unchecked(1350, 7, 1) });
    }

    // Since the library is `cdylib`, Rust doesn't test the snippets in the code, this is a manual
    // copy of the code mentioned in the readme.
    #[test]
    fn test_readme() {
        let fixed_point = Date::from_ymd(1404, 2, 13).unwrap(); // 2025, 5 (May), 3
        let mut new = fixed_point.clone();
        new.add_d(11);
        assert_eq!((new.y(), new.m(), new.d()), (1404, 2, 24));
    }
}
