[![Crates.io Version](https://img.shields.io/crates/v/jelal)](https://crates.io/crates/jelal)
[![docs.rs](https://img.shields.io/docsrs/jelal)](https://docs.rs/jelal/latest/jelal/)

---

> "_ASAD_!"  -[Captian Jelal](https://soundcloud.com/alisorena/nakhodajelal)

---

Jelal is a modern and lean Jalali (Persian/Iranian) calendar for developers.

Supported in:
- Rust
- JavaScript/TypeScript (using `wasm-pack`)
- Python (using `maturin`)
- C (using `cbindgen`)

Jump to [Building and Usage](#building-and-usage) for installation.

## Introduction

This library does not reinvent the wheel nor aims to be the fastest or the most
correct for all the ages (-99999 to year 99999). The goal of this project is to
be light and fairly correct by doing the absolute minimum and working mostly
around the contemporary centuries at best. Just to remind myself that doing less
is sometimes doing more. This approach makes for verbose configurations since
the exported functionalities use established tools like `cbindgen` and
`wasm-pack` which do not support macro expansion even though this is not an
issue since the size and scope of the library is really slim.

Most attempts at implementing of Jalali underestimate how much work it takes to
implement a calendar from scratch with libc level of features including
timezones, localization and else. Knowing that a calendar only actually concerns
"days," especially a calendar like Jalali which is a mirror of Gregorian (both
solar calendars). With that in mind, this crate tries to simply sync Jalali to
Gregorian by days which effectively dodges all of the complexity, issues and
features. Simply use your default Gregorian time libraries including libc's and
upon needing a date conversion do that by calling this to get the right idea
about what month and day is it in Jalali and format accordingly. The rest (day
of week, time of day, timezone and etc.) is exactly as your Gregorian library
suggests.

As such conversions are rarely actually needed, there is no point in being fast
either. The base of all the calculations will be the mature Gregorian
implementations and this little structs can be used just for the IO and other
interactions. Hence, this crate simply works and aims to be a single reference
for how exactly calculations about this calendar must be since there is nothing
but hard to read papers from 2000's on this issue, all text and not a single
formula.

Regardless, this crate is `no_std` for the ease of use cross boundaries.
Everything is in `const` so usage in compile-time is a viable option. Beyond
that, no dependencies is required! Once `no_core` is stable, core will be
stripped away too.

### Example and Design Motivation

This library does not calculate the date directly and works with a "Day Delta"
approach. Since both the Gregorian and Persian calendars are solar and almost
identical in most aspects, the aim for this library is to just do the least
possible logical process to convert the two different days (even if it results
in a somewhat primitive algorithm). Below an example of this approach is
provided.

Consider a fixed point in both calendars with verified equals (i.e Saturday 3rd
of May (5) 2025 is 13th of Ordibehesht (2) 1404, the date of the start of the
project).  To convert any other date based on that time, a "Day Delta" or day
difference with that day in Gregorian is needed. This is a trivial problem which
is done in almost every date and time library for most languages. In this
example, we try to convert Wednesday May 14 2025 to Jalali. Since this is
trivial, no other library is used to calculate the 11 day difference. Granted,
we run that in this library to add eleven days to the initial fixed-point like
below, which yields the correct results.

<!-- edit test_readme -->
```rust
let fixed_point = Date::from((1404, 2, 13)); // 2025, 5 (May), 3
assert_eq!(fixed_point.add_days(11), Date::from((1404, 2, 24)));
```

As explained above, this has the benefit of being simple and removes all the
responsiblity of calculating and matching the dates without engaging
localization, second counting or any other complication. The API is not the most
easy to use but the goal is not its user-friendliness and rather the ease of its
use in FFIs, in cross-boundaries and in environments with limited features with
one single code base.

#### Regarding `codegen` and Alternatives

This crate uses `codegen` local binary to generate the FFI files. The output is
already included in the sources with each commit so there is no need to run it
but to know the reason this square wheel exists while the general "calendar
parts" avoid re-inventing the wheel (as mentioned before) is important.
The output of the crate is some simple Rust code that is easy to work with for
tools like `cbindgen` and provides some occasional solutions for their
limitations (like "conditional attribute not found when using `pymethods`" in
`pyo3`).

Major crates like `icu4x` use `uniffi` or `diplomat` to create binds. `codegen`
is a hack compared to the aforementioned tools. However, after experiments,
these sophisticated tools proved to be excess in binding generation and
inconvenient in packaging department as their goals is to provide "good enough
bindings for many languages" rather than a "one click magic everything for one
language".

So in short, `codegen` was written to automatically create FFI compatible Rust
code from the main Rust modules without the clusters of `cfg` attributes and
burden of keeping everything in sync. All the while keeping the convenience of
tools like `maturin` and `wasm-pack`.

For more information, see the crate. This crate was not intended for publication
(at least for now) since it is yet to prove useful for any purpose and style of
coding beyond this project's.

## Building and Usage

If you are in a hurry, run the following command with either of the `verb`s
below:

```sh
cargo make $VERB
```

Important final verbs for the rushed:
- `build`: Rust build
- `cbindgen` (C): Creates the C compatible library and C headers for the C
  language (using nightly for the bindgen part)
- `wasm-pack` (JS/TS): Creates the library and its package with `wasm-pack`
- `maturin` (Python): Creates the library and its package with `maturin`

### Longer explanation

To build the library one could use a simple `cargo build` and generate the
headers using tools like `cbindgen` and else, which are not explained in this
README. If the user is not acquainted with the correct tools, the defaults are
prepared in `Makefile.toml` which can be used with `cargo build` (`makers`) to
build and install the project.

First, if `cargo-make` is not installed, install it.

```sh
cargo install cargo-make
```

After ensuring `cargo-make`, to build the project call:


```sh
# This is the same as calling `cargo build`.
cargo make build
```

To see the possible build tasks ("targets" in `make` jargon), run the following:

```sh
cargo make --list-category-steps Jelal
```

### Building for Release

To build for release in `cargo make`, simply pass `--profile release` right
after `make`:

```sh
cargo make --profile release $REST_OF_ARGUMENTS
```

As in `cargo`, the default is set to `debug` or `development` but `release` is
supported. Any invalid profile will behave as `debug`.

Note that flags in this mode are set to treat any warning as hard error.

### Build Requirements

Besides Rust utilities like `cargo` and `rustup`, the following are optionally
required. The dependencies will be automatically installed if using
`cargo-make`:
- `build`: no further dependencies
- `cbindgen`: requires `cbindgen 0.29.0` and a `nightly` since attributes for
  this tool are behind `cfg_attr` (`cargo install cbindgen`)
- `build-wasm`: `wasm32-unknown-unknown` tuple (`rustup target add
  wasm32-unknown-unknown`)
- `wasm-pack`: `build-wasm` requirements and also `wasm-pack` (`cargo install
  wasm-pack`)
- `maturin`: requires `maturin` (`cargo install maturin`)

The requirements for the tools mentioned above can be found on their resources.

The `makers` scripts are heavily dependent on the environment variables defined
in the `Makefile.toml`. Read the sources for their requirements.

### Installation

As of now, only C/Rust, C headers and Python package (via `pip`) can be
installed automatically.

#### C or Dynamic Library Installation

After building the library for C or Rust, run the following (with correct
priviledges) to install the library and optionally the headers (if generated).

```sh
cargo make install-lib

# or with `sudo`
sudo -E cargo make install-lib
```

Note that a common pitfall on dynamically linking is forgetting to set
`LD_LIBRARY_PATH` (more on that online, a good default is
`LD_LIBRARY_PATH="/usr/local/lib"`) which may make you think that the install
command had silently failed!

Also, the `rustup` command is usually installed by the user and in the `$HOME`
directory hence the `-E` after `sudo`.

#### Python Wheel Package

Like before, after building the `whl` for Python, run the following command to
install it (make sure your environment is configured):

```sh
cargo make install-wheel
```

This will install the most recently built `whl` file in your target directory.
If another behavior is needed, you should manually run the installation
commands.

## Contribute

The feature set seems to be enough for the goal of the project and the most
welcome contributions as for now are tests! The first and foremost important
goal of this project is to be "correct" rather than anything else. The languages
can be expanded further as well. For more TODO see the sources and the section
below. Please keep the simple conventions in the section ahead in mind while
developing.

### Style and Notes

- Whenever possible, order values, codes and all else in year, month, day order.
- Avoid abbrevations.
- Since `Makefile.toml` searches `Cargo.toml` `features` section, keep that
  simple, formatted and keep each key-value pair on one line with no `.`
  notation.
- Do not use any proc-macro if not necessary (`jelal_proc` was here before the
  `codegen` crate and will probably be removed)
- Keep changes in as small as possible commits with imperative short
  descriptions.
- Mention the changes in the changelog in the order of "Change", "Remove" and
  "Add".
- Run `cargo make pre-commit` before committing which does a lot of unnessary
  checks and takes a really long time to finish but still, better safe.
- In pervious versions, some features could not work together. This is not
  desirable so keep in mind how features interact if all or some are enabled
  together.

### Versioning

Before `1.0.0`, every minor version (`0.THIS.0`) bump *may* be a breaking change
for the dependent code using this library.

See the `CHANGELOG` file in the source for the news about each release.

### TODOs

- Add `staticlib.a` output
- Debloat `staticlib.a` output
- Write more tests
- Add Go support
- Add a class based C++ intermediate (possibly to `codegen`).
- Add Github build tools, lints, formatters and Clippy.
- Add Windows and MacOS support (maybe already there, but should be tested)
- Remove `jelal_proc`
- Clean `codegen`
- Add a script that builds and measures the most minimal and optimized (for
  size) version automatically to add it to this readme (are we still small?)
- Add a script that automatically adds the content of `test_readme` to this
  readme
- Add build script for examples and add them to tests

## License

The license of this crate and its components is exactly as described in the
`Cargo.toml`, [`MIT`] OR [`Apache-2.0`]. Some functions are copied (with minor
changes) from
[`unicode-org/icu4x/utils/calendrical_calculations/src/persian.rs`] which
probably lies under the "Software" clause of the free [`Unicode-3.0`] license
(see the exact file here: [`unicode-org/icu4x/LICENSE`]).

For the dependencies, consult their licenses but this crate aims to keep it at
[`MIT`] OR [`Apache-2.0`] dual license for the sake of simplicity, as for the
rest of the ecosystem with the exception of the [`Unicode-3.0`] mentioned above.

[`MIT`]: https://opensource.org/license/MIT
[`Apache-2.0`]: https://opensource.org/license/apache-2-0
[`Unicode-3.0`]: https://opensource.org/license/unicode-license-v3
[`unicode-org/icu4x/LICENSE`]: https://github.com/unicode-org/icu4x/blob/main/LICENSE
[`unicode-org/icu4x/utils/calendrical_calculations/src/persian.rs`]: https://github.com/unicode-org/icu4x/blob/main/utils/calendrical_calculations/src/persian.rs
