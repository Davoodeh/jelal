# `0.2.0`

## Change

- `extern "C"` and disable in any non-"c"-feature
- `const` and add it to any non-"wasm"-feature

## Add

- `Date::cmp` (`const fn`) for comparisons in comptime
- `jelal_proc::fn_attr` with `const`, `unconst`, `extern` and `unextern`
