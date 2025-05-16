![Crates.io Version](https://img.shields.io/crates/v/jelal?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fjelal)
![docs.rs](https://img.shields.io/docsrs/jelal?link=https%3A%2F%2Fdocs.rs%2Fjelal%2Flatest%2Fjelal%2F)

Jelal is a modern and lean Jalali (Persian/Iranian) calendar for developers.

Supported in:
- Rust
- JavaScript/TypeScript
- Python
- C

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
solar calendars). With that in mind, crate instead tries to simply sync Jalali
to Gregorian by days which effectively dodges all of the complexity, issues and
features. Simply use your default Gregorian time libraries including libc's and
upon needing a date conversion do that by calling this to get the right idea
about what month and day is it in Jalali and format accordingly.

As such conversions are rarely actually needed, there is no point in being fast
either. The base of all the calculations will be the mature Gregorian
implementations and this little structs can be used just for the IO and other
interactions. Hence, this crate simply works and aims to be a single reference
for how exactly calculations about this calendar must be since there is nothing
but hard to read papers from 2000's on this issue, all text and not a single
formula.

Regardless, this crate is `no_std` for the ease of use cross boundaries. No
dependencies are required! Once `no_core` is stable, core will be stripped away
too.

### Building and Usage

To build the library one could use a simple `cargo build` and generate the
headers using tools like `cbindgen` and else, which are not explained in this
README. If the user is not acquainted with the correct tools, the defaults are
prepared in `cmake` which can be used to build and install the project.

After ensuring `cmake`, create a directory in `target/` named `cmake`, cd into
it and run the commands:

```sh
mkdir target/cmake/
cd target/cmake/
cmake ../..
```

To build the project call:

```sh
cmake --build .
```

This is the same as calling `cargo build`.

To install the built artifacts, run:

``` sh
cmake --install .
```

Assuming the targets are generated, they will be installed.

Make sure you have the right access (sudo or admin). Also, a common pitfall on
dynamically linking is forgetting to set `LD_LIBRARY_PATH` (more on that online)
which may make you think that the install command had silently failed!

There are the variations (targets) below defined to use like
`cmake --build . --target ${NAME}` in place of `${NAME}`:
- `Lib` (default): Creates the library as the `cargo` would
- `Cbindgen` (C): `Lib` + C headers for the C language (using nightly)
- `WasmPack` (JS/TS): Creates the library and its package with `wasm-pack`
- `Maturin` (Python): Creates the library and its package with `maturin`
- `Wasm32`: Creates the library for `wasm32-unknown-unknown` tuple
- `Doc`: Creates the documents as `rustdoc` (`cargo doc`) would

#### Building for Release

To build for release in CMake, simply pass `-DCMAKE_BUILD_TYPE` as suggested [in
the
reference](https://cmake.org/cmake/help/latest/variable/CMAKE_BUILD_TYPE.html#variable:CMAKE_BUILD_TYPE).
As for `cargo`, the default is set to `Debug` (as if `-DCMAKE_BUILD_TYPE=Debug`)
but also `Release` is supported (`-DCMAKE_BUILD_TYPE=Release`).

#### Build Requirements

Besides Rust utilities like `cargo` and `rustup`, the following are optionally
required:
- `Lib`: no further dependencies
- `Cbindgen`: requires `cbindgen` and a `nightly` since attributes for this tool
  are behind `cfg_attr` (run `cargo install cbindgen`)
- `Wasm32`: `wasm32-unknown-unknown` tuple (run `rustup target add
  wasm32-unknown-unknown`)
- `WasmPack`: `Wasm32` requirements and also `wasm-pack` (run `cargo install
  wasm-pack`)
- `Maturin`: requires `maturin` (run `cargo install maturin`)

The requirements for the tools mentioned above can be found on their resources.

#### Installation

Installation as of now only installs the libraries for the host target and the C
headers.

### Contribute

The feature set seems to be enough for the goal of the project and the most
welcome contributions as for now are tests! The first and foremost important
goal of this project is to be "correct" rather than anything else, including
fast and unreadable. The languages can be expanded further as well. For
more TODO see the sources and the section below. Please keep the simple
conventions in the section ahead in mind while developing.

#### Table of Abbrevations, Orders and Style

- Always, the order of values, codes and all else is first Year, Month then Day.
- The accepted abbrevations in the code (not documentation) of this crate are as
  follow:
  - `Doy`: Day of year, `1..=366`
  - `Dom`: Day of month, `1..=31`
  - `Res`: Result
  - `y`: Year
  - `m`: Month
  - `d`: Day (not the day of the year or month, just days)
  - `ymd`: Year, month and day
  - `md`: Month and day
- No two abbrevations come after each other if not defined in this table (i.e.
  `y_doy`, not `ydoy`).
- Since `CMakeLists.txt` searches `Cargo.toml`, keep it simple, formatted and
  keep each key-value pair on one line with no `.` notation in the name.
- Do not use any (proc-)macro if not necessary (`bindgen` is here until [PyO3
  attribute issue](https://github.com/PyO3/pyo3/issues/5125) is resolved)
- Do not use any type not necessary (extra custom types limited for days and
  else, keep things verbose and primitive)
- CMake targets should resemble the tools and expressions used in their process
  in PascalCase.

#### TODOs

- Debloat staticlib.a output
- Write more tests
- Add Go support
- Add a class based C++ intermediate.
- Add Github build tools, lints, formatters and Clippy!
- Add Windows and MacOS support
