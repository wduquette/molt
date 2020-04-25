# set -- Set a variable's value

**Syntax: set *varName* ?*newValue*?**

Sets variable *varName* to the *newValue*, returning the *newValue*.  If
*newValue* is omitted, simply returns the variable's existing value, or
returns an error if there is no existing value.

The `set` command operates in the current scope, e.g., in
[`proc`](./proc.md) bodies it operates on the set of local variables.

See also: [`global`](./global.md)

## TCL Liens

* Molt does not support namespaces or namespace notation.
