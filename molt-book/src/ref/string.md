# string -- String Manipulation

**Syntax: string *subcommand* ?*args*...?**

This command manipulates Rust `String` values, considered as vectors of Rust `char` values.  
A `char` is a Unicode code point, not a grapheme; graphemes that consist of multiple code
points will be treated as multiple characters.

| Subcommand                                  | Description                                    |
| ------------------------------------------- | ---------------------------------------------- |
| [string cat](#string-cat)                   | Concatenates zero or more strings              |
| [string compare](#string-compare)           | Compares two strings lexicographically         |
| [string equal](#string-equal)               | Compares two strings for equality              |
| [string length](#string-length)             | String length in characters                    |

**TCL Liens**

* Supports a subset of the subcommands provided by the standard TCL `string` command.  The
  subset will increase over time.
* Does not currently support index syntax, e.g., `end-1`.

## string cat

**Syntax: string cat ?*args* ...?**

Returns the concatenation of zero or more strings.

## string compare

**Syntax: string compare ?*options*? *string1* *string2***

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

## string equal

**Syntax: string equal ?*options*? *string1* *string2***

Compares the two strings, returning `1` if they are equal, and `0` otherwise.

The options are as follows:

| Option           | Description                                          |
| ---------------- | ---------------------------------------------------- |
| -nocase          | The comparison is case-insensitive.                  |
| -length *length* | Only the first *length* characters will be compared. |

Notes:

* When `-nocase` is given, the strings are compared by converting them to lowercase using
  a naive method that may fail for more complex Unicode graphemes.

## string length

**Syntax: string length _string_**

Returns the length of the string in Rust characters.
