# expr -- Evaluate algebraic expressions

**Syntax: expr *expr***

Evaluates the expression, returning the result.

`expr` implements a little language that has a syntax separate from that of Molt. An
expression is composed of values and operators, with parentheses for grouping, just
as in C, Java, and so forth.  Values consist of numeric and boolean literals,
[function calls](#math-functions), variable and command interpolations, and double-quoted
and braced strings. Every value that looks like a number is treated as a number, and every
value that looks like a [boolean](#boolean-values) is treated as a boolean.

The [operators](#operators-and-precedence) permitted in expressions include most of those
permitted in C expressions, with a few additional ones  The operators have the same
meaning and precedence as in C.  Expressions can yield numeric or non-numeric results.

Integer computations are done with Rust's `i64` type; floating-point computations are
done with Rust's `f64` type.

## Examples

```tcl
expr {1 + 1}

set x 7.5
set y 3.4
expr {$x + $y}

expr {[mycommand] + 2}

expr {2*(1 + abs($x))}
```

## Operators and Precedence

The following table shows the operators in order of precedence.

| Operators                 | Details                                          |
| ------------------------- | ------------------------------------------------ |
| `- + ~ !`                 | Unary plus, minus, bit-wise not, and logical not |
| `* / %`                   | Multiplication, division, integer remainder      |
| `+ -`                     | Addition, subtraction                            |
| `<< >>`                   | Left and right shift.                            |
| `< > <= >=`               | Ordering relations (see below)                   |
| `== !=`                   | Equality, inequality (see below)                 |
| `eq ne`                   | String equality, inequality                      |
| `in ni`                   | List inclusion, exclusion                        |
| `&`                       | Bit-wise AND                                     |
| `^`                       | Bit-wise exclusive OR                            |
| <code>&#124;</code>       | Bit-wise OR                                      |
| `&&`                      | Logical AND, short circuiting                    |
| <code>&#124;&#124;</code> | Logical OR, short circuiting                     |
| `x ? y : z`               | Ternary "if-then-else" operator.                 |

## Boolean Values

* True values: any non-zero number, `true`, `yes`, `on`.
* False values: zero, `false`, `no`, `off`.
* Logical operators always return 0 or 1.
* By convention, predicate commands also return 0 or 1.

## Math Functions

Functions are written as "*name*(*argument*,...)".  Each argument is itself a complete
expression.

The following functions are available in Molt expressions:

**abs(*x*)** — Absolute value of *x*.

**double(*x*)** — Returns integer *x* as a floating-point value.

**int(*x*)** — Truncates floating-point value *x* and returns it as an integer.

**round(*x*)** — Rounds floating-point value *x* to the nearest integer and returns it as
an integer.

## TCL Liens

**Expr Command Syntax:** In standard TCL `expr` takes any number of arguments, which it
concatenates into a single expression for evaluation.  This means that variable and command
interpolation is done twice, once by the TCL parser and once by `expr`, which hurts
performance and can also be a source of subtle and confusing errors.  Consequently it is
almost always best to provide the expression as a single braced string, and so Molt's `expr`
takes a single argument.  This is unlikely to change.

**Expression Syntax:** Molt's expression parsing is meant to be consistent with TCL 7.6, with the
addition of the TCL 8.x `eq`, `ne`, `in`, and `ni` operators.

* Molt does not yet support the full range of math functions supported by TCL 7.6.
* Molt does not yet do precise float-to-string-to-float conversions, per TCL 8.6.  See  
  "String Representation of Floating Point Numbers" on the Tcler's Wiki expr page.
* Molt's handling of floating point arithmetic errors is still naive.

**Integer Division:** Integer division in Molt rounds down towards zero, following the example
of Rust, Python, C99, and many other languages.  Standard TCL rounds toward negative
infinity, a decision that dates to a time when the C standard did not define the correct
behavior and C compilers varied.  It seems reasonable that an extension language should do
something as basic as this in the same way as the host language.

**Possible Futures:** The following TCL 8.6 features are not on the road map at present,
but might be added in the future.

* Bignums
* The exponential operator, `**`
* The `tcl::mathfunc::` namespace, and the ability to define new functions in TCL code.
