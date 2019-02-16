//! The Expr command and parser
//!
//! * Ultimately, the command should probably move to commands.rs.
//!   But this is convenient for now.

use std::str::Chars;
use std::iter::Peekable;
use crate::*;
use crate::interp::Interp;

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

type ValueResult = Result<Value,ResultCode>;

/// A parsed value
#[derive(Debug)]
enum Value {
    Int(MoltInt),
    Float(MoltFloat),
    Str(String),
    None, // Equivalent to Str("").
}

/// Context for expr parsing

struct ExprInfo<'a> {
    // The full expr.
    original_expr: String,

    // The input iterator, e.g., the pointer to the next character.
    expr: Peekable<Chars<'a>>,

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
            expr: expr.chars().peekable(),
            token: -1,
            no_eval: false,
        }
    }

    fn next_is(&mut self, ch: char) -> bool {
        Some(&ch) == self.expr.peek()
    }

    // Are we at the end of the input expression?
    fn at_end(&mut self) -> bool {
        self.expr.peek().is_none()
    }

    // Skip whitespace characters
    fn skip_space(&mut self) {
        while let Some(c) = self.expr.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.expr.next();
        }
    }
}

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

/// Evaluate an expression and returns its value in string form.
pub fn molt_expr_string(interp: &mut Interp, string: &str) -> InterpResult {
    let value = expr_top_level(interp, string)?;

    match value {
        Value::Int(int) => molt_ok!("{}", int),
        Value::Float(flt) => molt_ok!("{}", flt), // TODO: better float->string logic
        Value::Str(str) => molt_ok!(str),
        Value::None => molt_ok!(""),
    }
}

/// Provides top-level functionality shared by molt_expr_string, molt_expr_int, etc.
fn expr_top_level<'a>(interp: &mut Interp, string: &'a str) -> ValueResult {
    let info = &mut ExprInfo::new(string);

    let result: Value = expr_get_value(interp, info, -1)?;

    if info.token != END {
        return molt_err!("syntax error in expression \"{}\"", string);
    }

    if let Value::Float(_) = result {
        // TODO: check for NaN, INF, and throw IEEE floating point error.
    }

    Ok(result)
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
                        match value {
                            Value::Int(int) => {
                                value = Value::Int(-int);
                            }
                            Value::Float(flt) => {
                                value = Value::Float(-flt);
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
                        match value {
                            Value::Int(int) => {
                                if int == 0 {
                                    value = Value::Int(1);
                                } else {
                                    value = Value::Int(0);
                                }
                            }
                            Value::Float(flt) => {
                                if flt == 0.0 {
                                    value = Value::Int(1);
                                } else {
                                    value = Value::Int(0);
                                }
                            }
                            _ => {
                                return illegal_type(&value, operator);
                            }
                        }
                    }
                    BIT_NOT => {
                        if let Value::Int(int) = value {
                            value = Value::Int(!int);
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

    let mut value2 = Value::None;

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
                return Ok(Value::Str("WTF".into()));
            } else {
                return syntax_error(info);
            }
        }

        if PREC_TABLE[operator as usize] <= prec {
            // TODO: What value should we be returning?
            return Ok(Value::Str("WTF".into()));
        }

        // If we're doing an AND or OR and the first operand already determines
        // the result, don't execute anything in the second operand: just parse.
        // Same style for ?: pairs.

        if operator == AND || operator == OR || operator == QUESTY {

        }
    }

    // Just to quiet the warnings for now.
    Ok(Value::None)
}

/// Lexical analyzer for the expression parser.  Parses a single value, operator, or other
/// syntactic element from an expression string.
///
/// ## Results
///
/// Returns an error result if an error occurs while doing lexical analysis or
/// executing an embedded command.  On success, info.token is set to the last token type,
/// and info is updated to point to the next token.  If the token is VALUE, the returned
/// Value contains it; otherwise, the value is Value::None.
///
/// TODO: It might be better to combine info.token and the value into one data object,
/// i.e., add Value::Op(i32) or make each token type a Value (and handle precedence).
/// But one step at a time.
fn expr_lex(interp: &mut Interp, info: &mut ExprInfo) -> ValueResult {
    // FIRST, skip white space.
    let mut p = info.expr.clone();

    while let Some(ch) = p.peek() {
        if ch.is_whitespace() {
            p.next();
        } else {
            break;
        }
    }


    info.skip_space();

    if info.at_end() {
        info.token = END;
        return Ok(Value::None);
    }

    // First try to parse the token as an integer or floating-point number.
    // Don't want to check for a number if the first character is "+"
    // or "-".  If we do, we might treat a binary operator as unary by
    // mistake, which will eventually cause a syntax error.

    // if !info.next_is('+') && !info.next_is('-') {
    //     if expr_looks_like_int(info) {
    //         // Convert value at next to int
    //         // Return error on overflow; next is unchanged.
    //         // Return Value::Int with token=VALUE.
    //     } else {
    //         // Convert value at next to double
    //         // Return error on overflow; next is unchanged.
    //         // Return Value::Float with token=VALUE.
    //     }
    // }



    Ok(Value::None) // TEMP
}

// fn expr_looks_like_int<'a>(ptr: &mut CPtr<'a>) -> bool {
    // // FIRST, skip whitespace
    // ptr.skip_while(|c| c.is_whitespace());
    //
    // if ptr.peek() == Some(&'+') || ptr.peek() == Some(&'-') {
    //     ptr.next();
    // }
//
//
//     true
// }

impl Value {
    fn is_numeric(&self) -> bool {
        match self {
            Value::Int(_) => true,
            Value::Float(_) => true,
            Value::Str(_) => false,
            Value::None => false,
        }
    }
}

// Return standard syntax error
fn syntax_error(info: &mut ExprInfo) -> ValueResult {
    molt_err!("syntax error in expression \"{}\"", info.original_expr)
}

// Return standard illegal type error
fn illegal_type(value: &Value, op: i32) -> ValueResult {
    let type_str = if let Value::Float(_) = value {
        "floating-point value"
    } else {
        "non-numeric string"
    };

    molt_err!("can't use {} as operand of \"{}\"", type_str, OP_STRINGS[op as usize])
}
