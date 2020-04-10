# molt-shell -- Molt Application Frameworks

The `molt-shell` crate provides application frameworks for use with the Molt TCL interpreter:

*   A interpreter shell (REPL)
*   A benchmark harness

The Molt Book (and the rustdoc) explain how to create each of these kinds of apps; or see
the `molt-app` crate for a straightforward example.  `molt-app` defines `moltsh`, which
provides a vanilla REPL, test tool, and benchmark tool, in one page of code.

The [`molt-sample` repo](http://github.com/wduquette/molt-sample) contains a sample Molt
extension, including a shell application and a library create, both of which define new
Molt commands.

See [The Molt Book](https://wduquette.github.io/molt) for more details, and
the [GitHub Repo](https://github.com/wduquette/molt) for issue tracking, etc.

## New in Molt 0.3

* Scripted REPL prompts

See the
[Annotated Change Log](https://wduquette.github.io/molt/changes.md) in the Molt Book for
the complete list of new features by version.
