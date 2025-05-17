# `0.2.0`

## Change

- `extern "C"` and disable in any non-"c"-feature
- `const` and add it to any non-"wasm"-feature
- location of `Date::*` consts to the root crate

## Add

- constants representing the Jalali/Persian date of the Epoch start (1970, 1, 1)
- `Date::cmp` (`const fn`) for comparisons in comptime
- `jelal_proc::fn_attr` with `const`, `unconst`, `extern` and `unextern`
