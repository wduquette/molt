//! molt-shell -- The Molt REPL and Test Harness
//!
//! This crate provides the code for adding the Molt REPL and Test Harness to a binary
//! crate.
//!
//! In each case, begin by creating a `molt::Interp` and adding any application-specific
//! extensions.  Then:
//!
//! * To invoke the REPL, use `molt_shell::repl`
//! * To execute a script, use `molt_shell::script`
//! * To execute the test harness on a Molt test script, use
//!   `molt_shell::test_harness`.

pub mod bench;
mod shell;
pub mod test_harness;

pub use bench::*;
pub use shell::*;
pub use test_harness::*;
