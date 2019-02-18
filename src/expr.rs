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
pub fn cmd_expr(interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "expr")?;

    molt_expr_string(interp, argv[1])
}

//------------------------------------------------------------------------------------------------
// Value Representation
//
// TODO: At some point we might want to make this a two-legged struct, a la TclObj.

type ValueResult = Result<Value,ResultCode>;

/// The value type.  Includes the parsed value.
#[derive(Debug,PartialEq,Eq,Copy,Clone)]
enum Type {
    Int,
    Float,
    String,
}

/// A parsed value.
///
/// **Note**: Originally did this as a struct containing an enum with associated values
/// for the data, but that complicated the logic.  We need to easily compare the types
/// of two values (which `if let` doesn't allow), and we need to be able to refer to a
/// type without reference to the typed value.
///
/// I could have used a union to save space, but we don't keep large numbers of these
/// around.
#[derive(Debug)]
struct Value {
    vtype: Type,
    int: MoltInt,
    flt: MoltFloat,
    str: String,
}

impl Value {
    fn none() -> Self {
        Self {
            vtype: Type::String,
            int: 0,
            flt: 0.0,
            str: String::new(),
        }
    }

    fn int(int: MoltInt) -> Self {
        Self {
            vtype: Type::Int,
            int: int,
            flt: 0.0,
            str: String::new(),
        }
    }

    fn float(flt: MoltFloat) -> Self {
        Self {
            vtype: Type::Float,
            int: 0,
            flt: flt,
            str: String::new(),
        }
    }

    fn string(string: &str) -> Self {
        Self {
            vtype: Type::String,
            int: 0,
            flt: 0.0,
            str: string.to_string(),
        }
    }

    // Only for checking integers.
    fn is_true(&self) -> bool {
        match self.vtype {
            Type::Int => self.int != 0,
            _ => {
                panic!("Value::is_true called for non-integer");
            }
        }
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

    // No Evaluation if > 0
    no_eval: i32,
}

impl<'a> ExprInfo<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            original_expr: expr.to_string(),
            expr: CharPtr::new(expr),
            token: -1,
            no_eval: 0,
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
        Type::Int => molt_ok!("{}", value.int),
        Type::Float => molt_ok!("{}", value.flt), // TODO: better float->string logic
        Type::String => molt_ok!(value.str),
    }
}

/// Evaluates an expression and returns its value as a Molt integer.
pub fn molt_expr_int(interp: &mut Interp, string: &str) -> Result<MoltInt, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        Type::Int => Ok(value.int),
        Type::Float => Ok(value.flt as MoltInt),
        _ => molt_err!("expression didn't have numeric value"),
    }
}

/// Evaluates an expression and returns its value as a Molt float.
pub fn molt_expr_float(interp: &mut Interp, string: &str) -> Result<MoltFloat, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        Type::Int => Ok(value.int as MoltFloat),
        Type::Float => Ok(value.flt),
        _ => molt_err!("expression didn't have numeric value"),
    }
}

/// Evaluates an expression and returns its value as a boolean.
pub fn molt_expr_bool(interp: &mut Interp, string: &str) -> Result<bool, ResultCode> {
    let value = expr_top_level(interp, string)?;

    match value.vtype {
        Type::Int => Ok(value.int != 0),
        Type::Float => Ok(value.flt != 0.0),
        Type::String => get_boolean(&value.str),
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

    if value.vtype == Type::Float {
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
    let mut value2 = Value::none();
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

            if info.no_eval == 0 {
                match operator {
                    UNARY_MINUS => {
                        match value.vtype {
                            Type::Int => {
                                value.int = -value.int;
                            }
                            Type::Float => {
                                value.flt = -value.flt;
                            }
                            _ => {
                                return illegal_type(value.vtype, operator);
                            }
                        }
                    }
                    UNARY_PLUS  => {
                        if !value.is_numeric() {
                            return illegal_type(value.vtype, operator);
                        }
                    }
                    NOT => {
                        match value.vtype {
                            Type::Int => {
                                // NOTE: Tcl uses !int here, but in Rust !int_value is a bitwise
                                // operator, not a logical one.
                                if value.int == 0 {
                                    value.int = 1;
                                } else {
                                    value.int = 0;
                                }
                            }
                            Type::Float => {
                                if value.flt == 0.0 {
                                    value = Value::int(1);
                                } else {
                                    value = Value::int(0);
                                }
                            }
                            _ => {
                                return illegal_type(value.vtype, operator);
                            }
                        }
                    }
                    BIT_NOT => {
                        if let Type::Int = value.vtype {
                            // Note: in Rust, unlike C, !int_value is a bitwise operator.
                            value.int = !value.int;
                        } else {
                            return illegal_type(value.vtype, operator);
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

    if !got_op {
        // This reads the next token, which we expect to be an operator.
        value2 = expr_lex(interp, info)?;
    }

    loop {
        operator = info.token;
        // ??? value2.pv.next = value2.pv.buffer;

        if operator < MULT || operator >= UNARY_MINUS {
            if operator == END || operator == CLOSE_PAREN || operator == COMMA {
                return Ok(value);
            } else {
                return syntax_error(info);
            }
        }

        if PREC_TABLE[operator as usize] <= prec {
            return Ok(value);
        }

        // If we're doing an AND or OR and the first operand already determines
        // the result, don't execute anything in the second operand: just parse.
        // Same style for ?: pairs.

        if operator == AND || operator == OR || operator == QUESTY {
            // For these operators, we need an integer value.  Convert or return
            // an error.
            match value.vtype {
                Type::Float => {
                    if value.flt == 0.0 {
                        value = Value::int(0);
                    } else {
                        value = Value::int(1);
                    }
                }
                Type::String => {
                    if info.no_eval == 0 {
                        return illegal_type(value.vtype, operator);
                    }
                    value = Value::int(0);
                }
                _ => {}
            }

            if (operator == AND && !value.is_true()) || (operator == OR && value.is_true()) {
                // Short-circuit; we don't care about the next operand, but it must be
                // syntactically correct.
                info.no_eval += 1;
                let _ = expr_get_value(interp, info, PREC_TABLE[operator as usize])?;
                info.no_eval -= 1;

                if operator == OR {
                    value = Value::int(1);
                }

                // Go on to the next operator.
                continue;
            } else if operator == QUESTY {
                return molt_err!("QUESTY not implemented yet!");
            } else {
                value2 = expr_get_value(interp, info, PREC_TABLE[operator as usize])?;
            }
        } else {
            value2 = expr_get_value(interp, info, PREC_TABLE[operator as usize])?;
        }

        if info.token < MULT
            && info.token != VALUE
            && info.token != END
            && info.token != COMMA
            && info.token != CLOSE_PAREN
        {
            return syntax_error(info);
        }

        if info.no_eval > 0 {
            continue;
        }

        // At this point we've got two values and an operator.  Check to make sure that the
        // particular data types are appropriate for the particular operator, and perform
        // type conversion if necessary.

        match operator {
            MULT | DIVIDE | PLUS | MINUS => {
            }
            _ => return molt_err!("Not yet implemented: check ops {}", OP_STRINGS[operator as usize]),
        }

        // Carry out the function of the specified operator.
        match operator {
            _ => return molt_err!("Not yet implemented: eval {}", OP_STRINGS[operator as usize]),
        }
    }
}

/// Lexical analyzer for the expression parser.  Parses a single value, operator, or other
/// syntactic element from an expression string.
///
/// ## Results
///
/// Returns an error result if an error occurs while doing lexical analysis or
/// executing an embedded command.  On success, info.token is set to the last token type,
/// and info is updated to point to the next token.  If the token is VALUE, the returned
/// Value contains it; otherwise, the value is Type::None.
///
/// TODO: It might be better to combine info.token and the value into one data object,
/// i.e., add Type::Op(i32) or make each token type a Value (and handle precedence).
/// But one step at a time.
fn expr_lex(_interp: &mut Interp, info: &mut ExprInfo) -> ValueResult {
    // FIRST, skip white space.
    let mut p = info.expr.clone();

    p.skip_while(|c| c.is_whitespace());

    if p.is_none() {
        info.token = END;
        info.expr = p;
        return Ok(Value::none());
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
            Ok(Value::none())
        }
        Some(')') => {
            info.token = CLOSE_PAREN;
            Ok(Value::none())
        }
        Some(',') => {
            info.token = COMMA;
            Ok(Value::none())
        }
        Some('*') => {
            info.token = MULT;
            Ok(Value::none())
        }
        Some('/') => {
            info.token = DIVIDE;
            Ok(Value::none())
        }
        Some('%') => {
            info.token = MOD;
            Ok(Value::none())
        }
        Some('+') => {
            info.token = PLUS;
            Ok(Value::none())
        }
        Some('-') => {
            info.token = MINUS;
            Ok(Value::none())
        }
        Some('?') => {
            info.token = QUESTY;
            Ok(Value::none())
        }
        Some(':') => {
            info.token = COLON;
            Ok(Value::none())
        }
        Some('<') => {
            p.skip();
            match p.peek() {
                Some('<') => {
                    info.token = LEFT_SHIFT;
                    p.skip();
                    info.expr = p;
                    Ok(Value::none())
                }
                Some('=') => {
                    info.token = LEQ;
                    p.skip();
                    info.expr = p;
                    Ok(Value::none())
                }
                _ => {
                    info.token = LESS;
                    Ok(Value::none())
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
                    Ok(Value::none())
                }
                Some('=') => {
                    info.token = GEQ;
                    p.skip();
                    info.expr = p;
                    Ok(Value::none())
                }
                _ => {
                    info.token = GREATER;
                    Ok(Value::none())
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
            Ok(Value::none())
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
            Ok(Value::none())
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
            Ok(Value::none())
        }
        Some('^') => {
            info.token = BIT_XOR;
            Ok(Value::none())
        }
        Some('|') => {
            molt_err!("Not yet implemented: {:?}", p.peek())
        }
        Some('~') => {
            info.token = BIT_NOT;
            Ok(Value::none())
        }
        Some(_) => {
            // TODO: if is alphabetic, see if it's a function.
            p.skip();
            info.expr = p;
            info.token = UNKNOWN;
            Ok(Value::none())
        }
        None => {
            p.skip();
            info.expr = p;
            info.token = UNKNOWN;
            Ok(Value::none())
        }
    }
}

/// Given a string (such as one coming from command or variable substitution) make a
/// Value based on the string.  The value will be floating-point or integer if possible,
/// or else it will just be a copy of the string.  Returns an error on failed numeric
/// conversions.
fn expr_parse_string(_interp: &mut Interp, string: &str) -> ValueResult {
    if !string.is_empty() {
        let mut value = Value::none();
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
                value = Value::int(int);
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
                    value = Value::float(flt);
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
/// Also, should probably make this return a new Value directly, instead of modifying
/// the old one.
fn expr_make_string(_interp: &mut Interp, value: &mut Value) {
    match value.vtype {
        Type::Int => {
            value.vtype=Type::String;
            value.str = format!("{}", value.int);
        }
        Type::Int => {
            value.vtype=Type::String;
            value.str = format!("{}", value.flt);
        }
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
            Type::Int => true,
            Type::Float => true,
            Type::String => false,
        }
    }
}

// Return standard syntax error
fn syntax_error(info: &mut ExprInfo) -> ValueResult {
    molt_err!("syntax error in expression \"{}\"", info.original_expr)
}

// Return standard illegal type error
fn illegal_type(bad_type: Type, op: i32) -> ValueResult {
    let type_str = if bad_type == Type::Float {
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
        if val1.vtype != val2.vtype {
            return false;
        }

        match &val1.vtype {
            Type::Int => {
                val1.int == val2.int
            }
            Type::Float => {
                val1.flt == val2.flt
            }
            Type::String => {
                val1.str == val2.str
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
