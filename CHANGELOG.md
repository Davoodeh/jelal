# Future (probably `0.5.0`)

## Change

- Default `headers` task to `cffi`.

# `0.4.1`

## Add

- Support for `deprecated` in `codegen`.
- Experimental support for a custom `codegen` replacement for `cbindgen` (namely
  `cffi`, use `make cffi` to test) to generate the C headers.

## Change

- codegen:
  - `ImplConst` items are now visited beforetheir duplicate global were
    produced.
  - `codegen` the crate now holds multiple binaries and the previous `main.rs`
    is renamed as `codegen.rs`.
  - to collapse documents (just a visual change for now)

# `0.4.0`

Total rewrite of the library with an emphasis on new types. This version
actually makes the library usable and out of the alpha to somewhat of a beta
version. Backward compatibility with the previous versions is no more hence this
changelog starts fresh.

To see the changelog of the previous versions use Git or any time machine.
