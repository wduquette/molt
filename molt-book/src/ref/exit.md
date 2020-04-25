# exit -- Exit the application

**Syntax: exit ?*returnCode*?**

Terminates the application by calling
[`std::process:exit()`](https://doc.rust-lang.org/std/process/fn.exit.html)
with the given *returnCode*, which must be an integer.  If not present,
the *returnCode* defaults to 0.
