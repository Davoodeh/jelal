//! Holds all the FFI related code when the respective configurations are in place.
//!
//! Except the values documented below, no functionalities is guaranteed to exist on this module.
//! See the source code for the internal `codegen` binary which creates the respective automatically
//! included `generated.rs` file here.
//!
//! Generally, `generated.rs` is a re-export of everything in this crate with multiple signatures
//! and other facilities that help bindgen tools read the code. The most notable difference between
//! functions exported here with the originals is the heavy usage of cloning and the removal of "new
//! types" in favor of primitives and delegating the conversion to `Into` (for receiving unsafe
//! inputs) and `transmute` (for outputing safe values).
//!
//! The aim of these binds is first and foremost the ease of usage.

mod generated;

#[cfg(feature = "c")]
pub(crate) use core::ffi::{c_char, c_int, c_long};

#[cfg(not(doc))]
pub use generated::*;

/// Equivalent to `struct tm` in standard `time.h`.
///
/// This is essentially `libc::tm` (except `Copy` and extra traits to be like other structs in the
/// library). As for any C struct, the usage, creation and behavior is a matter of suggestion rather
/// than rule but this library treats this struct as in the following. In short, the suggested
/// method to read it, is exactly as in C except for year which does not have an offset.
///
/// Assuming it is a Jalali date:
/// - [`Self::tm_year`]: represents the year as mentioned in its documents (non-zero).
/// - [`Self::tm_mon`]: represents month (0-11) where 0 is the first month of the year.
/// - [`Self::tm_yday`]: represents the ordinal where 0 is the first day of the year.
/// - [`Self::tm_mday`]: exactly as in `jelal`.
///
/// The rest are unsupported (at least for now) and will default to 0.
///
/// The main creation method for the Jalali interpretation of this struct is
/// [`crate::Date::to_jtm`]. There are no `from_jtm` equal since there are many ways interprete how
/// this should be done, (based on ordinal `yday` or `year`, `mon`, `mday` fields to name two).
#[cfg(feature = "c")]
#[derive(Clone, Eq, PartialEq)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct tm {
    /// Consult C documents. Not supported in `jelal`, defaults to 0.
    pub tm_sec: c_int,
    /// Consult C documents. Not supported in `jelal`, defaults to 0.
    pub tm_min: c_int,
    /// Consult C documents. Not supported in `jelal`, defaults to 0.
    pub tm_hour: c_int,
    /// Day of month (1-31), equal to [`crate::MonthDay::day`].
    pub tm_mday: c_int,
    /// Zero based month ID (0-11), equal to [`crate::Month`] minus 1 if it's Jalali.
    pub tm_mon: c_int,
    /// Year, or to be exact: "Year - 1900".
    ///
    /// For normal calculations this is often deducted by 1900.  In Jalali, one may ignore the -1900
    /// offset which makes it [`crate::Year`] if not zero.
    pub tm_year: c_int,
    /// The day of the week.
    ///
    /// As of now, this is not set in by `jelal`, defaults to 0.
    pub tm_wday: c_int,
    /// The ordinal, day of year indexed from 0, equal to [`crate::Ordinal`] - 1.
    pub tm_yday: c_int,
    /// Consult C documents. Not supported in `jelal`, defaults to 0.
    pub tm_isdst: c_int,
    /// Consult C documents. Not supported in `jelal`, defaults to 0.
    pub tm_gmtoff: c_long,
    /// Consult C documents. Not supported in `jelal`, defaults to 0 (null).
    pub tm_zone: *const c_char,
}

#[cfg(feature = "c")]
impl tm {
    /// Create a default (invalid) value.
    ///
    /// This is not the `default` nor public, not to encourage its hasty creation as it is an
    /// invalid value.
    pub(crate) const fn new_zero() -> Self {
        Self {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: core::ptr::null(),
        }
    }
}
