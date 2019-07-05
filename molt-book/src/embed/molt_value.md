# The Molt `Value` Type

The `Value` type is the standard representation in Rust of Molt values.  In the Tcl
language, "everything is a string"; which is to say, every value can be represented
as a string.  Many values—e.g., numbers and lists—also have a binary data representation,
but a single value can move from one binary data representation to another depending
on how it is used by the user.  Consider the following:

```tcl
set x [expr {2 + 3}]  ;# It's the integer 5.
puts "x=$x"           ;# It's converted to a string.
set y [lindex $x 0]   ;# It's converted to a one-element list.
```

Initially, the variable `x` contains a `Value` with only a data representation, the
integer 5.  Then `puts` needs it as a string, and so the `Value` acquires a string
representation as well, but retains its integer representation.  Then `lindex` needs
to look at it as a list, so the string is parsed into a Molt list and the 0th element
is returned.  The integer representation is lost and replaced by the list
representation. The `Value` type manages all of these transformations internally, with the effect that string-to-binary and binary-to-string conversions happen only when
absolutely necessary.

Note: A `Value`'s string representation is never lost, once acquired: semantically,
`Values` are immutable.  The data transformations that go on under the hood are an
aid to performance, but in principle the value is unchanged.

## Creating Values

`Values` can be created easily from a variety of kinds of input:

```
let a = Value::from("abc");                              // &str
let b = Value::from("def".to_string());                  // String
let c = Value::from(123);                                // MoltInt (i64)
let d = Value::from(45.67);                              // MoltFloat (f64)
let e = Value::from(true);                               // bool
let f = Value::from(&[Value::from(1), Value::from(2)]);  // &[Value]
```

And in fact, a `Value` can contain any Rust type that supports the `Display`,
`Debug`, and `FromStr` types via the `Value::from_other` method.  Such types are
called "external types in the Molt documentation set.

## Cloning Values

Because `Values` are immutable, they have been designed to be cheaply and easy cloned
with reference counting via the standard `Rc` type.

## Retrieving Data from Values

It is always possible to retrieve a `Value`'s data as a string:

```
let value = Value::from(5);
let text: String = value.to_string();
assert_eq!(&text, "5");
```

The `to_string` method creates a brand new `String` in the usual way; it is often better to use `as_string`, which returns the `Value`'s actual string rep as an `Rc<String>`:

```
let value = Value::from(5);
let text: Rc<String> = value.as_string();
assert_eq!(&*text, "5");
```

It is also possible to retrieve data representations; but since this isn't guaranteed to
work the relevant methods all return `Result<_,ResultCode>`.  (See
[The `MoltResult` type](./molt_result.md) for a discussion of `ResultCodes`.)  For
example,

```
let value = Value::from("123");
let x = value.as_int()?;
assert_eq!(x, 123);
```

## Retrieving Values of External Types

Values of external types can be retrieved as well using the `Value::as_copy` or
`Value::as_other` method, depending on whether the type implements the `Copy`
trait.  These are different than their peers, in that they return `Option<T>`
and `Option<Rc<T>>` rather than `Result<T,ResultCode>` or `Result<Rc<T>,ResultCode>`.
The reason is that Molt doesn't know what the appropriate
error message should be when it finds a value it can't convert into the external
type `T` and so returns `None`, leaving the error handling up to the client.

For this reason, when using an external type `MyType` with Molt it is usual to define a
function that converts a `Value` to a `Result<MyType,ResultCode>`.  If `MyType` is an
enum, for example, you might write this:

```rust
impl MyType {
    /// A convenience: retrieves the enumerated value, converting it from
    /// `Option<MyType>` into `Result<MyType,ResultCode>`.
    pub fn from_molt(value: &Value) -> Result<Self, ResultCode> {
        if let Some(x) = value.as_copy::<MyType>() {
            Ok(x)
        } else {
            Err(ResultCode::Error(Value::from("Not a MyType string")))
        }
    }
}
```
