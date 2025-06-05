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

#[cfg(not(doc))]
pub use generated::*;
