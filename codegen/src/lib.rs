//! Holds cross binary values that are hardcoded for jelal codegen.

pub mod resolve_type;
pub mod sift;
pub mod util;
pub mod visit_mut;

// features
pub const STD_FEATURE: &str = "std";
pub const PY_FEATURE: &str = "py";
pub const C_FEATURE: &str = "c";
pub const WASM_FEATURE: &str = "wasm";

/// Match the idents defined here.
pub const IDENTS: &[&str] = &["Date", "Month", "MonthDay", "Ordinal", "Year"];

/// Inside these files.
pub const FILES: [&str; 2] = ["lib.rs", "primitive.rs"];

/// Indicates the Rust output of the files.
///
/// This must be relative.
pub const OUTPUT: &str = "ffi/generated.rs";

/// Holds the path for the location of jelal's src directory.
pub const FILES_PREFIX: &str = "../src/";

/// Holds the name for jelal cratename.
pub const LIB_NAME: &str = "jelal";

/// Prefixes the given path so it will be in the jelal sources.
pub fn prefixed_path(path: &str) -> String {
    format!("{}{}", FILES_PREFIX, path)
}
