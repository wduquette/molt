# global -- Bring global into scope

**Syntax: global ?*varname* ...?**

Brings global variable(s) *varname* into scope in a
[`proc`](./proc.md) body.  This command has no effect if called in the
global scope.

## TCL Differences

At the script level, `global` works the same in Molt as in Standard
TCL.  However, Molt's internal implementation of variables is currently much
simpler than standard TCL's, e.g., no arrays, no namespaces.
