//! molt-shell -- The Molt REPL and Test Harness
//!
//! This crate provides the code for adding the Molt REPL and related tools to a binary
//! crate.
//!
//! In each case, begin by creating a `molt::Interp` and adding any application-specific
//! extensions.  Then:
//!
//! * To invoke the REPL, use [`molt_shell::repl`](./fn.repl.html).
//! * To execute a script, use [`molt_shell::script`](./fn.script.html).
//! * To execute the test harness on a Molt test script, use
//!   [`molt_shell::test_harness`](./test_harness/index.html).
//! * To execute the benchmark harness on a Molt test script, use
//!   [`molt_shell::bench`](./bench/index.html).

pub mod bench;
mod shell;

pub use bench::*;
pub use shell::*;
