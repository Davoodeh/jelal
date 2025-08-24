# Future (probably `0.5.0`)

## Change

- Default `headers` task to `cffi`.

## Remove

- deprecated `MonthDay::LEAP_LAST_MONTH_DAY_MAX`.
- deprecated `MonthDay::NON_LEAP_LAST_MONTH_DAY_MAX`.

# `0.4.2`

## Add

- `Ordering` to `generated` module
- `cmp` functions that are more primitive than their `ext_cmp` counterpart as a
  side-effect of `jelal_proc` removal

## Remove

- `jelal_proc` crate (version 0.5.0)

## Fix

- A bug in `codegen` where `Self` typed parameters in methods (not `self`) would
  generate invalid types which also effected functions. This bug does not effect
  previous versions but may effect codes in developement so an update is
  suggested.

# `0.4.1`

## Add

- Support for `deprecated` in `codegen`.
- Experimental support for a custom `codegen` replacement for `cbindgen` (namely
  `cffi`, use `make cffi` to test) to generate the C headers.
- `tm` (`time.h` struct) and `to_jtm` for easier creation of Jalali correct `tm`
  structs from `Date`
- `MonthDay::NON_LEAP_LAST_MAX_DAY` and deprecate what it replaces
  `MonthDay::NON_LEAP_LAST_MONTH_DAY_MAX` to be more aligned with other
  `*MAX_DAY` constants
- `MonthDay::LEAP_LAST_MAX_DAY` and deprecate what it replaces
  `MonthDay::LEAP_LAST_MONTH_DAY_MAX` to be more aligned with other `*MAX_DAY`
  constants

## Change

- `MonthDay::LEAP_LAST_MONTH_DAY_MAX` and
  `MonthDay::NON_LEAP_LAST_MONTH_DAY_MAX` to deprecated. Use the added
  alternatives.
- codegen:
  - `ImplConst` items are now visited beforetheir duplicate global were
    produced.
  - `codegen` the crate now holds multiple binaries and the previous `main.rs`
    is renamed as `codegen.rs`.
  - to collapse documents (just a visual change for now)
  - sift to whitelist `cfg` and `cfg_attr` attributes
  - to pass `cfg` attributes to `pymodule` (prevents `cfg` misbehavior)
  - allow `unused_imports`
  - use `crate::ffi::*` automatically

# `0.4.0`

Total rewrite of the library with an emphasis on new types. This version
actually makes the library usable and out of the alpha to somewhat of a beta
version. Backward compatibility with the previous versions is no more hence this
changelog starts fresh.

To see the changelog of the previous versions use Git or any time machine.
