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

/// A parsed value
#[derive(Debug)]
enum Value<'a> {
    Int(MoltInt),
    Float(MoltFloat),
    Str(&'a str),
}

/// Context for expr parsing

struct ExprContext<'a> {
    // The full expr.
    expr: String,

    // The input iterator
    chars: Peekable<Chars<'a>>,

    // Last token's type; see constants
    token: i32,
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
