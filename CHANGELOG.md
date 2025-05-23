# `0.4.0`

## Fix

- `is_valid_doy` not being available in Python

## Add

- `std` and `const` features for more control
- `Date::epoch` function to provide the Epoch date for unconst envs (like wasm)
- `Date::from_d_past_epoch` and `Date::from_d_before_epoch`
- `FromEpochDelta` trait and implementations for `core::time::Duration`,
  `std::time::SystemTime`
- build tests in CMake for all the possible combination of features, docs & test

# `0.3.0`

## Fix

- days in the second half of the year being calculated incorrectly in `Md`
- `Date::set_doy` not being strict enough and considering leap years
- `shift_day` internals not working correctly on occasions

## Change

- the feature invokation so it throws when mutually exclusive features are used
- all the functions to use explicit `export_name` with struct prefix
- Md API. It's totally reworked to be more strict

## Add

- `#[must_use]` for setters which may fail
- implementation of traits like `From` and `Default`
- [`Date::`]`is_valid_doy` for checking ordinal (day of year) for a given year

# `0.2.0`

## Change

- `extern "C"` and disable `repr(C)` in any non-"c"-feature 
- `const` and add it to any non-"wasm"-feature
- location of `Date::*` consts to the root crate

## Add

- constants representing the Jalali/Persian date of the Epoch start (1970, 1, 1)
- `Date::cmp` (`const fn`) for comparisons in comptime
- `jelal_proc::fn_attr` with `const`, `unconst`, `extern` and `unextern`
