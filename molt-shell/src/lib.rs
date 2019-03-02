//! molt-shell -- The Molt REPL and Test Harness
//!
//! This crate provides the code for adding the Molt REPL and Test Harness to a binary
//! crate.
//!
//! Details: TODO


mod shell;
mod test_harness;

pub use shell::*;
pub use test_harness::*;
