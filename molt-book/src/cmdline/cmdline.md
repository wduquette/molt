# Molt Command Line Tool

The `molt-app` crate provides a command line tool for use in development and
experimentation. The command line tool, called `molt`, has several subcommands:

[**molt shell**](./molt_shell.md) executes scripts and provides an
interactive REPL.

[**molt test**](./molt_test.md) executes Molt test suites, most notably
Molt's own test suite.

[**molt bench**](./molt_bench.md) executes Molt benchmarks.  This tool is
experimental, and is primarily for use in optimizing molt itself.

Note: the `molt-shell` crate provides the same features for use with customized Molt interpreters.
