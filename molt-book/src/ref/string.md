# string *subcommand* ?*arg* ...?

This command manipulates string values.

| Subcommand                                  | Description                                    |
| ------------------------------------------- | ---------------------------------------------- |
| [string cat](#string-cat-arg)               | Concatenates zero or more strings  |

**TCL Liens**

* Supports a subset of the subcommands provided by the standard TCL `string` command.  The
  subset will increase over time.
* Does not currently support index syntax, e.g., `end-1`.

## string cat ?*arg* ...?

Returns the concatenation of zero or more strings.
