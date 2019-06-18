//! The MoltValue Type
//!
//! The [`MoltValue`] struct is the standard representation of a data value
//! in the Molt language.  It represents a single immutable data value; the
//! data is reference-counted, so instances can be cloned efficiently.  The
//! data value can be any TCL data value: a number, a list, or any
//! arbitrary type (that meets certain requirements).
//!
//! In TCL "everything is a string"; thus, every `MoltValue` has a string
//! representation, or _string rep_.  But for efficiency with numbers, lists,
//! and user-defined binary data structures, the MoltValue also caches a
//! data representation, or _data rep_.
//!
//! A `MoltValue` can have just a string rep, just a data rep, or both.
//! Like the `Tcl_Obj` in standard TCL, the `MoltValue` is like a stork: it
//! can stand one leg, the other leg, or both legs.
//!
//! A client can ask the `MoltValue` for its string, which is always available
//! and will be computed from the data rep if it doesn't already exist.  (Once
//! computed, the string rep never changes.)  A client can also ask
//! the `MoltValue` for any other type it desires.  If the requested data rep
//! is already available, it will be returned; otherwise, the `MoltValue` will
//! attempt to parse it from the string_rep.  The last successful conversion is
//! cached for later.
//!
//! For example, consider the following sequence:
//!
//! * A computation yields a `MoltValue` containing the integer 5. The data rep is
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
//! `MoltValue` handles strings, integers, floating-point values, and lists as
//! special cases, since they are part of the language and are so frequently used.
//! In addition, a `MoltValue` can also contain any Rust struct that meets
//! certain requirements.
//!
//! # External Types
//!
//! Any struct that implements the `std::fmt::Display`, `std::fmt::Debug`,
//! and `std::str::FromStr` traits can be saved in a `MoltValue`.  The struct's
//! `Display` and `FromStr` trait implementations are used to do the string
//! rep/data rep conversions.  In particular:
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
//! TODO
//!
//! [`MoltValue`]: struct.MoltValue.html

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

/// The `MoltValue` type. See [the module level documentation](index.html) for more.
#[derive(Clone, Debug)]
pub struct MoltValue {
    string_rep: RefCell<Option<Rc<String>>>,
    data_rep: RefCell<Datum>,
}

impl Display for MoltValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl MoltValue {
    /// Creates a new `MoltValue` from the given string slice.
    ///
    /// Prefer [`from_string`](#method.from_string) if you have a newly created
    /// string for the `MoltValue` to take ownership of.
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// let value = MoltValue::from_str("My String Slice");
    /// assert_eq!(&*value.as_string(), "My String Slice");
    /// ```
    pub fn from_str(str: &str) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(Some(Rc::new(str.to_string()))),
            data_rep: RefCell::new(Datum::None),
        }
    }

    /// Creates a new `MoltValue` from the given String.
    ///
    /// Prefer [`from_str`](#method.from_string) if you have a string slice
    /// you'd otherwise have to create a new string from.
    /// 
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// let string = String::from("My New String");
    /// let value = MoltValue::from_string(string);
    /// assert_eq!(&*value.as_string(), "My New String");
    /// ```
    pub fn from_string(str: String) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(Some(Rc::new(str))),
            data_rep: RefCell::new(Datum::None),
        }
    }

    /// Returns the value's string representation as a reference-counted
    /// string.
    ///
    /// **Note**: This is the standard way of retrieving a `MoltValue`'s
    /// string rep, as unlike `to_string` it doesn't create a new `String`.
    ///
    /// # Example 
    ///
    /// ```
    /// use molt::MoltValue;
    /// let value = MoltValue::from_int(123);
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
    
    /// Creates a new `MoltValue` whose data representation is a `bool`.  The value's
    /// string representation will be "1" or "0".
    ///
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// let value = MoltValue::from_bool(true);
    /// assert_eq!(&*value.as_string(), "1");
    /// 
    /// let value = MoltValue::from_bool(false);
    /// assert_eq!(&*value.as_string(), "0");
    /// ```
    pub fn from_bool(flag: bool) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(Datum::Bool(flag)),
        }
    }

    /// Tries to return the `MoltValue` as a `bool`, parsing the
    /// value's string representation if necessary.
    /// 
    /// # Boolean Strings
    /// 
    /// The following string values can be interpreted as boolean values, regardless of case: `true`,
    /// `false`, `on`, `off`, `yes`, `no`, `1`, `0`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<bool,ResultCode> {
    /// let value = MoltValue::from_bool(true);
    /// let flag = value.as_bool()?;
    /// assert!(flag);
    /// 
    /// let value = MoltValue::from_str("no");
    /// let flag = value.as_bool()?;
    /// assert!(!flag);
    /// # Ok(true)
    /// # }
    /// ```
    pub fn as_bool(&self) -> Result<bool, ResultCode> { 
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have a boolean then just return it.
        if let Datum::Bool(flag) = *data_ref {
            return Ok(flag);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data_rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as an integer
        let str = (&*string_ref).as_ref().unwrap(); 
        let flag = MoltValue::parse_bool(&*str)?; 
        *data_ref = Datum::Bool(flag);
        Ok(flag)
    }

    // Parses a string as a boolean using the standard TCL syntax.
    // Returns a standard Molt error result.
    fn parse_bool(arg: &str) -> Result<bool, ResultCode> {
        let value: &str = &arg.to_lowercase();
        match value {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => molt_err!("expected boolean but got \"{}\"", arg),
        }
    }


    /// Creates a new `MoltValue` whose data representation is a `MoltInt`.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// 
    /// let value = MoltValue::from_int(123);
    /// assert_eq!(&*value.as_string(), "123");
    /// ```
    pub fn from_int(int: MoltInt) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(Datum::Int(int)),
        }
    }

    /// Tries to return the `MoltValue` as a `MoltInt`, parsing the
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
    /// use molt::MoltValue;
    /// use molt::types::MoltInt;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<MoltInt,ResultCode> {
    /// 
    /// let value = MoltValue::from_int(123);
    /// let int = value.as_int()?;
    /// assert_eq!(int, 123);
    /// 
    /// let value = MoltValue::from_str("OxFF");
    /// let int = value.as_int()?;
    /// assert_eq!(int, 255);
    /// # Ok(1)
    /// # }
    /// ```
    pub fn as_int(&self) -> Result<MoltInt, ResultCode> {
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have an integer then just return it.
        if let Datum::Int(int) = *data_ref {
            return Ok(int);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data_rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as an integer
        let str = (&*string_ref).as_ref().unwrap(); 
        let int = MoltValue::parse_int(&*str)?; 
        *data_ref = Datum::Int(int);
        Ok(int)
    }

    // Parses a string as an integer using the standard TCL syntax (except octal :-)
    // Returns a standard Molt error result.
    fn parse_int(arg: &str) -> Result<MoltInt, ResultCode> {
        let mut arg = arg;
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
            Err(_) => molt_err!("expected integer but got \"{}\"", arg),
        }
    }

    /// Creates a new `MoltValue` whose data representation is a `MoltFloat`.
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
    /// use molt::MoltValue;
    /// 
    /// let value = MoltValue::from_float(12.34);
    /// assert_eq!(&*value.as_string(), "12.34");
    /// ```
    pub fn from_float(flt: MoltFloat) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(Datum::Flt(flt)),
        }
    }

    /// Tries to return the `MoltValue` as a `MoltFloat`, parsing the
    /// value's string representation if necessary.
    ///
    /// # Floating-Point Syntax
    /// 
    /// Molt accepts the same floating-point strings as Rust's standard numeric parser.
    ///
    /// # Example
    ///
    /// ```
    /// use molt::MoltValue;
    /// use molt::types::MoltFloat;
    /// use molt::types::ResultCode;
    /// # fn dummy() -> Result<MoltFloat,ResultCode> {
    /// 
    /// let value = MoltValue::from_float(12.34);
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 12.34);
    /// 
    /// let value = MoltValue::from_str("23.45");
    /// let flt = value.as_float()?;
    /// assert_eq!(flt, 23.45);
    /// # Ok(1.0)
    /// # }
    /// ```
    pub fn as_float(&self) -> Result<MoltFloat, ResultCode> {
        let mut data_ref = self.data_rep.borrow_mut();
        let mut string_ref = self.string_rep.borrow_mut();

        // FIRST, if we have a float then just return it.
        if let Datum::Flt(flt) = *data_ref {
            return Ok(flt);
        }

        // NEXT, if we don't have a string_rep, get one from the current
        // data rep.
        if (*string_ref).is_none() {
            *string_ref = Some(Rc::new(data_ref.to_string()));
        }

        // NEXT, Try to parse the string_rep as a float
        // TODO: Currently uses the standard Rust parser.  That may
        // be OK, but I need to check.
        let str = (&*string_ref).as_ref().unwrap(); 
        let result = str.parse::<MoltFloat>();

        match result {
            Ok(flt) => {
                *data_ref = Datum::Flt(flt);
                Ok(flt)
            },
            Err(_) => {
                molt_err!("expected floating-point number but got \"{}\"", str)
            }
        }
    }

    /// Creates a new `MoltValue` whose data representation is a `MoltList`.
    ///
    /// # Example
    ///
    /// TODO
    pub fn from_list(list: MoltList) -> MoltValue {
        MoltValue {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(Datum::List(Rc::new(list))),
        }
    }

    /// Tries to return the `MoltValue` as a `MoltList`, parsing the
    /// value's string representation if necessary.
    ///
    /// TODO: Need to return Molt-compatible Err's.
    /// TODO: Need to add list parsing.
    ///
    /// # Example
    ///
    /// TODO
    pub fn as_list(&self) -> Result<Rc<MoltList>, ResultCode> {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let Datum::List(list) = &*data_ref {
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
        *data_ref = Datum::List(list.clone());

        Ok(list)
    }

    /// Creates a new `MoltValue` containing the given value of some user type.
    ///
    /// The type must implement `Display`, `Debug`, and `FromStr`, and the
    /// `Display` output must be compatible with the `FromStr` parser (and with
    /// TCL syntax).  The value will be reference counted.
    pub fn from_other<T: 'static>(value: T) -> MoltValue
    where
        T: Display + Debug,
    {
        MoltValue {
            string_rep: RefCell::new(None),
            data_rep: RefCell::new(Datum::Other(Rc::new(value))),
        }
    }

    /// Tries to interpret the `MoltValue` as a value of type `T`.
    ///
    /// The value is returned as an `Rc<T>`, as this allows the client to
    /// use the value freely.
    ///
    /// This method returns `Option` rather than `Result` because it is up
    /// to the caller to provide a meaningful error message.  It is normal
    /// for externally defined types to wrap this function in a function
    /// that does so.
    ///
    /// # Example
    ///
    /// TODO
    pub fn as_other<T: 'static>(&self) -> Option<Rc<T>>
    where
        T: Display + Debug + FromStr,
    {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let Datum::Other(other) = &*data_ref {
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
                *data_ref = Datum::Other(Rc::new(tval));
                return Some(out);
            }
        }

        // NEXT, we couldn't do it.
        None
    }

    /// Tries to interpret the `MoltValue` as a value of type `T`, returning
    /// a copy.
    ///
    /// This method returns `Option` rather than `Result` because it is up
    /// to the caller to provide a meaningful error message.  It is normal
    /// for externally defined types to wrap this function in a function
    /// that does so.
    ///
    /// # Example
    ///
    /// TODO
    pub fn as_copy<T: 'static>(&self) -> Option<T>
    where
        T: Display + Debug + FromStr + Copy,
    {
        let mut string_ref = self.string_rep.borrow_mut();
        let mut data_ref = self.data_rep.borrow_mut();

        // FIRST, if we have the desired type, return it.
        if let Datum::Other(other) = &*data_ref {
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
                *data_ref = Datum::Other(Rc::new(tval));
                return Some(*out);
            }
        }

        // NEXT, we couldn't do it.
        None
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
// Datum enum: a sum type for the different kinds of data_reps.

// The data representation for MoltValues.
#[derive(Clone, Debug)]
enum Datum {
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

    /// The MoltValue has no data rep at present.
    None,
}

impl Display for Datum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Datum::Bool(flag) => write!(f, "{}", if *flag { 1 } else { 0 }),
            Datum::Int(int) => write!(f, "{}", int),
            Datum::Flt(flt) => write!(f, "{}", flt),
            Datum::List(list) => write!(f, "{}", list_to_string(list)),
            Datum::Other(other) => write!(f, "{}", other),
            Datum::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::str::FromStr;

    #[test]
    fn to_string() {
        let val = MoltValue::from_string("abc".to_string());
        assert_eq!(*val.to_string(), "abc".to_string());

        let val2 = val.clone();
        assert_eq!(*val.to_string(), *val2.to_string());
    }

    #[test]
    fn as_string() {
        let val = MoltValue::from_string("abc".to_string());
        assert_eq!(*val.as_string(), "abc".to_string());

        let val2 = val.clone();
        assert_eq!(*val.as_string(), *val2.to_string());
    }

    #[test]
    fn from_as_bool() {
        let val = MoltValue::from_bool(true);
        assert_eq!(*val.to_string(), "1".to_string());

        let val = MoltValue::from_bool(false);
        assert_eq!(*val.to_string(), "0".to_string());

        let val = MoltValue::from_string("true".to_string());
        assert_eq!(val.as_bool(), Ok(true));
    }

    #[test]
    fn parse_bool() {
        assert_eq!(Ok(true), MoltValue::parse_bool("1"));
        assert_eq!(Ok(true), MoltValue::parse_bool("true"));
        assert_eq!(Ok(true), MoltValue::parse_bool("yes"));
        assert_eq!(Ok(true), MoltValue::parse_bool("on"));
        assert_eq!(Ok(true), MoltValue::parse_bool("TRUE"));
        assert_eq!(Ok(true), MoltValue::parse_bool("YES"));
        assert_eq!(Ok(true), MoltValue::parse_bool("ON"));
        assert_eq!(Ok(false), MoltValue::parse_bool("0"));
        assert_eq!(Ok(false), MoltValue::parse_bool("false"));
        assert_eq!(Ok(false), MoltValue::parse_bool("no"));
        assert_eq!(Ok(false), MoltValue::parse_bool("off"));
        assert_eq!(Ok(false), MoltValue::parse_bool("FALSE"));
        assert_eq!(Ok(false), MoltValue::parse_bool("NO"));
        assert_eq!(Ok(false), MoltValue::parse_bool("OFF"));
        assert_eq!(MoltValue::parse_bool("nonesuch"),
            molt_err!("expected boolean but got \"nonesuch\""));
    }

    #[test]
    fn from_as_int() {
        let val = MoltValue::from_int(5);
        assert_eq!(*val.to_string(), "5".to_string());
        assert_eq!(val.as_int(), Ok(5));
        assert_eq!(val.as_float(), Ok(5.0));

        let val = MoltValue::from_string("7".to_string());
        assert_eq!(*val.to_string(), "7".to_string());
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        // TODO: Note, 7.0 might not get converted to "7" long term.
        // In Standard TCL, its string_rep would be "7.0".  Need to address
        // MoltFloat formatting/parsing.
        let val = MoltValue::from_float(7.0);
        assert_eq!(*val.to_string(), "7".to_string());
        assert_eq!(val.as_int(), Ok(7));
        assert_eq!(val.as_float(), Ok(7.0));

        let val = MoltValue::from_string("abc".to_string());
        // assert_eq!(val.as_int(), Err("Not an integer".to_string()));
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"abc\""));
    }

    #[test]
    fn parse_int() {
        assert_eq!(MoltValue::parse_int("1"), Ok(1));
        assert_eq!(MoltValue::parse_int("-1"), Ok(-1));
        assert_eq!(MoltValue::parse_int("+1"), Ok(1));
        assert_eq!(MoltValue::parse_int("0xFF"), Ok(255));
        assert_eq!(MoltValue::parse_int("+0xFF"), Ok(255));
        assert_eq!(MoltValue::parse_int("-0xFF"), Ok(-255));

        assert_eq!(MoltValue::parse_int(""), molt_err!("expected integer but got \"\""));
        assert_eq!(MoltValue::parse_int("a"), molt_err!("expected integer but got \"a\""));
        assert_eq!(MoltValue::parse_int("0x"), molt_err!("expected integer but got \"0x\""));
        assert_eq!(MoltValue::parse_int("0xABGG"),
            molt_err!("expected integer but got \"0xABGG\""));
    }

    #[test]
    fn from_as_float() {
        let val = MoltValue::from_float(12.5);
        assert_eq!(*val.to_string(), "12.5".to_string());
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"12.5\""));
        // assert_eq!(val.as_int(), Err("Not an integer".to_string()));
        assert_eq!(val.as_float(), Ok(12.5));

        let val = MoltValue::from_string("7.8".to_string());
        assert_eq!(*val.to_string(), "7.8".to_string());
        assert_eq!(val.as_int(), molt_err!("expected integer but got \"7.8\""));
        // assert_eq!(val.as_int(), Err("Not an integer".to_string()));
        assert_eq!(val.as_float(), Ok(7.8));

        let val = MoltValue::from_int(5);
        assert_eq!(val.as_float(), Ok(5.0));

        let val = MoltValue::from_string("abc".to_string());
        assert_eq!(val.as_float(), 
            molt_err!("expected floating-point number but got \"abc\""));
    }

    #[test]
    fn from_as_list() {
        // NOTE: we aren't testing list formatting and parsing here; that's done in list.rs.
        // We *are* testing that MoltValue will use the list.rs code to convert strings to lists
        // and back again.
        let listval = MoltValue::from_list(vec!["abc".to_string(), "def".to_string()]);
        assert_eq!(*listval.as_string(), "abc def".to_string());

        let listval = MoltValue::from_string("qrs xyz".to_string());
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
        let myval = MoltValue::from_other(Flavor::SALTY);
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SALTY);

        // Give a String, get an Rc<Flavor> back.
        let myval = MoltValue::from_string("sweet".to_string());
        let result = myval.as_other::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(*out, Flavor::SWEET);

        // Flavor is Copy, so get a Flavor back
        let myval = MoltValue::from_other(Flavor::SALTY);
        let result = myval.as_copy::<Flavor>();
        assert!(result.is_some());
        let out = result.unwrap();
        assert_eq!(out, Flavor::SALTY);
    }

    // Sample external type, used for testing.

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum Flavor {
        SALTY,
        SWEET,
    }

    impl Flavor {
        // TODO: The error should be a Molt ResultCode.
        // TODO: This should move to the example.
        pub fn from_molt(value: &MoltValue) -> Result<Self, String> {
            if let Some(x) = value.as_copy::<Flavor>() {
                Ok(x)
            } else {
                Err("Not a flavor string".to_string())
            }
        }
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
