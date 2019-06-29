//! The Value Type
//!
//! The [`Value`] struct is the standard representation of a data value
//! in the Molt language.  It represents a single immutable data value; the
//! data is reference-counted, so instances can be cloned efficiently.  Its
//! content may be any TCL data value: a number, a list, a string, or a value of
//! an arbitrary type that meets certain requirements.
//!
//! In TCL, "everything is a string".  Every value is defined by its _string
//! representation_, e.g., "one two three" is the _string rep_ of a
//! list with three items, "one", "two", and "three".  A string that is a valid string
//! for multiple types can be interpreted as any of those types; for example, the
//! string "5" can be a string, the integer 5, or a list of one element, "5".
//!
//! # Comparisons
//!
//! If two `Value`'s are compared for equality in Rust, Rust compares their string reps.
//! In TCL expressions the `==` and `!=` operators compare numbers and the
//! `eq` and `ne` operators compare string reps.
//!
//! # Internal Representation
//!
//! "Everything is a string"; thus, every `Value` has a string
//! representation, or _string rep_.  But for efficiency with numbers, lists,
//! and user-defined binary data structures, the `Value` also caches a
//! data representation, or _data rep_.
//!
//! A `Value` can have just a string rep, just a data rep, or both.
//! Like the `Tcl_Obj` in standard TCL, the `Value` is like a stork: it
//! can stand one leg, the other leg, or both legs.
//!
//! A client can ask the `Value` for its string, which is always available
//! and will be computed from the data rep if it doesn't already exist.  (Once
//! computed, the string rep never changes.)  A client can also ask
//! the `Value` for any other type it desires.  If the requested data rep
//! is already available, it will be returned; otherwise, the `Value` will
//! attempt to parse it from the string_rep.  The last successful conversion is
//! cached for later.
//!
//! For example, consider the following sequence:
//!
//! * A computation yields a `Value` containing the integer 5. The data rep is
//!   a `MoltInt`, and the string rep is undefined.
//!
//! * The client asks for the string, and the string rep "5" is computed.
//!
//! * The client asks for the value's integer value.  It's available and is returned.
//!
//! * The client asks for the value's value as a MoltList.  This is possible, because
//!   the string "5" can be interpreted as a list of one element, the
//!   string "5".  A new data rep is computed and saved, replacing the previous one.
//!
//! With this scheme, long series of computations can be carried
//! out efficiently using only the the data rep, incurring the parsing cost at most
//! once, while preserving TCL's "everything is a string" semantics.
//!
//! Converting from one data rep to another is expensive, as it involves parsing
//! the string value.  Performance suffers when code switches rapidly from one data
//! rep to another, e.g., in a tight loop.  The effect, which is known as "shimmering",
//! can usually be avoided with a little care.
//!
//! `Value` handles strings, integers, floating-point values, and lists as
//! special cases, since they are part of the language and are so frequently used.
//! In addition, a `Value` can also contain _external types_: Rust types that meet
//! certain requirements.
//!
//! # External Types
//!
//! Any type that implements the `std::fmt::Display`, `std::fmt::Debug`,
//! and `std::str::FromStr` traits can be saved in a `Value`.  The struct's
//! `Display` and `FromStr` trait implementations are used to convert between
//! the string rep and data rep.
//!
//! * The `Display` implementation is responsible for producing the value's string rep.
//!
//! * The `FromStr` implementation is responsible for producing the value's data rep from
//!   a string, and so must be able to parse the `Display` implementation's
//!   output.
//!
//! * The string rep should be chosen so as to fit in well with TCL syntax, lest
//!   confusion, quoting hell, and comedy should ensue.  (You'll know it when you
//!   see it.)
//!
//! ## Example
//!
//! For example, the following code shows how to define an external type implementing
//! a simple enum.
//!
//! ```
//! use molt::types::*;
//! use std::fmt;
//! use std::str::FromStr;
//!
//! #[derive(Debug, PartialEq, Copy, Clone)]
//! pub enum Flavor {
//!     SALTY,
//!     SWEET,
//! }
//!
//! impl fmt::Display for Flavor {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         if *self == Flavor::SALTY {
//!             write!(f, "salty")
//!         } else {
//!             write!(f, "sweet")
//!         }
//!     }
//! }
//!
//! impl FromStr for Flavor {
//!     type Err = String;
//!
//!     fn from_str(value: &str) -> Result<Self, Self::Err> {
//!         let value = value.to_lowercase();
//!
//!         if value == "salty" {
//!             Ok(Flavor::SALTY)
//!         } else if value == "sweet" {
//!             Ok(Flavor::SWEET)
//!         } else {
//!            // The error message doesn't matter to Molt
//!            Err("Not a flavor string".to_string())
//!         }
//!     }
//! }
//!
//! impl Flavor {
//!     /// A convenience: retrieves the enumerated value, converting it from
//!     /// `Option<Flavor>` into `Result<Flavor,ResultCode>`.
//!     pub fn from_molt(value: &Value) -> Result<Self, ResultCode> {
//!         if let Some(x) = value.as_copy::<Flavor>() {
//!             Ok(x)
//!         } else {
//!             Err(ResultCode::Error(Value::from("Not a flavor string")))
//!         }
//!     }
//! }
//! ```
//!
//! [`Value`]: struct.Value.html

use crate::expr::Datum;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;
use crate::list::get_list;
use crate::list::list_to_string;
use crate::types::MoltList;
use crate::types::MoltInt;
use crate::types::MoltFloat;
use crate::types::ResultCode;

//-----------------------------------------------------------------------------
// Public Data Types

/// The `Value` type. See [the module level documentation](index.html) for more.
#[derive(Clone, Debug)]
pub struct Value {
    string_rep: RefCell<Option<Rc<String>>>,
    data_rep: RefCell<DataRep>,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl PartialEq for Value {
    // Two Values are equal if their string representations are equal.
    fn eq(&self, other: &Self) -> bool {
        *self.as_string() == *other.as_string()
    }
}

impl Eq for Value {}

impl From<String> for Value {
    /// Creates a new `Value` from the given String.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let string = String::from("My New String");
    /// let value = Value::from(string);
    /// assert_eq!(&*value.as_string(), "My New String");
    /// ```
    fn from(str: String) -> Self {
        Self {
            string_rep: RefCell::new(Some(Rc::new(str))),
            data_rep: RefCell::new(DataRep::None),
        }
    }
}

impl From<&str> for Value {
    /// Creates a new `Value` from the given string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from("My String Slice");
    /// assert_eq!(&*value.as_string(), "My String Slice");
    /// ```
    fn from(str: &str) -> Self {
        Self {
            string_rep: RefCell::new(Some(Rc::new(str.to_string()))),
            data_rep: RefCell::new(DataRep::None),
        }
    }
}

// TODO: Not clear why this is needed when we've `impl From<&str>`, but it makes the
// molt_err! macro happy.
impl From<&String> for Value {
    /// Creates a new `Value` from the given string reference.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from("My String Slice");
    /// assert_eq!(&*value.as_string(), "My String Slice");
    /// ```
    fn from(str: &String) -> Self {
        Self {
            string_rep: RefCell::new(Some(Rc::new(str.to_string()))),
            data_rep: RefCell::new(DataRep::None),
        }
    }
}

impl From<bool> for Value {
    /// Creates a new `Value` whose data representation is a `bool`.  The value's
    /// string representation will be "1" or "0".
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from(true);
    /// assert_eq!(&*value.as_string(), "1");
    ///
    /// let value = Value::from(false);
    /// assert_eq!(&*value.as_string(), "0");
    /// ```
    fn from(flag: bool) -> Self {
        Self {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(DataRep::Bool(flag)),
        }
    }
}

impl From<MoltInt> for Value {
    /// Creates a new `Value` whose data representation is a `MoltInt`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let value = Value::from(123);
    /// assert_eq!(&*value.as_string(), "123");
    /// ```
    fn from(int: MoltInt) -> Self {
        Self {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(DataRep::Int(int)),
        }
    }
}

impl From<MoltFloat> for Value {
    /// Creates a new `Value` whose data representation is a `MoltFloat`.
    ///
    /// # String Representation
    ///
    /// The string representation of the value will be however Rust's `format!` macro
    /// formats floating point numbers by default.  **Note**: this isn't quite what we
    /// want; Standard TCL goes to great lengths to ensure that the formatted string
    /// will yield exactly the same floating point number when it is parsed.  Rust
    /// will format the number `5.0` as `5`, turning it into a integer if parsed naively. So
    /// there is more work to be done here.
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let value = Value::from(12.34);
    /// assert_eq!(&*value.as_string(), "12.34");
    /// ```
    fn from(flt: MoltFloat) -> Self {
        Self {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(DataRep::Flt(flt)),
        }
    }
}

impl From<MoltList> for Value {
    /// Creates a new `Value` whose data representation is a `MoltList`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    ///
    /// let list = vec![Value::from(1234), Value::from("abc")];
    /// let value = Value::from(list);
    /// assert_eq!(&*value.as_string(), "1234 abc");
    /// ```
    fn from(list: MoltList) -> Self {
        Self {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(DataRep::List(Rc::new(list))),
        }
    }
}

impl Value {
    /// Returns the empty `Value`, a value whose string representation is the empty
    /// string.
    pub fn empty() -> Value {
        Value::from("")
    }

    // /// Creates a new `Value` from the given String.
    // ///
    // /// Prefer [`new`](#method.new) if you have a string slice
    // /// you'd otherwise have to create a new string from.
    // ///
    // /// # Example
    // ///
    // /// ```
    // /// use molt::types::Value;
    // /// let string = String::from("My New String");
    // /// let value = Value::from_string(string);
    // /// assert_eq!(&*value.as_string(), "My New String");
    // /// ```
    // pub fn from_string(str: String) -> Value {
    //     Value {
    //         string_rep: RefCell::new(Some(Rc::new(str))),
    //         data_rep: RefCell::new(DataRep::None),
    //     }
    // }

    /// Returns the value's string representation as a reference-counted
    /// string.
    ///
    /// **Note**: This is the standard way of retrieving a `Value`'s
    /// string rep, as unlike `to_string` it doesn't create a new `String`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// let value = Value::from(123);
    /// assert_eq!(&*value.as_string(), "123");
    /// ```
    pub fn as_string(&self) -> Rc<String> {
        // FIRST, if there's already a string, return it.
        let mut string_ref = self.string_rep.borrow_mut();

        if let Some(str) = &*string_ref {
            return Rc::clone(str);
        }

        // NEXT, if there's no string there must be data.  Convert the data to a string,
        // and save it for next time.
        let data_ref = self.data_rep.borrow();
        let new_string = Rc::new((*data_ref).to_string());

        *string_ref = Some(new_string.clone());

        new_string
    }

    /// Tries to return the `Value` as a `bool`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Boolean Strings
    ///
    /// The following are valid boolean strings, regardless of case: `true`,
    /// `false`, `on`, `off`, `yes`, `no`, `1`, `0`.  Note that other numeric strings are
    /// _not_ valid boolean strings.
    ///
    /// # Numeric Values
    ///
    /// Non-negative numbers are interpreted as true; zero is interpreted as false.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<bool,ResultCode> {
    /// // All of the following can be interpreted as booleans.
    /// let value = Value::from(true);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from("no");
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// let value = Value::from(5);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from(0);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// let value = Value::from(1.1);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, true);
    ///
    /// let value = Value::from(0.0);
    /// let flag = value.as_bool()?;
    /// assert_eq!(flag, false);
    ///
    /// // Numeric strings can not, unless evaluated as expressions.
    /// let value = Value::from("123");
    /// assert!(value.as_bool().is_err());
    /// # Ok(true)
    /// # }
    /// ```
    pub fn as_bool(&self) -> Result<bool, ResultCode> {
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have a boolean then just return it.
        if let DataRep::Bool(flag) = *data_ref {
            return Ok(flag);
        }

        // NEXT, if we have a number return whether it's zero or not.
        if let DataRep::Int(int) = *data_ref {
            return Ok(int != 0);
        }

        if let DataRep::Flt(flt) = *data_ref {
            return Ok(flt != 0.0);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data_rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as a boolean
        let str = (&*string_ref).as_ref().unwrap();
        let flag = Value::get_bool(&*str)?;
        *data_ref = DataRep::Bool(flag);
        Ok(flag)
    }

    /// Converts a string argument into a boolean, returning an error on failure.
    ///
    /// Molt accepts the following strings as Boolean values:
    ///
    /// * **true**: `true`, `yes`, `on`, `1`
    /// * **false**: `false`, `no`, `off`, `0`
    ///
    /// Parsing is case-insensitive, and leading and trailing whitespace are ignored.
    ///
    /// This method does not evaluate expressions; use `molt::expr` to evaluate boolean
    /// expressions.
    ///
    /// # Example
    ///
    /// ```
    /// # use molt::types::*;
    /// # fn dummy() -> Result<bool,ResultCode> {
    /// let arg = "yes";
    /// let flag = Value::get_bool(arg)?;
    /// assert!(flag);
    /// # Ok(flag)
    /// # }
    /// ```
    pub fn get_bool(arg: &str) -> Result<bool, ResultCode> {
        let orig = arg;
        let value: &str = &arg.trim().to_lowercase();
        match value {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => molt_err!("expected boolean but got \"{}\"", orig),
        }
    }

    /// Tries to return the `Value` as a `MoltInt`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Integer Syntax
    ///
    /// Molt accepts decimal integer strings, and hexadecimal integer strings
    /// with a `0x` prefix.  Strings may begin with a unary "+" or "-".  Hex
    /// digits may be in upper or lower case.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltInt;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<MoltInt,ResultCode> {
    ///
    /// let value = Value::from(123);
    /// let int = value.as_int()?;
    /// assert_eq!(int, 123);
    ///
    /// let value = Value::from("OxFF");
    /// let int = value.as_int()?;
    /// assert_eq!(int, 255);
    /// # Ok(1)
    /// # }
    /// ```
    pub fn as_int(&self) -> Result<MoltInt, ResultCode> {
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have an integer then just return it.
        if let DataRep::Int(int) = *data_ref {
            return Ok(int);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data_rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as an integer
        let str = (&*string_ref).as_ref().unwrap();
        let int = Value::get_int(&*str)?;
        *data_ref = DataRep::Int(int);
        Ok(int)
    }

    // Parses a string as an integer using the standard TCL syntax (except octal :-)
    // Returns a standard Molt error result.
    pub fn get_int(arg: &str) -> Result<MoltInt, ResultCode> {
        let orig = arg;
        let mut arg = arg.trim();
        let mut minus = 1;

        if arg.starts_with('+') {
            arg = &arg[1..];
        } else if arg.starts_with('-') {
            minus = -1;
            arg = &arg[1..];
        }

        let parse_result = if arg.starts_with("0x") {
            MoltInt::from_str_radix(&arg[2..], 16)
        } else {
            arg.parse::<MoltInt>()
        };

        match parse_result {
            Ok(int) => Ok(minus * int),
            Err(_) => molt_err!("expected integer but got \"{}\"", orig),
        }
    }

    /// Tries to return the `Value` as a `MoltFloat`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Floating-Point Syntax
    ///
    /// Molt accepts the same floating-point strings as Rust's standard numeric parser.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltFloat;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<MoltFloat,ResultCode> {
    ///
    /// let value = Value::from(12.34);
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 12.34);
    ///
    /// let value = Value::from("23.45");
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 23.45);
    /// # Ok(1.0)
    /// # }
    /// ```
    pub fn as_float(&self) -> Result<MoltFloat, ResultCode> {
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have a float then just return it.
        if let DataRep::Flt(flt) = *data_ref {
            return Ok(flt);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as a float
        let str = (&*string_ref).as_ref().unwrap();
        let flt = Value::get_float(&*str)?;
        *data_ref = DataRep::Flt(flt);
        Ok(flt)
    }

    // Parses a string as an integer using the standard TCL syntax (except octal :-)
    // Returns a standard Molt error result.
    pub fn get_float(arg: &str) -> Result<MoltFloat, ResultCode> {
        let arg_trim = arg.trim();

        match arg_trim.parse::<MoltFloat>() {
            Ok(flt) => Ok(flt),
            Err(_) => molt_err!("expected floating-point number but got \"{}\"", arg),
        }
    }

    /// Tries to return the `Value` as a `MoltList`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::types::Value;
    /// use molt::types::MoltList;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<String,ResultCode> {
    ///
    /// let value = Value::from("1234 abc");
    /// let list = value.as_list()?;
    /// assert_eq!(list.len(), 2);
    ///
    /// assert_eq!(list[0], Value::from("1234"));
    /// assert_eq!(list[1], Value::from("abc"));
    ///
    /// # Ok("dummy".to_string())
    /// # }
    /// ```
    pub fn as_list(&self) -> Result<Rc<MoltList>, ResultCode> {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let DataRep::List(list) = &*data_ref {
            return Ok(list.clone());
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, try to parse the string_rep as a list.
        let str = (&*string_ref).as_ref().unwrap();
        let list = Rc::new(get_list(str)?);
        *data_ref = DataRep::List(list.clone());

        Ok(list)
    }

    /// Creates a new `Value` containing the given value of some user type.
    ///
    /// The user type must meet certain constraints; see the
    /// [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the constraints; we can create
    /// a `Value` containing one as follows.  Notice that the `Value` ownership of its input:
    ///
    /// ```ignore
    /// let color: HexColor::new(0x11u8, 0x22u8, 0x33u8);
    /// let value = Value::from_other(color);
    ///
    /// // Retrieve the value's string rep.
    /// assert_eq!(&*value.as_string(), "#112233");
    /// ```
    ///
    /// See [`Value::as_other`](#method.as_other) and
    /// [`Value::as_copy`](#method.as_copy) for examples of how to
    /// retrieve a `MyType` value from a `Value`.
    pub fn from_other<T: 'static>(value: T) -> Value
    where
        T: Display + Debug,
    {
        Value {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(DataRep::Other(Rc::new(value))),
        }
    }

    /// Tries to interpret the `Value` as a value of external type `T`, parsing
    /// the string representation if necessary.
    ///
    /// The user type must meet certain constraints; see the
    /// [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// # Return Value
    ///
    /// The value is returned as an `Rc<T>`, as this allows the client to
    /// use the value freely and clone it efficiently if needed.
    ///
    /// This method returns `Option<Rc<T>>` rather than `Result<Rc<T>,ResultCode>`
    /// because it is up to the caller to provide a meaningful error message.
    /// It is normal for externally defined types to wrap this function in a function
    /// that does so; see the [module level documentation](index.html) for an example.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the constraints; we can create
    /// a `Value` containing one and retrieve it as follows.
    ///
    /// ```ignore
    /// // Just a normal Molt string
    /// let value = Value::from("#112233");
    ///
    /// // Retrieve it as an Option<Rc<HexColor>>:
    /// let color = value.as_other::<HexColor>()
    ///
    /// if color.is_some() {
    ///     let color = color.unwrap();
    ///     let r = *color.red();
    ///     let g = *color.green();
    ///     let b = *color.blue();
    /// }
    /// ```
    pub fn as_other<T: 'static>(&self) -> Option<Rc<T>>
    where
        T: Display + Debug + FromStr,
    {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let DataRep::Other(other) = &*data_ref {
            // other is an &Rc<MoltAny>
            let result = other.clone().downcast::<T>();

            if result.is_ok() {
                // Let's be sure we're really getting what we wanted.
                let out: Rc<T> = result.unwrap();
                return Some(out);
            }
        }

        // NEXT, if we don't have a string_rep, get one.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, can we parse it as a T?  If so, save it back to
        // the data_rep, and return it.
        if let Some(str) = &*string_ref {
            if let Ok(tval) = str.parse::<T>() {
                let tval = Rc::new(tval);
                let out = tval.clone();
                *data_ref = DataRep::Other(Rc::new(tval));
                return Some(out);
            }
        }

        // NEXT, we couldn't do it.
        None
    }

    /// Tries to interpret the `Value` as a value of type `T`, parsing the string
    /// representation if necessary, and returning a copy.
    ///
    /// The user type must meet certain constraints; and in particular it must
    /// implement `Copy`. See the [module level documentation](index.html) for details on
    /// how to define an external type for use with Molt.
    ///
    /// This method returns `Option` rather than `Result` because it is up
    /// to the caller to provide a meaningful error message.  It is normal
    /// for externally defined types to wrap this function in a function
    /// that does so.
    ///
    /// # Example
    ///
    /// Suppose we have a type `HexColor` that meets the normal external type
    /// constraints and also supports copy; we can create a `Value` containing one and
    /// retrieve it as follows.
    ///
    /// ```ignore
    /// // Just a normal Molt string
    /// let value = Value::from("#112233");
    ///
    /// // Retrieve it as an Option<HexColor>:
    /// let color = value.as_copy::<HexColor>()
    ///
    /// if color.is_some() {
    ///     let color = color.unwrap();
    ///     let r = color.red();
    ///     let g = color.green();
    ///     let b = color.blue();
    /// }
    /// ```
    pub fn as_copy<T: 'static>(&self) -> Option<T>
    where
        T: Display + Debug + FromStr + Copy,
    {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let DataRep::Other(other) = &*data_ref {
            // other is an &Rc<MoltAny>
            let result = other.clone().downcast::<T>();

            if result.is_ok() {
                // Let's be sure we're really getting what we wanted.
                let out: Rc<T> = result.unwrap();
                return Some(*out);
            }
        }

        // NEXT, if we don't have a string_rep, get one.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, can we parse it as a T?  If so, save it back to
        // the data_rep, and return it.
        if let Some(str) = &*string_ref {
            if let Ok(tval) = str.parse::<T>() {
                let tval = Rc::new(tval);
                let out = tval.clone();
                *data_ref = DataRep::Other(Rc::new(tval));
                return Some(*out);
            }
        }

        // NEXT, we couldn't do it.
        None
    }

    /// For use by `expr::expr` in parsing out `Values`.
    pub(crate) fn already_number(&self) -> Option<Datum> {
        let data_ref = self.data_rep.borrow();

        match *data_ref {
            DataRep::Flt(flt) => Some(Datum::float(flt)),
            DataRep::Int(int) => Some(Datum::int(int)),
            _ => None
        }
    }
}

//-----------------------------------------------------------------------------
// The MoltAny Trait: a tool for handling external types.

/// This trait allows us to except "other" types, and still compute their
/// string rep on demand.
trait MoltAny: Any + Display + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl dyn MoltAny {
    /// Is this value a value of the desired type?
    pub fn is<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Downcast an `Rc<MoltAny>` to an `Rc<T>`
    fn downcast<T: 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if self.is::<T>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as _)) }
        } else {
            Err(self)
        }
    }
}

impl<T: Any + Display + Debug> MoltAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

//-----------------------------------------------------------------------------
// DataRep enum: a sum type for the different kinds of data_reps.

// The data representation for Values.
#[derive(Clone, Debug)]
enum DataRep {
    /// A Boolean
    Bool(bool),

    /// A Molt integer
    Int(MoltInt),

    /// A Molt float
    Flt(MoltFloat),

    /// A Molt List
    List(Rc<MoltList>),

    /// An external data type
    Other(Rc<dyn MoltAny>),

    /// The Value has no data rep at present.
    None,
}

impl Display for DataRep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataRep::Bool(flag) => write!(f, "{}", if *flag { 1 } else { 0 }),
            DataRep::Int(int) => write!(f, "{}", int),
            DataRep::Flt(flt) => write!(f, "{}", flt),
            DataRep::List(list) => write!(f, "{}", list_to_string(&*list)),
            DataRep::Other(other) => write!(f, "{}", other),
            DataRep::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::str::FromStr;

    #[test]
    fn from_string() {
        // Using From<String>
        let val = Value::from("xyz".to_string());
        assert_eq!(&*val.to_string(), "xyz");

        // Using Into
        let val: Value = String::from("Fred").into();
        assert_eq!(&*val.to_string(), "Fred");
    }

    #[test]
    fn from_str_ref() {
        // Using From<&str>
        let val = Value::from("xyz");
        assert_eq!(&*val.to_string(), "xyz");

        // Using Into
        let val: Value = "Fred".into();
        assert_eq!(&*val.to_string(), "Fred");
    }

    #[test]
    fn clone_string() {
        // Values with just string reps can be cloned and have equal string reps.
        let val = Value::from("abc");
        let val2 = val.clone();
        assert_eq!(*val.to_string(), *val2.to_string());
    }

    #[test]
    fn as_string() {
        let val = Value::from("abc");
        assert_eq!(*val.as_string(), "abc".to_string());

        let val2 = val.clone();
        assert_eq!(*val.as_string(), *val2.to_string());
    }

    #[test]
    fn compare() {
        let val = Value::from("123");
        let val2 = Value::from(123);
        let val3 = Value::from(456);

        assert_eq!(val, val2);
        assert_ne!(val, val3);
    }

    #[test]
    fn from_bool() {
        // Using From<bool>
        let val = Value::from(true);
        assert_eq!(&*val.to_string(), "1");

        let val = Value::from(false);
        assert_eq!(&*val.to_string(), "0");
    }

    #[test]
    fn as_bool() {
        // Can convert string to bool.
        let val = Value::from("true");
        assert_eq!(val.as_bool(), Ok(true));

        // Non-zero numbers are true; zero is false.
        let val = Value::from(5);
        assert_eq!(val.as_bool(), Ok(true));

        let val = Value::from(0);
        assert_eq!(val.as_bool(), Ok(false));

        let val = Value::from(5.5);
        assert_eq!(val.as_bool(), Ok(true));

        let val = Value::from(0.0);
        assert_eq!(val.as_bool(), Ok(false));
    }

    #[test]
    fn get_bool() {
        // Test the underlying boolean value parser.
        assert_eq!(Ok(true), Value::get_bool("1"));
        assert_eq!(Ok(true), Value::get_bool("true"));
        assert_eq!(Ok(true), Value::get_bool("yes"));
        assert_eq!(Ok(true), Value::get_bool("on"));
        assert_eq!(Ok(true), Value::get_bool("TRUE"));
        assert_eq!(Ok(true), Value::get_bool("YES"));
        assert_eq!(Ok(true), Value::get_bool("ON"));
        assert_eq!(Ok(false), Value::get_bool("0"));
        assert_eq!(Ok(false), Value::get_bool("false"));
        assert_eq!(Ok(false), Value::get_bool("no"));
        assert_eq!(Ok(false), Value::get_bool("off"));
        assert_eq!(Ok(false), Value::get_bool("FALSE"));
        assert_eq!(Ok(false), Value::get_bool("NO"));
        assert_eq!(Ok(false), Value::get_bool("OFF"));
        assert_eq!(Ok(true), Value::get_bool(" true "));
        assert_eq!(Value::get_bool("nonesuch"),
            molt_err!("expected boolean but got \"nonesuch\""));
        assert_eq!(Value::get_bool(" Nonesuch "),
            molt_err!("expected boolean but got \" Nonesuch \""));
    }

    #[test]
    fn from_as_int() {
        let val = Value::from(5);
        assert_eq!(&*val.as_string(), "5");
        assert_eq!(val.as_int(), Ok(5));
        assert_eq!(val.as_float(), Ok(5.0));

        let val = Value::from("7");
        assert_eq!(&*val.as_string(), "7");
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        // TODO: Note, 7.0 might not get converted to "7" long term.
        // In Standard TCL, its string_rep would be "7.0".  Need to address
        // MoltFloat formatting/parsing.
        let val = Value::from(7.0);
        assert_eq!(&*val.as_string(), "7");
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        let val = Value::from("abc");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"abc\""));
    }

    #[test]
    fn get_int() {
        // Test the internal integer parser
        assert_eq!(Value::get_int("1"), Ok(1));
        assert_eq!(Value::get_int("-1"), Ok(-1));
        assert_eq!(Value::get_int("+1"), Ok(1));
        assert_eq!(Value::get_int("0xFF"), Ok(255));
        assert_eq!(Value::get_int("+0xFF"), Ok(255));
        assert_eq!(Value::get_int("-0xFF"), Ok(-255));
        assert_eq!(Value::get_int(" 1 "), Ok(1));

        assert_eq!(Value::get_int(""), molt_err!("expected integer but got \"\""));
        assert_eq!(Value::get_int("a"), molt_err!("expected integer but got \"a\""));
        assert_eq!(Value::get_int("0x"), molt_err!("expected integer but got \"0x\""));
        assert_eq!(Value::get_int("0xABGG"),
            molt_err!("expected integer but got \"0xABGG\""));
        assert_eq!(Value::get_int(" abc "),
            molt_err!("expected integer but got \" abc \""));
    }

    #[test]
    fn from_as_float() {
        let val = Value::from(12.5);
        assert_eq!(&*val.as_string(), "12.5");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"12.5\""));
        assert_eq!(val.as_float(), Ok(12.5));

        let val = Value::from("7.8");
        assert_eq!(&*val.as_string(), "7.8");
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"7.8\""));
        assert_eq!(val.as_float(), Ok(7.8));

        let val = Value::from(5);
        assert_eq!(val.as_float(), Ok(5.0));

        let val = Value::from("abc");
        assert_eq!(val.as_float(),
            molt_err!("expected floating-point number but got \"abc\""));
    }

    #[test]
    fn get_float() {
        // Test the internal float parser.
        // NOTE: At present, it relies on the standard Rust float parser, so only
        // check special case behavior.
        assert_eq!(Value::get_float("1"), Ok(1.0));
        assert_eq!(Value::get_float("2.3"), Ok(2.3));
        assert_eq!(Value::get_float(" 4.5 "), Ok(4.5));

        assert_eq!(Value::get_float("abc"),
            molt_err!("expected floating-point number but got \"abc\""));
        assert_eq!(Value::get_float(" abc "),
            molt_err!("expected floating-point number but got \" abc \""));
    }

    #[test]
    fn from_as_list() {
        // NOTE: we aren't testing list formatting and parsing here; that's done in list.rs.
        // We *are* testing that Value will use the list.rs code to convert strings to lists
        // and back again.
        let listval = Value::from(vec![Value::from("abc"), Value::from("def")]);
        assert_eq!(&*listval.as_string(), "abc def");

        let listval = Value::from("qrs xyz");
        let result = listval.as_list();

        assert!(result.is_ok());

        if let Ok(rclist) = result {
            assert_eq!(rclist.len(), 2);
            assert_eq!(rclist[0].to_string(), "qrs".to_string());
            assert_eq!(rclist[1].to_string(), "xyz".to_string());
        }
    }

    #[test]
    fn from_to_flavor() {
        // Give a Flavor, get an Rc<Flavor> back.
        let myval = Value::from_other(Flavor::SALTY);
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SALTY);

        // Give a String, get an Rc<Flavor> back.
        let myval = Value::from("sweet");
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SWEET);

        // Flavor is Copy, so get a Flavor back
        let myval = Value::from_other(Flavor::SALTY);
        let result = myval.as_copy::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, Flavor::SALTY);
    }

    #[test]
    fn already_number() {
        // Can retrieve a DataRep::Int as a Datum::Int.
        let value = Value::from(123);
        let out = value.already_number();
        assert!(out.is_some());
        assert_eq!(out.unwrap(), Datum::int(123));

        // Can retrieve a DataRep::Flt as a Datum::Flt.
        let value = Value::from(45.6);
        let out = value.already_number();
        assert!(out.is_some());
        assert_eq!(out.unwrap(), Datum::float(45.6));

        // Other values, None.
        let value = Value::from("123");
        assert!(value.already_number().is_none());
    }

    // Sample external type, used for testing.

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Flavor {
        SALTY,
        SWEET,
    }

    impl FromStr for Flavor {
        type Err = String;

        fn from_str(value: &str) -> Result<Self, Self::Err> {
            let value = value.to_lowercase();

            if value == "salty" {
                Ok(Flavor::SALTY)
            } else if value == "sweet" {
                Ok(Flavor::SWEET)
            } else {
                Err("Not a flavor string".to_string())
            }
        }
    }

    impl fmt::Display for Flavor {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if *self == Flavor::SALTY {
                write!(f, "salty")
            } else {
                write!(f, "sweet")
            }
        }
    }
}
