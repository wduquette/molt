# Expression Parsing

Notes on the expression parser.  This information derives from:

*   The ["expr" wiki page](https://wiki.tcl-lang.org/page/expr)
*   The Tcl 7.6 source code
*   Discovery learning

## Preliminary Notes

*   An expression consists of values and operators.
*   Operators are the same as in C, with the same precedence.
    *   Plus a few, e.g., eq, ne, in, ni
    *   Minus the assignment operators
    *   `+, -, *, /, %, **, ==, !=, <, >, <=, >=, &&, ||, !`
    *   Bitwise operators
    *   Ternary `?:`
    *   See below for the list supported by Molt, with precedence.
    *   See [the wiki page](https://wiki.tcl-lang.org/page/expr) for the
        full set.
*   Values may be:
    *   Variable interpolations
    *   Script interpolations
    *   Quoted strings
    *   Braced strings
    *   Bare strings that look like numbers or booleans.
    *   These things are always evaluated as individual values, *not*
        sub-expressions.
    *   Ultimately, a value can be interpreted as a number (integer or
        floating), a boolean, or a string, depending on what operators it's
        used with.
        *   But unquoted literals will be interpreted as numeric or boolean.
*   Boolean Values
    * True values: any non-zero number, `true`, `yes`, `on`.
    * False values: zero, `false`, `no`, `off`.
    * Logical operators always return 0 or 1.
    * By convention, predicate commands also return 0 or 1.
    * Boolean constants may only be used with logical operators.
        *   Or as the entire expression, in which case they are returned
            "as is".
        *   But this is just normal `expr` behavior: any expression consisting
            of a single value just returns the value.
        *   Point is, boolean constants do not become boolean values until
            they are used with boolean operators.
*   Literal strings must be quoted or braced.
*   Parentheses delimit subexpressions.
*   Logical AND and OR are short-circuiting
    *   AND only evaluates its second argument if its first is non-zero
    *   OR only evaluates its second argument if its first is zero
    *   This includes command interpolation!  If the second argument is
        an interpolated command, it won't be expanded unless necessary!
*   Function calls consist of:
    *   functionName ( expr [, expr...] )

## TCL Liens

The following features of TCL expressions will *not* be implemented, at least
at first.

*   Bignums
*   Exponential operator: `**`
*   Bitwise operators: `~ | & << >> ^`
*   Ternary operator: `?:`
*   Function calls
*   Precise float-to-string-to-float conversions.  See "String Representation
    of Floating Point Numbers" on the Wiki expr page.

## Operators and Precedence

The following table shows the operators in order of precedence.

| Operators   | Details                                     |
| ----------- | ------------------------------------------- |
| `- + !`     | Unary plus, minus, and logical not          |
| `* / %`     | Multiplication, division, integer remainder |
| `+ -`       | Addition, subtraction                       |
| `< > <= >=` | Ordering relations                          |
| `== !=`     | Equality, inequality                        |
| `eq ne`     | String equality, inequality                 |
| `in ni`     | List inclusion, exclusion                   |
| `&&`        | Logical AND, short circuiting               |
| `||`        | Logical OR, short circuiting                |

## Handling of Booleans

* True values: any non-zero number, `true`, `yes`, `on`.
* False values: zero, `false`, `no`, `off`.
* Logical operators always return 0 or 1.
* By convention, predicate commands also return 0 or 1.
