//! The Expr command and parser
//!
//! * Ultimately, the command should probably move to commands.rs.
//!   But this is convenient for now.

use crate::char_ptr::CharPtr;
use crate::*;
use crate::interp::Interp;

//------------------------------------------------------------------------------------------------
// expr command
//
// TODO: Move to commands.rs

/// # expr expr
///
/// Evaluates an expression and returns its result.
///
/// ## TCL Liens
///
/// In standard TCL, `expr` takes any number of arguments which it combines into
/// a single expression for evaluation.  However, it is well understood in the
/// TCL community that you should "brace your expressions", i.e., `expr` should
/// always be written with a single braced argument, e.g.,
///
/// ```tcl
/// expr {$x + $y}
/// ```
///
/// Otherwise, the interpreter does two rounds of variable and command interpolation,
/// one as part of the normal command parsing, and one as part of the expression
/// parsing.  This is horrible for performance, and can also lead to subtle errors
/// if the expression parser expands things it shouldn't.  Consequently, Molt
/// requires a single argument.
pub fn cmd_expr(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "expr")?;

    molt_err!("TODO")
}

//------------------------------------------------------------------------------------------------
// Value Representation
//
// TODO: At some point we might want to make this a two-legged struct, a la TclObj.

type ValueResult = Result<Value,ResultCode>;

/// The value type.  Includes the parsed value.
#[derive(Debug)]
enum ValueType {
    Int(MoltInt),
    Float(MoltFloat),
    Str(String),
    None, // Equivalent to Str("").
}

/// A parsed value
#[derive(Debug)]
struct Value {
    vtype: ValueType,
}

impl Value {
    fn new() -> Self {
        Self {
            vtype: ValueType::None,
        }
    }

    fn int(int: MoltInt) -> Self {
        Self { vtype: ValueType::Int(int) }
    }

    fn float(flt: MoltFloat) -> Self {
        Self { vtype: ValueType::Float(flt) }
    }

    fn string(string: &str) -> Self {
        Self { vtype: ValueType::Str(string.into()) }
    }
}

//------------------------------------------------------------------------------------------------
// Parsing Context

/// Context for expr parsing
struct ExprInfo<'a> {
    // The full expr.
    original_expr: String,

    // The input iterator, e.g., the pointer to the next character.
    expr: CharPtr<'a>,

    // Last token's type; see constants
    token: i32,

    // No Evaluation flag.
    // TODO: consider moving to Interp.
    no_eval: bool,
}

impl<'a> ExprInfo<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            original_expr: expr.to_string(),
            expr: CharPtr::new(expr),
            token: -1,
            no_eval: false,
        }
    }
}

//------------------------------------------------------------------------------------------------
// Constants and Lookup Tables

// Token constants
//
// The token types are defined below.  In addition, there is a table
// associating a precedence with each operator.  The order of types
// is important.  Consult the code before changing it.

const VALUE: i32 = 0;
const OPEN_PAREN: i32 = 1;
const CLOSE_PAREN: i32 = 2;
const COMMA: i32 = 3;
const END: i32 = 4;
const UNKNOWN: i32 = 5;

// Tokens 6 and 7 are unused.

// Binary operators:
const MULT: i32 = 8;
const DIVIDE: i32 = 9;
const MOD: i32 = 10;
const PLUS: i32 = 11;
const MINUS: i32 = 12;
const LEFT_SHIFT: i32 = 13;
const RIGHT_SHIFT: i32 = 14;
const LESS: i32 = 15;
const GREATER: i32 = 16;
const LEQ: i32 = 17;
const GEQ: i32 = 18;
const EQUAL: i32 = 19;
const NEQ: i32 = 20;
const BIT_AND: i32 = 21;
const BIT_XOR: i32 = 22;
const BIT_OR: i32 = 23;
const AND: i32 = 24;
const OR: i32 = 25;
const QUESTY: i32 = 26;
const COLON: i32 = 27;

// Unary operators:
const UNARY_MINUS: i32 = 28;
const UNARY_PLUS: i32 = 29;
const NOT: i32 = 30;
const BIT_NOT: i32 = 31;

// Precedence table.  The values for non-operator token types are ignored.

const PREC_TABLE: [i32; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    12, 12, 12, // MULT, DIVIDE, MOD
    11, 11, // PLUS, MINUS
    10, 10, // LEFT_SHIFT, RIGHT_SHIFT
    9, 9, 9, 9, // LESS, GREATER, LEQ, GEQ
    8, 8, // EQUAL, NEQ
    7, // BIT_AND
    6, // BIT_XOR
    5, // BIT_OR
    4, // AND
    3, // OR
    2, // QUESTY
    1, // COLON
    13, 13, 13, 13, // UNARY_MINUS, UNARY_PLUS, NOT, BIT_NOT
];

const OP_STRINGS: [&str; 32] = [
    "VALUE", "(", ")", ",", "END", "UNKNOWN", "6", "7",
    "*", "/", "%", "+", "-", "<<", ">>", "<", ">", "<=",
    ">=", "==", "!=", "&", "^", "|", "&&", "||", "?", ":",
    "-", "+", "!", "~"
];

//------------------------------------------------------------------------------------------------
// Public API

/// Evaluates an expression and returns its value in string form.
pub fn molt_expr_string(interp: &mut Interp, string: &str) -> InterpResult {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        ValueType::Int(int) => molt_ok!("{}", int),
        ValueType::Float(flt) => molt_ok!("{}", flt), // TODO: better float->string logic
        ValueType::Str(str) => molt_ok!(str),
        ValueType::None => molt_ok!(""),
    }
}

/// Evaluates an expression and returns its value as a Molt integer.
pub fn molt_expr_int(interp: &mut Interp, string: &str) -> Result<MoltInt, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        ValueType::Int(int) => Ok(int),
        ValueType::Float(flt) => Ok(flt as MoltInt),
        _ => molt_err!("expression didn't have numeric value"),
    }
}

/// Evaluates an expression and returns its value as a Molt float.
pub fn molt_expr_float(interp: &mut Interp, string: &str) -> Result<MoltFloat, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        ValueType::Int(int) => Ok(int as MoltFloat),
        ValueType::Float(flt) => Ok(flt),
        _ => molt_err!("expression didn't have numeric value"),
    }
}

/// Evaluates an expression and returns its value as a boolean.
pub fn molt_expr_bool(interp: &mut Interp, string: &str) -> Result<bool, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        ValueType::Int(int) => Ok(int != 0),
        ValueType::Float(flt) => Ok(flt != 0.0),
        ValueType::Str(str) => get_boolean(&str),
        ValueType::None => get_boolean(""),
    }
}

//------------------------------------------------------------------------------------------------
// Expression Internals

/// Provides top-level functionality shared by molt_expr_string, molt_expr_int, etc.
fn expr_top_level<'a>(interp: &mut Interp, string: &'a str) -> ValueResult {
    let info = &mut ExprInfo::new(string);

    let value = expr_get_value(interp, info, -1)?;

    if info.token != END {
        return molt_err!("syntax error in expression \"{}\"", string);
    }

    if let ValueType::Float(_) = value.vtype {
        // TODO: check for NaN, INF, and throw IEEE floating point error.
    }

    Ok(value)
}

/// Parse a "value" from the remainder of the expression in info.
/// The `prec` is a precedence value; treat any unparenthesized operator
/// with precedence less than or equal to `prec` as the end of the
/// expression.
fn expr_get_value<'a>(interp: &mut Interp, info: &'a mut ExprInfo, prec: i32) -> ValueResult {
    // There are two phases to this procedure.  First, pick off an initial value.
    // Then, parse (binary operator, value) pairs until done.
    let mut got_op = false;
    let mut value = expr_lex(interp, info)?;
    let mut operator: i32 = -1;

    if info.token == OPEN_PAREN {
        // Parenthesized sub-expression.
        value = expr_get_value(interp, info, -1)?;

        if info.token != CLOSE_PAREN {
            return molt_err!("unmatched parentheses in expression \"{}\"", info.original_expr);
        }
    } else {
        if info.token == MINUS {
            info.token = UNARY_MINUS;
        }

        if info.token == PLUS {
            info.token = UNARY_PLUS;
        }

        if info.token >= UNARY_MINUS {
            // Process unary operators
            operator = info.token;
            value = expr_get_value(interp, info, PREC_TABLE[info.token as usize])?;

            if !info.no_eval {
                match operator {
                    UNARY_MINUS => {
                        match value.vtype {
                            ValueType::Int(int) => {
                                value.vtype = ValueType::Int(-int);
                            }
                            ValueType::Float(flt) => {
                                value.vtype = ValueType::Float(-flt);
                            }
                            _ => {
                                return illegal_type(&value, operator);
                            }
                        }
                    }
                    UNARY_PLUS  => {
                        if !value.is_numeric() {
                            return illegal_type(&value, operator);
                        }
                    }
                    NOT => {
                        match value.vtype {
                            ValueType::Int(int) => {
                                if int == 0 {
                                    value.vtype = ValueType::Int(1);
                                } else {
                                    value.vtype = ValueType::Int(0);
                                }
                            }
                            ValueType::Float(flt) => {
                                if flt == 0.0 {
                                    value.vtype = ValueType::Int(1);
                                } else {
                                    value.vtype = ValueType::Int(0);
                                }
                            }
                            _ => {
                                return illegal_type(&value, operator);
                            }
                        }
                    }
                    BIT_NOT => {
                        if let ValueType::Int(int) = value.vtype {
                            value.vtype = ValueType::Int(!int);
                        } else {
                            return illegal_type(&value, operator);
                        }
                    }
                    _ => {
                        return molt_err!("unknown unary op: \"{}\"", operator);
                    }
                }
            }
            got_op = true;
        } else if info.token != VALUE {
            return syntax_error(info);
        }
    }

    // Got the first operand.  Now fetch (operator, operand) pairs

    let mut value2 = Value::new();

    if !got_op {
        // TODO: There is serious magic going on here in the TCL code.
        // with the value's "ParseValue" struct.
        value2 = expr_lex(interp, info)?;
    }

    loop {
        operator = info.token;
        // ??? value2.pv.next = value2.pv.buffer;

        if operator < MULT || operator >= UNARY_MINUS {
            if operator == END || operator == CLOSE_PAREN || operator == COMMA {
                // goto done, Ok
                // TODO: What value are we returning?
                // It appears that interp->result was set by something called
                // by this routine.
                return Ok(Value::new());
            } else {
                return syntax_error(info);
            }
        }

        if PREC_TABLE[operator as usize] <= prec {
            // TODO: What value should we be returning?
            return Ok(Value::new());
        }

        // If we're doing an AND or OR and the first operand already determines
        // the result, don't execute anything in the second operand: just parse.
        // Same style for ?: pairs.

        if operator == AND || operator == OR || operator == QUESTY {

        }
    }

    // Just to quiet the warnings for now.
    Ok(Value::new())
}

/// Lexical analyzer for the expression parser.  Parses a single value, operator, or other
/// syntactic element from an expression string.
///
/// ## Results
///
/// Returns an error result if an error occurs while doing lexical analysis or
/// executing an embedded command.  On success, info.token is set to the last token type,
/// and info is updated to point to the next token.  If the token is VALUE, the returned
/// Value contains it; otherwise, the value is ValueType::None.
///
/// TODO: It might be better to combine info.token and the value into one data object,
/// i.e., add ValueType::Op(i32) or make each token type a Value (and handle precedence).
/// But one step at a time.
fn expr_lex(_interp: &mut Interp, info: &mut ExprInfo) -> ValueResult {
    // FIRST, skip white space.
    let mut p = info.expr.clone();

    p.skip_while(|c| c.is_whitespace());

    if p.is_none() {
        info.token = END;
        info.expr = p;
        return Ok(Value::new());
    }

    // First try to parse the token as an integer or floating-point number.
    // Don't want to check for a number if the first character is "+"
    // or "-".  If we do, we might treat a binary operator as unary by
    // mistake, which will eventually cause a syntax error.

    if !p.is('+') && !p.is('-') {
        if expr_looks_like_int(&p) {
            // There's definitely an integer to parse; parse it.
            let token = util::read_int(&mut p).unwrap();
            let int = get_int(&token)?;
            info.token = VALUE;
            info.expr = p;
            return Ok(Value::int(int));
        } else if let Some(token) = util::read_float(&mut p) {
            p.skip_while(|c| c.is_whitespace());

            if p.is_none() {
                info.token = VALUE;
                info.expr = p;
                return Ok(Value::float(get_float(&token)?));
            }
        }
    }

    // It isn't a number, so the next character will determine what it is.
    info.expr = p.clone();
    info.expr.skip();

    match p.peek() {
        Some('$') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('[') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('"') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('{') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('(') => {
            info.token = OPEN_PAREN;
            Ok(Value::new())
        }
        Some(')') => {
            info.token = CLOSE_PAREN;
            Ok(Value::new())
        }
        Some(',') => {
            info.token = COMMA;
            Ok(Value::new())
        }
        Some('*') => {
            info.token = MULT;
            Ok(Value::new())
        }
        Some('/') => {
            info.token = DIVIDE;
            Ok(Value::new())
        }
        Some('%') => {
            info.token = MOD;
            Ok(Value::new())
        }
        Some('+') => {
            info.token = PLUS;
            Ok(Value::new())
        }
        Some('-') => {
            info.token = MINUS;
            Ok(Value::new())
        }
        Some('?') => {
            info.token = QUESTY;
            Ok(Value::new())
        }
        Some(':') => {
            info.token = COLON;
            Ok(Value::new())
        }
        Some('<') => {
            p.skip();
            match p.peek() {
                Some('<') => {
                    info.token = LEFT_SHIFT;
                    p.skip();
                    info.expr = p;
                    Ok(Value::new())
                }
                Some('=') => {
                    info.token = LEQ;
                    p.skip();
                    info.expr = p;
                    Ok(Value::new())
                }
                _ => {
                    info.token = LESS;
                    Ok(Value::new())
                }
            }
        }
        Some('>') => {
            p.skip();
            match p.peek() {
                Some('>') => {
                    info.token = RIGHT_SHIFT;
                    p.skip();
                    info.expr = p;
                    Ok(Value::new())
                }
                Some('=') => {
                    info.token = GEQ;
                    p.skip();
                    info.expr = p;
                    Ok(Value::new())
                }
                _ => {
                    info.token = GREATER;
                    Ok(Value::new())
                }
            }
        }
        Some('=') => {
            p.skip();
            if let Some('=') = p.peek() {
                info.token = EQUAL;
                p.skip();
                info.expr = p;
            } else {
                info.token = UNKNOWN;
            }
            Ok(Value::new())
        }
        Some('!') => {
            p.skip();
            if let Some('=') = p.peek() {
                info.token = NEQ;
                p.skip();
                info.expr = p;
            } else {
                info.token = NOT;
            }
            Ok(Value::new())
        }
        Some('&') => {
            p.skip();
            if let Some('=') = p.peek() {
                info.token = AND;
                p.skip();
                info.expr = p;
            } else {
                info.token = BIT_AND;
            }
            Ok(Value::new())
        }
        Some('^') => {
            info.token = BIT_XOR;
            Ok(Value::new())
        }
        Some('|') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('~') => {
            info.token = BIT_NOT;
            Ok(Value::new())
        }
        Some(_) => {
            // TODO: if is alphabetic, see if it's a function.
            p.skip();
            info.expr = p;
            info.token = UNKNOWN;
            Ok(Value::new())
        }
        None => {
            p.skip();
            info.expr = p;
            info.token = UNKNOWN;
            Ok(Value::new())
        }
    }
}

/// Given a string (such as one coming from command or variable substitution) make a
/// Value based on the string.  The value will be floating-point or integer if possible,
/// or else it will just be a copy of the string.  Returns an error on failed numeric
/// conversions.
fn expr_parse_string(_interp: &mut Interp, string: &str) -> ValueResult {
    if !string.is_empty() {
        let mut value = Value::new();
        let mut p = CharPtr::new(string);

        if expr_looks_like_int(&p) {
            // FIRST, skip leading whitespace.
            p.skip_while(|c| c.is_whitespace());

            // NEXT, get the integer token from it.  We know there has to be something,
            // since it "looks like int".
            let token = util::read_int(&mut p).unwrap();

            // NEXT, did we read the whole string?  If not, it isn't really an integer.
            // Otherwise, drop through and return it as a string.
            p.skip_while(|c| c.is_whitespace());

            if p.is_none() {
                let int = get_int(&token)?;
                value.vtype = ValueType::Int(int);
                return Ok(value);
            }
        } else {
            // FIRST, see if it's a double. Skip leading whitespace.
            p.skip_while(|c| c.is_whitespace());

            // NEXT, see if we can get a float token from it.
            // since it "looks like int".
            if let Some(token) = util::read_float(&mut p) {
                // Did we read the whole string?  If not, it isn't really a float.
                // Otherwise, drop through and return it as a string.
                p.skip_while(|c| c.is_whitespace());

                if p.is_none() {
                    let flt = get_float(&token)?;
                    value.vtype = ValueType::Float(flt);
                    return Ok(value);
                }
            }
        }
    }

    Ok(Value::string(string))
}

/// Converts a value from int or double representation to a string, if it wasn't
/// already.
///
/// **Note:** In the TCL code, the interp is used for the floating point precision.
/// At some point I might add that.
fn expr_make_string(_interp: &mut Interp, value: &mut Value) {
    match value.vtype {
        ValueType::Int(val) => value.vtype=ValueType::Str(format!("{}", val)),
        ValueType::Float(val) => value.vtype=ValueType::Str(format!("{}", val)),
        _ => {},
    }
}

fn expr_looks_like_int<'a>(ptr: &CharPtr<'a>) -> bool {
    // FIRST, skip whitespace
    let mut p = ptr.clone();
    p.skip_while(|c| c.is_whitespace());

    if p.is('+') || p.is('-') {
        p.skip();
    }

    if !p.is_digit() {
        return false;
    }
    p.skip();

    while p.is_digit() {
        p.skip();
    }

    !p.is('.') && !p.is('e') && !p.is('E')
}

impl Value {
    fn is_numeric(&self) -> bool {
        match self.vtype {
            ValueType::Int(_) => true,
            ValueType::Float(_) => true,
            ValueType::Str(_) => false,
            ValueType::None => false,
        }
    }
}

// Return standard syntax error
fn syntax_error(info: &mut ExprInfo) -> ValueResult {
    molt_err!("syntax error in expression \"{}\"", info.original_expr)
}

// Return standard illegal type error
fn illegal_type(value: &Value, op: i32) -> ValueResult {
    let type_str = if let ValueType::Float(_) = value.vtype {
        "floating-point value"
    } else {
        "non-numeric string"
    };

    molt_err!("can't use {} as operand of \"{}\"", type_str, OP_STRINGS[op as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_expr_looks_like_int(str: &str) -> bool {
        let p = CharPtr::new(str);

        expr_looks_like_int(&p)
    }

    // Note: comparing floating point values for equality is usually a mistake.  In this
    // case, we are simply converting simple floating-point values to and from strings, and
    // verifying that we got the number we expected, so this is probably OK.
    #[allow(clippy::float_cmp)]
    fn veq(val1: &Value, val2: &Value) -> bool {
        match &val1.vtype {
            ValueType::Int(v1) => {
                match &val2.vtype {
                    ValueType::Int(v2) => v1 == v2,
                    _ => false,
                }
            }
            ValueType::Float(v1) => {
                match &val2.vtype {
                    ValueType::Float(v2) => v1 == v2,
                    _ => false,
                }
            }
            ValueType::Str(v1) => {
                match &val2.vtype {
                    ValueType::Str(v2) => v1 == v2,
                    ValueType::None => v1.is_empty(),
                    _ => false,
                }
            }
            ValueType::None => {
                match &val2.vtype {
                    ValueType::Str(v2) => v2.is_empty(),
                    ValueType::None => true,
                    _ => false,
                }
            }
        }
    }

    #[test]
    fn test_expr_looks_like_int() {
        assert!(call_expr_looks_like_int("1"));
        assert!(call_expr_looks_like_int("+1"));
        assert!(call_expr_looks_like_int("-1"));
        assert!(call_expr_looks_like_int("123"));
        assert!(call_expr_looks_like_int("123a"));
        assert!(!call_expr_looks_like_int(""));
        assert!(!call_expr_looks_like_int("a"));
        assert!(!call_expr_looks_like_int("123."));
        assert!(!call_expr_looks_like_int("123e"));
        assert!(!call_expr_looks_like_int("123E"));
        assert!(!call_expr_looks_like_int("."));
        assert!(!call_expr_looks_like_int("e"));
        assert!(!call_expr_looks_like_int("E"));
    }

    #[test]
    fn test_expr_make_string() {
        let mut interp = Interp::new();

        let mut value = Value::int(123);
        expr_make_string(&mut interp, &mut value);
        assert!(veq(&value, &Value::string("123")));

        let mut value = Value::float(1.1);
        expr_make_string(&mut interp, &mut value);
        assert!(veq(&value, &Value::string("1.1")));

        let mut value = Value::string("abc");
        expr_make_string(&mut interp, &mut value);
        assert!(veq(&value, &Value::string("abc")));
    }

    #[test]
    fn test_expr_parse_string() {
        let mut interp = Interp::new();

        let result = expr_parse_string(&mut interp, "");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::string("")));

        let result = expr_parse_string(&mut interp, "abc");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::string("abc")));

        let result = expr_parse_string(&mut interp, " 123abc");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::string(" 123abc")));

        let result = expr_parse_string(&mut interp, " 123.0abc");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::string(" 123.0abc")));

        let result = expr_parse_string(&mut interp, " 123   ");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::int(123)));

        let result = expr_parse_string(&mut interp, " 1.0   ");
        assert!(result.is_ok());
        assert!(veq(&result.unwrap(), &Value::float(1.0)));

        let result = expr_parse_string(&mut interp, "1234567890123456789012345678901234567890");
        assert!(result.is_err());

        // Should have an example of a float overflow/underflow, but I've not found a literal
        // string that gives one.
    }
}
