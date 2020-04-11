# string *subcommand* ?*arg* ...?

This command manipulates string values.

| Subcommand                                  | Description                                    |
| ------------------------------------------- | ---------------------------------------------- |
| [string cat](#string-cat-arg)               | Concatenates zero or more strings  |
| [string compare](#string-compare-options-string1-string2) | Compares two strings lexicographically |

**TCL Liens**

* Supports a subset of the subcommands provided by the standard TCL `string` command.  The
  subset will increase over time.
* Does not currently support index syntax, e.g., `end-1`.

## string cat ?*arg* ...?

Returns the concatenation of zero or more strings.

## string compare ?*options*? *string1* *string2*

Compares the two strings lexicographically, returning `-1` if *string1* is less than *string2*,
`0` if they are equal, and `1` if *string1* is greater than *string2*.

The options are as follows:

| Option           | Description                                          |
| ---------------- | ---------------------------------------------------- |
| -nocase          | The comparison is case-insensitive.                  |
| -length *length* | Only the first *length* characters will be compared. |

Notes:

* When `-nocase` is given, the strings are compared by converting them to lowercase using
  a naive method that may fail for more complex Unicode graphemes.
