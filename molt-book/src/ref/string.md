# string -- String manipulation

**Syntax: string *subcommand* ?*args*...?**

| Subcommand                            | Description                                    |
| ------------------------------------- | ---------------------------------------------- |
| [string cat](#string-cat)             | Concatenates zero or more strings              |
| [string compare](#string-compare)     | Compares two strings lexicographically         |
| [string equal](#string-equal)         | Compares two strings for equality              |
| [string first](#string-first)         | Finds first occurrence of a string             |
| [string last](#string-last)           | Finds last occurrence of a string              |
| [string length](#string-length)       | String length in characters                    |
| [string map](#string-map)             | Maps keys to values in a string                |
| [string range](#string-range)         | Extracts a substring                           |
| [string tolower](#string-tolower)     | Converts a string to lower case                |
| [string toupper](#string-toupper)     | Converts a string to upper case                |
| [string trim](#string-trim)           | Trims leading and trailing whitespace          |
| [string trimleft](#string-trimleft)   | Trims leading whitespace                       |
| [string trimright](#string-trimright) | Trims trailing whitespace                      |

## TCL Liens

* Supports a subset of the subcommands provided by the standard TCL `string` command.  The
  subset will increase over time.
* Does not currently support index syntax, e.g., `end-1`, for the `string first`,
  `string last`, and `string range` commands.  These commands accept simple numeric indices only.

## Molt Strings and Unicode

Molt strings are exactly and identically Rust `String` values, and are treated at the TCL
level as vectors of Rust `char` values. A Rust `char` is a "Unicode scalar value", and is
also (in most cases) a Unicode code point.  It is not a not a grapheme; graphemes that
consist of multiple code points will be treated as multiple characters.  This is more or
less the same as Standard TCL, but Unicode being what it is there may be edge cases where
behavior will differ slightly.

## string cat
---
**Syntax: string cat ?*args* ...?**

Returns the concatenation of zero or more strings.

## string compare
---
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
---
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

## string first
---
**Syntax: string first *needleString haystackString* ?*startIndex*?**

Returns the index of the first occurrence of the *needleString* in the *haystackString*, or
-1 if the *needleString* is not found.  If the *startIndex* is given, the search will begin
at the *startIndex*.

## string last
---
**Syntax: string last *needleString haystackString* ?*startIndex*?**

Returns the index of the last occurrence of the *needleString* in the *haystackString*, or
-1 if the *needleString* is not found.  If the *startIndex* is given, the search will begin
at the *startIndex*.

## string length
---
**Syntax: string length _string_**

Returns the length of the string in Rust characters.

## string map
---
**Syntax: string map ?-nocase? _mapping string_**

Replaces old substrings in *string* with new ones based on the key/value pairs in *mapping*,
which is a dictionary or flat key/value list.  If `-nocase` is given, substring matches will
be case-insensitive.  The command iterates through the string in a single pass, checking for
each key in order, so that earlier key replacements have no effect on later key replacements.

## string range
---
**Syntax: string range *string* *first* *last***

Returns the substring of *string* starting with the character whose index is *first* and
ending with the character whose index is *last*.  Values of *first* that are less than 0 are
treated as 0, and values of *last* that are greater than the index of the last character in the
string are treated as that index.

## string tolower
---
**Syntax: string tolower _string_**

Converts the *string* to all lower case, using the standard Rust `String::to_lowercase` method.

**TCL Liens**: Tcl 8.6 provides for optional *first* and *last* indices; only the text in that
range is affected.

## string toupper
---
**Syntax: string toupper _string_**

Converts the *string* to all upper case, using the standard Rust `String::to_uppercase` method.

**TCL Liens**: Tcl 8.6 provides for optional *first* and *last* indices; only the text in that
range is affected.

## string trim
---
**Syntax: string trim _string_**

Returns *string* trimmed of leading and trailing whitespace by the standard Rust `String::trim`
method.

## string trimleft
---
**Syntax: string trimleft _string_**

Returns *string* trimmed of leading whitespace by the standard Rust `String::trim_start`
method.

## string trimright
---
**Syntax: string trimright _string_**

Returns *string* trimmed of trailing whitespace by the standard Rust `String::trim_end`
method.
