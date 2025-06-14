#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_main, no_std)]
#![cfg_attr(feature = "py", allow(unsafe_op_in_unsafe_fn))] // python, staticmethods and unsafe new

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
#[allow(unused_imports)] // conditionally used
extern crate jelal_proc;

use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

mod traits;

pub use traits::*;

jelal_proc::forbid_mutual_feature!("const", "wasm");

#[cfg(feature = "py")]
use pyo3::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "py")]
#[pymodule]
fn jelal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_non_leap_correction, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_doy, m)?)?;
    m.add_function(wrap_pyfunction!(is_leap_year, m)?)?;
    m.add_function(wrap_pyfunction!(max_doy, m)?)?;
    m.add_class::<Md>()?;
    m.add_class::<Date>()?;
    Ok(())
}

#[cfg(all(not(test), not(any(feature = "wasm", feature = "std"))))] // suppress duplicate error
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Counter for years.
pub type Year = i32;

/// Counter for months of a year.
pub type Month = u16;

/// Counter for consecutive days
pub type Day = u32;

/// Counter for days of a week or month.
pub type Dom = u16;

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
pub const EPOCH_DOM: Dom = 11;

/// 1970, 1, 1 in Jalali.
#[cfg(feature = "const")]
pub const EPOCH_DATE: Date = Date::epoch();

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
#[cfg_attr(feature = "const", fn_attr(const))]
pub fn is_non_leap_correction(year: Year) -> bool {
    #[cfg(not(feature = "const"))]
    {
        NON_LEAP_CORRECTION.binary_search(&year).is_ok()
    }
    #[cfg(feature = "const")]
    {
        if year < NON_LEAP_CORRECTION[0]
            || year > NON_LEAP_CORRECTION[NON_LEAP_CORRECTION.len() - 1]
        {
            return false;
        }

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
#[cfg_attr(feature = "const", fn_attr(const))]
pub fn is_leap_year(year: Year) -> bool {
    if year >= NON_LEAP_CORRECTION[0] && is_non_leap_correction(year) {
        return false;
    }

    // no previous year so assume no
    let Some(prev) = year.checked_sub(1) else {
        return false;
    };

    if year > NON_LEAP_CORRECTION[0] && is_non_leap_correction(prev) {
        return true;
    }

    let year = year as i64; // why?
    (25 * year + 11).rem_euclid(33) < 8
}

/// The number of days in a given year.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
#[cfg_attr(feature = "py", pyfunction)]
#[cfg_attr(feature = "const", fn_attr(const))]
pub fn max_doy(y: Year) -> Doy {
    if is_leap_year(y) {
        366
    } else {
        365
    }
}

/// Check if the given day can be a valid day number (ordinal) in the given year.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "c", unsafe(no_mangle), fn_attr(extern "C"))]
#[cfg_attr(feature = "py", pyfunction)]
#[cfg_attr(feature = "const", fn_attr(const))]
pub fn is_valid_doy(y: Year, doy: Doy) -> bool {
    doy > 0 && doy < SECOND_HALF_MAX_DOY || (doy == SECOND_HALF_MAX_DOY && is_leap_year(y))
}

/// A month and day of the month in a sample leap year of Jalali.
///
/// This is basically between a dumb tuple and a basic intermediate struct for languages that do not
/// support functions returning tuples.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "c", repr(C))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pyclass(get_all))]
pub struct Md {
    m: Month,
    d: Dom,
}

impl Md {
    // Construct a new instance but since Option<Self> is not wise to export, keep hidden.

    /// Create an instance if the day exists in the given month (assuming leap year).
    pub const fn from_md(m: Month, d: Dom) -> Option<Self> {
        match (m, d) {
            (1..=12, 1..=30) | (1..=6, 31) => Some(Self { m, d }),
            _ => None,
        }
    }

    /// Create an instance from the day of the year if not larger than a leap year count.
    pub const fn from_doy(doy: Doy) -> Option<Self> {
        if doy > SECOND_HALF_MAX_DOY || doy < 1 {
            return None;
        }

        Some(unsafe { Self::from_doy_unchecked(doy) })
    }

    /// [`Self::from_doy`] but with no checks.
    ///
    /// # Safety
    /// - Must be a valid day of year (1..=366).
    pub const unsafe fn from_doy_unchecked(doy: Doy) -> Self {
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

    /// Unwrap the month into the last day of the month for a 0 day.
    ///
    /// This is used for mathematical computations and internally. Do not use in creating new
    /// instances.
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
    /// Getter for the month.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn m(&self) -> Dom {
        self.m
    }

    /// Getter for the day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn d(&self) -> Dom {
        self.d
    }

    /// Tell what day of year is this month and day (reverse of [`Self::from_doy`]).
    ///
    /// This will return constant 1 if the initialization of this struct has failed and the values
    /// are invalid.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn doy(&self) -> Doy {
        let offset = match self.m as Doy {
            m @ 1..=6 => (m - 1) * FIRST_HALF_MAX_DOM as Doy,
            m @ 7..=12 => (m - 7) * SECOND_HALF_MAX_DOM as Doy + FIRST_HALF_MAX_DOY,
            _ => return 1,
        };
        offset + self.d as Doy
    }

    /// Change the month if the day is a valid day in that month (assuming leap year).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_set_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_m(&mut self, m: Month) -> bool {
        match Self::from_md(m, self.d) {
            Some(v) => {
                *self = v;
                true
            }
            None => false,
        }
    }

    /// Change the day if the day is a valid day in that month (assuming leap year).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_set_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_d(&mut self, d: Dom) -> bool {
        match Self::from_md(self.m, d) {
            Some(v) => {
                *self = v;
                true
            }
            None => false,
        }
    }

    /// Count how many months and days is from the start of the year (reverse of [`Self::doy`]).
    ///
    /// Add a month for each 30/31 days considering month days from the start of the year.
    ///
    /// Inputs must be from (1..=366) to ensure correct calculation.
    ///
    /// This returns the number of months (0 to strictly under 12) that must be added or removed and
    /// the remaining days (0 to strictly under 31).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_set_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_doy(&mut self, doy: Doy) -> bool {
        match Self::from_doy(doy) {
            Some(v) => {
                *self = v;
                true
            }
            None => false,
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", py_attr(pymethods, staticmethod))]
impl Md {
    /// Create a new valid instance (1, 1), when not const-time use [`Default`].
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "md_default"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn default() -> Self {
        Self { m: 1, d: 1 }
    }
}

impl From<Date> for Md {
    fn from(value: Date) -> Self {
        value.md()
    }
}

impl Default for Md {
    fn default() -> Self {
        Self::default()
    }
}

impl From<(Month, Dom)> for Md {
    fn from((m, d): (Month, Dom)) -> Self {
        Self { m, d }
    }
}

/// Jalali equivalent of the Date in whatever measures.
///
/// Since this struct works by measuring days. Does not concern leap seconds or smaller units.
///
/// This calendar does not have a year 0! Skips over 0 to -1. The minimum and maximum years
/// representable are the same as [`Year::MIN`] and [`Year::MAX`].
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
    #[cfg_attr(feature = "const", fn_attr(const))]
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
    #[cfg_attr(feature = "const", fn_attr(const))]
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
    #[cfg_attr(feature = "const", fn_attr(const))]
    fn shift_d_from_start_y(&mut self, mut d: Day, toward_past: bool) {
        if d == 0 {
            return;
        }

        // set to 1 temporarily not to upset year functions in a leap event
        // This function overrides the doy so does not matter if the original value is lost
        if !self.set_doy(1) {
            unreachable!()
        }

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
    #[cfg_attr(feature = "c", unsafe(export_name = "date_from_ymd_unchecked"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub unsafe fn from_ymd_unchecked(y: Year, m: Month, d: Dom) -> Self {
        Self {
            y,
            doy: unsafe { Md::from_md(m, d).unwrap_unchecked() }.doy(),
        }
    }

    /// Create a new year with unchecked arguments (non-zero year, month and day).
    ///
    /// # Safety
    /// - Year must not be 0, if so, replace it with [`Y0_REPLACEMENT`] (-1)
    /// - If 12/30 (366th day of the year, the leap) is selected, the year must be a leap year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(
        feature = "c",
        unsafe(export_name = "date_from_y_doy_unchecked"),
        fn_attr(extern "C"),
    )]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub unsafe fn from_y_doy_unchecked(y: Year, doy: Doy) -> Self {
        Self { y, doy }
    }

    /// Return the first day of a given year (year 0 is the same as -1).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_from_y"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn from_y(mut y: Year) -> Self {
        Self::ensure_y(&mut y);
        Self { y, doy: 1 }
    }

    /// Create an instance with given days past the Epoch.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_from_d_past_epoch"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn from_d_past_epoch(d: Day) -> Self {
        let mut epoch = Self::epoch();
        epoch.add_d(d);
        epoch
    }

    /// Create an instance with given days past the Epoch.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(
        feature = "c",
        unsafe(export_name = "date_from_d_before_epoch"),
        fn_attr(extern "C")
    )]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn from_d_before_epoch(d: Day) -> Self {
        let mut epoch = Self::epoch();
        epoch.sub_d(d);
        epoch
    }

    /// Return the Epoch date (for unconst environments).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_epoch"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn epoch() -> Self {
        Self {
            y: EPOCH_YEAR,
            doy: EPOCH_DOY,
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "py", pymethods)]
impl Date {
    // getters

    /// Getter for the year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_y"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn y(&self) -> Year {
        self.y
    }

    /// Getter for the month.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn m(&self) -> Dom {
        self.md().m
    }

    /// Getter for the day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn d(&self) -> Dom {
        self.md().d
    }

    /// Getter for the day and month.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_md"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn md(&self) -> Md {
        // SAFETY: the day of the year is always valid in this struct
        unsafe { Md::from_doy_unchecked(self.doy()) }
    }

    /// Getter for the day of the year. What day of year it is (0..=365 for 366 max days).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn doy(&self) -> Doy {
        self.doy
    }

    /// Is this year leap (see [`is_leap_year`]).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_is_leap_year"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.y())
    }

    /// Return the number of days in this year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_max_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn max_doy(&self) -> Doy {
        max_doy(self.y())
    }

    /// Check if a given day of year (ordinal) can be a valid day for this year or not.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_is_valid_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn is_valid_doy(&self) -> bool {
        is_valid_doy(self.y(), self.doy())
    }

    /// Return how many days past since Epoch (0 if its before the Epoch).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_d_past_epoch"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn d_past_epoch(&self) -> Day {
        // TODO make this conditional, if a time library is available, use that instead as it should
        // be more efficient
        let last_y = self.y();
        let mut d: Day = 0;

        // const loop from epoch to the given year
        let mut current_y = EPOCH_YEAR;
        while current_y < last_y {
            d = d.saturating_add(max_doy(current_y) as Day);
            current_y += 1;
        }

        let doy = self.doy();
        if doy >= EPOCH_DOY {
            d = d.saturating_add((self.doy() - EPOCH_DOY) as Day);
        } else {
            d = d.saturating_sub((EPOCH_DOY - self.doy()) as Day);
        }

        d
    }

    // setters

    /// Set the year of this month and day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_set_y"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_y(&mut self, mut y: Year) -> bool {
        Self::ensure_y(&mut y);

        if !is_valid_doy(y, self.doy()) {
            return false;
        }

        self.y = y;

        true
    }

    /// Set the month of year to the given number.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_set_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_m(&mut self, m: Month) -> bool {
        self.set_md(m, self.d())
    }

    /// Set what day of month it should be.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_set_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_d(&mut self, d: Dom) -> bool {
        self.set_md(self.m(), d)
    }

    /// Set the month and day of the year.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_set_md"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_md(&mut self, m: Month, d: Dom) -> bool {
        match Md::from_md(m, d) {
            Some(v) => self.set_doy(v.doy()),
            None => false,
        }
    }

    /// Set the day of the year if in valid range (1..=366) with respect to leap years.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_set_doy"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn set_doy(&mut self, doy: Doy) -> bool {
        if !is_valid_doy(self.y(), doy) {
            return false;
        }

        self.doy = doy;
        true
    }

    /// Add a year to the calendar (saturating).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_add_y"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn add_y(&mut self, y: Year) -> bool {
        self.set_y(self.y.saturating_add(y))
    }

    /// Sub a year to the calendar (saturating).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_sub_y"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn sub_y(&mut self, y: Year) -> bool {
        self.set_y(self.y.saturating_sub(y))
    }

    /// Set the date to `m` months before this day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_sub_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn sub_m(&mut self, mut m: Month) -> bool {
        // see add_m variant for an introduction to understanding this function
        let this_m = self.m();
        let mut new = Self::from_y(self.y());
        if this_m <= m {
            // since doy is 1 this won't panic
            if !new.add_y(m as Year / 12) {
                unreachable!()
            }
            m = m % 12 + 12;
        }

        // `this_m - m` guaranteed not to underflow for the checks above
        if m != 0 && !new.set_md(this_m - m, self.d()) {
            return false;
        }

        *self = new;
        true
    }

    /// Set the date to `m` months after this day (saturating).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_add_m"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    #[must_use]
    pub fn add_m(&mut self, mut m: Month) -> bool {
        // by creating a new instance, this is essentially atomic... if any setter fails: no change
        // calcualate the new year and make sure its supported
        let mut new = Self::from_y(self.y().saturating_add(m as Year / 12));
        m %= 12;
        if m == 0 {
            m = 12;
        }
        // doy is 1 which means the first day of any month if changed and m%12 implies valid m
        if !new.set_md(m, self.d()) {
            return false;
        }

        *self = new;
        true
    }

    /// Set the date to `d` days before this day.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_sub_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn sub_d(&mut self, d: Day) -> bool {
        let doy = self.doy() as Day;

        // since the algorithm is crude and having doy <= d makes an overflow (probably),
        // first we need to add the d and then add back the doy which acts like a carry.
        self.shift_d_from_start_y(d, true);
        self.shift_d_from_start_y(doy, false);
        true
    }

    /// Set the date to `d` days after this day (saturating).
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_add_d"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn add_d(&mut self, d: Day) -> bool {
        let doy = self.doy() as Day;
        self.shift_d_from_start_y(doy.saturating_add(d), false);
        true
    }

    /// Compare two dates and return true if the first is more (later) than the given.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_gt"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn gt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }

    /// Compare two dates and return true if the first is more (later) than the given or equal.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_gte"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn gte(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal | Ordering::Greater)
    }

    /// Compare two dates and return true if the first is less (earlier) than the given.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_lt"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn lt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }

    /// Compare two dates and return true if the first is less (earlier) than the given or equal.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_lte"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
    pub fn lte(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less | Ordering::Equal)
    }

    // const trait impls
    //
    // Do not use getters in these codes so they break on change of inner structures which
    // forces the implementor to revise the code

    /// Check equality to another, in Rust use [`Eq`] if not in comptime.
    #[cfg_attr(feature = "wasm", wasm_bindgen)]
    #[cfg_attr(feature = "c", unsafe(export_name = "date_eq"), fn_attr(extern "C"))]
    #[cfg_attr(feature = "const", fn_attr(const))]
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

    #[test]
    fn test_set_doy_leap_for_leap() {
        let mut d = Date { y: 1403, doy: 365 };
        assert!(d.is_leap_year());
        assert!(d.set_doy(366));
        assert!(d.doy() == 366);
    }

    #[test]
    fn test_set_doy_leap_for_non_leap() {
        let mut d = Date { y: 1404, doy: 365 };
        assert!(!d.is_leap_year());
        assert!(!d.set_doy(366));
        assert!(d.doy() == 365);
    }

    #[test]
    fn test_add_12_month_leap_invalid() {
        let mut d = Date::from_ymd(1403, 12, 30).unwrap();
        assert!(d.y() == 1403);
        assert!(d.m() == 12);
        assert!(d.d() == 30);
        assert!(d.doy() == 366);
        assert!(!d.add_m(12));
        assert!(d.y() == 1403);
        assert!(d.m() == 12);
        assert!(d.d() == 30);
        assert!(d.doy() == 366);
    }

    #[test]
    fn test_add_12_month_valid() {
        let mut d = Date::from_ymd(1403, 12, 29).unwrap();
        assert!(d.y() == 1403);
        assert!(d.m() == 12);
        assert!(d.d() == 29);
        assert!(d.doy() == 365);
        assert!(d.add_m(12));
        assert!(d.y() == 1404);
        assert!(d.m() == 12);
        assert!(d.d() == 29);
        assert!(d.doy() == 365);
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

    #[test]
    fn test_is_leap_year_min_i32() {
        assert!(!is_leap_year(i32::MIN))
    }

    #[test]
    fn test_d_past_epoch() {
        // past
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR - 1, EPOCH_MONTH - 1, EPOCH_DOM - 1)
                .unwrap()
                .d_past_epoch(),
            0
        );
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR, EPOCH_MONTH - 1, EPOCH_DOM - 1)
                .unwrap()
                .d_past_epoch(),
            0
        );
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR, EPOCH_MONTH, EPOCH_DOM - 1)
                .unwrap()
                .d_past_epoch(),
            0
        );
        // same
        assert_eq!(Date::epoch().d_past_epoch(), 0);

        // future
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR, EPOCH_MONTH + 1, EPOCH_DOM)
                .unwrap()
                .d_past_epoch(),
            30
        );
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR, 12, 29).unwrap().d_past_epoch(),
            (365 - EPOCH_DOY) as Day
        );
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR + 1, 1, 1).unwrap().d_past_epoch(),
            (365 - EPOCH_DOY + 1) as Day
        );
        assert_eq!(
            Date::from_ymd(EPOCH_YEAR + 1, 1, 2).unwrap().d_past_epoch(),
            (365 - EPOCH_DOY + 2) as Day
        );
    }
}
